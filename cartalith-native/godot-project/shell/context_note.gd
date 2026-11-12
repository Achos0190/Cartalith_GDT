extends RefCounted

## PC-5 (`PC_SHELL_PROPOSAL.md` §3.4 "The Note tab", milestone PC-5; Ruling DY, VQ1, VQ4, VQ6): the Context
## panel's Note tab -- the subject's own note, edited in place.
##
## **What it is.** A view of the vault editor, never a second one. The tab owns a frame (`_slot`) and asks
## the vault window (`DccApp.vault_window`, `VaultWindow.dock_note`) to build ITS editor (`markdown_editor.gd`,
## coloured Write by default, Preview a toggle, the conflict banner, autosave, the session undo and Revert to
## opened…) into it. Every write is the window's `_write_now`, behind the hash guard; the quit flush
## (`DccApp._vault_safe_to_exit`, pause, focus-out) is the window's `flush_for_exit`. This file holds no text,
## no path of the note's and no writer: it finds the note, hosts the frame, and draws what surrounds the
## editor.
##
## **One note focus, one writer, defined.** The note focus is the single pair (`VaultWindow._browse_path`,
## the editor built for it). It is in exactly one of three places: this tab's frame, the vault window's body,
## or nowhere. This tab docks only while the window is hidden (`dock_note` answers `window` otherwise); the
## window, when it opens or rebuilds, frees the docked editor first (`VaultWindow._clear`) and says so
## (`dock_released`). A note open "in both" is therefore never two editors: it is one, wherever it was last
## put, and the other host shows a notice. Proved in `_pc5note_probe` (two editors never alive together, one
## write per edit, the window's open flushing the tab's text, an outside change giving one banner).
##
## **No typed text is lost.** A selection that moves while the note holds unsaved text flushes it through the
## guarded write first. When that write is REFUSED (a conflict, a deleted or read-only note, a vanished vault)
## the old editor stays on screen with its banner and the switch is held: the tab says what the selection is
## now and shows the new subject's note as soon as a save lands (`dock_saved`). Switching tabs hides the
## frame, the editor is kept alive underneath and its autosave keeps running; pinning only changes which
## subject the tab follows; closing the panel or quitting is the window's flush. Removing the tab releases the
## dock (`release_dock`), which saves first and keeps the editor if it cannot.
##
## **What it must never do.** Write a note except through the window; keep a second copy of the selection
## (it is handed the subject per refresh); show a plausible value for something absent (a subject with no note
## says so and offers to create one; one the vault cannot bind says why, never a blank editor); create a note
## as a side effect (creation is a button, with the template choice, VQ2).
##
## Preloaded by `right_dock.gd` (`ContextNote`), no `class_name`, so it needs no `--headless --import` pass.

## `DccApp`.
var app
## The tab's frame body: `_top` (notices), `_slot` (the editor's host, kept between refreshes), `_bottom`
## (dated entries). Built once by `build_into()`.
var _top: VBoxContainer
var _slot: VBoxContainer
var _bottom: VBoxContainer
## The subject (a `DccApp.selection` value) the last refresh was for.
var _subject: Dictionary = {}
## True while a switch is held because the note on screen could not be saved (see the header).
var _held := false
## The dated-entry rows of the last refresh: `[{button, start, end}]`, so a year move re-lights without a
## rebuild.
var _rows: Array = []
## Whether the window's signals are connected yet.
var _wired := false

## A subject kind that the vault can bind a note to, and the vault's own name for it
## (`cartalith_vault::EntityKind`). Settlement is addressed by its `tid`, never its index (an index shifts
## when another settlement is deleted). Landmark is bound too (VE-9, Ruling EI): its selection id is an index
## into `bridge.landmarks()`, so `resolve` turns it into the vault entity id through the landmark's key
## (`vault_landmark_entity_id`) and names it by its generated or user-given name (`landmark_names`). Culture,
## conflict and note are not bound here.
## **Label and icon are bound too** (Ruling EQ item 2, edit-from-context): a hand-placed label's or a placed
## icon's selection id is its index in the engine's store, which shifts when an earlier item is deleted, so
## `resolve` turns it into the vault entity id through the engine (`vault_label_entity_id` /
## `vault_icon_entity_id`: the item's cell, never the index) and refuses, saying why, when two items share a cell.
const BINDABLE := {"settlement": "settlement", "faction": "faction", "landmark": "landmark",
	"label": "label", "icon": "icon"}


func _init(a) -> void:
	app = a


