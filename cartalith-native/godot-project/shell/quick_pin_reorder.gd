extends RefCounted
## CQ-6's touch reorder (`CUSTOM_QUICK_BAR_SCOPE.md` CQ-6; the section 12 table's "long-press then drag"): a pin held under a
## finger for the long-press time is LIFTED, dragged along its bar (the tablet's chip row, a horizontal axis) or its list (the
## phone's drawer, a vertical one), and dropped in a new place. One implementation for both; the two callers own the gesture's
## start and the store, this owns the lifted state, the drop marker and the answer.
##
## ## What it is, and the order it commits
##
## The caller hands it `items`, the pin controls that are on screen, in order (the Assistant chip, `＋`, `⋯` and the drawer's
## Customise row are never in it: they are not pins and cannot move). `update()` takes the finger's position in GLOBAL canvas
## coordinates and finds the drop index: the number of OTHER items whose centre the finger has passed, which is the index
## the lifted pin takes in the list once it is out of its old place. The marker (a thin accent bar, a `top_level` child of the
## host so no container lays it out) sits in the gap that index names. `finish()` answers `{moved, from, to}`; the caller moves
## the stored pin (`DccSettings.quick_bar_move_visible_to`) and redraws.
##
## ## What it must never do
##
## Write the store itself (the caller does, through the one order function the Customise window's Up/Down and the chip menu's
## Move left/right also use), draw anything while no pin is lifted, or leave a pin dimmed after `finish()`/`cancel()`.
## The order is the user's own `[quick_bar]` user setting; it is never saved into a world (`DccSettings`, scope section 0).

## A lifted pin's alpha, and the marker's thickness in canvas units. Labelled judgements: dim enough to read as "picked up",
## not so dim that its label is lost; 3 units is visible on a 44-unit chip without hiding a neighbour's border.
const LIFT_ALPHA := 0.45
const MARKER_THICK := 3.0

var _host: Control
var _horizontal := true
## `func() -> Array` of the pin controls on screen, in order.
var _items: Callable
var _marker: ColorRect
var _item: Control
var _from := -1
var _to := -1
var _origin := 0.0
var _travelled := 0.0
var active := false

## `host` carries the drop marker; `horizontal` is the axis the items run along; `items` is `func() -> Array`.
func _init(host: Control, horizontal: bool, items: Callable) -> void:
	_host = host
	_horizontal = horizontal
	_items = items

## The lifted control (for tests), null while nothing is lifted.
func item() -> Control:
	return _item

## The drop index the finger names now, `-1` while nothing is lifted (for tests).
func target() -> int:
	return _to

## Lifts `item` with the finger at `global_pos`. False (and nothing lifted) when `item` is not one of the items on screen.
func begin(item: Control, global_pos: Vector2) -> bool:
	var list: Array = _items.call()
	var at := list.find(item)
	if at < 0:
		return false
	cancel()
	_item = item
	_from = at
	_to = at
	_origin = _axis(global_pos)
	_travelled = 0.0
	active = true
	item.modulate.a = LIFT_ALPHA
	_marker = ColorRect.new()
	_marker.name = "ReorderMarker"
	_marker.color = DccTheme.c("accent")
	_marker.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_marker.top_level = true
	_marker.z_index = 20
	_host.add_child(_marker)
	_place_marker()
	return true

## Follows the finger: finds the drop index and moves the marker to its gap.
func update(global_pos: Vector2) -> void:
	if not active or not is_instance_valid(_item):
		return
	_travelled = maxf(_travelled, absf(_axis(global_pos) - _origin))
	var others: Array = _items.call().filter(func(c): return c != _item)
	var p := _axis(global_pos)
	var to := 0
	for c in others:
		if _centre(c as Control) < p:
			to += 1
	_to = to
	_place_marker()

## How far the finger has gone from where it lifted, along the axis, in canvas units.
func travelled() -> float:
	return _travelled

## Ends the gesture: un-dims the pin, removes the marker. `moved` is whether the drop index differs from the pin's own place.
func finish() -> Dictionary:
	var out := {"moved": active and _to != _from, "from": _from, "to": _to}
	cancel()
	return out

## Drops the lifted state without a result (a press elsewhere, a freed pin, a swap).
func cancel() -> void:
	if _item != null and is_instance_valid(_item):
		_item.modulate.a = 1.0
	if _marker != null and is_instance_valid(_marker):
		if _marker.get_parent() != null:
			_marker.get_parent().remove_child(_marker)
		_marker.queue_free()
	_marker = null
	_item = null
	_from = -1
	_to = -1
	active = false

func _axis(p: Vector2) -> float:
	return p.x if _horizontal else p.y

func _centre(c: Control) -> float:
	var r := c.get_global_rect()
	return _axis(r.get_center())

## Puts the marker in the gap the drop index names: before the item now at that index among the others, or after the last.
func _place_marker() -> void:
	if _marker == null:
		return
	var others: Array = _items.call().filter(func(c): return c != _item)
	if others.is_empty():
		_marker.visible = false
		return
	_marker.visible = true
	var r: Rect2
	var before := _to < others.size()
	r = (others[_to] as Control).get_global_rect() if before else (others[others.size() - 1] as Control).get_global_rect()
	if _horizontal:
		var x := (r.position.x - 2.0 - MARKER_THICK * 0.5) if before else (r.end.x + 2.0 - MARKER_THICK * 0.5)
		_marker.global_position = Vector2(x, r.position.y)
		_marker.size = Vector2(MARKER_THICK, r.size.y)
	else:
		var y := (r.position.y - 2.0 - MARKER_THICK * 0.5) if before else (r.end.y + 2.0 - MARKER_THICK * 0.5)
		_marker.global_position = Vector2(r.position.x, y)
		_marker.size = Vector2(r.size.x, MARKER_THICK)

## What a lifted pin's event means, from either form a finger arrives in: `"motion"` (a mouse move or a screen drag),
## `"release"` (the left button or the touch lifting) or `""`. A finger arrives as an `InputEventScreenTouch` AND its emulated
## mouse event, and a control may be handed only one of the two on the lift (the screen touch's release reached a chip while the
## emulated mouse release did not, measured in `_quickbar_tablet_probe.gd`), so BOTH end the lift; the second finds nothing lifted.
static func kind_of(ev: InputEvent) -> String:
	if ev is InputEventMouseMotion or ev is InputEventScreenDrag:
		return "motion"
	if ev is InputEventMouseButton and (ev as InputEventMouseButton).button_index == MOUSE_BUTTON_LEFT and not (ev as InputEventMouseButton).pressed:
		return "release"
	if ev is InputEventScreenTouch and not (ev as InputEventScreenTouch).pressed:
		return "release"
	return ""

## The event's position in its control's coordinates (every one of the four classes above carries `position`).
static func pos_of(ev: InputEvent) -> Vector2:
	return (ev as Object).get("position")
