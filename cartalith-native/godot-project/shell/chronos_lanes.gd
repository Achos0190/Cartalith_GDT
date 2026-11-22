extends Control
## The timeline ladder's drawing: L0 (a minimal strip) and L1 (four lanes) over one year axis (Ruling GT,
## `design/timeline-designs-2026-11-22/timeline_designs.html` option C; `CHRONOS_TIMELINE_DESIGN.md` s.10 build note
## "Ruling GT"). It replaces `chronos_spans.gd`'s bars-only lane.
##
## **It owns no data and no clock.** `chronos_items.gd` hands it the track (`lo`, `hi` = the strip's `tl_range()`), the
## period bands, the events, the L0 landmarks and the cursor; a click or tap answers through `on_pick(item)` (the
## strip's one year writer), `on_cluster(items)` (L1: more than one mark under the finger, so L2 filtered to them),
## `on_expand()` (L0: a press that hit no landmark) and `on_swipe(dir)` (phone: -1 up a level, +1 down).
##
## **L0** draws the period bands (with their names where they fit), at most `LANDMARKS_MAX` landmark ticks, the cursor
## and the axis. No chips, no text but the band names and the axis years.
## **L1** adds one row of lanes per `LANES` entry (plus `Ownership` when the strip hands it rows for a selection):
## every event is one mark, points apart from ranges, packed into at most `SUB_ROWS_MAX` sub-rows so overlapping marks
## do not hide each other; a label is drawn only where it fits before the next mark of its sub-row (never wrapped,
## never overlapping). **The mark's shape says where it came from** and its colour says nothing, except that a vault
## `#colour` tag colours that mark only (T10): generated = filled disc/bar; a live note = a ring/outlined bar; an
## imported copy = a filled square/bar with a square cap; **both, shown once** = a ring with a dot.
## **It draws nothing it was not given**, and an item it cannot place (no year) is not drawn rather than drawn at 0.

const LANES := ["Founding", "Conflict", "Journey", "Notes"]
## The lane names as the strip prints them: the desktop's, the tablet's shorter ones, and the phone's none
## (`design` s.C "Every level": unlabelled lanes on the phone; a tap shows one title below).
const LANE_TITLES := {"Founding": "Founding", "Conflict": "Conflict", "Journey": "Journey", "Notes": "From the vault", "Ownership": "Ownership"}
const LANE_TITLES_SHORT := {"Founding": "Founding", "Conflict": "Conflict", "Journey": "Journey", "Notes": "Vault", "Ownership": "Owners"}
## At most this many landmark ticks at L0 (Ruling GT, T5).
const LANDMARKS_MAX := 5
## Sub-rows a lane may grow to; marks that need a further one share the last (a labelled judgement: three rows keep the
## strip under a third of a phone window at 30 px).
const SUB_ROWS_MAX := 3
## The hit tolerance in px: a finger on touch (the design's "nearest item within 22 px"), a pointer otherwise.
const PICK_R_TOUCH := 22.0
const PICK_R_POINTER := 7.0
## A vertical drag this long, and clearly more vertical than horizontal, is a swipe (phone only).
const SWIPE_PX := 40.0
## Movement beyond this between press and release is a drag, not a click.
const DRAG_PX := 12.0
## A lane with no marks is this tall (px): its title and a rule, not a row of targets.
const EMPTY_LANE_H := 14.0
## Chronos' eight named colours (README, "Colour is opt-in and small"), as the shell paints them. Anything else is read as hex.
const CHRONOS_COLOURS := {
	"red": Color("d9534f"), "orange": Color("e8913a"), "yellow": Color("d8b830"), "green": Color("4aa564"),
	"blue": Color("4a8fd9"), "purple": Color("8a63c9"), "grey": Color("8a8f98"), "pink": Color("d96fa8"),
}

var level := 0
var lo := 0
var hi := 1
var cursor_year := 0
## `[{name, from, to, color?, synthetic}]`, ordered; `synthetic` = centuries standing in for no `@` lines.
var bands: Array = []
## The marks of L1: every event of the model after the strip's filters.
var events: Array = []
## L0's at most five landmark items.
var landmarks: Array = []
var on_pick: Callable = Callable()
var on_cluster: Callable = Callable()
var on_expand: Callable = Callable()
var on_swipe: Callable = Callable()
var phone := false
var touch := false
var short_titles := false
## The item the strip last moved the year to, ringed in the accent.
var selected_key := ""

