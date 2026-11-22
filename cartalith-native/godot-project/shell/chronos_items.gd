extends VBoxContainer
## The timeline ladder (Ruling GT, 2026-11-22; `design/timeline-designs-2026-11-22/timeline_designs.html` option C, with
## every default of T2-T10; `CHRONOS_TIMELINE_DESIGN.md` s.10 "Ruling GT"). It replaces the chip list of
## `CHRONOS_TIMELINE_DESIGN.md` s.2 (CT-2) at the same two mount points: `app.gd::_build_timeline_expanded` (desktop and
## tablet) and `dcc_shell.gd::_build_phone_sim_strip` (phone). Four levels, each one adding detail:
##
##   L0  a minimal strip: period bands from the vault's `@` lines (centuries when there are none), at most five landmark
##       ticks (the `=` markers, then conflicts, then capitals' foundings), the cursor and one count line.
##   L1  four lanes (Founding, Conflict, Journey, From the vault), a point against a range, a label only where it fits;
##       `Ownership` joins only when a place or faction is selected.
##   L2  a filterable table of every event: a search box and three filter menus (Kind, Source, Faction).
##   L3  "Open as document": `WorldGen.timeline_open_document` writes `_cartalith/Consolidated timeline.md` (generated,
##       read-only, `cartalith: generated-timeline`, never over an unmarked file) and the vault editor opens it with a banner.
##
## **It owns no data and no clock.** Every mark is `WorldGen.timeline_items`' answer (the read model, the dedup of a live
## note event and its imported copy into one row, and `ev_kind`/`color` are Rust, with tests); a click or tap calls the
## shell's one year writer `tl_set_year()` / `tl_set_date()`. The vault is **never edited** by anything here but the L3 file.
##
## **Navigation** (Ruling GT): a click or tap climbs a level (an L0 press that hits no landmark, "All N events", a
## cluster at L1); Back, Esc or the chevron steps down one (`go_down()`; `DccShell`'s back chain calls it before the
## exit gate); on the phone a vertical swipe on the canvas climbs or descends. The strip rests at L0 on the phone and
## L1 elsewhere, remembered **per device** (`timeline_items/level` in `DccSettings.CONFIG_PATH`); L2 is never a rest
## state. Filters and the L2 search survive a strip rebuild (the strip is rebuilt on every selection change) in statics.
##
## **The badge is passive** (Ruling GE: *"a passive badge, no automatic prompt"*): the vault scan is read-budgeted and
## paged (`WorldGen.timelines_scan`), runs on a timer one page per tick, and only adds a count and a button; the prompt
## opens when the user presses it. `_scan` is static so a strip rebuild does not restart it.

const ChronosPrompt := preload("res://shell/chronos_prompt.gd")
const ChronosLanes := preload("res://shell/chronos_lanes.gd")

