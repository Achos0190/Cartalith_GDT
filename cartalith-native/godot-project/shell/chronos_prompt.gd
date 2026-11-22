extends AcceptDialog
## "Timelines in your vault", the quiet form (Ruling GT, design option C "The prompt in this option"; `CHRONOS_TIMELINE_DESIGN.md`
## s.4.3, s.6; CT-4, CT-5, CT-7). Opened only by pressing the passive badge on the strip (`chronos_items.gd`); never automatically
## (Ruling GE).
##
## **One line per timeline**: a status dot, the note's name (no path), the event count and the span or a plain verdict. One line
## opens at a time and shows **a preview on the shared axis**: hollow dashed marks where its events would land, over the filled
## marks already on the timeline, so duplicates and gaps are seen at a glance. **One segmented choice, applied on click** with an
## Undo line: Keep and Ignore apply at once; Link applies at once for a note already linked to a map item and otherwise (like
## Import) opens its entity picker first, which is why those two are marked "...". The epoch offset, Re-sync and Compare founding
## sit behind "More". The footer keeps one bulk action, "Leave the rest as they are" (Keep for every undecided timeline; there is
## no bulk Import, design s.4.3).
##
## **Defaults** (design s.4.3): a note already linked to a map item reads "already live on the timeline" and defaults to Link
## (no action: the strip already shows it live); an unlinked or private note defaults to Keep, the one choice that changes
## nothing and loses nothing.
##
## **It never edits a note.** Every action is a call on `WorldGen`: `timeline_decide` (the decision row and, for Import, the
## imported events), `vault_attach` (the existing knowledge-link flow) and `timeline_decide_undo`. The window does not block the
## map (`exclusive = false`).

var _shell: Node = null
var _cands: Array = []
var _on_change: Callable = Callable()
var _existing: Array = []
var _choice: Dictionary = {}
var _entity_pick: Dictionary = {}
var _offset_pick: Dictionary = {}
var _status: Dictionary = {}
var _undo: Dictionary = {}
var _cards: VBoxContainer = null
var _open_key := ""
var _summary: Label = null
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

## `existing` is the strip's events (for the preview's filled marks).
func setup(shell: Node, cands: Array, on_change: Callable, existing: Array = []) -> void:
	_shell = shell
	_cands = cands.duplicate()
	_on_change = on_change
	_existing = existing
	title = "Timelines in your vault"
	exclusive = false
	max_size = Vector2i(760, 620)
	name = "ChronosPrompt"
	ok_button_text = "Done"
	# AcceptDialog resets the minimum size of every Button in its own button row when its theme applies
	# (see `context_window.gd`, "The holder is not a Button"), so its Done could never reach 44 dp: the
	# stock button is hidden and a Done of our own sits in the content, where its height holds.
	get_ok_button().hide()
	var pad := MarginContainer.new()
	for side in ["left", "right", "top", "bottom"]:
		pad.add_theme_constant_override("margin_" + side, 8)
	add_child(pad)
	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 8)
	pad.add_child(root)
	_summary = DccTheme.mono_label("", "text", DccTheme.FS_SMALL, 0)
	_summary.name = "PromptSummary"
	root.add_child(_summary)
	var note := DccTheme.mono_label("nothing changes until you choose", "text_ghost", DccTheme.FS_MICRO, 0)
	root.add_child(note)
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	# The body must leave room for the title bar, the summary, the bulk row and the Done button inside the
	# window, or Done lands below the fold (it did at 450x975 and at 1280x800): the scroll's own minimum is
	# a share of the viewport, never a fixed 340.
	scroll.custom_minimum_size = Vector2(0, mini(340, int(_view_h() * 0.35)))
	root.add_child(scroll)
	_cards = VBoxContainer.new()
	_cards.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_cards.add_theme_constant_override("separation", 6)
	scroll.add_child(_cards)
	var bulk := HFlowContainer.new()
	bulk.add_theme_constant_override("separation", 8)
	root.add_child(bulk)
	var keep_all := DccWidgets.segment(bulk, "Leave the rest as they are", func(): _bulk("keep"))
	keep_all.name = "KeepAll"
	keep_all.tooltip_text = "Keep every timeline you have not decided: the notes and the map stay as they are."
	var done := DccWidgets.chip(bulk, "Done", func(): hide(), true)
	done.name = "Done"
	done.size_flags_horizontal = Control.SIZE_SHRINK_END
	for b in [keep_all, done]:
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
	_summary.text = "%d timeline%s in your vault" % [_cands.size(), "" if _cands.size() == 1 else "s"]
	if _cands.is_empty():
		_cards.add_child(DccTheme.mono_label("No timelines found.", "text_ghost", DccTheme.FS_SMALL, 0))
		return
	for cd in _cands:
		_card(cd)