## `[{rect: Rect2, item: Dictionary, lane: String, row: int, label: String}]` in local px, rebuilt by `_layout()`;
## probes read it.
var marks: Array = []
## `{lane: Rect2}` the lanes' bands, for labels and hit rows.
var lane_rects: Dictionary = {}
var _plot_x0 := 0.0
var _plot_x1 := 1.0
var _band_h := 14.0
var _axis_h := 14.0
var _press := Vector2.ZERO
var _pressing := false

func _init() -> void:
	name = "ChronosLanes"
	mouse_filter = Control.MOUSE_FILTER_STOP

func _font() -> Font:
	return DccTheme.mono()

func _fs() -> int:
	return maxi(9, DccTheme.role_px("fs_timeline"))

func _tw(s: String) -> float:
	return _font().get_string_size(s, HORIZONTAL_ALIGNMENT_LEFT, -1, _fs()).x

## One call to replace everything the canvas shows.
func set_model(p_level: int, p_lo: int, p_hi: int, p_cursor: int, p_bands: Array, p_events: Array, p_landmarks: Array) -> void:
	level = p_level
	lo = p_lo
	hi = maxi(p_hi, p_lo + 1)
	cursor_year = p_cursor
	bands = p_bands
	events = p_events
	landmarks = p_landmarks
	_layout()
	queue_redraw()

func set_cursor(y: int) -> void:
	cursor_year = y
	queue_redraw()

func _x(year: float) -> float:
	return _plot_x0 + (year - float(lo)) / float(hi - lo) * (_plot_x1 - _plot_x0)

func _end_of(it: Dictionary) -> float:
	if it.has("end_year"):
		return float(it["end_year"])
	if bool(it.get("open_end", false)):
		return maxf(float(hi), float(it["start_year"]))
	return float(it["start_year"])

func _is_range(it: Dictionary) -> bool:
	return it.has("end_year") or bool(it.get("open_end", false))

func _lane_h() -> float:
	return 30.0 if touch else 22.0

func _sub_h() -> float:
	return 22.0 if touch else 16.0

## The lanes this model draws: the four of the design, then Ownership only when the strip gave it rows.
func lane_names() -> Array:
	var out: Array = LANES.duplicate()
	for it in events:
		if String((it as Dictionary).get("lane", "")) == "Ownership":
			out.append("Ownership")
			break
	return out

## Greedy interval packing per lane, then labels where they fit. Sets `custom_minimum_size.y`.
func _layout() -> void:
	marks = []
	lane_rects = {}
	var w := maxf(size.x, 200.0)
	var gutter := 0.0
	if level >= 1 and not phone:
		gutter = 96.0 if not short_titles else 58.0
	_plot_x0 = gutter + 4.0
	_plot_x1 = w - 8.0
	if level == 0:
		custom_minimum_size.y = float(maxi(44, DccTheme.role_px("btn_min_h"))) if (touch or phone) else 44.0
		return
	var y := _band_h
	for ln in lane_names():
		var lane: String = ln
		var rows_end: Array = []
		var lane_marks: Array = []
		var items: Array = events.filter(func(i): return String(i.get("lane", "")) == lane)
		items.sort_custom(func(a, b): return int(a["start_year"]) < int(b["start_year"]))
		for it in items:
			var d: Dictionary = it
			var x0 := maxf(_x(float(d["start_year"])), _plot_x0)
			var x1 := maxf(minf(_x(_end_of(d)), _plot_x1), x0 + 8.0) if _is_range(d) else x0 + 8.0
			x0 -= 4.0 if not _is_range(d) else 0.0
			var r := -1
			for i in rows_end.size():
				if float(rows_end[i]) + 2.0 <= x0:
					r = i
					break
			if r < 0:
				if rows_end.size() < SUB_ROWS_MAX:
					rows_end.append(0.0)
					r = rows_end.size() - 1
				else:
					r = SUB_ROWS_MAX - 1
			rows_end[r] = maxf(float(rows_end[r]), x1)
			lane_marks.append({"x0": x0, "x1": x1, "item": d, "row": r, "lane": lane, "label": ""})
		var nrows := maxi(1, rows_end.size())
		# a lane with nothing in it keeps its title but not its height (the tablet's strip is short of room)
		var lh := maxf(_lane_h(), float(nrows) * _sub_h() + 4.0) if not items.is_empty() else EMPTY_LANE_H
		lane_rects[lane] = Rect2(0.0, y, w, lh)
		# labels: a mark's title goes right of it, only where it ends before the next mark of its sub-row
		for r in nrows:
			var row_marks: Array = lane_marks.filter(func(m): return int(m["row"]) == r)
			for i in row_marks.size():
				var m: Dictionary = row_marks[i]
				var lim: float = float((row_marks[i + 1] as Dictionary)["x0"]) - 4.0 if i + 1 < row_marks.size() else _plot_x1
				var title := String((m["item"] as Dictionary)["title"])
				var room := lim - (float(m["x1"]) + 4.0)
				if not phone and room >= _tw(title):
					m["label"] = title
		for m in lane_marks:
			var rr := int((m as Dictionary)["row"])
			var cy := y + 2.0 + (float(rr) + 0.5) * (lh - 4.0) / float(nrows)
			marks.append({"rect": Rect2(float(m["x0"]), cy - _sub_h() * 0.5, float(m["x1"]) - float(m["x0"]), _sub_h()),
				"item": m["item"], "lane": m["lane"], "row": rr, "label": m["label"], "cy": cy})
		y += lh
	custom_minimum_size.y = y + _axis_h + 4.0

