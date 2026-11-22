extends AcceptDialog
## "Timelines found in your vault" (`CHRONOS_TIMELINE_DESIGN.md` s.4.3, s.6; CT-4, CT-5, CT-7). Opened only by
## pressing the passive badge on the strip (`chronos_items.gd`); never automatically (Ruling GE).
##
## One card per timeline: the note, the form that found it, the event count and year range, a three-line
## preview, and the four choices Link / Import / Keep / Ignore. **Defaults, and why** (design s.4.3): a note
## already linked to a map item defaults to Link (no action: the strip already shows it live); an unlinked or
## private note defaults to Keep, the one choice that changes nothing and loses nothing. There is no bulk
## Import: "Apply Keep to all" and "Ignore all" are the only bulk actions.
##
## **It never edits a note.** Every action is a call on `WorldGen`: `timeline_decide` (the decision row and,
## for Import, the imported events), `vault_attach` (the existing knowledge-link flow) and
## `timeline_decide_undo`. The window does not block the map (`exclusive = false`).

var _shell: Node = null
var _cands: Array = []
var _on_change: Callable = Callable()
var _choice: Dictionary = {}
var _entity_pick: Dictionary = {}
var _offset_pick: Dictionary = {}
var _status: Dictionary = {}
var _undo: Dictionary = {}
var _cards: VBoxContainer = null
## Per card key: the last re-sync's undo token, and the founding year a "set" replaced (`{index, was}`).
var _resync_undo: Dictionary = {}
var _founded_undo: Dictionary = {}

## The tap-target height in physical px: 44 dp on the phone (`DccTheme.PHONE_TAP_MIN` scaled by the phone's
## unit, as `DccShell._ptap` does), the shell's own button role elsewhere. Shared with `chronos_items.gd`
## so a chip on the strip and a chip in the prompt cannot disagree (the phone's chips were `btn_min_h`, which
## is below 44 dp there).
static func tap_h() -> int:
	if DccTheme.is_phone():
		return ceili(DccTheme.PHONE_TAP_MIN * DccTheme.phone_scale())
	return DccTheme.role_px("btn_min_h")

func setup(shell: Node, cands: Array, on_change: Callable) -> void:
	_shell = shell
	_cands = cands.duplicate()
	_on_change = on_change
	title = "Timelines found in your vault"
	exclusive = false
	max_size = Vector2i(760, 620)
	name = "ChronosPrompt"
	ok_button_text = "Done"
	# AcceptDialog resets the minimum size of every Button in its own button row when its theme applies
	# (see `context_window.gd`, "The holder is not a Button"), so its Done could never reach 44 dp: the
	# stock button is hidden and a Done of our own sits in the content, where its height holds.
	get_ok_button().hide()
	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 8)
	root.custom_minimum_size = Vector2(0, 0)
	add_child(root)
	var note := DccTheme.mono_label("Nothing has been changed. Keep leaves the note and the map alone; Link binds the note to a map item; Import copies the events onto the timeline (the note is never edited).", "text_ghost", DccTheme.FS_MICRO, 0)
	note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	root.add_child(note)
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	# The body must leave room for the title bar, the note, the bulk row and the Done button inside the
	# window, or Done lands below the fold (it did at 450x975 and at 1280x800): the scroll's own minimum is
	# a share of the viewport, never a fixed 340.
	scroll.custom_minimum_size = Vector2(0, mini(340, int(_view_h() * 0.35)))
	root.add_child(scroll)
	_cards = VBoxContainer.new()
	_cards.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_cards.add_theme_constant_override("separation", 10)
	scroll.add_child(_cards)
	var bulk := HFlowContainer.new()
	bulk.add_theme_constant_override("separation", 8)
	root.add_child(bulk)
	var keep_all := DccWidgets.segment(bulk, "Apply Keep to all", func(): _bulk("keep"))
	keep_all.name = "KeepAll"
	var ignore_all := DccWidgets.segment(bulk, "Ignore all", func(): _bulk("ignore"))
	ignore_all.name = "IgnoreAll"
	var done := DccWidgets.chip(bulk, "Done", func(): hide(), true)
	done.name = "Done"
	done.size_flags_horizontal = Control.SIZE_SHRINK_END
	for b in [keep_all, ignore_all, done]:
		(b as Button).custom_minimum_size.y = maxf((b as Button).custom_minimum_size.y, float(tap_h()))
	_build_cards()

