extends VBoxContainer
## The vault editor's side panel (VE-5 of `VAULT_EDITOR_PROPOSAL.md` s.3.2, s.3.5): **Links**
## (backlinks and unlinked mentions), **Outline**, **Properties** and **Tags**, over the note open in
## the editor beside it. Held by `vault_window.gd` without a `class_name`, as `markdown_editor.gd` is.
##
## ## What it must never do
##
## - **Parse Markdown.** Every list comes from the `cartalith-vault` crate through the bridge
##   (`vault_file_backlinks`, `vault_file_mentions`, `vault_link_spans`, `vault_mention_spans`,
##   `vault_link_mention`, `vault_note_headings`, `vault_note_properties`, `vault_frontmatter_set` /
##   `_remove`, `vault_tag_index`). GDScript here holds no second parser (s.5).
## - **Write a note by any route but the hash guard.** The two edits that touch the text of the OPEN note
##   (a property, and the editor's own jump) go through `apply_text`, which is the editor's one-undo-step
##   replace, so the autosave (`vault_window.gd::_write_now`) is still the only writer of that note. The
##   one edit that touches ANOTHER note (an unlinked mention's Link button) reads it with
##   `vault_read_file_for_edit`, edits the text with `vault_link_mention`, and writes it with
##   `vault_write_file` against the hash it just read, so a note changed outside in between is refused
##   and left alone, never overwritten.
## - **Lose typed text.** A property is committed on Enter or when its field loses focus, never on a
##   timer, and a rebuild of the panel while a field of it has focus is skipped (the person is typing).
##
## ## The pieces that are the host's
##
## The host (`VaultWindow`) hands in Callables, so this file knows neither the window nor the editor:
## `live_text`, `note_rel`, `open_note(rel, line)`, `jump_to(line, heading)`, `apply_text(text)` and `say(msg)`.
## The host calls `on_text_changed()` after an edit and `on_saved()` after a save.

## Node names that carry data (a note path, a property key, a tag) go through `validate_node_name()`,
## because a Node name may not hold `.` `:` `@` `/` `"` `%` and a path always does; the probe finds them
## by the same call.

## The tabs, in order, with their labels. The keys are the node-name suffixes the probe finds.
const TABS := [["links", "Links"], ["outline", "Outline"], ["props", "Properties"], ["tags", "Tags"]]

## How many unlinked mentions to list. The crate opens one note per candidate, so this is a labelled
## judgement (a long list is a chore to read anyway), not a measurement.
const MENTION_MAX := 40

## Rows shown per tag in the Tags tab before "more" (a tag on many notes should not bury the rest).
const TAG_NOTES_SHOWN := 30

## The outline rebuilds this long after the last edit (seconds): the text is parsed by the crate, so it
## waits for a pause. A labelled judgement, as the footer's refresh is.
const REFRESH_SEC := 0.45

## Notes the Tags index opens at most (the crate says how many it opened; see `tag_index`). Equals the
## shell's note cap of the moment; VE-9 raises the cap and this follows it.
const TAG_SCAN_MAX := 2000

var bridge
var phone := false
var live_text: Callable = Callable()
var note_rel: Callable = Callable()
var open_note: Callable = Callable()
var jump_to: Callable = Callable()
var apply_text: Callable = Callable()
var say: Callable = Callable()
## Called by the phone's "Note" button, which closes the sheet and shows the editor again.
var on_close: Callable = Callable()
## VE-6. `rename_note()` starts the rename flow for the open note (the host owns the prompt, the plan,
## the write and the report; this panel only offers the button). `last_rename()` answers the host's record
## of the last rename that moved a note, `{}` when none, so the tab can offer its undo; `undo_rename()` runs it.
var rename_note: Callable = Callable()
var last_rename: Callable = Callable()
var undo_rename: Callable = Callable()

## The active tab key (one of `TABS`).
var tab := "links"
## Tag index of the last scan, `{}` until a scan; dropped when a note is saved (it may be out of date).
var _tags: Dictionary = {}
## A tag whose notes are listed open in the Tags tab (`""` for none).
var _tag_open := ""
var _tab_buttons := {}
## True while `rebuild` is tearing the old rows down. Freeing a focused `LineEdit` emits its
## `focus_exited`, which would commit the field a second time, from inside the teardown; the commit
## handlers check this and stand down.
var _rebuilding := false
var _body: VBoxContainer
var _timer: Timer