func _notification(what: int) -> void:
	if what == NOTIFICATION_RESIZED:
		_layout()
		queue_redraw()

func _tint(it: Dictionary, fallback: Color) -> Color:
	var c := String(it.get("color", ""))
	if c == "":
		return fallback
	if CHRONOS_COLOURS.has(c.to_lower()):
		return CHRONOS_COLOURS[c.to_lower()]
	if Color.html_is_valid(c):
		return Color.html(c)
	return fallback

## How a mark is drawn is where it came from (see the file header); returns "gen", "note", "imported" or "both".
static func origin_of(it: Dictionary) -> String:
	var s := String(it.get("source", ""))
	if s == "Imported":
		return "both" if bool(it.get("also_linked", false)) else "imported"
	if s == "Note":
		return "note"
	return "gen"

func _draw() -> void:
	var f := _font()
	var fs := _fs()
	var w := maxf(size.x, 200.0)
	var h := maxf(size.y, custom_minimum_size.y)
	var ink := DccTheme.c("text")
	var dim := DccTheme.c("text_dim")
	var ghost := DccTheme.c("text_ghost")
	var plot_h := h - _axis_h
	# period bands, behind everything; alternating washes so adjacent periods read as two
	var wash_a := DccTheme.c("accent_wash")
	var wash_b := DccTheme.c("accent_wash_2")
	for i in bands.size():
		var b: Dictionary = bands[i]
		var x0 := clampf(_x(float(b["from"])), _plot_x0, _plot_x1)
		var x1 := clampf(_x(float(b["to"])), _plot_x0, _plot_x1)
		if x1 - x0 < 1.0:
			continue
		var col := _tint(b, wash_a if i % 2 == 0 else wash_b)
		if col != wash_a and col != wash_b:
			col.a = 0.22
		draw_rect(Rect2(x0, 0.0, x1 - x0, plot_h), col)
		var nm := String(b.get("name", ""))
		if nm != "" and _tw(nm) + 6.0 <= x1 - x0:
			draw_string(f, Vector2(x0 + 4.0, _band_h - 3.0 if level >= 1 else plot_h * 0.5 + fs * 0.35), nm.to_upper(), HORIZONTAL_ALIGNMENT_LEFT, x1 - x0 - 4.0, fs - 1, ghost)
	if level == 0:
		for it in landmarks:
			var d: Dictionary = it
			var x := _x(float(d["start_year"]))
			var col := _tint(d, dim if String(d.get("source", "")) != "Conflict" else DccTheme.c("warn"))
			draw_line(Vector2(x, 4.0), Vector2(x, plot_h - 4.0), col, 1.5)
			draw_colored_polygon(PackedVector2Array([Vector2(x - 3.5, 4.0), Vector2(x + 3.5, 4.0), Vector2(x, 10.0)]), col)
			if String(d.get("key", "")) == selected_key:
				draw_arc(Vector2(x, 7.0), 6.0, 0.0, TAU, 16, DccTheme.c("accent"), 1.5)
	else:
		# lane separators and titles
		for ln in lane_rects:
			var r: Rect2 = lane_rects[ln]
			draw_line(Vector2(0.0, r.position.y), Vector2(w, r.position.y), DccTheme.c("line"), 1.0)
			if not phone:
				var titles: Dictionary = LANE_TITLES_SHORT if short_titles else LANE_TITLES
				draw_string(f, Vector2(4.0, r.position.y + minf(r.size.y, _lane_h()) * 0.5 + fs * 0.35), String(titles.get(ln, ln)), HORIZONTAL_ALIGNMENT_LEFT, _plot_x0 - 8.0, fs, dim)
		for m in marks:
			_draw_mark(m, f, fs, ink)
	# the cursor's year label owns its slot on the axis row; a tick label that would touch it is skipped
	var cs := str(cursor_year)
	var ctw := _tw(cs)
	var cxl := _x(float(clampi(cursor_year, lo, hi)))
	var clx := clampf(cxl - ctw * 0.5, 0.0, w - ctw)
	# axis: nice ticks, labels that do not collide
	var span := float(hi - lo)
	var step := _nice_step(span / 6.0)
	var t := ceilf(float(lo) / step) * step
	var last_x := -1000.0
	while t <= float(hi):
		var x := _x(t)
		draw_line(Vector2(x, plot_h), Vector2(x, plot_h + 3.0), ghost, 1.0)
		var s := str(int(t))
		var tw := _tw(s)
		if x - tw * 0.5 > last_x + 6.0 and x + tw * 0.5 < w and (x + tw * 0.5 < clx - 4.0 or x - tw * 0.5 > clx + ctw + 4.0):
			draw_string(f, Vector2(x - tw * 0.5, h - 2.0), s, HORIZONTAL_ALIGNMENT_LEFT, -1, fs - 1, ghost)
			last_x = x + tw * 0.5
		t += step
	# the cursor
	var cx := _x(float(clampi(cursor_year, lo, hi)))
	draw_line(Vector2(cx, 0.0), Vector2(cx, plot_h), ink, 1.0)
	var lx := clx
	draw_rect(Rect2(lx - 2.0, h - _axis_h + 1.0, ctw + 4.0, _axis_h - 1.0), DccTheme.c("bg"))
	draw_string(f, Vector2(lx, h - 2.0), cs, HORIZONTAL_ALIGNMENT_LEFT, -1, fs - 1, DccTheme.c("accent"))