## Builds the frame's structure into `body` once. A second call is a no-op.
func build_into(body: Control) -> void:
	if _slot != null and is_instance_valid(_slot):
		return
	_top = VBoxContainer.new()
	_top.name = "NoteTop"
	_top.add_theme_constant_override("separation", 4)
	body.add_child(_top)
	_slot = VBoxContainer.new()
	_slot.name = "NoteSlot"
	_slot.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_slot.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_child(_slot)
	_bottom = VBoxContainer.new()
	_bottom.name = "NoteBottom"
	_bottom.add_theme_constant_override("separation", 2)
	body.add_child(_bottom)
	## A frame that leaves the tree (the panel rebuilt for a class swap, the app closing) hands the editor
	## back: saved if it can be, kept with its text if it cannot.
	_slot.tree_exiting.connect(func():
		var vw = app.get("vault_window")
		if vw != null and is_instance_valid(vw) and vw.is_docked():
			vw.release_dock())
	_wire()


## Connects the vault window's signals once.
func _wire() -> void:
	if _wired:
		return
	var vw = app.get("vault_window")
	if vw == null:
		return
	_wired = true
	vw.dock_released.connect(func(): refresh_soon())
	vw.dock_saved.connect(func():
		if _held:
			refresh_soon())
	vw.visibility_changed.connect(func():
		if not vw.visible:
			refresh_soon())
	vw.dock_link_select = _on_link_select


## A refresh on the next idle frame, coalesced by the panel's own rebuild: the signals above can fire inside
## a handler that is about to free the node it came from.
func refresh_soon() -> void:
	if app.right_dock_ctrl != null:
		app.right_dock_ctrl.refresh_note.call_deferred()


## `VaultWindow.dock_link_select`: a link followed in the editor names place or faction `(kind, id)`. Selects
## it, and the panel follows, unless the panel is pinned (a pinned tab keeps its subject; the note then opens
## in the tab instead). Returns whether it selected.
func _on_link_select(kind: String, id: int, _path: String) -> bool:
	if app.right_dock_ctrl != null and app.right_dock_ctrl.is_pinned():
		return false
	app.select(kind, id)
	return true


## The note for `subject`, as a Dictionary with a `state`:
##   `none`         nothing is selected;
##   `unsupported`  the subject's kind has no note binding here (plus `why`);
##   `novault`      no vault folder is connected (plus `name`);
##   `no_note`      the subject has no linked note (plus `vk`, `vid`, `name`);
##   `note`         it has one (plus `rel`, `extra` = how many more links it has, `vk`, `vid`, `name`).
## Reads the link store and the bridge; writes nothing.
func resolve(subject: Dictionary) -> Dictionary:
	var kind := String(subject.get("kind", ""))
	if kind == "":
		return {"state": "none"}
	var nm := String(app.right_dock_ctrl.subject_name(subject)) if app.right_dock_ctrl != null else ""
	if not BINDABLE.has(kind):
		var why: String = {
			"culture": "A culture has no note binding on this screen yet.",
			"conflict": "A conflict has no note: a vault note binds to a place or a faction.",
		}.get(kind, "This kind of subject has no note binding.")
		return {"state": "unsupported", "why": why, "name": nm}
	var bridge = app.bridge
	var vid := int(subject.get("id", -1))
	if kind == "landmark":
		## The selection id is an index into the landmark list; the note binds to the vault entity id made
		## from the landmark's key. A landmark that is not (or is no longer) placed says so: no id is invented.
		var lms: Array = bridge.landmarks()
		var names: Array = bridge.landmark_names()
		if vid < 0 or vid >= lms.size() or vid >= names.size():
			return {"state": "none"}
		nm = String((names[vid] as Dictionary).get("name", nm))
		var lm: Dictionary = lms[vid]
		var got: Dictionary = bridge.vault_landmark_entity_id("%s@%d,%d" % [String(lm.get("kind", "")), int(lm.get("x", 0)), int(lm.get("y", 0))])
		if not bool(got.get("ok", false)) or not got.has("entity_id"):
			return {"state": "unsupported", "why": "This landmark cannot be bound to a note now: %s" % String(got.get("error", "no vault id")), "name": nm}
		vid = int(got["entity_id"])
	if kind == "label" or kind == "icon":
		var live: Dictionary = bridge.label_get(vid) if kind == "label" else bridge.icon_get(vid)
		if vid < 0 or live.is_empty():
			return {"state": "none"}
		var eid: Dictionary = bridge.vault_label_entity_id(vid) if kind == "label" else bridge.vault_icon_entity_id(vid)
		if not bool(eid.get("ok", false)) or not eid.has("entity_id"):
			return {"state": "unsupported", "why": "This %s cannot be bound to a note now: %s" % [kind,
				String(eid.get("error", "no vault id"))], "name": nm}
		vid = int(eid["entity_id"])
		nm = String(eid.get("label", nm))
	if kind == "settlement":
		var all: Array = bridge.settlements()
		if vid < 0 or vid >= all.size():
			return {"state": "none"}
		vid = int((all[vid] as Dictionary).get("tid", 0))
		if vid == 0:
			return {"state": "unsupported", "why": "This settlement has no stable id yet, so no note can be bound to it.", "name": nm}
	if not bool(bridge.vault_info().get("bound", false)):
		return {"state": "novault", "name": nm, "vk": String(BINDABLE[kind]), "vid": vid}
	var links: Array = bridge.vault_links_for(String(BINDABLE[kind]), vid)
	for l in links:
		var path := String((l as Dictionary).get("path", ""))
		if path != "":
			return {"state": "note", "rel": path, "extra": links.size() - 1, "vk": String(BINDABLE[kind]),
				"vid": vid, "name": nm}
	return {"state": "no_note", "vk": String(BINDABLE[kind]), "vid": vid, "name": nm}


