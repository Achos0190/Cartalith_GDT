extends Node
## Click-through probe: the marker information card does not eat the next map click
## (owner choice "Click-through", 2026-11-14; `OUTSTANDING_WORK.md` row "Marker card click-through",
## `STATUS.md` "Marker card click-through").
##
##   godot --path . --resolution 1600x1000 _ctthru_probe.tscn
##
## **Windowed, never `--headless`**, from a wiped scratch user dir (`APPDATA` pointed at an empty
## folder: Godot 4.7 has no `--user-data-dir`). Real input only, through `Input.parse_input_event()`.
## Exit 0 pass, 1 fail, 2 abort (bad argument, watchdog, no world).
##
## What it protects. CL-3 (`d2a8b6a3`) opens the information card on a left click on a marker, and the card
## was a `PopupPanel` whose outside click Godot closes AND consumes: the next click on the map only closed the
## card. The owner's rule is that a click elsewhere closes the card and does what it would otherwise have done.
## With a card open on marker A:
##   B  a click on another settlement selects it AND opens ITS card, in that one click;
##   E  a click on an empty map point closes the card and clears the selection (bare ground's own action);
##   S  the same for a third settlement (a settlement click is a select);
##   W  a wheel notch on the map zooms it, and the card is gone;
##   M  a middle-button drag on the map pans it, and the card is gone;
##   K  a shortcut chord (Ctrl+D) pressed with a card open reaches the shell (selection cleared) and closes the card;
##   I  a click INSIDE the card leaves it open and the selection alone;
##   X  Escape closes the card and changes nothing else;
##   R  a RIGHT-click card is the same `ContextCard` and behaves the same way: the outside left click selects
##      the settlement under it and opens that one's card (`MAP_CONTEXT_SCOPE.md` 9.4's "tap outside closes the
##      surface only" is superseded for the card by the owner's choice; the ring keeps its own rule).
## Every leg asserts the card's state as well as the effect, and the control legs (a click with no card open)
## prove a click does select, so a leg cannot pass because the click never reached the map.

var _fails := 0
var _checks := 0
var _app: Node
var _bridge


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _move(at: Vector2, mask := 0) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 1)
	mm.button_mask = mask
	Input.parse_input_event(mm)
	await _settle(2)


func _button(at: Vector2, idx: MouseButton, down: bool) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = idx
	ev.pressed = down
	ev.position = at
	ev.global_position = at
	if idx == MOUSE_BUTTON_MIDDLE and down:
		ev.button_mask = MOUSE_BUTTON_MASK_MIDDLE
	Input.parse_input_event(ev)
	await _settle(2)


func _click_at(at: Vector2, idx: MouseButton = MOUSE_BUTTON_LEFT) -> void:
	await _move(at)
	await _button(at, idx, true)
	await _button(at, idx, false)
	await _settle(6)


func _wheel(at: Vector2) -> void:
	await _move(at)
	await _button(at, MOUSE_BUTTON_WHEEL_UP, true)
	await _button(at, MOUSE_BUTTON_WHEEL_UP, false)
	await _settle(6)


func _middle_drag(from: Vector2, by: Vector2) -> void:
	await _move(from)
	await _button(from, MOUSE_BUTTON_MIDDLE, true)
	for i in 4:
		await _move(from + by * float(i + 1) / 4.0, MOUSE_BUTTON_MASK_MIDDLE)
	await _button(from + by, MOUSE_BUTTON_MIDDLE, false)
	await _settle(6)


func _key(code: Key, ctrl := false) -> void:
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = code
		ev.physical_keycode = code
		ev.ctrl_pressed = ctrl
		ev.pressed = down
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(4)


func _card() -> Object:
	return _app.context_broker.card


func _card_open() -> bool:
	var c = _card()
	return c != null and is_instance_valid(c) and c.visible


## The marker the open card is for, as "kind:id", or "" with no card open.
func _card_for() -> String:
	if not _card_open():
		return ""
	var hits: Array = (_app.context_broker.last_request as Dictionary).get("hits", [])
	if hits.is_empty():
		return ""
	return "%s:%d" % [String((hits[0] as Dictionary).get("kind", "")), int((hits[0] as Dictionary).get("id", -1))]


func _sel_is(i: int) -> bool:
	return _app.selection == {"kind": "settlement", "id": i}


## Screen position of settlement `i`'s marker (the pc1select probe's own mapping).
func _screen_of(i: int) -> Vector2:
	var ov = _app.viewport.overlay
	var s: Dictionary = _bridge.settlements()[i]
	return (ov as Control).get_global_transform() * ov._cell_to_screen(Vector2(float(s.get("x", 0)), float(s.get("y", 0))), ov._displayed_rect())