## The note's name for a line: the file stem, no path (design: "a status dot, the note name (no path)").
static func note_name(c: Dictionary) -> String:
	return String(c["rel"]).get_file().get_basename()

## The one-line verdict after the count: what the timeline already is, or its span.
func _verdict(c: Dictionary) -> String:
	var st := String(c.get("standing", "undecided"))
	if st == "decided":
		return "decided: %s" % String(c.get("previous", ""))
	if st == "changed":
		return "changed since you decided (was %s)" % String(c.get("previous", ""))
	if bool(c.get("linked", false)):
		return "already live on the timeline"
	if c.has("year_min"):
		return "%d–%d" % [int(c["year_min"]), int(c["year_max"])]
	return ""

func _card(c: Dictionary) -> void:
	var key := _key(c)
	if not _choice.has(key):
		_choice[key] = _default_choice(c)
	var box := VBoxContainer.new()
	box.name = "Card_" + key.md5_text().substr(0, 8)
	box.add_theme_constant_override("separation", 4)
	_cards.add_child(box)
	var st := String(c.get("standing", "undecided"))
	var n := int(c["count"])
	var line := Button.new()
	line.name = "Line"
	line.flat = true
	line.alignment = HORIZONTAL_ALIGNMENT_LEFT
	line.focus_mode = Control.FOCUS_NONE
	line.custom_minimum_size.y = float(tap_h())
	# a line is a sentence, so it wraps rather than pushing the dialog wider than a phone (measured at 450 px: "already live on" was cut)
	line.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	line.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	line.add_theme_font_override("font", DccTheme.mono())
	line.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	var dot := "●" if st != "decided" else "○"
	var priv := "  private" if c.has("private") else ""
	var open_now := _open_key == key
	line.text = "%s  %s %s%s  ·  %d event%s  ·  %s" % ["▾" if open_now else "▸", dot, note_name(c), priv, n, "" if n == 1 else "s", _verdict(c)]
	line.tooltip_text = "%s (%s). Press to see where its events would land." % [String(c["rel"]), String(c["form_label"])]
	box.add_child(line)
	var detail := VBoxContainer.new()
	detail.name = "Detail"
	detail.add_theme_constant_override("separation", 6)
	detail.visible = open_now
	box.add_child(detail)
	line.pressed.connect(func():
		_open_key = "" if _open_key == key else key
		_build_cards())
	_status[key] = _detail(detail, c, key)
	box.add_child(HSeparator.new())