func _view_h() -> float:
	return float(_shell.get_viewport_rect().size.y) if _shell != null else 600.0

func _key(c: Dictionary) -> String:
	return "%s|%s|%d" % [String(c["rel"]), String(c["form"]), int(c["ordinal"])]

func _default_choice(c: Dictionary) -> String:
	if bool(c.get("linked", false)) and not c.has("private"):
		return "link"
	return "keep"

func _bridge() -> Object:
	if _shell == null or not _shell.has_method("_find_engine_bridge"):
		return null
	return _shell._find_engine_bridge()

func _build_cards() -> void:
	for ch in _cards.get_children():
		_cards.remove_child(ch)
		ch.queue_free()
	if _cands.is_empty():
		_cards.add_child(DccTheme.mono_label("No timelines found.", "text_ghost", DccTheme.FS_SMALL, 0))
		return
	for cd in _cands:
		_card(cd)

func _card(c: Dictionary) -> void:
	var key := _key(c)
	if not _choice.has(key):
		_choice[key] = _default_choice(c)
	var box := VBoxContainer.new()
	box.name = "Card_" + key.md5_text().substr(0, 8)
	box.add_theme_constant_override("separation", 4)
	_cards.add_child(box)
	var range_text := ""
	if c.has("year_min"):
		range_text = "  %d to %d" % [int(c["year_min"]), int(c["year_max"])]
	var head := "%s  ·  %s  ·  %d event%s%s" % [String(c["rel"]), String(c["form_label"]), int(c["count"]), "" if int(c["count"]) == 1 else "s", range_text]
	if bool(c.get("linked", false)):
		head += "  [linked]"
	if c.has("private"):
		head += "  [private: %s]" % String(c["private"])
	var h := DccTheme.mono_label(head, "text", DccTheme.FS_SMALL, 0)
	h.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(h)
	for line in (c.get("preview", PackedStringArray()) as PackedStringArray):
		box.add_child(DccTheme.mono_label("    " + line, "text_ghost", DccTheme.FS_MICRO, 0))
	var st := String(c.get("standing", "undecided"))
	var st_text := "undecided"
	if st == "decided":
		st_text = "decided: %s" % String(c.get("previous", ""))
	elif st == "changed":
		st_text = "changed since you decided (was %s)" % String(c.get("previous", ""))
	var stl := DccTheme.mono_label(st_text, "accent" if st != "undecided" else "text_ghost", DccTheme.FS_MICRO, 0)
	stl.name = "Standing"
	_status[key] = stl
	box.add_child(stl)
	if c.has("private") and String(_choice[key]) in ["import", "link"]:
		pass
	var row := HFlowContainer.new()
	row.add_theme_constant_override("h_separation", 6)
	box.add_child(row)
	var chips := {}
	for word in ["link", "import", "keep", "ignore"]:
		var w: String = word
		var chip := DccWidgets.segment(row, w.capitalize(), func(): pass)
		chip.name = "Choice_" + w
		chips[w] = chip
		DccWidgets.set_segment_on(chip, String(_choice[key]) == w)
		chip.custom_minimum_size.y = maxf(chip.custom_minimum_size.y, float(tap_h()))
	# the entity + offset row, shown for Link (when unlinked) and Import
	var pick := HFlowContainer.new()
	pick.add_theme_constant_override("h_separation", 6)
	box.add_child(pick)
	var opt := OptionButton.new()
	opt.name = "Entity"
	opt.custom_minimum_size.y = float(tap_h())
	_fill_entities(opt)
	pick.add_child(opt)
	_entity_pick[key] = opt
	var off := SpinBox.new()
	off.name = "EpochOffset"
	off.min_value = -100000
	off.max_value = 100000
	off.step = 1
	off.value = 0
	off.prefix = "epoch offset "
	off.tooltip_text = "Added to the note's years. 0 means the note's year is the engine's year (year 0 is the generated world)."
	pick.add_child(off)
	_offset_pick[key] = off
	var warn := DccTheme.mono_label("", "accent", DccTheme.FS_MICRO, 0)
	warn.name = "Warning"
	warn.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(warn)
	var apply := DccWidgets.chip(row, "Apply", func(): pass, true)
	apply.name = "Apply"
	apply.custom_minimum_size.y = maxf(apply.custom_minimum_size.y, float(tap_h()))
	var undo := DccWidgets.segment(row, "Undo", func(): pass)
	undo.name = "Undo"
	undo.visible = false
	var sync_pick := func():
		var w := String(_choice[key])
		pick.visible = w == "import" or (w == "link" and not bool(c.get("linked", false)))
		off.visible = w == "import"
		warn.text = ""
		if c.has("private") and (w == "import" or w == "link"):
			warn.text = "This note is private (%s); its events will be saved in the project file, flagged private, and left out of the assistant and of exports." % String(c["private"])
		for ww in chips:
			DccWidgets.set_segment_on(chips[ww], ww == w)
	for word in chips:
		var w2: String = word
		(chips[w2] as Button).pressed.connect(func():
			_choice[key] = w2
			sync_pick.call())
	apply.pressed.connect(func(): _apply(c, key, stl, undo, warn))
	undo.pressed.connect(func(): _do_undo(key, stl, undo))
	sync_pick.call()
	_extras(box, c, key, stl)
	box.add_child(HSeparator.new())