func _local(at: Vector2) -> Vector2:
	return ((_app.viewport.overlay as Control).get_global_transform().affine_inverse()) * at


## The part of the map not under a dock, bar or the navpad: the overlay's rect cut by every control that draws
## over it (the docks, the viewport area's own rect), shrunk by `pad`.
func _free_rect(pad: float) -> Rect2:
	var r: Rect2 = (_app.viewport.overlay as Control).get_global_rect()
	for d in [_app.left_dock, _app.right_dock]:
		if d != null and d.visible:
			var dr: Rect2 = (d as Control).get_global_rect()
			if dr.position.x < r.get_center().x:
				var cut := dr.end.x - r.position.x
				r = Rect2(r.position.x + maxf(cut, 0.0), r.position.y, r.size.x - maxf(cut, 0.0), r.size.y)
			else:
				r = Rect2(r.position.x, r.position.y, maxf(dr.position.x - r.position.x, 0.0), r.size.y)
	return r.grow(-pad)


## Settlements whose marker lies inside the visible map, is picked as ITSELF by the overlay at its own pixel,
## and is at least `gap` px from every other settlement. Named ones first.
func _clickable(gap: float) -> Array:
	var ov = _app.viewport.overlay
	var r: Rect2 = _free_rect(50.0)
	var all: Array = _bridge.settlements()
	var pts: Array = []
	for i in all.size():
		pts.append(_screen_of(i))
	var out: Array = []
	for i in all.size():
		var p: Vector2 = pts[i]
		if not r.has_point(p):
			continue
		var m: Dictionary = ov.marker_at(_local(p))
		if m.is_empty() or String(m["kind"]) != "settlement" or int(m["id"]) != i:
			continue
		var lone := true
		for j in all.size():
			if j != i and p.distance_to(pts[j]) < gap:
				lone = false
				break
		if lone:
			out.append(i)
	return out


## A map pixel with no marker, no hit of any kind, `gap` px clear of every settlement and outside `avoid`.
func _empty_point(gap: float, avoid: Rect2) -> Vector2:
	var ov = _app.viewport.overlay
	var r: Rect2 = _free_rect(60.0)
	var pts: Array = []
	for i in _bridge.settlements().size():
		pts.append(_screen_of(i))
	var y := r.position.y
	while y < r.end.y:
		var x := r.position.x
		while x < r.end.x:
			var p := Vector2(x, y)
			x += 37.0
			if avoid.has_point(p):
				continue
			var near := false
			for q in pts:
				if p.distance_to(q) < gap:
					near = true
					break
			if near:
				continue
			var lp := _local(p)
			if not (ov.marker_at(lp) as Dictionary).is_empty():
				continue
			var req: Dictionary = ov.request_at(lp, "click")
			if req.is_empty() or not (req.get("hits", []) as Array).is_empty():
				continue
			return p
		y += 37.0
	return Vector2(-1, -1)


## Closes any open card by Escape, so each leg starts from "no card, same view".
func _reset() -> void:
	if _card_open():
		await _key(KEY_ESCAPE)
	await _settle(4)


## A finger's tap (a real `InputEventScreenTouch` press and release; Godot emulates the mouse from it, which is
## what the map's touch routing reads).
func _tap(at: Vector2) -> void:
	for down in [true, false]:
		var ev := InputEventScreenTouch.new()
		ev.index = 0
		ev.pressed = down
		ev.position = at
		Input.parse_input_event(ev)
		await _settle(3)
	## A phone tap waits out the double-tap window before it acts (`viewport_host.gd`'s `_last_tap`).
	await _settle(45)