## The detail of the open line: the preview axis, the segmented choice with its picker, the verdict sentence, the undo line
## and "More".
func _detail(box: VBoxContainer, c: Dictionary, key: String) -> Label:
	var pv := Preview.new()
	pv.name = "Preview"
	pv.setup(_existing, c.get("events", []))
	box.add_child(pv)
	var verdict := DccTheme.mono_label("", "text_ghost", DccTheme.FS_MICRO, 0)
	verdict.name = "Verdict"
	verdict.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(verdict)
	var row := HFlowContainer.new()
	row.name = "Choices"
	row.add_theme_constant_override("h_separation", 6)
	box.add_child(row)
	var labels := {"link": "Link to…", "import": "Import…", "keep": "Keep in note", "ignore": "Ignore"}
	if bool(c.get("linked", false)):
		labels["link"] = "Show live"
	var chips := {}
	for word in ["link", "import", "keep", "ignore"]:
		var w: String = word
		var chip := DccWidgets.segment(row, String(labels[w]), func(): pass)
		chip.name = "Choice_" + w
		chips[w] = chip
		DccWidgets.set_segment_on(chip, String(_choice[key]) == w and String(c.get("standing", "undecided")) == "decided")
		chip.custom_minimum_size.y = maxf(chip.custom_minimum_size.y, float(tap_h()))
	# the entity picker: only when Link (an unlinked note) or Import is chosen
	var pick := HFlowContainer.new()
	pick.name = "Picker"
	pick.add_theme_constant_override("h_separation", 6)
	pick.visible = false
	box.add_child(pick)
	var opt := OptionButton.new()
	opt.name = "Entity"
	opt.custom_minimum_size.y = float(tap_h())
	_fill_entities(opt, c)
	pick.add_child(opt)
	_entity_pick[key] = opt
	var go := DccWidgets.chip(pick, "Apply", func(): pass, true)
	go.name = "Apply"
	go.custom_minimum_size.y = maxf(go.custom_minimum_size.y, float(tap_h()))
	var stl := DccTheme.mono_label("", "accent", DccTheme.FS_MICRO, 0)
	stl.name = "Standing"
	stl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(stl)
	var warn := DccTheme.mono_label("", "accent", DccTheme.FS_MICRO, 0)
	warn.name = "Warning"
	warn.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(warn)
	var undo := DccWidgets.segment(box, "Undo", func(): pass)
	undo.name = "Undo"
	undo.visible = false
	undo.custom_minimum_size.y = maxf(undo.custom_minimum_size.y, float(tap_h()))
	undo.pressed.connect(func(): _do_undo(key, stl, undo))
	var st := String(c.get("standing", "undecided"))
	if st == "decided":
		stl.text = "decided: %s" % String(c.get("previous", ""))
	elif st == "changed":
		stl.text = "changed since you decided (was %s)" % String(c.get("previous", ""))
	_offset_pick[key] = _more(box, c, key, stl)
	go.pressed.connect(func(): _apply(c, key, stl, undo, warn))
	for word in chips:
		var w2: String = word
		(chips[w2] as Button).pressed.connect(func():
			_choice[key] = w2
			stl.text = ""
			for ww in chips:
				DccWidgets.set_segment_on(chips[ww], ww == w2)
			warn.text = ""
			if c.has("private") and (w2 == "import" or w2 == "link"):
				warn.text = "This note is private (%s); its events will be saved in the project file, flagged private, and left out of the assistant and of exports." % String(c["private"])
			match w2:
				"keep", "ignore":
					pick.visible = false
					_apply(c, key, stl, undo, warn)
				"link":
					if bool(c.get("linked", false)):
						pick.visible = false
						_apply(c, key, stl, undo, warn)
					else:
						pick.visible = true
						go.text = "Link"
				"import":
					pick.visible = true
					go.text = "Import"
			verdict.text = _verdict_sentence(c, key))
	verdict.text = _verdict_sentence(c, key)
	return stl

## One sentence for what the timeline would do to the map, from the founding comparison when the note is linked to a
## settlement (design: "1 event agrees with a founding year", never "0 duplicates" before an import could know it).
func _verdict_sentence(c: Dictionary, key: String) -> String:
	var b := _bridge()
	var pk := _picked(key)
	if b == null or pk.is_empty() or String(pk["kind"]) != "settlement":
		return ""
	var rows: Array = b.world_gen.timeline_founding_check(String(c["rel"]), String(c["form"]), int(c["ordinal"]), int(pk["id"]))
	var agree := 0
	var conflict := 0
	for r in rows:
		match String((r as Dictionary)["verdict"]):
			"agrees":
				agree += 1
			"conflict":
				conflict += 1
	if agree + conflict == 0:
		return ""
	return "%d event%s agree%s with a founding year · %d disagree%s" % [agree, "" if agree == 1 else "s", "s" if agree == 1 else "", conflict, "s" if conflict == 1 else ""]