## The entity the card's picker names now: `{kind, id}`, or `{}` with none.
func _picked(key: String) -> Dictionary:
	var opt: OptionButton = _entity_pick.get(key)
	if opt == null or opt.item_count == 0 or opt.selected < 0:
		return {}
	return opt.get_item_metadata(opt.selected)

## CT-7's two actions beside the four choices (design s.4.5): **Re-sync** for a timeline already imported
## (added / removed / changed counts, each group taken or left, one undo) and the **founding comparison**
## for a settlement (the note's founding events against the settlement's own year; a conflict shows both
## years and offers "Set the settlement's founding year", never applied silently). Neither edits a note.
func _extras(box: VBoxContainer, c: Dictionary, key: String, stl: Label) -> void:
	var row := HFlowContainer.new()
	row.name = "Extras"
	row.add_theme_constant_override("h_separation", 6)
	box.add_child(row)
	var out := DccTheme.mono_label("", "text_ghost", DccTheme.FS_MICRO, 0)
	out.name = "ExtrasText"
	out.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(out)
	var acts := HFlowContainer.new()
	acts.name = "ExtrasActions"
	acts.add_theme_constant_override("h_separation", 6)
	box.add_child(acts)
	var resync := DccWidgets.segment(row, "Re-sync", func(): pass)
	resync.name = "Resync"
	resync.tooltip_text = "Compare the note's lines with the events imported from it: what was added, removed or changed. Nothing is applied until you choose."
	var found := DccWidgets.segment(row, "Compare founding", func(): pass)
	found.name = "FoundCheck"
	found.tooltip_text = "Compare the note's founding events with the chosen settlement's own founding year."
	for bt in [resync, found]:
		(bt as Button).custom_minimum_size.y = maxf((bt as Button).custom_minimum_size.y, float(tap_h()))
	resync.pressed.connect(func(): _show_resync(c, key, out, acts))
	found.pressed.connect(func(): _show_founding(c, key, out, acts))

func _clear(n: Node) -> void:
	for ch in n.get_children():
		n.remove_child(ch)
		ch.queue_free()