## Redraws the tab for `subject` (the panel's subject: the pinned one, else what it shows). Docks the
## subject's note, or says why there is none. A no-op while the vault window is generating-gated. Never
## discards text: see the header.
func refresh(subject: Dictionary) -> void:
	if _slot == null or not is_instance_valid(_slot):
		return
	_wire()
	_subject = subject.duplicate()
	for c in _top.get_children():
		_top.remove_child(c)
		c.queue_free()
	for c in _bottom.get_children():
		_bottom.remove_child(c)
		c.queue_free()
	_rows.clear()
	_held = false
	var vw = app.vault_window
	var r := resolve(subject)
	if String(r["state"]) == "note":
		var res: Dictionary = vw.dock_note(String(r["rel"]), _slot)
		if bool(res.get("ok", false)):
			if int(r.get("extra", 0)) > 0:
				_say("%s has %d more linked note%s: Records ▸ Markdown vault lists them." % [String(r["name"]),
					int(r["extra"]), "" if int(r["extra"]) == 1 else "s"])
			_build_dated(r)
			return
		match String(res.get("why", "")):
			"window":
				_notice("NoteInWindow", "This note is open in the vault window, which holds its editor. Close the window to edit it here.")
			"held":
				_held = true
				_notice("NoteHeld", "The selection is now %s, but %s is not saved yet, so it stays here. Resolve the message above, or press Save now; this tab then shows %s." % [
					String(r["name"]), String(res.get("path", "")).get_file(), String(r["name"])])
			_:
				_notice("NoteUnreadable", "%s's linked note (%s) could not be read. The reason is on the status line." % [
					String(r["name"]), String(r["rel"])])
		return
	## Nothing to show in the editor: let a docked one go (saved first), or keep it held.
	if vw.is_docked() or vw.dock_dirty():
		if not vw.release_dock():
			_held = true
			_notice("NoteHeld", "The selection is now %s, but the note on screen (%s) is not saved yet, so it stays here. Resolve the message above." % [
				String(r.get("name", "something else")), vw.docked_path().get_file()])
			return
	_empty(r)


## The empty states: each says what is true and what to do, in a frame that holds no editor.
func _empty(r: Dictionary) -> void:
	match String(r["state"]):
		"none":
			_notice("NoteNothing", "Nothing is selected. Select a place, a faction, a landmark, a label or an icon to read and write its note here.")
		"unsupported":
			_notice("NoteUnsupported", String(r["why"]))
		"novault":
			_notice("NoteNoVault", "No vault folder is connected, so %s has no note to show. A vault is a folder of Markdown files this app reads and writes." % String(r["name"]))
			var b := DccWidgets.action(_top, "Connect a vault…", func(): app.open_vault_browse(), true)
			b.name = "NoteConnect"
		"no_note":
			_notice("NoteNone", "%s has no note yet." % String(r["name"]))
			_build_create(r)


## A notice line in the frame's top box, named for probes.
func _notice(node_name: String, text: String) -> Label:
	var l := DccWidgets.note(_top, text)
	l.name = node_name
	return l


## A quieter line (it does not replace the editor).
func _say(text: String) -> void:
	var l := DccTheme.label(text, "text_dim", DccTheme.FS_TINY)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.custom_minimum_size.x = 160
	l.name = "NoteExtra"
	_top.add_child(l)