## "More": the epoch offset, Re-sync and Compare founding (design: behind one disclosure). Returns the epoch SpinBox, which
## `_apply` reads for an Import. Neither action edits a note.
func _more(box: VBoxContainer, c: Dictionary, key: String, stl: Label) -> SpinBox:
	var more := VBoxContainer.new()
	more.name = "MoreBox"
	more.visible = false
	more.add_theme_constant_override("separation", 4)
	var tog := DccWidgets.segment(box, "More ▸", func(): pass)
	tog.name = "More"
	tog.custom_minimum_size.y = maxf(tog.custom_minimum_size.y, float(tap_h()))
	tog.pressed.connect(func():
		more.visible = not more.visible
		tog.text = "More ▾" if more.visible else "More ▸")
	box.add_child(more)
	var off := SpinBox.new()
	off.name = "EpochOffset"
	off.min_value = -100000
	off.max_value = 100000
	off.step = 1
	off.value = 0
	off.prefix = "epoch offset "
	off.tooltip_text = "Added to the note's years on Import. 0 means the note's year is the engine's year (year 0 is the generated world)."
	more.add_child(off)
	_extras(more, c, key, stl)
	return off

## The entity the card's picker names now: `{kind, id}`, or `{}` with none.
func _picked(key: String) -> Dictionary:
	var opt: OptionButton = _entity_pick.get(key)
	if opt == null or opt.item_count == 0 or opt.selected < 0:
		return {}
	return opt.get_item_metadata(opt.selected)

## CT-7's two actions (design s.4.5): **Re-sync** for a timeline already imported (added / removed / changed counts, each
## group taken or left, one undo) and the **founding comparison** for a settlement (the note's founding events against the
## settlement's own year; a conflict shows both years and offers "Set the settlement's founding year", never applied
## silently). Neither edits a note.
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

## The preview on the shared axis (design: hollow dashed marks where the timeline's events would land, over the filled marks
## already there). One axis from the earliest to the latest year of either set; a range is a bar, a point a dot.
class Preview extends Control:
	var have: Array = []
	var incoming: Array = []
	var lo := 0.0
	var hi := 1.0
	func setup(existing: Array, inc: Array) -> void:
		have = existing
		incoming = inc
		# the axis is the incoming timeline's span plus 5 %: the timeline already on the strip is drawn only where it falls
		# inside it (a world's founding years can sit centuries earlier and would squash the preview into a corner)
		var first := true
		for e in inc:
			var d: Dictionary = e
			var a := float(d.get("start", 0))
			var z := float(d.get("end", a))
			lo = a if first else minf(lo, a)
			hi = z if first else maxf(hi, z)
			first = false
		if hi <= lo:
			hi = lo + 1.0
		var pad := (hi - lo) * 0.05
		lo -= pad
		hi += pad
		have = have.filter(func(e): return float(e["start_year"]) >= lo and float(e["start_year"]) <= hi)
		custom_minimum_size = Vector2(160, 46)
		mouse_filter = Control.MOUSE_FILTER_IGNORE
	func _x(y: float) -> float:
		return 6.0 + (y - lo) / (hi - lo) * maxf(size.x - 12.0, 1.0)
	func _draw() -> void:
		var ink := DccTheme.c("text_dim")
		var acc := DccTheme.c("accent")
		var f := DccTheme.mono()
		draw_line(Vector2(6, 26), Vector2(size.x - 6, 26), DccTheme.c("line"), 1.0)
		for e in have:
			var d: Dictionary = e
			var x0 := _x(float(d["start_year"]))
			var x1 := _x(float(d["end_year"])) if d.has("end_year") else x0
			if x1 - x0 < 2.0:
				draw_circle(Vector2(x0, 26), 3.0, ink)
			else:
				draw_rect(Rect2(x0, 23, x1 - x0, 6), ink)
		for e in incoming:
			var d: Dictionary = e
			var x0 := _x(float(d["start"]))
			var x1 := _x(float(d["end"])) if d.has("end") else x0
			if x1 - x0 < 2.0:
				draw_arc(Vector2(x0, 26), 4.0, 0.0, TAU, 12, acc, 1.5)
			else:
				var r := Rect2(x0, 21, x1 - x0, 10)
				draw_dashed_line(r.position, Vector2(r.end.x, r.position.y), acc, 1.5, 4.0)
				draw_dashed_line(Vector2(r.position.x, r.end.y), r.end, acc, 1.5, 4.0)
				draw_dashed_line(r.position, Vector2(r.position.x, r.end.y), acc, 1.5, 4.0)
				draw_dashed_line(Vector2(r.end.x, r.position.y), r.end, acc, 1.5, 4.0)
		draw_string(f, Vector2(4, 44), str(int(lo)), HORIZONTAL_ALIGNMENT_LEFT, -1, 10, DccTheme.c("text_ghost"))
		var s := str(int(hi))
		draw_string(f, Vector2(size.x - 4 - f.get_string_size(s, HORIZONTAL_ALIGNMENT_LEFT, -1, 10).x, 44), s, HORIZONTAL_ALIGNMENT_LEFT, -1, 10, DccTheme.c("text_ghost"))

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