func _show_resync(c: Dictionary, key: String, out: Label, acts: Container) -> void:
	_clear(acts)
	var b := _bridge()
	var pk := _picked(key)
	if b == null or pk.is_empty():
		out.text = "no map item to re-sync onto"
		return
	var plan: Dictionary = b.world_gen.timeline_resync_plan(String(c["rel"]), String(c["form"]), int(c["ordinal"]), String(pk["kind"]), int(pk["id"]))
	if not bool(plan.get("ok", false)):
		out.text = "re-sync unavailable: %s" % String(plan.get("error", ""))
		return
	var n_add := int(plan["added"])
	var n_rem := int(plan["removed"])
	var n_chg := int(plan["changed"])
	out.text = "Re-sync: %d added, %d removed (kept, marked source removed), %d changed since import." % [n_add, n_rem, n_chg]
	if n_add + n_rem + n_chg == 0:
		return
	var takes := {}
	for g in [["added", n_add], ["changed", n_chg], ["removed", n_rem]]:
		var cb := CheckBox.new()
		cb.name = "Take_" + String(g[0])
		cb.text = "take %s (%d)" % [g[0], g[1]]
		cb.button_pressed = int(g[1]) > 0
		cb.disabled = int(g[1]) == 0
		cb.custom_minimum_size.y = float(tap_h())
		acts.add_child(cb)
		takes[g[0]] = cb
	var apply := DccWidgets.chip(acts, "Apply re-sync", func(): pass, true)
	apply.name = "ApplyResync"
	apply.custom_minimum_size.y = maxf(apply.custom_minimum_size.y, float(tap_h()))
	apply.pressed.connect(func():
		var got: Dictionary = b.world_gen.timeline_resync_apply(String(c["rel"]), String(c["form"]), int(c["ordinal"]), String(pk["kind"]), int(pk["id"]),
			(takes["added"] as CheckBox).button_pressed, (takes["changed"] as CheckBox).button_pressed, (takes["removed"] as CheckBox).button_pressed)
		if not bool(got.get("ok", false)):
			out.text = "not applied: %s" % String(got.get("error", ""))
			return
		_resync_undo[key] = String(got["undo"])
		out.text = "Re-synced (one undo step)."
		_clear(acts)
		var undo := DccWidgets.segment(acts, "Undo re-sync", func(): pass)
		undo.name = "UndoResync"
		undo.custom_minimum_size.y = maxf(undo.custom_minimum_size.y, float(tap_h()))
		undo.pressed.connect(func():
			var u: Dictionary = b.world_gen.timeline_resync_undo(String(_resync_undo.get(key, "")))
			out.text = "Re-sync undone." if bool(u.get("ok", false)) else "could not undo: %s" % String(u.get("error", ""))
			_clear(acts)
			if _on_change.is_valid():
				_on_change.call())
		if _on_change.is_valid():
			_on_change.call())

func _show_founding(c: Dictionary, key: String, out: Label, acts: Container) -> void:
	_clear(acts)
	var b := _bridge()
	var pk := _picked(key)
	if b == null or pk.is_empty() or String(pk["kind"]) != "settlement":
		out.text = "Founding comparison needs a settlement chosen in the list above."
		return
	var tid := int(pk["id"])
	var rows: Array = b.world_gen.timeline_founding_check(String(c["rel"]), String(c["form"]), int(c["ordinal"]), tid)
	var lines: Array = []
	var conflict := {}
	for r in rows:
		var d: Dictionary = r
		match String(d["verdict"]):
			"agrees":
				lines.append("%s: %d agrees with the settlement's founding year." % [String(d["title"]), int(d["note_year"])])
			"conflict":
				lines.append("%s: the note says %d, the settlement was founded %s." % [String(d["title"]), int(d["note_year"]), str(int(d["founded"])) if d.has("founded") else "(no year recorded)"])
				if conflict.is_empty():
					conflict = d
	if lines.is_empty():
		out.text = "No founding event in this note to compare (compared at the note's own years)."
		return
	out.text = "\n".join(lines)
	if conflict.is_empty():
		return
	var index := -1
	var all: Array = b.settlements()
	for i in all.size():
		if int((all[i] as Dictionary).get("tid", -1)) == tid:
			index = i
	if index < 0:
		return
	var y := int(conflict["note_year"])
	var set_btn := DccWidgets.chip(acts, "Set the settlement's founding year to %d" % y, func(): pass, true)
	set_btn.name = "SetFounded"
	set_btn.custom_minimum_size.y = maxf(set_btn.custom_minimum_size.y, float(tap_h()))
	set_btn.pressed.connect(func():
		var was: Dictionary = all[index]
		if not b.civ_edit_settlement(index, {"founded": y}):
			out.text = "could not set the founding year"
			return
		_founded_undo[key] = {"index": index, "was": int(was["founded"]) if was.has("founded") else null}
		out.text = "The settlement's founding year is now %d." % y
		_clear(acts)
		var undo := DccWidgets.segment(acts, "Undo founding year", func(): pass)
		undo.name = "UndoFounded"
		undo.custom_minimum_size.y = maxf(undo.custom_minimum_size.y, float(tap_h()))
		undo.pressed.connect(func():
			var u: Dictionary = _founded_undo.get(key, {})
			if not u.is_empty():
				b.civ_edit_settlement(int(u["index"]), {"founded": int(u["was"])} if u["was"] != null else {"founded_clear": true})
			out.text = "Founding year restored."
			_clear(acts)))

