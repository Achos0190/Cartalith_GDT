extends VBoxContainer
## The consolidated timeline's rows on the year strip (`CHRONOS_TIMELINE_DESIGN.md` s.2, s.6; Ruling GD/GE
## option (a); CT-2, CT-4). Built by `app.gd::_build_timeline_expanded` (desktop and tablet) and by
## `dcc_shell.gd::_build_phone_sim_strip` (phone, compact: the body stays shut until "Items" is pressed).
##
## **It owns no data and no clock.** Every row is `WorldGen.timeline_items(filter)`'s answer (the read model
## and its filters are Rust, with tests); a click calls the shell's one year writer `tl_set_year()` (and
## `tl_set_day()` when the row has a day), exactly as VE-7's chronos rows do. No new signal, no new writer.
##
## Filters (design s.2.2): scope (selection follows / all items), kind (seven lanes), faction. They persist
## per device in the `timeline_items` section of `DccSettings.CONFIG_PATH`, beside the layer toggles, and not
## in the world save. The strip says "N hidden by filters" so an empty list is never read as "no data".
##
## **The badge is passive** (Ruling GE: *"a passive badge, no automatic prompt"*). The vault scan is
## read-budgeted and paged (`WorldGen.timelines_scan`), runs on a timer one page per tick, and only ever
## adds a count and a button; the prompt opens when the user presses the badge. `_scan` is static so a strip
## rebuild does not restart it.

const ChronosPrompt := preload("res://shell/chronos_prompt.gd")
const ChronosSpans := preload("res://shell/chronos_spans.gd")