## Builds the tab row and the body. Call once, after the host's Callables are set.
func setup() -> void:
	name = "VaultSidePanel"
	add_theme_constant_override("separation", 6)
	if phone:
		var back := DccWidgets.action(self, "‹ Note", func():
			if on_close.is_valid():
				on_close.call())
		back.name = "SideClose"
		back.tooltip_text = "Back to the note."
		_floor(back)
	var row := HFlowContainer.new()
	row.name = "SideTabs"
	row.add_theme_constant_override("h_separation", 4)
	row.add_theme_constant_override("v_separation", 4)
	add_child(row)
	for t in TABS:
		var key := String(t[0])
		var b := DccWidgets.segment(row, String(t[1]), func(): show_tab(key))
		b.name = "SideTab_" + key
		_floor(b)
		_tab_buttons[key] = b
	var scroll := ScrollContainer.new()
	scroll.name = "SideScroll"
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.add_theme_stylebox_override("panel", DccTheme.empty())
	add_child(scroll)
	_body = VBoxContainer.new()
	_body.name = "SideBody"
	_body.add_theme_constant_override("separation", 4)
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(_body)
	_timer = Timer.new()
	_timer.one_shot = true
	_timer.wait_time = REFRESH_SEC
	_timer.timeout.connect(_on_timer)
	add_child(_timer)
	show_tab(tab)


## The 44 dp touch floor on the phone (`DccWidgets.chip` and `segment` do not apply it themselves).
func _floor(b: Button) -> void:
	if phone:
		b.custom_minimum_size.y = maxf(b.custom_minimum_size.y, 44.0)


## Switches to `key` and draws it.
func show_tab(key: String) -> void:
	tab = key
	for k in _tab_buttons:
		DccWidgets.set_segment_on(_tab_buttons[k], k == key)
	rebuild()


## The host calls this after every edit of the open note. The outline and the properties follow the
## text, a moment after the last key.
func on_text_changed() -> void:
	if tab == "outline" or tab == "props":
		_timer.start()


## The host calls this after the open note was written. Backlinks and the tag index may have moved.
func on_saved() -> void:
	_tags = {}
	if tab == "links" or tab == "tags":
		rebuild()


func _on_timer() -> void:
	if _field_has_focus():
		_timer.start()  ## the person is typing in one of our fields: ask again later, do not rebuild under them
		return
	rebuild()


## Whether a text field of this panel holds the keyboard.
func _field_has_focus() -> bool:
	var f := get_viewport().gui_get_focus_owner() if is_inside_tree() else null
	return f != null and f is LineEdit and is_ancestor_of(f)


## Redraws the active tab from the live data.
func rebuild() -> void:
	if _body == null:
		return
	_rebuilding = true
	for c in _body.get_children():
		_body.remove_child(c)
		c.queue_free()
	_rebuilding = false
	match tab:
		"links":
			_build_links()
		"outline":
			_build_outline()
		"props":
			_build_props()
		"tags":
			_build_tags()


func _rel() -> String:
	return String(note_rel.call()) if note_rel.is_valid() else ""


func _text() -> String:
	return String(live_text.call()) if live_text.is_valid() else ""


func _say(msg: String) -> void:
	if say.is_valid():
		say.call(msg)


## A wrapping caption line.
func _note(text: String) -> Label:
	var l := DccTheme.label(text, "text_ghost", DccTheme.FS_SMALL)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.custom_minimum_size.x = 120
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_body.add_child(l)
	return l


func _heading(text: String) -> void:
	var l := DccTheme.mono_label(text, "text_dim", DccTheme.FS_SMALL, 1)
	_body.add_child(l)


## A full-width row button with wrapped text, 44 dp tall on the phone.
func _row_button(text: String, tip: String, on_press: Callable) -> Button:
	var b := Button.new()
	b.text = text
	b.tooltip_text = tip
	b.flat = true
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	b.custom_minimum_size = Vector2(120, 0)
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	b.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	b.add_theme_color_override("font_color", DccTheme.c("accent"))
	b.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	b.pressed.connect(on_press)
	_floor(b)
	return b


# -- Links ------------------------------------------------------------------------