func _fill_entities(opt: OptionButton) -> void:
	var b := _bridge()
	if b == null:
		return
	var sel: Variant = _shell.get("selection")
	var want := -1
	var i := 0
	for s in b.settlements():
		var sd: Dictionary = s
		opt.add_item("settlement · %s" % String(sd.get("name", "")), i)
		opt.set_item_metadata(opt.item_count - 1, {"kind": "settlement", "id": int(sd.get("tid", 0)), "label": String(sd.get("name", ""))})
		if sel is Dictionary and String((sel as Dictionary).get("kind", "")) == "settlement" and int((sel as Dictionary).get("id", -2)) == i:
			want = opt.item_count - 1
		i += 1
	for fa in b.get_factions():
		var fd: Dictionary = fa
		opt.add_item("faction · %s" % String(fd.get("name", "")), 1000 + int(fd.get("id", 0)))
		opt.set_item_metadata(opt.item_count - 1, {"kind": "faction", "id": int(fd.get("id", 0)), "label": String(fd.get("name", ""))})
	if want >= 0:
		opt.select(want)
	elif opt.item_count > 0:
		opt.select(0)

func _apply(c: Dictionary, key: String, stl: Label, undo: Button, warn: Label) -> void:
	var b := _bridge()
	if b == null:
		return
	var word := String(_choice[key])
	var kind := ""
	var id := 0
	if word == "link" or word == "import":
		var linked_id := _existing_link(b, String(c["rel"]))
		if word == "link" and bool(c.get("linked", false)) and not linked_id.is_empty():
			kind = String(linked_id["entity_kind"])
			id = int(linked_id["entity_id"])
		else:
			var opt: OptionButton = _entity_pick[key]
			if opt.item_count == 0:
				stl.text = "no map item to bind to"
				return
			var md: Dictionary = opt.get_item_metadata(opt.selected)
			kind = String(md["kind"])
			id = int(md["id"])
			if word == "link" and linked_id.is_empty():
				var att: Dictionary = b.vault_attach(kind, id, String(md["label"]), String(c["rel"]), "")
				if not bool(att.get("ok", false)):
					stl.text = "could not link: %s" % String(att.get("error", ""))
					return
	var off: SpinBox = _offset_pick[key]
	var got: Dictionary = b.world_gen.timeline_decide(String(c["rel"]), String(c["form"]), int(c["ordinal"]), word, kind, id, int(off.value))
	if not bool(got.get("ok", false)):
		stl.text = "not applied: %s" % String(got.get("error", ""))
		return
	var msg := "decided: %s" % word
	if word == "import":
		msg += " (added %d, already on the timeline %d)" % [(got["added"] as PackedInt64Array).size(), int(got["duplicates"])]
		_undo[key] = String(got["undo"])
		undo.visible = true
	else:
		_undo[key] = String(got["undo"])
		undo.visible = true
	stl.text = msg
	if got.has("private_warning"):
		warn.text = String(got["private_warning"])
	if _on_change.is_valid():
		_on_change.call()

func _existing_link(b: Object, rel: String) -> Dictionary:
	for l in b.vault_all_links():
		if String((l as Dictionary).get("path", "")) == rel:
			return l
	return {}

func _do_undo(key: String, stl: Label, undo: Button) -> void:
	var b := _bridge()
	if b == null or not _undo.has(key):
		return
	var got: Dictionary = b.world_gen.timeline_decide_undo(String(_undo[key]))
	if bool(got.get("ok", false)):
		stl.text = "undone"
		undo.visible = false
		_undo.erase(key)
		if _on_change.is_valid():
			_on_change.call()

func _bulk(word: String) -> void:
	var b := _bridge()
	if b == null:
		return
	var n := 0
	for c in _cands:
		var cd: Dictionary = c
		if String(cd.get("standing", "undecided")) == "decided":
			continue
		var got: Dictionary = b.world_gen.timeline_decide(String(cd["rel"]), String(cd["form"]), int(cd["ordinal"]), word, "", 0, 0)
		if bool(got.get("ok", false)):
			n += 1
			_status[_key(cd)].text = "decided: %s" % word
	if _on_change.is_valid():
		_on_change.call()
	title = "Timelines found in your vault - %s applied to %d" % [word, n]