## The kinds the ladder asks the read model for. Snapshot and Ownership join only with a selection (below).
const BASE_KINDS := ["Founding", "Conflict", "Journey", "Note", "Imported"]
## The Kind filter's words; `Notes` is the lane the vault's two sources share ("Note" in the menu).
const KIND_WORDS := ["Founding", "Conflict", "Journey", "Note"]
const SOURCE_WORDS := ["Cartalith", "Note (live)", "Imported", "Note + Imported"]
const MONTH_DAYS := [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
## Notes read per scan page and the pause between pages (a labelled judgement: 200 reads is a few
## milliseconds of file I/O; the pause leaves the frames to the UI).
const SCAN_PAGE := 200
const SCAN_PAUSE_SEC := 0.25
## The L2 table's height as a share of the window, floored and capped (a labelled judgement: the table is read beside
## the map, which stays live, so it must not take the screen).
const TABLE_SHARE := 0.34
const TABLE_SHARE_PHONE := 0.24
const TABLE_SHARE_TABLET := 0.22
## One century, in years, for the bands drawn when the vault has no `@` lines (Ruling GT, T8).
const CENTURY := 100

## The vault scan, shared by every instance: `candidates` (accumulated pages), `next` (-1 when covered),
## `done`, and `error` (empty when fine). Reset by `invalidate_scan()`.
static var _scan := {"candidates": [], "next": 0, "done": false, "error": ""}
## Survive a strip rebuild: the level (-1 = not yet read from the device settings), the filters and the cluster.
static var _s_level := -1
static var _s_kinds: Array = []
static var _s_sources: Array = []
static var _s_factions: Array = []
static var _s_search := ""
static var _s_cluster: Array = []
static var _s_filter_open := false

var _shell: Node = null
var _compact := false
var level := 1
var _up: Button = null
var _down: Button = null
var _count: Label = null
var _badge: Button = null
var _filter_btn: Button = null
var _all_btn: Button = null
var _canvas: Control = null
var _picked: Label = null
var _filters: HFlowContainer = null
var _search: LineEdit = null
var _menus: Dictionary = {}
var _nof: Label = null
var _doc_btn: Button = null
var _cluster_chip: Button = null
var _table: Tree = null
var _note: Label = null
var _sheet: AcceptDialog = null
## The model, rebuilt by `_refresh()`: every non-period item (with `lane`), the periods, and the track.
var _events: Array = []
var _periods: Array = []
var _bands: Array = []
var _landmarks: Array = []
var _shown: Array = []
var _total := 0
var _off := 0
var _last_sel := ""

## Builds the head, the canvas, the filter bar and the table. `compact` is the phone.
func setup(shell: Node, compact: bool) -> VBoxContainer:
	_shell = shell
	_compact = compact
	name = "ChronosItems"
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	add_theme_constant_override("separation", 4)
	level = _initial_level()
	var head := HFlowContainer.new()
	head.name = "LadderHead"
	head.add_theme_constant_override("h_separation", 6)
	head.add_theme_constant_override("v_separation", 4)
	head.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	add_child(head)
	_down = DccWidgets.segment(head, "⌄ Back", go_down)
	_down.name = "LadderDown"
	_down.tooltip_text = "Step down one level (Back or Esc does the same)."
	_up = DccWidgets.segment(head, "⌃", go_up)
	_up.name = "LadderUp"
	_up.tooltip_text = "Show more: lanes, then every event in a table."
	_count = DccTheme.mono_label("", "text_ghost", _fs(), 0)
	_count.name = "LadderCount"
	head.add_child(_count)
	_badge = DccWidgets.segment(head, "", _open_prompt)
	_badge.name = "TimelinesBadge"
	_badge.visible = false
	_all_btn = DccWidgets.segment(head, "", func(): _set_level(2))
	_all_btn.name = "AllEvents"
	_filter_btn = DccWidgets.segment(head, "Filter", func():
		_s_filter_open = not _s_filter_open
		_sync_visibility())
	_filter_btn.name = "FilterToggle"
	for b in [_down, _up, _badge, _all_btn, _filter_btn]:
		if _compact:
			(b as Button).add_theme_font_size_override("font_size", _fs())
		(b as Button).custom_minimum_size.y = maxf((b as Button).custom_minimum_size.y, float(ChronosPrompt.tap_h()))
	_canvas = ChronosLanes.new()
	_canvas.phone = shell.is_phone() if shell != null and shell.has_method("is_phone") else compact
	_canvas.touch = _canvas.phone or DccTheme.is_tablet()
	_canvas.short_titles = DccTheme.is_tablet() or _canvas.phone
	_canvas.on_pick = _pick_item
	_canvas.on_cluster = _open_cluster
	_canvas.on_expand = func(): _set_level(1)
	_canvas.on_swipe = func(dir: int):
		if dir < 0:
			go_up()
		else:
			go_down()
	add_child(_canvas)
	_picked = DccTheme.mono_label("", "text_dim", DccTheme.role_px("fs_timeline"), 0)
	_picked.name = "LadderPicked"
	_picked.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_picked.visible = false
	add_child(_picked)
	_build_filters()
	_table = Tree.new()
	_table.name = "LadderTable"
	_table.hide_root = true
	_table.column_titles_visible = true
	_table.select_mode = Tree.SELECT_ROW
	_table.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_table.add_theme_font_override("font", DccTheme.mono())
	_table.add_theme_font_size_override("font_size", maxi(10, _fs()))
	_table.item_mouse_selected.connect(_on_table_clicked)
	add_child(_table)
	_note = DccTheme.mono_label("", "text_ghost", _fs(), 0)
	_note.name = "LadderNote"
	_note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_note.visible = false
	add_child(_note)
	move_child(_note, 1)
	if _shell != null:
		if _shell.has_signal("timeline_records_changed") and not _shell.timeline_records_changed.is_connected(_on_records_changed):
			_shell.timeline_records_changed.connect(_on_records_changed)
		if _shell.has_signal("timeline_changed") and not _shell.timeline_changed.is_connected(_on_cursor):
			_shell.timeline_changed.connect(_on_cursor)
		# the Ownership and Snapshot lanes follow the selection (they are drawn for a selected place or faction only)
		if _shell.has_signal("selection_changed") and not _shell.selection_changed.is_connected(_on_selection):
			_shell.selection_changed.connect(_on_selection)
	return self

## The strip builds this before it is in the tree, so the first read and the scan wait for `_ready`.
func _ready() -> void:
	_refresh()
	_start_scan()

## A selection changed: only a change of the *entity* (`settlement:7`, `faction:2`, or none) matters to the lanes, so the
## model is re-read then and not on every hover-following update of the selection signal.
func _on_selection(_sel: Dictionary) -> void:
	if not is_inside_tree():
		return
	var k := _selection_key()
	if k != _last_sel:
		_last_sel = k
		_refresh.call_deferred()

func _exit_tree() -> void:
	if _shell != null:
		if _shell.has_signal("selection_changed") and _shell.selection_changed.is_connected(_on_selection):
			_shell.selection_changed.disconnect(_on_selection)
		if _shell.has_signal("timeline_records_changed") and _shell.timeline_records_changed.is_connected(_on_records_changed):
			_shell.timeline_records_changed.disconnect(_on_records_changed)
		if _shell.has_signal("timeline_changed") and _shell.timeline_changed.is_connected(_on_cursor):
			_shell.timeline_changed.disconnect(_on_cursor)

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

# -- levels ------------------------------------------------------------------------------------------------

## The remembered level of this device, or the form's default: L0 on the phone, L1 elsewhere (Ruling GT, T4).
func _initial_level() -> int:
	if _s_level >= 0:
		return _s_level
	var cfg := ConfigFile.new()
	cfg.load(DccSettings.CONFIG_PATH)
	var dflt := 0 if _compact else 1
	_s_level = clampi(int(cfg.get_value("timeline_items", "level", dflt)), 0, 1)
	return _s_level

func _remember(l: int) -> void:
	var rest := mini(l, 1)
	if rest == _s_level:
		return
	_s_level = rest
	var cfg := ConfigFile.new()
	cfg.load(DccSettings.CONFIG_PATH)
	cfg.set_value("timeline_items", "level", rest)
	cfg.save(DccSettings.CONFIG_PATH)

func _set_level(l: int) -> void:
	level = clampi(l, 0, 2)
	_remember(level)
	if level < 2:
		_s_cluster = []
	_sync_sheet()
	_refresh()

## L2 is a **sheet**, not a body under the lanes: the strip's other rows already fill the room under the map (measured
## windowed: inline, the table started at y 867 of 1000 on the tablet and at 677 of 900 on the desktop, its bottom off the
## window on both). The design draws it so: the tablet's 520 dp sheet, the phone's sheet, and on the desktop a panel beside the
## map, which stays live (the sheet is not modal and sits at the right edge).

## Opens or closes the L2 sheet to match `level`. The filter bar, the note line and the table are *moved* into it (so there
## is one table, not two) and moved back on close. Closing the sheet by its own X, Esc or the system Back is stepping down
## to L1.
func _sync_sheet() -> void:
	if level == 2 and _sheet == null:
		_sheet = AcceptDialog.new()
		_sheet.name = "LadderSheet"
		_sheet.title = "Timeline events"
		_sheet.exclusive = false
		_sheet.get_ok_button().hide()
		var pad := MarginContainer.new()
		for side in ["left", "right", "top", "bottom"]:
			pad.add_theme_constant_override("margin_" + side, 8)
		_sheet.add_child(pad)
		var box := VBoxContainer.new()
		box.name = "LadderSheetBody"
		box.add_theme_constant_override("separation", 6)
		pad.add_child(box)
		add_child(_sheet)
		for n in [_note, _filters, _table]:
			(n as Node).reparent(box, false)
		_table.size_flags_vertical = Control.SIZE_EXPAND_FILL
		_table.custom_minimum_size.y = 200.0
		_sheet.visibility_changed.connect(func():
			if _sheet != null and not _sheet.visible and level == 2:
				_set_level(1))
		var vp := get_viewport_rect().size
		if _compact:
			_sheet.popup_centered(Vector2i(int(vp.x) - 16, int(vp.y * 0.78)))
		elif DccTheme.is_tablet():
			_sheet.popup_centered(Vector2i(mini(int(vp.x) - 16, 560), int(vp.y * 0.78)))
		else:
			var w := mini(int(vp.x) - 16, 760)
			_sheet.popup(Rect2i(int(vp.x) - w - 8, 52, w, int(vp.y * 0.72)))
	elif level < 2 and _sheet != null:
		var sh := _sheet
		_sheet = null
		for n in [_table, _filters, _note]:
			(n as Node).reparent(self, false)
		move_child(_note, 1)
		_table.size_flags_vertical = Control.SIZE_FILL
		sh.queue_free()

## One level up (more detail); a no-op at L2.
func go_up() -> void:
	if level < 2:
		_set_level(level + 1)

## One level down; returns false at L0 so a caller (Esc, the back chain) knows it did nothing.
func go_down() -> bool:
	if level <= 0:
		return false
	_set_level(level - 1)
	return true

func _unhandled_key_input(ev: InputEvent) -> void:
	if ev is InputEventKey and (ev as InputEventKey).pressed and not (ev as InputEventKey).echo \
			and (ev as InputEventKey).keycode == KEY_ESCAPE and level > 0 and is_visible_in_tree():
		if go_down():
			get_viewport().set_input_as_handled()

## Visibility per level; one place so L0/L1/L2 cannot disagree.
func _sync_visibility() -> void:
	var phone_l0 := _compact and level == 0
	_canvas.visible = true
	_canvas.level = mini(level, 1)
	_down.visible = level > 0
	_up.visible = level == 0
	_count.visible = not phone_l0 and not (_compact and level == 2)
	_all_btn.visible = level == 1
	_filter_btn.visible = level == 1
	_filters.visible = level == 2 or (level == 1 and _s_filter_open)
	_search.visible = level == 2
	_doc_btn.visible = level == 2
	_table.visible = level == 2
	_cluster_chip.visible = level == 2 and not _s_cluster.is_empty()
	_picked.visible = level < 2 and _picked.text != "" and not (_compact or DccTheme.is_tablet())
	var nf := _filter_count()
	_filter_btn.text = "Filter (%d)" % nf if nf > 0 else "Filter"
	DccWidgets.set_segment_on(_filter_btn, _s_filter_open or nf > 0)

# -- the model ---------------------------------------------------------------------------------------------

## Rebuilds the model from the Rust read model and redraws the visible level.
func _refresh() -> void:
	if _canvas == null:
		return
	_update_badge()
	var b := _bridge()
	if b == null:
		_events = []
		_periods = []
		_total = 0
		_count.text = "no world"
		_canvas.set_model(mini(level, 1), 0, 1, 0, [], [], [])
		_sync_visibility()
		_fill_table()
		return
	var got: Dictionary = b.world_gen.timeline_items({"kinds": PackedStringArray(BASE_KINDS)})
	if not bool(got.get("ok", false)):
		_count.text = String(got.get("error", "unavailable"))
		return
	var items: Array = (got.get("items", []) as Array).duplicate()
	items.append_array(_selection_rows(b))
	_events = []
	_periods = []
	for it in items:
		var d: Dictionary = it
		if String(d.get("lane", "")) == "Snapshot":
			d["lane"] = "Ownership"
		if String(d.get("ev_kind", "")) == "period":
			_periods.append(d)
		else:
			_events.append(d)
	_total = _events.size()
	var rng: Dictionary = _shell.tl_range() if _shell != null and _shell.has_method("tl_range") else {"min": 0, "max": 1}
	var lo := int(rng["min"])
	var hi := int(rng["max"])
	_bands = _make_bands(lo, hi)
	_landmarks = _pick_landmarks(b)
	for e in _events:
		(e as Dictionary)["era"] = _era_of(int(e["start_year"]))
	_sync_visibility()
	_rebuild_menus(b)
	_shown = _filtered()
	var cur := int(_shell.tl_year()) if _shell != null else 0
	_feed_canvas(lo, hi, cur)
	_all_btn.text = "All %d event%s ›" % [_total, "" if _total == 1 else "s"]
	_fill_table()
	_update_badge()

## Hands the canvas the marks that lie on its axis. The axis is the strip's track (`tl_range()`), which does not widen
## for a founding year (CT view issue 1: a founding at -635 dragged the whole track back), so an event outside it has no
## place to be drawn and is **counted, not drawn at the edge** ("N off the axis"); every event stays in the L2 table.
func _feed_canvas(lo: int, hi: int, cur: int) -> void:
	var on: Array = _shown.filter(func(i): return _reaches(i, lo, hi))
	_off = _shown.size() - on.size()
	_canvas.set_model(mini(level, 1), lo, hi, cur, _bands, on if level >= 1 else [], _landmarks)
	var parts: Array = ["%d event%s" % [_total, "" if _total == 1 else "s"]]
	## The phone's count line is shorter: at its body size (12, was 10) the long form was wider than the transport panel
	## and widened the whole strip past the screen edge (frame `L1_transport_phone`, 2026-11-22).
	parts.append("%d period%s" % [_periods.size(), "" if _periods.size() == 1 else "s"] if not _periods.is_empty() else ("no periods" if _compact else "no periods (centuries shown)"))
	if _off > 0 and level >= 1:
		parts.append("%d off the axis" % _off if _compact else "%d off the axis (all events lists them)" % _off)
	_count.text = " · ".join(PackedStringArray(parts))

## True when the item's years touch `[lo, hi]`: a range that starts before the axis and runs into it is drawn (clipped).
func _reaches(it: Dictionary, lo: int, hi: int) -> bool:
	var a := int(it["start_year"])
	var z := int(it["end_year"]) if it.has("end_year") else (hi if bool(it.get("open_end", false)) else a)
	return z >= lo and a <= hi

## Snapshot and Ownership rows: only for a selected place (its own spans) or faction (every span it holds), as the old
## spans lane drew them (a world has one ownership span per settlement per owner, thousands in a big one).
func _selection_rows(b: Object) -> Array:
	var sel: Variant = _shell.get("selection") if _shell != null else null
	if not (sel is Dictionary) or (sel as Dictionary).is_empty():
		return []
	var d := {"kinds": PackedStringArray(["Ownership"])}
	var kind := String((sel as Dictionary).get("kind", ""))
	if kind == "faction":
		d["factions"] = PackedInt32Array([int((sel as Dictionary).get("id", -1))])
	else:
		var key := _selection_key()
		if key == "":
			return []
		d["entities"] = PackedStringArray([key])
	var own: Dictionary = b.world_gen.timeline_items(d)
	var snap: Dictionary = b.world_gen.timeline_items({"kinds": PackedStringArray(["Snapshot"])})
	var out: Array = []
	if bool(own.get("ok", false)):
		out.append_array(own["items"])
	if bool(snap.get("ok", false)):
		out.append_array(snap["items"])
	return out

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

## Period bands: the vault's `@` lines (Ruling GT, T8: vault periods only), or centuries across the track when there
## are none. An open-ended period runs to the end of the track.
func _make_bands(lo: int, hi: int) -> Array:
	var out: Array = []
	if not _periods.is_empty():
		var ps: Array = _periods.duplicate()
		ps.sort_custom(func(a, c): return int(a["start_year"]) < int(c["start_year"]))
		for p in ps:
			var d: Dictionary = p
			var band := {"name": String(d["title"]), "from": int(d["start_year"]), "to": int(d["end_year"]) if d.has("end_year") else hi, "synthetic": false}
			if d.has("color"):
				band["color"] = String(d["color"])
			out.append(band)
		return out
	var step := CENTURY if hi - lo <= 24 * CENTURY else 10 * CENTURY
	var y := int(floorf(float(lo) / float(step))) * step
	while y < hi:
		out.append({"name": "%ds" % y, "from": y, "to": y + step, "synthetic": true})
		y += step
	return out

## The period an event's start falls in (the last one starting at or before it), or "".
func _era_of(year: int) -> String:
	var name_ := ""
	for b in _bands:
		var d: Dictionary = b
		if bool(d.get("synthetic", false)):
			return ""
		if int(d["from"]) <= year and year <= int(d["to"]):
			name_ = String(d["name"])
	return name_

## At most `ChronosLanes.LANDMARKS_MAX` landmarks (Ruling GT, T5): the vault's `=` markers first, then conflicts, then
## capitals' foundings; within a tier, if there are more than the room left, spread evenly over the years so the ticks do
## not bunch at one end. Returned in year order.
func _pick_landmarks(b: Object) -> Array:
	var capitals := {}
	for s in b.settlements():
		var sd: Dictionary = s
		if bool(sd.get("capital", false)) or String(sd.get("kind", "")) in ["capital", "metropolis"]:
			capitals[int(sd.get("tid", -1))] = true
	var markers: Array = _events.filter(func(i): return String(i.get("ev_kind", "")) == "marker")
	var conflicts: Array = _events.filter(func(i): return String(i["source"]) == "Conflict")
	var founds: Array = _events.filter(func(i): return String(i["source"]) == "Founding" and capitals.has(int(i.get("entity_id", -1))))
	var out: Array = []
	for tier in [markers, conflicts, founds]:
		var room: int = ChronosLanes.LANDMARKS_MAX - out.size()
		if room <= 0:
			break
		var t: Array = tier.duplicate()
		t.sort_custom(func(a, c): return int(a["start_year"]) < int(c["start_year"]))
		if t.size() <= room:
			out.append_array(t)
		else:
			for k in room:
				out.append(t[int(roundf(float(k) * float(t.size() - 1) / maxf(1.0, float(room - 1))))] if room > 1 else t[t.size() / 2])
	out.sort_custom(func(a, c): return int(a["start_year"]) < int(c["start_year"]))
	return out

func _source_word(it: Dictionary) -> String:
	match String(it["source"]):
		"Note":
			return "Note (live)"
		"Imported":
			return "Note + Imported" if bool(it.get("also_linked", false)) else "Imported"
	return "Cartalith"

func _kind_word(it: Dictionary) -> String:
	var lane := String(it.get("lane", ""))
	return "Note" if lane == "Notes" else lane

## The events the strip's filters leave (empty list = no restriction; groups AND, values OR).
func _filtered() -> Array:
	var out: Array = []
	var q := _s_search.strip_edges().to_lower()
	for it in _events:
		var d: Dictionary = it
		if not _s_kinds.is_empty() and not _s_kinds.has(_kind_word(d)):
			continue
		if not _s_sources.is_empty() and not _s_sources.has(_source_word(d)):
			continue
		if not _s_factions.is_empty():
			var fs: PackedInt32Array = d.get("factions", PackedInt32Array())
			if not _s_factions.any(func(f): return fs.has(int(f))):
				continue
		if q != "" and not (String(d["title"]).to_lower().contains(q) or str(int(d["start_year"])).contains(q)):
			continue
		out.append(d)
	return out

func _filter_count() -> int:
	return (1 if not _s_kinds.is_empty() else 0) + (1 if not _s_sources.is_empty() else 0) + (1 if not _s_factions.is_empty() else 0) + (1 if _s_search.strip_edges() != "" else 0)

# -- the filter bar ----------------------------------------------------------------------------------------

func _build_filters() -> void:
	_filters = HFlowContainer.new()
	_filters.name = "LadderFilters"
	_filters.add_theme_constant_override("h_separation", 6)
	_filters.add_theme_constant_override("v_separation", 4)
	_filters.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	add_child(_filters)
	_search = LineEdit.new()
	_search.name = "LadderSearch"
	_search.placeholder_text = "Search events"
	_search.clear_button_enabled = true
	_search.text = _s_search
	_search.custom_minimum_size = Vector2(150, float(ChronosPrompt.tap_h()))
	_search.text_changed.connect(func(t: String):
		_s_search = t
		_on_filters_changed())
	_filters.add_child(_search)
	for k in ["Kind", "Source", "Faction"]:
		var mb := MenuButton.new()
		mb.name = "Menu_" + String(k)
		mb.flat = false
		mb.custom_minimum_size.y = float(ChronosPrompt.tap_h())
		mb.add_theme_font_override("font", DccTheme.mono())
		mb.add_theme_font_size_override("font_size", maxi(10, _fs()))
		_filters.add_child(mb)
		_menus[k] = mb
	_nof = DccTheme.mono_label("", "text_ghost", _fs(), 0)
	_nof.name = "LadderNOf"
	_filters.add_child(_nof)
	_cluster_chip = DccWidgets.segment(_filters, "", func():
		_s_cluster = []
		_on_filters_changed())
	_cluster_chip.name = "ClusterChip"
	if _compact:
		_cluster_chip.add_theme_font_size_override("font_size", _fs())
	_cluster_chip.custom_minimum_size.y = maxf(_cluster_chip.custom_minimum_size.y, float(ChronosPrompt.tap_h()))
	_doc_btn = DccWidgets.chip(_filters, "Open as document", _open_document, true)
	_doc_btn.name = "OpenAsDocument"
	_doc_btn.tooltip_text = "Write the whole timeline as a read-only Markdown document in Chronos syntax (_cartalith/Consolidated timeline.md in your vault) and open it. It is regenerated each time and never replaces a file of yours. Private notes' events are left out."
	_doc_btn.custom_minimum_size.y = maxf(_doc_btn.custom_minimum_size.y, float(ChronosPrompt.tap_h()))

## The ladder's text size: the strip's role size on desktop and tablet; the phone's body size on the phone, where the
## strip role (10) and `segment()`'s 10 were the finding "count, Back and Filter read about 10 px on the 6T" (2026-11-22).
func _fs() -> int:
	return DccTheme.FS_BODY if _compact else DccTheme.role_px("fs_timeline")

## Fills the three menus from the current model (the faction list changes with the world). A checkable row toggles and
## the popup stays open.
func _rebuild_menus(b: Object) -> void:
	var kinds: Array = KIND_WORDS.duplicate()
	if _events.any(func(i): return String(i.get("lane", "")) == "Ownership"):
		kinds.append("Ownership")
	_fill_menu("Kind", kinds.map(func(k): return [k, k]), _s_kinds)
	_fill_menu("Source", SOURCE_WORDS.map(func(k): return [k, k]), _s_sources)
	var roster: Array = b.get_factions() if b != null else []
	var rows: Array = []
	var ids := {}
	for fa in roster:
		var fd: Dictionary = fa
		ids[int(fd.get("id", -1))] = true
		rows.append([String(fd.get("name", "Faction %d" % int(fd.get("id", 0)))), int(fd.get("id", 0))])
	# a saved faction filter whose faction is gone would silently hide every row (Ruling GK item 3)
	if not bool(b.get("generating")):
		_s_factions = _s_factions.filter(func(f): return ids.has(int(f)))
	_fill_menu("Faction", rows, _s_factions)

func _fill_menu(which: String, rows: Array, active: Array) -> void:
	var mb: MenuButton = _menus[which]
	var pm := mb.get_popup()
	pm.clear()
	pm.hide_on_checkable_item_selection = false
	for pc in pm.id_pressed.get_connections():
		pm.id_pressed.disconnect(pc["callable"])
	for i in rows.size():
		pm.add_check_item(String(rows[i][0]), i)
		pm.set_item_checked(i, active.has(rows[i][1]))
		pm.set_item_metadata(i, rows[i][1])
	mb.text = "%s: %s ▾" % [which, "all" if active.is_empty() else str(active.size())]
	pm.id_pressed.connect(func(id: int):
		var v: Variant = pm.get_item_metadata(id)
		var arr: Array = _s_kinds if which == "Kind" else (_s_sources if which == "Source" else _s_factions)
		if arr.has(v):
			arr.erase(v)
		else:
			arr.append(v)
		pm.set_item_checked(id, arr.has(v))
		mb.text = "%s: %s ▾" % [which, "all" if arr.is_empty() else str(arr.size())]
		_on_filters_changed())

func _on_filters_changed() -> void:
	_shown = _filtered()
	var cur := int(_shell.tl_year()) if _shell != null else 0
	var rng: Dictionary = _shell.tl_range()
	_feed_canvas(int(rng["min"]), int(rng["max"]), cur)
	_sync_visibility()
	_fill_table()

# -- the table (L2) ----------------------------------------------------------------------------------------

func _columns() -> Array:
	if _compact:
		return ["Year", "Event"]
	if DccTheme.is_tablet():
		return ["Year", "Event", "Kind"]
	return ["Year", "Event", "Kind", "Source", "Era"]

func _when_text(it: Dictionary) -> String:
	var s := str(int(it["start_year"]))
	if it.has("end_year"):
		s += "–" + str(int(it["end_year"]))
	elif bool(it.get("open_end", false)):
		s += "–"
	return s

func _fill_table() -> void:
	if _table == null or not _table.visible:
		return
	var cols := _columns()
	_table.clear()
	_table.columns = cols.size()
	for c in cols.size():
		_table.set_column_title(c, String(cols[c]))
		_table.set_column_expand(c, c == 1)
		_table.set_column_custom_minimum_width(c, 92 if c == 0 else (130 if c >= 2 else 0))
	var root := _table.create_item()
	var rows: Array = _shown
	if not _s_cluster.is_empty():
		rows = _shown.filter(func(i): return _s_cluster.has(String(i["key"])))
	rows = rows.duplicate()
	rows.sort_custom(func(a, c): return int(a["start_year"]) < int(c["start_year"]))
	var cur := int(_shell.tl_year()) if _shell != null else 0
	var rng: Dictionary = _shell.tl_range() if _shell != null else {"min": -1000000000, "max": 1000000000}
	for it in rows:
		var d: Dictionary = it
		var row := _table.create_item(root)
		if row == null:
			break
		row.set_metadata(0, d)
		row.set_text(0, _when_text(d))
		var mark: String = {"gen": "●", "note": "○", "imported": "■", "both": "◉"}[ChronosLanes.origin_of(d)]
		row.set_text(1, ("%s %s" % [mark, String(d["title"])]) if _compact else String(d["title"]) + ("  [private]" if bool(d.get("private", false)) else ""))
		## Ruling HF: an event outside the track keeps its year; the table says so (the canvas does not draw it).
		if not _reaches(d, int(rng["min"]), int(rng["max"])):
			row.set_text(1, row.get_text(1) + ("  (before the range)" if int(d["start_year"]) < int(rng["min"]) else "  (after the range)"))
			row.set_tooltip_text(1, row.get_text(1)) ## the column can clip the note on a narrow sheet
		if cols.size() > 2:
			row.set_text(2, _kind_word(d))
		if cols.size() > 3:
			row.set_text(3, _source_word(d))
			row.set_text(4, String(d.get("era", "")))
	_tint_rows()
	_nof.text = "%d of %d" % [rows.size(), _total]
	_table.custom_minimum_size.y = _table_height()
	_table.tooltip_text = ""

## Tints the row that starts at the cursor's year and clears the rest. Called when the cursor moves; **never rebuilds
## the table**, because a row click moves the year from inside the Tree's own signal and a `clear()` there left the
## Tree without a root (`create_item` returned null).
func _tint_rows() -> void:
	if _table == null or _table.get_root() == null:
		return
	var cur := int(_shell.tl_year())
	var row := _table.get_root().get_first_child()
	while row != null:
		var d: Variant = row.get_metadata(0)
		var on: bool = d is Dictionary and int((d as Dictionary)["start_year"]) == cur
		for c in _table.columns:
			if on:
				row.set_custom_bg_color(c, DccTheme.c("accent_wash"))
			else:
				row.clear_custom_bg_color(c)
		row = row.get_next()

func _table_height() -> float:
	if _sheet != null:
		return 200.0
	var vh := float(get_viewport_rect().size.y) if is_inside_tree() else 800.0
	return clampf(vh * (TABLE_SHARE_PHONE if _compact else (TABLE_SHARE_TABLET if DccTheme.is_tablet() else TABLE_SHARE)), 160.0, 420.0)

func _on_table_clicked(_pos: Vector2, button: int) -> void:
	if button != MOUSE_BUTTON_LEFT:
		return
	var row := _table.get_selected()
	if row == null:
		return
	var d: Variant = row.get_metadata(0)
	if d is Dictionary:
		_pick_item(d)

## A mark, a landmark or a row was pressed: the year (and day) it starts on, in one write; then the strip says what it was.
func _pick_item(it: Dictionary) -> void:
	if _shell == null:
		return
	var y := int(it["start_year"])
	if it.has("start_month"):
		_shell.tl_set_date(y, _day_of_year(int(it["start_month"]), int(it.get("start_day", 1))), true)
	else:
		_shell.tl_set_year(y, true)
	_canvas.selected_key = String(it.get("key", ""))
	_picked.text = ChronosLanes.describe(it)
	# touch forms have no spare row: the tapped title takes the count line's place until the next refresh
	var own_row := not (_compact or DccTheme.is_tablet())
	if not own_row:
		_count.text = _picked.text
	_picked.visible = level < 2 and own_row
	_canvas.queue_redraw()

func _open_cluster(hit: Array) -> void:
	_s_cluster = []
	for it in hit:
		_s_cluster.append(String((it as Dictionary)["key"]))
	_cluster_chip.text = "%d events here ✕" % _s_cluster.size()
	_set_level(2)

## The cursor moved (any writer): the canvas redraws its line and the table re-tints its row, no rebuild.
func _on_cursor() -> void:
	if _canvas == null or not is_inside_tree():
		return
	_canvas.set_cursor(int(_shell.tl_year()))
	if level == 2:
		_tint_rows()

## The 0-based day of the 365-day calendar (`chronos::MonthDay::day_of_year`).
static func _day_of_year(month: int, day: int) -> int:
	var before := 0
	for i in range(clampi(month, 1, 12) - 1):
		before += int(MONTH_DAYS[i])
	return before + day - 1

# -- L3 ----------------------------------------------------------------------------------------------------

## "Open as document": the Rust call writes (or refuses) `_cartalith/Consolidated timeline.md`; on success the vault
## editor opens it with its read-only banner (`VaultWindow._build_editor_pane`); on a refusal the strip says why and
## **nothing was written**.
func _open_document() -> void:
	var b := _bridge()
	if b == null or _shell == null:
		return
	var got: Dictionary = b.world_gen.timeline_open_document(String(b.world_gen.get_world_name()))
	if not bool(got.get("ok", false)):
		_note.text = String(got.get("error", "could not write the document"))
		_note.visible = true
		return
	_note.text = "Wrote %s (%d lines). Generated and read-only; it is rewritten each time you open it." % [String(got["rel"]), int(got["written"])]
	_note.visible = true
	var vw: Variant = _shell.get("vault_window")
	if vw != null and is_instance_valid(vw):
		vw.note_written_elsewhere(String(got["rel"]))
		vw.open_note(String(got["rel"]))

# -- the passive badge -------------------------------------------------------------------------------------

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

## "1 vault timeline new" beside the counts (the design's L0 count line), a button that opens the quiet prompt.
func _update_badge() -> void:
	if _badge == null:
		return
	var n := attention_count()
	_badge.visible = n > 0
	if n > 0:
		_badge.text = "%d vault timeline%s new" % [n, "" if n == 1 else "s"]
		_badge.tooltip_text = "Timelines found in your notes. Nothing has been changed; press to look at each."

func _open_prompt() -> void:
	var d: AcceptDialog = ChronosPrompt.new()
	_shell.add_child(d)
	d.setup(_shell, candidates(), func():
		invalidate_scan()
		_start_scan()
		_refresh(), _events)
	d.popup_centered(Vector2i(760, 560) if not _shell.is_phone() else Vector2i(420, 760))