## Backlinks (notes that link here) and unlinked mentions (notes that name this one in prose), from the
## backlink index. With no index the tab says so and offers to build it: an empty list would read as
## "nothing links here", which is a claim the vault never made (`vault_backlink_stats().built`).
func _build_links() -> void:
	var rel := _rel()
	if rel == "":
		_note("No note is open.")
		return
	var stats: Dictionary = bridge.vault_backlink_stats()
	if not bool(stats.get("built", false)):
		_note("The link index has not been built, so backlinks and mentions are unknown (not zero).")
		var b := DccWidgets.action(_body, "Build the index", func():
			bridge.vault_refresh_backlinks(2000)
			rebuild())
		b.name = "SideBuildIndex"
		_floor(b)
		return
	var back: Array = bridge.vault_file_backlinks(rel)
	var mentions: Array = bridge.vault_file_mentions(rel, MENTION_MAX)
	_heading("BACKLINKS  %d" % back.size())
	if back.is_empty():
		_note("No note links here yet.")
	for bl in back:
		var d: Dictionary = bl
		var src := String(d.get("rel", ""))
		var n := int(d.get("count", 1))
		var row := _row_button("%s%s" % [src, "  ×%d" % n if n > 1 else ""], "Opens %s at the first link to this note." % src, func(): open_backlink(src))
		row.name = ("Backlink_" + src).validate_node_name()
		_body.add_child(row)
	_heading("UNLINKED MENTIONS  %d" % mentions.size())
	if mentions.is_empty():
		_note("No other note names this one without linking it.")
	for m in mentions:
		var md: Dictionary = m
		var src2 := String(md.get("rel", ""))
		var box := VBoxContainer.new()
		box.add_theme_constant_override("separation", 2)
		_body.add_child(box)
		var open := _row_button(src2, "Opens %s at the mention." % src2, func(): open_mention(src2))
		open.name = ("Mention_" + src2).validate_node_name()
		box.add_child(open)
		var ex := DccTheme.label(String(md.get("excerpt", "")), "text_ghost", DccTheme.FS_SMALL)
		ex.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		ex.custom_minimum_size.x = 120
		ex.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		box.add_child(ex)
		var link := DccWidgets.action(box, "Link", func(): link_mention(src2))
		link.name = ("MentionLink_" + src2).validate_node_name()
		link.tooltip_text = "Turns the first mention in %s into a link to this note. Written to that note through the same hash guard as any save." % src2
		_floor(link)


## The 0-based line of the first link in `source` that resolves to `target_rel`, or 0 when none is found
## (the index and the note can disagree if the note changed since the last refresh). Reads one note.
func backlink_line(source: String, target_rel: String) -> int:
	var text: String = bridge.vault_read_file(source)
	for s in bridge.vault_link_spans(text):
		var sd: Dictionary = s
		if bool(sd.get("external", false)) or not sd.has("target"):
			continue
		var r: Dictionary = bridge.vault_resolve_link(source, String(sd["target"]))
		if bool(r.get("ok", false)) and String(r.get("status", "")) == "unique" and String(r.get("path", "")) == target_rel:
			return int(sd.get("line", 0))
	return 0


## Opens the backlinking note `source` at the line of its link to this note.
func open_backlink(source: String) -> void:
	if open_note.is_valid():
		open_note.call(source, backlink_line(source, _rel()))


## The name the crate looks for in prose: this note's file stem with `-` and `_` read as spaces
## (`VaultSession::file_mentions`'s own derivation, so the two agree on what a mention is).
static func mention_name(rel: String) -> String:
	return rel.get_file().get_basename().replace("-", " ").replace("_", " ")


## The 0-based line of the first unlinked mention of this note in `source`, or 0.
func mention_line(source: String) -> int:
	var text: String = bridge.vault_read_file(source)
	var spans: Array = bridge.vault_mention_spans(text, mention_name(_rel()))
	return int((spans[0] as Dictionary).get("line", 0)) if not spans.is_empty() else 0


func open_mention(source: String) -> void:
	if open_note.is_valid():
		open_note.call(source, mention_line(source))