static func _nice_step(raw: float) -> float:
	var p := pow(10.0, floorf(log(maxf(raw, 1.0)) / log(10.0)))
	for m in [1.0, 2.0, 5.0, 10.0]:
		if p * m >= raw:
			return maxf(1.0, p * m)
	return maxf(1.0, p * 10.0)

func _draw_mark(m: Dictionary, f: Font, fs: int, ink: Color) -> void:
	var it: Dictionary = m["item"]
	var r: Rect2 = m["rect"]
	var cy: float = m["cy"]
	var base := DccTheme.c("warn") if String(it.get("source", "")) == "Conflict" else (DccTheme.c("accent") if String(it.get("source", "")) == "Ownership" else ink)
	var col := _tint(it, base)
	var o := origin_of(it)
	var x0 := r.position.x + (0.0 if _is_range(it) else 4.0)
	var is_marker := String(it.get("ev_kind", "")) == "marker"
	if _is_range(it):
		var bar := Rect2(r.position.x, cy - 3.5, maxf(r.size.x, 6.0), 7.0)
		match o:
			"note":
				draw_rect(bar, col, false, 1.5)
			"both":
				draw_rect(bar, col, false, 1.5)
				draw_circle(Vector2(bar.position.x + 4.0, cy), 2.0, col)
			"imported":
				draw_rect(bar, Color(col, 0.65))
				draw_rect(Rect2(bar.end.x - 3.0, bar.position.y, 3.0, 7.0), col)
			_:
				draw_rect(bar, col)
		if bool(it.get("open_end", false)):
			draw_colored_polygon(PackedVector2Array([Vector2(bar.end.x, cy - 4.5), Vector2(bar.end.x + 4.0, cy), Vector2(bar.end.x, cy + 4.5)]), col)
	elif is_marker:
		draw_colored_polygon(PackedVector2Array([Vector2(x0, cy - 5.0), Vector2(x0 + 5.0, cy), Vector2(x0, cy + 5.0), Vector2(x0 - 5.0, cy)]), col)
	else:
		match o:
			"note":
				draw_arc(Vector2(x0, cy), 4.0, 0.0, TAU, 14, col, 1.5)
			"both":
				draw_arc(Vector2(x0, cy), 5.0, 0.0, TAU, 14, col, 1.5)
				draw_circle(Vector2(x0, cy), 2.0, col)
			"imported":
				draw_rect(Rect2(x0 - 3.5, cy - 3.5, 7.0, 7.0), col)
			_:
				draw_circle(Vector2(x0, cy), 4.0, col)
	if String(it.get("key", "")) == selected_key:
		draw_rect(Rect2(r.position.x - 3.0, cy - 8.0, maxf(r.size.x, 8.0) + 6.0, 16.0), DccTheme.c("accent"), false, 1.5)
	var lab: String = m["label"]
	if lab != "":
		draw_string(f, Vector2(r.end.x + 4.0, cy + fs * 0.35), lab, HORIZONTAL_ALIGNMENT_LEFT, -1, fs, DccTheme.c("text_dim"))