## The lanes, in `ItemSource::ALL`'s order; the Kind filter's wire words.
const KINDS := ["Founding", "Snapshot", "Ownership", "Conflict", "Journey", "Note", "Imported"]
## At most this many rows are drawn, the ones nearest the cursor; a labelled judgement (about two lines of
## chips at 1600 px), the same cap discipline as `TL_STRIP_PILLS_MAX`.
const ROWS_MAX := 24
## A world with `civ.founding_year` on has one Founding row per settlement (217 in the probe world), which
## crowds the 24-row cap and pushes the user's own events off the strip. More than this many Founding rows
## collapse into one cluster chip ("Founding x217 ..."), pressed to open them; a selected item's own rows
## (a handful) are below it and stay as rows. A labelled judgement: about one chip line.
const FOUNDING_GROUP_MAX := 5
## The spans lane draws at most this many bars when no settlement or faction is selected (an ownership span
## per settlement per owner is thousands in a big world); above it the lane says how many it holds and asks
## for a selection, never a silent subset.
const SPANS_MAX := 60
const MONTH_DAYS := [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
## Notes read per scan page and the pause between pages (a labelled judgement: 200 reads is a few
## milliseconds of file I/O; the pause leaves the frames to the UI).
const SCAN_PAGE := 200
const SCAN_PAUSE_SEC := 0.25

## The vault scan, shared by every instance: `candidates` (accumulated pages), `next` (-1 when covered),
## `done`, and `error` (empty when fine). Reset by `invalidate_scan()`.
static var _scan := {"candidates": [], "next": 0, "done": false, "error": ""}

var _shell: Node = null
var _compact := false
var _scope_all := false
var _kinds: Array = []
var _factions: Array = []
var _body: Control = null
## What the Items button shows and hides: `_body` itself on the desktop and tablet, a `ScrollContainer` around
## it on the phone (the phone strip is a bottom panel that grows upward, and the filter chips at 44 dp are
## four lines tall, so an unbounded body pushed the rows under the sheet; it is bounded to a share of the
## window and scrolls).
var _holder: Control = null
var _list: HFlowContainer = null
var _count: Label = null
var _badge: Button = null
var _items_btn: Button = null
var _hidden_total := 0
var _founding_open := false
var _spans: Control = null
var _spans_note: Label = null

## Builds the toolbar and the rows. `compact` is the phone: only the "Items" button and the badge show until
## the button opens the body.
func setup(shell: Node, compact: bool) -> VBoxContainer:
	_shell = shell
	_compact = compact
	name = "ChronosItems"
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	add_theme_constant_override("separation", 4)
	_load_prefs()
	var head := HFlowContainer.new()
	head.add_theme_constant_override("h_separation", 6)
	head.add_theme_constant_override("v_separation", 4)
	head.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.name = "ChronosItemsHead"
	add_child(head)
	head.add_child(DccTheme.mono_label("ITEMS", "text_ghost", DccTheme.role_px("fs_timeline"), 2))
	_items_btn = DccWidgets.segment(head, "Items", _on_items_pressed)
	_items_btn.name = "ItemsToggle"
	_items_btn.custom_minimum_size.y = maxf(_items_btn.custom_minimum_size.y, float(ChronosPrompt.tap_h()))
	_items_btn.tooltip_text = "Every dated item the map knows (founding years, recorded years, ownership, conflicts, journeys, note events, imported events), filterable. A click moves the year."
	_badge = DccWidgets.segment(head, "", _open_prompt)
	_badge.name = "TimelinesBadge"
	_badge.custom_minimum_size.y = maxf(_badge.custom_minimum_size.y, float(ChronosPrompt.tap_h()))
	_badge.visible = false
	_body = VBoxContainer.new()
	_body.name = "ChronosItemsBody"
	_body.add_theme_constant_override("separation", 4)
	if compact:
		var sc := ScrollContainer.new()
		sc.name = "ChronosItemsScroll"
		sc.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
		sc.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		sc.visible = false
		_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		sc.add_child(_body)
		add_child(sc)
		_holder = sc
	else:
		add_child(_body)
		_holder = _body
	_build_filters()
	_list = HFlowContainer.new()
	_list.name = "ChronosItemsList"
	_list.add_theme_constant_override("h_separation", 6)
	_list.add_theme_constant_override("v_separation", 4)
	_list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_body.add_child(_list)
	_count = DccTheme.mono_label("", "text_ghost", DccTheme.role_px("fs_timeline"), 0)
	_count.name = "ChronosItemsCount"
	_body.add_child(_count)
	_spans = ChronosSpans.new()
	_spans.visible = false
	_spans.on_pick = _pick_item
	_body.add_child(_spans)
	_spans_note = DccTheme.mono_label("", "text_ghost", DccTheme.role_px("fs_timeline"), 0)
	_spans_note.name = "ChronosSpansNote"
	_spans_note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_spans_note.visible = false
	_body.add_child(_spans_note)
	if _shell != null and _shell.has_signal("timeline_records_changed") \
			and not _shell.timeline_records_changed.is_connected(_on_records_changed):
		_shell.timeline_records_changed.connect(_on_records_changed)
	return self

## The strip builds this before it is in the tree, so the first read and the scan wait for `_ready`.
func _ready() -> void:
	_refresh()
	_start_scan()

func _bridge() -> Object:
	if _shell == null or not _shell.has_method("_find_engine_bridge"):
		return null
	var b: Object = _shell._find_engine_bridge()
	if b == null or not bool(b.get("has_world")):
		return null
	return b

func _on_records_changed() -> void:
	if is_inside_tree():
		_refresh()
		_start_scan()

func _on_items_pressed() -> void:
	_holder.visible = not _holder.visible
	if _compact and _holder.visible:
		# 38 % of the window: the transport and the sheet below keep the rest (a labelled judgement)
		_holder.custom_minimum_size.y = floorf(get_viewport_rect().size.y * 0.38)
	_refresh()

## The filter chips: scope, the seven kinds, the factions. Lit = in force; a kind or faction group with none lit
## means "no restriction".
func _build_filters() -> void:
	var f := HFlowContainer.new()
	f.name = "ChronosFilters"
	f.add_theme_constant_override("h_separation", 6)
	f.add_theme_constant_override("v_separation", 4)
	f.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_body.add_child(f)
	var scope := DccWidgets.segment(f, "All items" if _scope_all else "Selection follows", func(): pass)
	scope.name = "Scope"
	scope.tooltip_text = "Selection follows: the rows for the selected settlement or faction (nothing selected shows all). All items: every row."
	scope.pressed.connect(func():
		_scope_all = not _scope_all
		scope.text = "All items" if _scope_all else "Selection follows"
		_save_prefs()
		_refresh())
	for k in KINDS:
		var kind: String = k
		var chip := DccWidgets.segment(f, kind, func(): pass)
		chip.name = "Kind_" + kind
		DccWidgets.set_segment_on(chip, _kinds.has(kind))
		chip.tooltip_text = "Show only the %s lane (press more lanes to add them; none lit shows every lane)." % kind
		chip.pressed.connect(func():
			if _kinds.has(kind):
				_kinds.erase(kind)
			else:
				_kinds.append(kind)
			DccWidgets.set_segment_on(chip, _kinds.has(kind))
			_save_prefs()
			_refresh())
	var b := _bridge()
	if b != null:
		var roster: Array = b.get_factions()
		_prune_factions(b, roster)
		var n := 0
		for fa in roster:
			n += 1
			if n > 8:
				break
			var fd: Dictionary = fa
			var fid: int = int(fd.get("id", n))
			var chip := DccWidgets.segment(f, String(fd.get("name", "Faction %d" % fid)), func(): pass)
			chip.name = "Faction_%d" % fid
			DccWidgets.set_segment_on(chip, _factions.has(fid))
			chip.tooltip_text = "Only items involving this faction."
			chip.pressed.connect(func():
				if _factions.has(fid):
					_factions.erase(fid)
				else:
					_factions.append(fid)
				DccWidgets.set_segment_on(chip, _factions.has(fid))
				_save_prefs()
				_refresh())
	var export := DccWidgets.segment(f, "Export view", _export_view)
	export.name = "ExportView"
	export.tooltip_text = "Copy the filtered view as a Chronos block to the clipboard, to paste into a note. Private notes' events are never written. It is not read back."
	for c in f.get_children():
		if c is Button:
			(c as Button).custom_minimum_size.y = maxf((c as Button).custom_minimum_size.y, float(ChronosPrompt.tap_h()))

## Ruling GK item 3 (owner 2026-11-22): drops every saved faction filter id that `roster`
## (`get_factions()` of a live world) no longer holds -- a faction deleted since, or all of them
## (Ruling GI) -- and saves the pruned list, so a stale id cannot silently hide every row.
## Does nothing while `b` is generating: `get_factions()` answers `[]` then (the worker holds
## `WorldGen`), which is not "no faction". Callers pass a live bridge only (`_bridge()`).
func _prune_factions(b: Object, roster: Array) -> void:
	if bool(b.get("generating")):
		return
	var ids := {}
	for fa in roster:
		ids[int((fa as Dictionary).get("id", -1))] = true
	var kept := _factions.filter(func(fid): return ids.has(int(fid)))
	if kept.size() != _factions.size():
		_factions = kept
		_save_prefs()

## The filter as `WorldGen.timeline_items` takes it.
func filter_dict() -> Dictionary:
	var b := _bridge()
	if b != null:
		_prune_factions(b, b.get_factions())
	var d := {}
	if not _kinds.is_empty():
		d["kinds"] = PackedStringArray(_kinds)
	if not _factions.is_empty():
		d["factions"] = PackedInt32Array(_factions)
	if not _scope_all:
		var key := _selection_key()
		if key != "":
			d["entities"] = PackedStringArray([key])
	# the ownership lane is lazy: it is only computed when it is asked for by name
	return d

## `settlement:<tid>` / `faction:<id>` for the shell's selection, or "" with none.
func _selection_key() -> String:
	if _shell == null:
		return ""
	var sel: Variant = _shell.get("selection")
	if not (sel is Dictionary) or (sel as Dictionary).is_empty():
		return ""
	var kind := String((sel as Dictionary).get("kind", ""))
	var id := int((sel as Dictionary).get("id", -1))
	var b := _bridge()
	if b == null:
		return ""
	if kind == "settlement":
		var all: Array = b.settlements()
		if id < 0 or id >= all.size():
			return ""
		return "settlement:%d" % int((all[id] as Dictionary).get("tid", 0))
	if kind == "faction":
		return "faction:%d" % id
	return ""

## Rebuilds the rows from the Rust read model. Called on the records signal and on every filter change.
func _refresh() -> void:
	if _list == null:
		return
	_update_badge()
	if not _holder.visible:
		if _items_btn != null:
			_items_btn.text = "Items"
		return
	for c in _list.get_children():
		_list.remove_child(c)
		c.queue_free()
	var b := _bridge()
	if b == null:
		_count.text = "no world"
		return
	var got: Dictionary = b.world_gen.timeline_items(filter_dict())
	if not bool(got.get("ok", false)):
		_count.text = String(got.get("error", "unavailable"))
		return
	var items: Array = got.get("items", [])
	_hidden_total = int(got.get("hidden", 0))
	if _items_btn != null:
		_items_btn.text = "Items (%d)" % items.size()
	var cur: int = int(_shell.tl_year()) if _shell != null else 0
	var shown: Array = items.duplicate()
	var founding: Array = shown.filter(func(i): return String(i["source"]) == "Founding")
	var clustered := founding.size() > FOUNDING_GROUP_MAX and not _founding_open and not _kinds.has("Founding")
	if clustered:
		shown = shown.filter(func(i): return String(i["source"]) != "Founding")
		_cluster_button(_list, founding)
	elif founding.size() > FOUNDING_GROUP_MAX and _founding_open and not _kinds.has("Founding"):
		_cluster_button(_list, founding)
	if shown.size() > ROWS_MAX:
		shown.sort_custom(func(a, c): return absi(int(a["start_year"]) - cur) < absi(int(c["start_year"]) - cur))
		shown = shown.slice(0, ROWS_MAX)
		shown.sort_custom(func(a, c): return int(a["start_year"]) < int(c["start_year"]))
	for it in shown:
		_row_button(_list, it)
	var more := items.size() - shown.size() - (founding.size() if clustered else 0)
	var parts: Array = []
	if more > 0:
		parts.append("+%d more (the %d nearest the cursor are shown)" % [more, ROWS_MAX])
	if _hidden_total > 0:
		parts.append("%d hidden by filters" % _hidden_total)
	if items.is_empty():
		parts.append("no dated items" if _hidden_total == 0 else "nothing matches the filters")
	_count.text = " · ".join(parts)
	_refresh_spans(cur)

## The one chip that stands for every Founding row; pressing it opens (or closes) them.
func _cluster_button(parent: Control, founding: Array) -> Button:
	var lo := 0
	var hi := 0
	for i in founding.size():
		var y := int((founding[i] as Dictionary)["start_year"])
		lo = y if i == 0 else mini(lo, y)
		hi = y if i == 0 else maxi(hi, y)
	var text := "Founding x%d (%d to %d) - press to %s" % [founding.size(), lo, hi, "close" if _founding_open else "open"]
	var b := DccWidgets.segment(parent, text, func(): pass)
	b.name = "FoundingCluster"
	b.tooltip_text = "Every settlement's founding year, grouped so they do not crowd out the rest. Press to list the %d nearest the cursor." % ROWS_MAX
	b.custom_minimum_size.y = maxf(b.custom_minimum_size.y, float(ChronosPrompt.tap_h()))
	DccWidgets.set_segment_on(b, _founding_open)
	b.pressed.connect(func():
		_founding_open = not _founding_open
		_refresh())
	return b

## The spans lane: ownership spans, conflicts and journeys drawn as bars on one axis. Two queries, because
## the two halves scale differently. **Conflicts and journeys** are few and global, so they are always drawn
## (narrowed only by the strip's own Kind and faction chips). **Ownership spans** are one per settlement per
## owner (hundreds), and the Rust lane computes them only when named, so they are drawn only for the selected
## settlement (its own key) or faction (the faction filter), and with no selection the lane says how many it
## is holding back instead of drawing a silent subset. "All items" scope draws every one, up to `SPANS_MAX`.
func _refresh_spans(cur: int) -> void:
	if _spans == null:
		return
	var b := _bridge()
	_spans.set_items([], cur)
	_spans_note.visible = false
	if b == null:
		return
	var note := ""
	var rows: Array = []
	var glob: Array = ["Conflict", "Journey"]
	if not _kinds.is_empty():
		glob = glob.filter(func(k): return _kinds.has(k))
	if not glob.is_empty():
		rows.append_array(_span_rows(b, {"kinds": PackedStringArray(glob)}))
	if _kinds.is_empty() or _kinds.has("Ownership"):
		var d := {"kinds": PackedStringArray(["Ownership"])}
		var drawn := true
		if not _scope_all:
			var sel: Variant = _shell.get("selection")
			var key := _selection_key()
			if sel is Dictionary and String((sel as Dictionary).get("kind", "")) == "faction":
				d["factions"] = PackedInt32Array([int((sel as Dictionary).get("id", -1))])
			elif key != "":
				d["entities"] = PackedStringArray([key])
			else:
				drawn = false
		if drawn:
			var own := _span_rows(b, d)
			if own.size() > SPANS_MAX:
				note = "%d ownership spans - narrow with a faction chip or a selection to draw them" % own.size()
			else:
				rows.append_array(own)
		else:
			note = "ownership spans are drawn for the selected settlement or faction"
	_spans.set_items(rows, cur)
	if _spans.overflow > 0:
		note = ("+%d more spans not drawn (%d rows)" % [_spans.overflow, ChronosSpans.ROWS_MAX]) + (" · " + note if note != "" else "")
	_spans_note.text = note
	_spans_note.visible = note != ""

## The model's rows for `d`, narrowed to those with a length, plus the strip's faction chips.
func _span_rows(b: Object, d: Dictionary) -> Array:
	if not _factions.is_empty() and not d.has("factions"):
		d["factions"] = PackedInt32Array(_factions)
	var got: Dictionary = b.world_gen.timeline_items(d)
	if not bool(got.get("ok", false)):
		return []
	return (got["items"] as Array).filter(func(i): return i.has("end_year") or bool(i.get("open_end", false)))

## A bar or a row was pressed: the year (and day) it starts on, in one write.
func _pick_item(it: Dictionary) -> void:
	if _shell == null:
		return
	var y := int(it["start_year"])
	if it.has("start_month"):
		_shell.tl_set_date(y, _day_of_year(int(it["start_month"]), int(it.get("start_day", 1))), true)
	else:
		_shell.tl_set_year(y, true)

func _date_text(it: Dictionary, prefix: String, year_key: String) -> String:
	var y := int(it[year_key])
	var m := prefix + "_month"
	if it.has(m):
		var s := "%d-%02d" % [y, int(it[m])]
		if it.has(prefix + "_day"):
			s += "-%02d" % int(it[prefix + "_day"])
		return s
	return str(y)

func _row_button(parent: Control, it: Dictionary) -> Button:
	var when := _date_text(it, "start", "start_year")
	if it.has("end_year"):
		when += "~" + _date_text(it, "end", "end_year")
	elif bool(it.get("open_end", false)):
		when += "~"
	var text := "%s  %s" % [when, String(it["title"])]
	if bool(it.get("private", false)):
		text += "  [private]"
	var b := DccWidgets.segment(parent, text, func(): pass)
	b.name = "Item_" + String(it["key"]).replace(":", "_")
	b.set_meta("key", String(it["key"]))
	b.set_meta("tl_year", int(it["start_year"]))
	b.tooltip_text = "%s · %s. Moves the year to %s." % [String(it["source"]), String(it["title"]), when.split("~")[0]]
	b.pressed.connect(func(): _pick_item(it))
	b.custom_minimum_size.y = maxf(b.custom_minimum_size.y, float(ChronosPrompt.tap_h()))
	return b

## The 0-based day of the 365-day calendar (`chronos::MonthDay::day_of_year`).
static func _day_of_year(month: int, day: int) -> int:
	var before := 0
	for i in range(clampi(month, 1, 12) - 1):
		before += int(MONTH_DAYS[i])
	return before + day - 1

func _export_view() -> void:
	var b := _bridge()
	if b == null:
		return
	var got: Dictionary = b.world_gen.timeline_export(filter_dict())
	if not bool(got.get("ok", false)):
		return
	DisplayServer.clipboard_set(String(got["text"]))
	if _count != null:
		_count.text = "copied %d line(s) as a Chronos block%s" % [int(got["written"]),
			(", %d could not be written" % (got["skipped"] as PackedStringArray).size()) if (got["skipped"] as PackedStringArray).size() > 0 else ""]

# -- the passive badge -----------------------------------------------------------------------------------

## Pages the vault scan, one `SCAN_PAGE` per tick, until covered. A note not read is not decided; the badge
## counts only what has been looked at.
func _start_scan() -> void:
	if bool(_scan["done"]) or _shell == null or not is_inside_tree():
		return
	var b := _bridge()
	if b == null:
		# the phone strip is built before any world exists: look again shortly, not never
		get_tree().create_timer(1.0).timeout.connect(_start_scan)
		return
	if bool(_scan.get("busy", false)):
		return
	_scan["busy"] = true
	var got: Dictionary = b.world_gen.timelines_scan(int(_scan["next"]), SCAN_PAGE)
	_scan["busy"] = false
	if not bool(got.get("ok", false)):
		_scan["error"] = String(got.get("error", ""))
		if _scan["error"].contains("no vault"):
			# a vault may be connected later; look again, cheaply, while the strip is up
			get_tree().create_timer(5.0).timeout.connect(_start_scan)
		else:
			_scan["done"] = true
		return
	_scan["candidates"].append_array(got.get("candidates", []))
	var nxt := int(got.get("next", -1))
	_scan["next"] = nxt
	if nxt < 0:
		_scan["done"] = true
	_update_badge()
	if nxt >= 0:
		get_tree().create_timer(SCAN_PAUSE_SEC).timeout.connect(_start_scan)

## Forgets the scan so the next strip build starts again (after a decision, or a vault change).
static func invalidate_scan() -> void:
	_scan = {"candidates": [], "next": 0, "done": false, "error": ""}

## Candidates that still need attention: undecided, or changed since the decision.
static func attention_count() -> int:
	var n := 0
	for c in _scan["candidates"]:
		if String((c as Dictionary).get("standing", "undecided")) != "decided":
			n += 1
	return n

static func candidates() -> Array:
	return _scan["candidates"]

func _update_badge() -> void:
	if _badge == null:
		return
	var n := attention_count()
	_badge.visible = n > 0
	if n > 0:
		_badge.text = "! %d timeline%s found in the vault - Review" % [n, "" if n == 1 else "s"]
		_badge.tooltip_text = "Timelines found in your notes. Nothing has been changed; press to choose what to do with each."

func _open_prompt() -> void:
	var d: AcceptDialog = ChronosPrompt.new()
	_shell.add_child(d)
	d.setup(_shell, candidates(), func():
		invalidate_scan()
		_start_scan()
		_refresh())
	d.popup_centered(Vector2i(760, 560) if not _shell.is_phone() else Vector2i(420, 760))

# -- device settings -------------------------------------------------------------------------------------

func _load_prefs() -> void:
	var cfg := ConfigFile.new()
	cfg.load(DccSettings.CONFIG_PATH)
	_scope_all = bool(cfg.get_value("timeline_items", "scope_all", false))
	_kinds = []
	for k in String(cfg.get_value("timeline_items", "kinds", "")).split(",", false):
		if KINDS.has(k):
			_kinds.append(k)
	_factions = []
	for f in String(cfg.get_value("timeline_items", "factions", "")).split(",", false):
		if f.is_valid_int():
			_factions.append(int(f))

func _save_prefs() -> void:
	var cfg := ConfigFile.new()
	cfg.load(DccSettings.CONFIG_PATH)
	cfg.set_value("timeline_items", "scope_all", _scope_all)
	cfg.set_value("timeline_items", "kinds", ",".join(PackedStringArray(_kinds)))
	var fs: Array = []
	for f in _factions:
		fs.append(str(f))
	cfg.set_value("timeline_items", "factions", ",".join(PackedStringArray(fs)))
	cfg.save(DccSettings.CONFIG_PATH)
