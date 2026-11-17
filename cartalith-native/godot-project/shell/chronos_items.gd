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

## The lanes, in `ItemSource::ALL`'s order; the Kind filter's wire words.
const KINDS := ["Founding", "Snapshot", "Ownership", "Conflict", "Journey", "Note", "Imported"]
## At most this many rows are drawn, the ones nearest the cursor; a labelled judgement (about two lines of
## chips at 1600 px), the same cap discipline as `TL_STRIP_PILLS_MAX`.
const ROWS_MAX := 24
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
var _list: HFlowContainer = null
var _count: Label = null
var _badge: Button = null
var _items_btn: Button = null
var _hidden_total := 0

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
	_items_btn.tooltip_text = "Every dated item the map knows (founding years, recorded years, ownership, conflicts, journeys, note events, imported events), filterable. A click moves the year."
	_badge = DccWidgets.segment(head, "", _open_prompt)
	_badge.name = "TimelinesBadge"
	_badge.visible = false
	_body = VBoxContainer.new()
	_body.name = "ChronosItemsBody"
	_body.add_theme_constant_override("separation", 4)
	_body.visible = not compact
	add_child(_body)
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
	_body.visible = not _body.visible
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
			(c as Button).custom_minimum_size.y = maxf((c as Button).custom_minimum_size.y, float(DccTheme.role_px("btn_min_h")))

## The filter as `WorldGen.timeline_items` takes it.
func filter_dict() -> Dictionary:
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
	if not _body.visible:
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
	if shown.size() > ROWS_MAX:
		shown.sort_custom(func(a, c): return absi(int(a["start_year"]) - cur) < absi(int(c["start_year"]) - cur))
		shown = shown.slice(0, ROWS_MAX)
		shown.sort_custom(func(a, c): return int(a["start_year"]) < int(c["start_year"]))
	for it in shown:
		_row_button(_list, it)
	var more := items.size() - shown.size()
	var parts: Array = []
	if more > 0:
		parts.append("+%d more (the %d nearest the cursor are shown)" % [more, ROWS_MAX])
	if _hidden_total > 0:
		parts.append("%d hidden by filters" % _hidden_total)
	if items.is_empty():
		parts.append("no dated items" if _hidden_total == 0 else "nothing matches the filters")
	_count.text = " · ".join(parts)

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
	var y := int(it["start_year"])
	var doy := -1
	if it.has("start_month"):
		doy = _day_of_year(int(it["start_month"]), int(it.get("start_day", 1)))
	b.pressed.connect(func():
		if _shell == null:
			return
		_shell.tl_set_year(y, true)
		if doy >= 0:
			_shell.tl_set_day(doy))
	b.custom_minimum_size.y = maxf(b.custom_minimum_size.y, float(DccTheme.role_px("btn_min_h")))
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