## What goes inside `[[ ]]` to name `target_rel` from `source`: the bare file name when that resolves to
## exactly this note from there, else the path without `.md`, so a link never lands on a twin.
func link_target(source: String, target_rel: String) -> String:
	var stem := target_rel.get_file().get_basename()
	var r: Dictionary = bridge.vault_resolve_link(source, stem)
	if bool(r.get("ok", false)) and String(r.get("status", "")) == "unique" and String(r.get("path", "")) == target_rel:
		return stem
	return target_rel.trim_suffix(".md")


## **Link**: turns the first unlinked mention of this note in `source` into `[[Name]]` (or `[[Name|as
## written]]` when the case differs, which the crate decides) and writes `source` through the hash guard.
## Returns `{ok, error?, rel, line?}`; the status line says what happened either way. Never forces: a
## refused write (changed outside, read-only, vanished) leaves `source` as it was on disk.
func link_mention(source: String) -> Dictionary:
	var target_rel := _rel()
	var r: Dictionary = bridge.vault_read_file_for_edit(source)
	if not bool(r.get("ok", false)):
		_say("Could not read %s: %s" % [source, String(r.get("error", ""))])
		return {"ok": false, "error": String(r.get("error", ""))}
	var text := String(r["text"])
	var spans: Array = bridge.vault_mention_spans(text, mention_name(target_rel))
	if spans.is_empty():
		_say("%s no longer mentions this note without a link." % source)
		rebuild()
		return {"ok": false, "error": "no mention"}
	var sp: Dictionary = spans[0]
	var lk: Dictionary = bridge.vault_link_mention(text, int(sp["start"]), int(sp["end"]), link_target(source, target_rel))
	if not bool(lk.get("ok", false)):
		_say("Not linked: %s" % String(lk.get("error", "")))
		return {"ok": false, "error": String(lk.get("error", ""))}
	var w: Dictionary = bridge.vault_write_file(source, String(lk["text"]), String(r.get("hash", "")))
	if not bool(w.get("ok", false)):
		_say("Not linked: %s was not written (%s). Nothing was changed." % [source, String(w.get("error", ""))])
		return {"ok": false, "error": String(w.get("error", ""))}
	bridge.vault_refresh_backlinks(2000)
	_say("Linked in %s." % source)
	rebuild()
	return {"ok": true, "rel": source, "line": int(sp.get("line", 0))}


# -- Outline ----------------------------------------------------------------------

## The note's headings, indented by level; a click jumps to the heading in both views (the host's
## `jump_to`). Fenced code and the frontmatter are skipped by the crate.
func _build_outline() -> void:
	var hs: Array = bridge.vault_note_headings(_text())
	if hs.is_empty():
		_note("This note has no headings.")
		return
	for h in hs:
		var hd: Dictionary = h
		var lvl := clampi(int(hd.get("level", 1)), 1, 6)
		var txt := String(hd.get("text", ""))
		var line := int(hd.get("line", 0))
		var row := HBoxContainer.new()
		row.name = "Outline_%d" % line
		_body.add_child(row)
		var pad := Control.new()
		pad.custom_minimum_size.x = 12.0 * float(lvl - 1)
		row.add_child(pad)
		var b := _row_button(txt, "Jumps to this heading.", func(): _jump(line, txt))
		b.name = "OutlineJump_%d" % line
		row.add_child(b)


func _jump(line: int, heading: String) -> void:
	if jump_to.is_valid():
		jump_to.call(line, heading)


# -- Properties -------------------------------------------------------------------

## The frontmatter as a form: one row per property (a text field for a scalar, chips and an add field
## for a list, a remove button), plus an add-property row. Every edit is the crate's
## `frontmatter_set` / `frontmatter_remove` on the editor's LIVE text, applied as one undo step, so
## every other line of the note stays byte-identical and the autosave writes it. A block the crate
## cannot read safely (malformed, a repeated key) is listed read-only with the reason.
func _build_props() -> void:
	_build_rename()
	var info: Dictionary = bridge.vault_note_properties(_text())
	var present := bool(info.get("present", false))
	var editable := bool(info.get("editable", true))
	var props: Array = info.get("properties", [])
	if not present:
		_note("This note has no properties yet. Adding one creates the block at the top.")
	elif not editable:
		_note("These properties cannot be read safely (a malformed line or a repeated name), so they are shown but not editable here. Fix them in the text.")
	for p in props:
		var pd: Dictionary = p
		_build_prop_row(String(pd.get("key", "")), bool(pd.get("list", false)), pd.get("values", PackedStringArray()), editable)
	if editable:
		_build_add_row()


