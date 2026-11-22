extends Control
## The spans lane of the consolidated timeline (`CHRONOS_TIMELINE_DESIGN.md` s.2.3; CT-2 leftovers 4 and 5):
## every item of the model that has a length -- an ownership span (Ownership lane, derived from the
## civilisation timeline's own snapshots by `civ_settlement_ownership_periods`), a conflict, a journey whose
## saved plan gives `calendar_days` -- drawn as a bar on one year axis, packed into rows so none overlap,
## with the cursor as a vertical line. It owns no data: `chronos_items.gd` hands it the rows
## `WorldGen.timeline_items` answered, and a press calls `on_pick(item)` (the strip's one year writer).
##
## **A bar is never narrower than `MIN_BAR_PX`**: a journey of 30 days on a 2 000-year axis is a sub-pixel
## width, and an item that cannot be seen is the lie this lane exists to remove (a journey drawn as a point
## was the leftover). The minimum is a labelled judgement, the width of a finger-readable tick.
## **It draws nothing it was not given**; with no spans it is not shown at all.

const MIN_BAR_PX := 4.0
const ROW_PX := 9
const ROW_GAP := 2
## At most this many rows are drawn (labelled judgement, the strip is 4-5 rows of chips tall already);
## bars that need a further row are counted in `overflow`, never silently dropped.
const ROWS_MAX := 6

var items: Array = []
var cursor_year := 0
## `Callable(item: Dictionary)`, called when a bar is pressed.
var on_pick: Callable = Callable()
## Bars that did not fit `ROWS_MAX` rows.
var overflow := 0

var _lo := 0.0
var _hi := 1.0
## `[{rect: Rect2, item: Dictionary}]` in local px, rebuilt by `_layout`; the hit test and the probe read it.
var bars: Array = []

func _init() -> void:
	name = "ChronosSpans"
	mouse_filter = Control.MOUSE_FILTER_STOP

## Replaces the rows and re-packs. `year_now` is the cursor.
func set_items(rows: Array, year_now: int) -> void:
	items = rows
	cursor_year = year_now
	visible = not items.is_empty()
	_layout()
	queue_redraw()

func _year_end(it: Dictionary) -> float:
	if it.has("end_year"):
		return float(it["end_year"])
	# an open range runs to the cursor (or its own start, whichever is later)
	return maxf(float(cursor_year), float(it["start_year"]))

func _x(year: float, w: float) -> float:
	return (year - _lo) / maxf(_hi - _lo, 1.0) * w

## Greedy interval packing over the year axis; rows are the lanes, `bars` the result.
func _layout() -> void:
	bars = []
	overflow = 0
	if items.is_empty():
		custom_minimum_size.y = 0
		return
	_lo = INF
	_hi = -INF
	for it in items:
		_lo = minf(_lo, float(it["start_year"]))
		_hi = maxf(_hi, _year_end(it))
	_lo = minf(_lo, float(cursor_year))
	_hi = maxf(_hi, float(cursor_year))
	var w := maxf(size.x, 200.0)
	var sorted: Array = items.duplicate()
	sorted.sort_custom(func(a, b): return float(a["start_year"]) < float(b["start_year"]))
	var row_end: Array = []
	for it in sorted:
		var x0 := _x(float(it["start_year"]), w)
		var x1 := maxf(_x(_year_end(it), w), x0 + MIN_BAR_PX)
		var r := -1
		for i in row_end.size():
			if float(row_end[i]) + 1.0 <= x0:
				r = i
				break
		if r < 0:
			if row_end.size() >= ROWS_MAX:
				overflow += 1
				continue
			row_end.append(0.0)
			r = row_end.size() - 1
		row_end[r] = x1
		bars.append({"rect": Rect2(x0, r * (ROW_PX + ROW_GAP), x1 - x0, ROW_PX), "item": it})
	custom_minimum_size.y = maxf(1.0, float(row_end.size() * (ROW_PX + ROW_GAP)))

func _notification(what: int) -> void:
	if what == NOTIFICATION_RESIZED:
		_layout()
		queue_redraw()

func _color_of(it: Dictionary) -> Color:
	match String(it.get("source", "")):
		"Ownership":
			return DccTheme.c("accent")
		"Conflict":
			return DccTheme.c("warn")
		_:
			return DccTheme.c("text_dim")

func _draw() -> void:
	for b in bars:
		var it: Dictionary = b["item"]
		draw_rect(b["rect"], _color_of(it))
		if bool(it.get("open_end", false)):
			# an open range ends in a tip, so it reads as "still going", not as a short bar
			var r: Rect2 = b["rect"]
			draw_colored_polygon(PackedVector2Array([Vector2(r.end.x, r.position.y - 1), Vector2(r.end.x + 4, r.position.y + r.size.y * 0.5), Vector2(r.end.x, r.end.y + 1)]), _color_of(it))
	var cx := _x(float(cursor_year), maxf(size.x, 200.0))
	draw_line(Vector2(cx, 0), Vector2(cx, size.y), DccTheme.c("text"), 1.0)

func _bar_at(p: Vector2) -> Dictionary:
	for b in bars:
		if (b["rect"] as Rect2).grow(2.0).has_point(p):
			return b["item"]
	return {}

func _get_tooltip(at_position: Vector2) -> String:
	var it := _bar_at(at_position)
	if it.is_empty():
		return ""
	return "%s · %s (%d to %s)" % [String(it["source"]), String(it["title"]), int(it["start_year"]), str(int(it["end_year"])) if it.has("end_year") else "now"]

func _gui_input(ev: InputEvent) -> void:
	if ev is InputEventMouseButton and (ev as InputEventMouseButton).pressed and (ev as InputEventMouseButton).button_index == MOUSE_BUTTON_LEFT:
		var it := _bar_at((ev as InputEventMouseButton).position)
		if not it.is_empty() and on_pick.is_valid():
			on_pick.call(it)
			accept_event()