## **Create *name*.md** (§3.4 "No note yet"): the template choice, then the note is created and bound to the
## subject, never silently. The template list is the vault's (`vault_templates`); a vault with none offers a
## blank note (a title line), which is the labelled judgement here: the proposal says "from the kind's
## template" and Cartalith ships none (Ruling BF). The bridge calls are the ones the vault window's own
## form makes (`vault_suggested_path`, `vault_create_from_template`, `vault_attach`).
func _build_create(r: Dictionary) -> void:
	var bridge = app.bridge
	var templates: Array = bridge.vault_templates()
	var labels: Array = ["(blank note)"]
	var rels: Array = [""]
	for t in templates:
		labels.append(String((t as Dictionary).get("label", "?")))
		rels.append(String((t as Dictionary).get("rel", "")))
	var pick := {"i": 0}
	var ch := DccWidgets.choice(_top, "Template", labels, 0, func(i: int): pick["i"] = i,
		"The vault's own templates. Cartalith ships none; a blank note holds the title only.")
	ch.name = "NoteTemplate"
	var suggested: String = bridge.vault_suggested_path(String(r["vk"]), String(r["name"]))
	var edit := LineEdit.new()
	edit.name = "NotePath"
	edit.text = suggested
	edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	edit.tooltip_text = "Where the new note goes, relative to the vault folder."
	_top.add_child(edit)
	var b := DccWidgets.action(_top, "Create %s" % suggested.get_file(), func(): _create(r, rels[int(pick["i"])], edit.text.strip_edges()), true)
	b.name = "NoteCreate"
	b.tooltip_text = "Creates the note from the chosen template and links it to %s. It never overwrites an existing file." % String(r["name"])


## Creates the note and binds it, then redraws (the tab shows it). A refusal says why and changes nothing.
func _create(r: Dictionary, template_rel: String, rel: String) -> void:
	if rel == "":
		app.set_status("hint", "Give the new note a path.", "accent")
		return
	var bridge = app.bridge
	var made: Dictionary
	if template_rel == "":
		made = bridge.vault_create_file(rel, "# %s\n" % String(r["name"]))
	else:
		made = bridge.vault_create_from_template(template_rel, rel, String(r["name"]))
	if not bool(made.get("ok", false)):
		app.set_status("hint", "Create: %s" % String(made.get("error", "refused")), "accent")
		return
	var a: Dictionary = bridge.vault_attach(String(r["vk"]), int(r["vid"]), String(r["name"]), rel, "")
	if bool(a.get("ok", false)):
		app.vault_window.store_changed.emit()
		app.set_status("hint", "%s created and linked to %s." % [rel, String(r["name"])], "text")
	else:
		app.set_status("hint", "%s created, but could not be linked: %s" % [rel, String(a.get("error", ""))], "accent")
	refresh(_subject)


## The note's dated entries (authored ` ```chronos ` lines, read through `vault_entity_chronos`): a row per
## event, a press moves the year (`DccApp.tl_set_year`, the one writer), and the row the year falls in is
## lit. Below the editor so the note's own text stays the thing on screen. Absent when the note has none.
func _build_dated(r: Dictionary) -> void:
	var got: Dictionary = app.bridge.vault_entity_chronos(String(r["vk"]), int(r["vid"]))
	if not bool(got.get("ok", false)):
		return
	var events: Array = got.get("events", [])
	if events.is_empty():
		return
	var head := DccTheme.label("DATED ENTRIES · press a row to move the year", "text_dim", DccTheme.FS_TINY)
	head.name = "NoteDatedHead"
	_bottom.add_child(head)
	var cap := 12
	for i in mini(events.size(), cap):
		var e: Dictionary = events[i]
		var start := int(e.get("start", 0))
		var yr := str(start)
		if e.has("end"):
			yr += " – %d" % int(e["end"])
		var label := "%s   %s" % [yr, String(e.get("name", ""))]
		var b := DccWidgets.chip(_bottom, label, func(): app.tl_set_year(start, true), false, 7, 2)
		b.name = "NoteDated_%d" % i
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		b.tooltip_text = String(e.get("description", "Moves the timeline to this year."))
		_rows.append({"button": b, "start": start, "end": int(e["end"]) if e.has("end") else start, "ranged": e.has("end")})
	if events.size() > cap:
		_bottom.add_child(DccTheme.label("+%d more in the note" % (events.size() - cap), "text_ghost", DccTheme.FS_TINY))
	on_timeline()


## The year moved: re-light the row it falls in. A ranged entry is lit while the year is inside it; of the
## single-year entries, the latest one at or before the year is.
func on_timeline() -> void:
	if _rows.is_empty():
		return
	var y: int = app.tl_year()
	var latest_point := -1
	for i in _rows.size():
		var row: Dictionary = _rows[i]
		if not bool(row["ranged"]) and int(row["start"]) <= y:
			if latest_point < 0 or int(row["start"]) >= int((_rows[latest_point] as Dictionary)["start"]):
				latest_point = i
	for i in _rows.size():
		var row: Dictionary = _rows[i]
		var b: Button = row["button"]
		if not is_instance_valid(b):
			continue
		var lit: bool = (int(row["start"]) <= y and y <= int(row["end"])) if bool(row["ranged"]) else i == latest_point
		b.set_meta("lit", lit)
		b.add_theme_color_override("font_color", DccTheme.c("accent") if lit else DccTheme.c("text"))