func _build_prop_row(key: String, is_list: bool, values, editable: bool) -> void:
	var box := VBoxContainer.new()
	box.name = ("Prop_" + key).validate_node_name()
	box.add_theme_constant_override("separation", 2)
	_body.add_child(box)
	var head := HBoxContainer.new()
	box.add_child(head)
	var kl := DccTheme.mono_label(key, "text_dim", DccTheme.FS_SMALL, 1)
	kl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(kl)
	if editable:
		var rm := DccWidgets.chip(head, "✕", func(): remove_property(key))
		rm.name = ("PropRemove_" + key).validate_node_name()
		rm.tooltip_text = "Removes the property %s from the note. Ctrl+Z in the note brings it back." % key
		_floor(rm)
	if is_list:
		var flow := HFlowContainer.new()
		flow.add_theme_constant_override("h_separation", 4)
		flow.add_theme_constant_override("v_separation", 4)
		box.add_child(flow)
		var items: Array = []
		for v in values:
			items.append(String(v))
		for i in items.size():
			var idx := i
			var c := DccWidgets.chip(flow, "%s%s" % [items[i], "  ✕" if editable else ""], func():
				if editable:
					set_list(key, _without(items, idx)))
			c.name = ("PropItem_%s_%d" % [key, i]).validate_node_name()
			_floor(c)
		if editable:
			var add := LineEdit.new()
			add.name = ("PropAdd_" + key).validate_node_name()
			add.placeholder_text = "add…"
			add.custom_minimum_size.x = 90
			add.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			add.text_submitted.connect(func(t: String):
				if t.strip_edges() != "":
					set_list(key, items + [t.strip_edges()]))
			_floor_field(add)
			flow.add_child(add)
	else:
		var f := LineEdit.new()
		f.name = ("PropValue_" + key).validate_node_name()
		var cur := String(values[0]) if values.size() > 0 else ""
		f.text = cur
		f.editable = editable
		f.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		f.custom_minimum_size.x = 100
		## Committed on Enter or on losing focus, only if it changed: typing is never written mid-word.
		var commit := func(t: String):
			if not _rebuilding and t != cur:
				set_scalar(key, t)
		f.text_submitted.connect(commit)
		f.focus_exited.connect(func(): commit.call(f.text))
		_floor_field(f)
		box.add_child(f)


func _floor_field(f: LineEdit) -> void:
	if phone:
		f.custom_minimum_size.y = maxf(f.custom_minimum_size.y, 44.0)


func _without(items: Array, idx: int) -> Array:
	var out: Array = items.duplicate()
	out.remove_at(idx)
	return out


## The add-property row: a name, a value (a comma-separated value is NOT split: one scalar), an Add button.
## Rename (VE-6): one button, and, after a rename that moved a note, a line saying so with its undo. The
## flow itself (prompt, plan, write, report) is the host's, because it must go through the host's one writer
## and its one note focus; this tab never touches a file.
func _build_rename() -> void:
	_heading("RENAME")
	var b := _row_button("Rename this note…", "Changes the note's file name and rewrites the links to it in your other notes. You see how many first.", func():
		if rename_note.is_valid():
			rename_note.call())
	b.name = "NoteRename"
	_body.add_child(b)
	var last: Dictionary = last_rename.call() if last_rename.is_valid() else {}
	if not last.is_empty():
		_note("Renamed %s to %s." % [String(last.get("old", "")), String(last.get("new", ""))])
		var u := _row_button("Undo the rename", "Renames the note back and puts the links back. Refused if the old name is taken, and it says what it could not put back.", func():
			if undo_rename.is_valid():
				undo_rename.call())
		u.name = "NoteRenameUndo"
		_body.add_child(u)