func _fill_entities(opt: OptionButton, c: Dictionary) -> void:
	var b := _bridge()
	if b == null:
		return
	var sel: Variant = _shell.get("selection")
	var want := -1
	var linked := _existing_link(b, String(c["rel"]))
	var i := 0
	for s in b.settlements():
		var sd: Dictionary = s
		opt.add_item("settlement \u00b7 %s" % String(sd.get("name", "")), i)
		opt.set_item_metadata(opt.item_count - 1, {"kind": "settlement", "id": int(sd.get("tid", 0)), "label": String(sd.get("name", ""))})
		if not linked.is_empty() and String(linked.get("entity_kind", "")) == "settlement" and int(linked.get("entity_id", -1)) == int(sd.get("tid", 0)):
			want = opt.item_count - 1
		elif linked.is_empty() and sel is Dictionary and String((sel as Dictionary).get("kind", "")) == "settlement" and int((sel as Dictionary).get("id", -2)) == i:
			want = opt.item_count - 1
		i += 1
	for fa in b.get_factions():
		var fd: Dictionary = fa
		opt.add_item("faction \u00b7 %s" % String(fd.get("name", "")), 1000 + int(fd.get("id", 0)))
		opt.set_item_metadata(opt.item_count - 1, {"kind": "faction", "id": int(fd.get("id", 0)), "label": String(fd.get("name", ""))})
		if not linked.is_empty() and String(linked.get("entity_kind", "")) == "faction" and int(linked.get("entity_id", -1)) == int(fd.get("id", 0)):
			want = opt.item_count - 1
	if want >= 0:
		opt.select(want)
	elif opt.item_count > 0:
		opt.select(0)

## Applies the card's choice **on click** (Ruling GT, T7) and shows the one-line result with its Undo. One decision is one
## undo step (`timeline_decide_undo`).
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
	var msg: String = {"keep": "Kept in the note", "ignore": "Ignored", "link": "Linked", "import": "Imported"}[word]
	if word == "import":
		msg += ": added %d, already on the timeline %d" % [(got["added"] as PackedInt64Array).size(), int(got["duplicates"])]
	_undo[key] = String(got["undo"])
	c["standing"] = "decided"
	c["previous"] = word
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
		for c in _cands:
			if _key(c) == key:
				(c as Dictionary)["standing"] = "undecided"
				(c as Dictionary)["previous"] = ""
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
			cd["standing"] = "decided"
			cd["previous"] = word
	if _on_change.is_valid():
		_on_change.call()
	_build_cards()
	_summary.text += "  ·  %s applied to %d" % [word, n]