## The marks under local point `p`: in `p`'s own sub-row, those whose extent grown by the pick radius reaches `p.x`.
func marks_at(p: Vector2) -> Array:
	var rad := PICK_R_TOUCH if touch else PICK_R_POINTER
	var out: Array = []
	if level == 0:
		for it in landmarks:
			var d: Dictionary = it
			if absf(_x(float(d["start_year"])) - p.x) <= rad:
				out.append(d)
		return out
	# the row `p` is in: the nearest mark row vertically within the lane under it
	var best_row := -1
	var best_lane := ""
	var best_dy := INF
	for m in marks:
		var dy := absf(float(m["cy"]) - p.y)
		if dy < best_dy:
			best_dy = dy
			best_row = int(m["row"])
			best_lane = String(m["lane"])
	if best_dy > _sub_h() * 0.75:
		return out
	for m in marks:
		if int(m["row"]) != best_row or String(m["lane"]) != best_lane:
			continue
		var r: Rect2 = m["rect"]
		if p.x >= r.position.x - rad and p.x <= r.end.x + rad:
			out.append(m["item"])
	return out

func _get_tooltip(at_position: Vector2) -> String:
	var hit := marks_at(at_position)
	if hit.is_empty():
		return ""
	if hit.size() > 1:
		return "%d events here. Press to list them." % hit.size()
	return describe(hit[0])

## One line naming an item, where it came from and, for a note event that is also imported, both origins.
static func describe(it: Dictionary) -> String:
	var when := str(int(it["start_year"]))
	if it.has("end_year"):
		when += " to " + str(int(it["end_year"]))
	elif bool(it.get("open_end", false)):
		when += " to now"
	var src := String(it.get("source", ""))
	var origin := ""
	match origin_of(it):
		"note":
			origin = " · note (live)"
		"imported":
			origin = " · imported"
		"both":
			origin = " · note (live) + imported"
	if it.has("rel") and origin != "":
		origin += " · " + String(it["rel"]).get_file().get_basename()
	return "%s · %s (%s)%s" % [src if origin == "" else "Vault", String(it["title"]), when, origin]

func _gui_input(ev: InputEvent) -> void:
	if ev is InputEventMouseButton and (ev as InputEventMouseButton).button_index == MOUSE_BUTTON_LEFT:
		var mb := ev as InputEventMouseButton
		if mb.pressed:
			_press = mb.position
			_pressing = true
			accept_event()
			return
		if not _pressing:
			return
		_pressing = false
		accept_event()
		var d := mb.position - _press
		# a drag is not a click: past a finger's wobble the release ends the gesture, whether or not it was a swipe
		if d.length() > DRAG_PX and not (phone and absf(d.y) >= SWIPE_PX and absf(d.y) > 1.5 * absf(d.x)):
			return
		if phone and absf(d.y) >= SWIPE_PX and absf(d.y) > 1.5 * absf(d.x) and on_swipe.is_valid():
			on_swipe.call(-1 if d.y < 0.0 else 1)
			return
		_click(mb.position)

func _click(p: Vector2) -> void:
	var hit := marks_at(p)
	if hit.size() == 1:
		if on_pick.is_valid():
			on_pick.call(hit[0])
	elif hit.size() > 1 and level >= 1:
		if on_cluster.is_valid():
			on_cluster.call(hit)
	elif hit.size() > 1:
		if on_pick.is_valid():
			on_pick.call(hit[0])
	elif level == 0 and on_expand.is_valid():
		on_expand.call()