func _build_add_row() -> void:
	_heading("ADD A PROPERTY")
	var row := HFlowContainer.new()
	row.add_theme_constant_override("h_separation", 4)
	row.add_theme_constant_override("v_separation", 4)
	_body.add_child(row)
	var k := LineEdit.new()
	k.name = "PropNewKey"
	k.placeholder_text = "name"
	k.custom_minimum_size.x = 80
	k.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_floor_field(k)
	row.add_child(k)
	var v := LineEdit.new()
	v.name = "PropNewValue"
	v.placeholder_text = "value"
	v.custom_minimum_size.x = 80
	v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_floor_field(v)
	row.add_child(v)
	var go := func():
		if k.text.strip_edges() != "":
			set_scalar(k.text.strip_edges(), v.text)
	v.text_submitted.connect(func(_t: String): go.call())
	var add := DccWidgets.action(row, "Add", go)
	add.name = "PropNewAdd"
	_floor(add)


## Applies a frontmatter edit to the editor's text. `result` is the crate's `{ok, text|error}`.
func _apply(result: Dictionary, what: String) -> bool:
	if not bool(result.get("ok", false)):
		_say("Not changed: %s" % String(result.get("error", "")))
		rebuild()
		return false
	if result.has("text") and apply_text.is_valid():
		apply_text.call(String(result["text"]))
	_say(what)
	## Deferred: this runs inside the handler of a widget the rebuild is about to free.
	rebuild.call_deferred()
	return true


## Sets a scalar property on the live text. Public so the probe drives the same path as the field.
func set_scalar(key: String, value: String) -> bool:
	return _apply(bridge.vault_frontmatter_set(_text(), key, value), "Property %s set." % key)


## Sets a list property (its items in order).
func set_list(key: String, items: Array) -> bool:
	return _apply(bridge.vault_frontmatter_set(_text(), key, PackedStringArray(items)), "Property %s changed." % key)


## Removes a property. The crate answers `removed: false` (no `text`) when there is none: said, not an error.
func remove_property(key: String) -> bool:
	var r: Dictionary = bridge.vault_frontmatter_remove(_text(), key)
	if bool(r.get("ok", false)) and not bool(r.get("removed", false)):
		_say("There is no property %s." % key)
		rebuild()
		return false
	return _apply(r, "Property %s removed." % key)


# -- Tags -------------------------------------------------------------------------

## The vault's tags, most-used first, with the notes carrying each (opened by tapping the tag). Scanned
## on first view and when asked: the crate opens at most `TAG_SCAN_MAX` notes and says how many it
## opened of how many exist, which this tab repeats when it is partial instead of presenting the
## count as the whole. Tapping a tag lists its notes from this index, each one opening the note (a
## departure from s.3.5's "searches for it": the index is exact where a text search for `#tag` is not).
func _build_tags() -> void:
	if _tags.is_empty():
		var r: Dictionary = bridge.vault_tag_index(TAG_SCAN_MAX)
		if not bool(r.get("ok", false)):
			_note("Tags are unavailable: %s" % String(r.get("error", "no vault is connected")))
			return
		_tags = r
	var rows: Array = _tags.get("tags", [])
	var scanned := int(_tags.get("scanned", 0))
	var total := int(_tags.get("total", 0))
	_heading("TAGS  %d" % rows.size())
	if scanned < total:
		_note("Partial: %d of %d notes were read, so counts may be low." % [scanned, total])
	if rows.is_empty():
		_note("No tags found in %d note%s." % [scanned, "" if scanned == 1 else "s"])
	for t in rows:
		var td: Dictionary = t
		var tag := String(td.get("tag", ""))
		var notes: Array = td.get("notes", [])
		var open := _tag_open == tag
		var b := _row_button("%s #%s  ·  %d" % ["▾" if open else "▸", tag, notes.size()], "Lists the notes tagged #%s." % tag, func():
			_tag_open = "" if _tag_open == tag else tag
			rebuild())
		b.name = ("Tag_" + tag).validate_node_name()
		_body.add_child(b)
		if open:
			var shown := 0
			for n in notes:
				if shown >= TAG_NOTES_SHOWN:
					_note("… and %d more." % (notes.size() - shown))
					break
				shown += 1
				var rel := String(n)
				var nb := _row_button("      " + rel, "Opens %s." % rel, func():
					if open_note.is_valid():
						open_note.call(rel, 0))
				nb.name = ("TagNote_" + rel).validate_node_name()
				_body.add_child(nb)
	var rescan := DccWidgets.action(_body, "Rescan", func():
		_tags = {}
		rebuild())
	rescan.name = "SideRescan"
	_floor(rescan)