## The phone's own surface, `phone_menu.gd`'s peek card (a long press opens it; here `resolve()` does, with the
## touch source, as `_ctxphone_probe.gd` establishes). Its scrim is `MOUSE_FILTER_STOP`, so the tap that closed the
## sheet never reached the map. With the sheet open, a tap on a settlement above it must select it AND close the
## sheet; a tap on the sheet's own handle must not close it. The control tap (no sheet) proves a tap selects.
func _phone_legs() -> void:
	var menu = _app._phone_menu
	var ov = _app.viewport.overlay
	print("== phone: tap-through the peek sheet ==")
	var vr: Rect2 = get_viewport().get_visible_rect()
	var r: Rect2 = (ov as Control).get_global_rect().intersection(Rect2(vr.position + Vector2(40, 140), vr.size - Vector2(80, 140 + 300)))
	var cand: Array = []
	_app.viewport.reset_view()
	await _settle(8)
	_app.viewport.zoom_step(0.4)
	await _settle(60)
	print("  info phone map rect %s, zoom %.2f, window %s" % [str((ov as Control).get_global_rect()), _app.viewport.zoom(), str(vr)])
	for i in _bridge.settlements().size():
		var p := _screen_of(i)
		var m: Dictionary = ov.marker_at(_local(p))
		if r.has_point(p) and not m.is_empty() and String(m["kind"]) == "settlement" and int(m["id"]) == i:
			cand.append(i)
	## Only markers with the map itself under the finger: the phone draws buttons (the navpad, tool chips) over it.
	var clear: Array = []
	for i in cand:
		await _move(_screen_of(int(i)))
		var h = get_viewport().gui_get_hovered_control()
		if h == null or h == ov or (ov as Node).is_ancestor_of(h):
			clear.append(i)
	cand = clear
	if cand.size() < 2:
		print("[FATAL] fewer than two tappable settlements on the phone (%d)" % cand.size())
		get_tree().quit(2)
		return
	var a := int(cand[0])
	var b := int(cand[1])
	await _move(_screen_of(b))
	print("  info hovered at B: %s; marker %s" % [str(get_viewport().gui_get_hovered_control()), str(ov.marker_at(_local(_screen_of(b))))])
	await _tap(_screen_of(b))
	_ok(_sel_is(b), "control: a tap on settlement B, no sheet open, selects it (%s)" % str(_app.selection))
	_app.clear_selection()
	await _settle(4)
	## Open the peek sheet on empty ground (any hit-free point), then tap B above the sheet.
	var req_pt := _empty_point(45.0, Rect2())
	if req_pt.x < 0.0:
		req_pt = _screen_of(b) + Vector2(60, 0)
	var raw2: Dictionary = ov.request_at(_local(req_pt), "touch")
	_app.context_broker.resolve(raw2)
	await _settle(8)
	_ok(menu.peek_card_is_open(), "the peek sheet is open")
	var panel: Rect2 = menu.peek_panel_rect()
	var pb := _screen_of(b)
	_ok(not panel.has_point(pb), "settlement B (%s) is above the sheet (%s)" % [str(pb), str(panel)])
	await _tap(pb)
	_ok(not menu.peek_card_is_open(), "the tap outside the sheet closed it")
	_ok(_sel_is(b), "...and the same tap selected settlement B (%s)" % str(_app.selection))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _ready() -> void:
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(2)
		return
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_bridge = _app.bridge
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await _settle(2)
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _settle(10)
	await _settle(30)
	if not bool(_bridge.has_world):
		print("[FATAL] no world")
		get_tree().quit(2)
		return
	_app.select_domain("civilization")
	_app.arm_tool("inspect")
	await _settle(8)
	if _app.is_phone():
		await _phone_legs()
		return
	var cands := _clickable(16.0)
	var ov0 = _app.viewport.overlay
	var inr := 0
	var mk := 0
	var rr0: Rect2 = _free_rect(50.0)
	print("  info free rect %s left %s right %s" % [str(rr0), str((_app.left_dock as Control).get_global_rect()), str((_app.right_dock as Control).get_global_rect())])
	for i0 in _bridge.settlements().size():
		var p0 := _screen_of(i0)
		if rr0.has_point(p0):
			inr += 1
			var m0: Dictionary = ov0.marker_at(_local(p0))
			if not m0.is_empty() and String(m0["kind"]) == "settlement" and int(m0["id"]) == i0:
				mk += 1
	print("  info settlements %d, in view %d, picked as themselves %d, overlay rect %s" % [_bridge.settlements().size(), inr, mk, str(rr0)])
	if cands.size() < 3:
		print("[FATAL] fewer than three isolated visible settlements (%d)" % cands.size())
		get_tree().quit(2)
		return
	var a := int(cands[0])
	var b := int(cands[1])
	var c := int(cands[2])
	print("== settlements A=%d B=%d C=%d ==" % [a, b, c])

	print("== control: a click with no card open selects and opens the card ==")
	await _click_at(_screen_of(a))
	_ok(_sel_is(a), "control: the click selected A")
	_ok(_card_for() == "settlement:%d" % a, "control: the card is open on A (%s)" % _card_for())
	if not _card_open():
		get_tree().quit(1)
		return
	var card_rect: Rect2 = _card().panel_rect()
	print("  info card rect %s" % str(card_rect))
	var clear := card_rect.grow(30.0)
	## B and C must not sit under A's card, or the click would land on the card.
	var others: Array = cands.filter(func(i): return i != a and not clear.has_point(_screen_of(i)))
	if others.size() < 2:
		print("[FATAL] not two settlements clear of A's card")
		get_tree().quit(2)
		return
	b = int(others[0])
	c = int(others[1])
	var empty := _empty_point(45.0, clear)
	_ok(empty.x >= 0.0, "an empty map point clear of A's card was found (%s)" % str(empty))

	print("== B: with A's card open, one click on settlement B ==")
	await _click_at(_screen_of(b))
	_ok(_sel_is(b), "B: the selection is B after ONE click (%s)" % str(_app.selection))
	_ok(_card_for() == "settlement:%d" % b, "B: the card is open on B, not A, not closed (%s)" % _card_for())

	print("== E: with a card open, one click on an empty map point ==")
	_ok(_card_open(), "E: precondition, a card is open")
	await _click_at(empty)
	_ok(not _card_open(), "E: the card is closed")
	_ok(_app.selection.is_empty(), "E: the empty point cleared the selection (%s)" % str(_app.selection))

	print("== S: A's card open, one click on settlement C ==")
	await _click_at(_screen_of(a))
	_ok(_card_for() == "settlement:%d" % a, "S: precondition, A's card is open")
	await _click_at(_screen_of(c))
	_ok(_sel_is(c) and _card_for() == "settlement:%d" % c, "S: C selected and C's card open (%s, %s)" % [str(_app.selection), _card_for()])

	print("== W: card open, one wheel notch on the map ==")
	await _reset()
	await _click_at(_screen_of(a))
	_ok(_card_open(), "W: precondition, a card is open")
	var z0: float = _app.viewport.zoom()
	await _wheel(empty)
	_ok(_app.viewport.zoom() > z0, "W: the wheel zoomed the map (%.3f -> %.3f)" % [z0, _app.viewport.zoom()])
	_ok(not _card_open(), "W: and the card is gone")

	print("== M: card open, a middle-button drag on the map ==")
	await _reset()
	## The zoom changed above; take fresh points.
	cands = _clickable(16.0)
	await _click_at(_screen_of(a)) if cands.has(a) else await _click_at(_screen_of(int(cands[0])))
	_ok(_card_open(), "M: precondition, a card is open")
	var cam: Vector2 = _app.viewport._camera.position
	var from := _empty_point(45.0, (_card().panel_rect() as Rect2).grow(20.0))
	await _middle_drag(from, Vector2(-60, -40))
	var moved: Vector2 = _app.viewport._camera.position - cam
	_ok(moved.distance_to(Vector2(-60, -40)) < 4.0, "M: the map panned by the drag (moved %s)" % str(moved))
	_ok(not _card_open(), "M: and the card is gone")

	print("== K: a shortcut chord with a card open ==")
	await _reset()
	cands = _clickable(16.0)
	var km := int(cands[0])
	await _click_at(_screen_of(km))
	_ok(_card_open() and _sel_is(km), "K: precondition, the card is open on the selected marker")
	await _key(KEY_D, true)
	_ok(_app.selection.is_empty(), "K: Ctrl+D (Edit > Deselect) reached the shell and cleared the selection (%s)" % str(_app.selection))
	_ok(not _card_open(), "K: and the card is gone")

	print("== I: a click INSIDE the card, and Escape ==")
	await _reset()
	cands = _clickable(16.0)
	var m := int(cands[0])
	await _click_at(_screen_of(m))
	_ok(_card_open() and _sel_is(m), "I: precondition, the card is open on the selected marker")
	var pr: Rect2 = _card().panel_rect()
	await _click_at(pr.position + Vector2(5, 5))
	_ok(_card_open() and _sel_is(m), "I: a click on the card's own edge leaves it open and the selection alone")
	await _key(KEY_ESCAPE)
	_ok(not _card_open(), "X: Escape closes the card")
	_ok(_sel_is(m), "X: and leaves the selection alone")

	print("== R: a RIGHT-click card is click-through too ==")
	await _reset()
	await _click_at(_screen_of(m), MOUSE_BUTTON_RIGHT)
	_ok(_card_open(), "R: the right click opened a card")
	var rr: Rect2 = _card().panel_rect() if _card_open() else Rect2()
	cands = _clickable(16.0).filter(func(i): return i != m and not rr.grow(30.0).has_point(_screen_of(i)))
	if cands.is_empty():
		print("  info no settlement clear of the right-click card; leg R's outside click skipped")
	else:
		var n := int(cands[0])
		await _click_at(_screen_of(n))
		_ok(_sel_is(n), "R: the outside click selected the settlement under it (%s)" % str(_app.selection))
		_ok(_card_for() == "settlement:%d" % n, "R: and its own card replaced the right-click card (%s)" % _card_for())
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
