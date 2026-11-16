extends Node
## The popped-out City Viewer's marker card (`OUTSTANDING_WORK.md`, "City map look and clickable markers", the leftover
## "a popped-out viewer opens no card"; `CITY_MAP_LOOK_SCOPE.md` sixth build note; `POPOUT_WINDOWS_SCOPE.md` section 11 on
## dialogs in pop-outs). Against the real shell, a real native pop-out window and real events pushed into it.
##
##   godot --path . _popcard_probe.tscn -- --shots <dir> [--tag <name>]
##
## **Windowed**, desktop form only, under a wiped `APPDATA`. The probe opens a second OS window (the viewer popped out),
## so the caller holds the machine's window lock. Exit 0 held, 1 a check failed, 2 could not run.
##
## What it protects:
##   P  the viewer popped out: a left click on a pin (pushed into the pop-out window with `Window.push_input`) opens the
##      card INSIDE that window (its parent is the viewer, it is embedded, no third OS window), its drawn panel lies wholly
##      in the window and within 40 px of the pin it was opened for, with the NOTE section naming the town.
##   E  the same with the pin pushed to the window's bottom-right corner, where the card must flip and clamp INSIDE the
##      pop-out (a card that measured the main window's bounds would hang off the pop-out).
##   N  the Note section works there (Ruling FJ): the card's one Note... row opens the vault on the town, the vault's place
##      panel creates a note bound to it, and the card then reads Open note.
##   K  click-through in the pop-out: with a card open, a click on another pin opens THAT pin's card (the press is not
##      swallowed).
##   D  docked again: the same click opens the card under the app, embedded in the main window, at the pin as before
##      (anchor = dialog position + canvas transform); popped again it goes back inside the pop-out.

const Pin := preload("res://shell/pin_marker.gd")
const VAULT := "user://popcard_probe_vault"

var _fails := 0
var _checks := 0
var _shots := ""
var _tag := "base"
var _app: Node
var _bridge
var _root := ""
var _reached_end := false


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(n: int = 4) -> void:
	for _i in n:
		await get_tree().process_frame


func _nwin() -> int:
	return DisplayServer.get_window_list().size()


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(1 if _fails > 0 else 0)


func _shot(w: Window, name: String) -> void:
	if _shots == "":
		return
	await RenderingServer.frame_post_draw
	var img: Image = w.get_texture().get_image()
	img.save_png(_shots.path_join("popcard_%s_%s.png" % [_tag, name]))


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--shots" and i + 1 < args.size():
			_shots = String(args[i + 1])
			i += 2
		elif a == "--tag" and i + 1 < args.size():
			_tag = String(args[i + 1])
			i += 2
		else:
			print("ABORT: unknown argument %s" % a)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("ABORT: run windowed; this probe opens a native window")
		get_tree().quit(2)
		return
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("WATCHDOG")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.4).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_ok(not _app.is_phone() and not DccTheme.is_tablet(), "the shell booted into the desktop form")
	DccSettings.set_popout_enabled(false)
	PopoutHost.forget_choices(get_tree())
	_bridge = _app.bridge
	_bridge.generate({"seed": 1200, "width_km": 1200.0, "grid_w": 512, "grid_h": 384, "archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _settle(20)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	if not _bridge.has_world:
		_ok(false, "a world generated")
		_finish()
		return
	await _run()
	## A script error inside `_run` ends the coroutine early and would otherwise read as a pass: the last leg sets this.
	_ok(_reached_end, "the probe ran to its last check")
	_finish()


func _town() -> int:
	var places: Array = _bridge.settlements()
	for k in places.size():
		var kind := String((places[k] as Dictionary).get("kind", ""))
		if kind in ["city", "capital", "town", "metropolis"]:
			var lay: Array = _bridge.urban_layouts(PackedInt32Array([k]))
			if lay.size() > 0 and (lay[0] as Dictionary).has("market"):
				return k
	return -1


func _find_button(root: Node, prefix: String) -> Button:
	for ch in root.get_children():
		if ch is Button and String((ch as Button).text).to_lower().begins_with(prefix.to_lower()) and (ch as Button).is_visible_in_tree():
			return ch
		var r := _find_button(ch, prefix)
		if r != null:
			return r
	return null


func _card():
	var b = _app.context_broker
	return b.card if b != null else null


func _card_open() -> bool:
	var c = _card()
	return c != null and c.visible


func _close_card() -> void:
	var c = _card()
	if c != null and c.visible:
		c.hide()
	await _settle(3)


func _row(prefix: String, kind: String = "") -> Variant:
	var c = _card()
	if c == null:
		return null
	for r in c.drawn_rows():
		if String(r.get("text", "")).begins_with(prefix) and (kind == "" or String(r.get("kind", "")) == kind):
			return r
	return null


## Pushes a left press then release into window `w` at `at` (the window's own coordinates), through the window's own
## GUI routing: the path a real mouse event takes, not a call into the viewer's handler.
func _click_in(w: Window, at: Vector2) -> void:
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		ev.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
		w.push_input(ev)
		await _settle(2)
	await _settle(8)


## Distance from `p` to rectangle `r` (0 inside).
func _dist(r: Rect2, p: Vector2) -> float:
	var dx := maxf(maxf(r.position.x - p.x, p.x - r.end.x), 0.0)
	var dy := maxf(maxf(r.position.y - p.y, p.y - r.end.y), 0.0)
	return Vector2(dx, dy).length()


## The pin's tip (the card's anchor), in the viewer window's own coordinates.
func _tip_in_window(cv, k: int) -> Vector2:
	return cv._canvas.get_global_transform_with_canvas() * cv.pin_tip(k)


## The pin's head, in the viewer window's own coordinates.
func _head_in_window(cv, k: int) -> Vector2:
	var pr: float = cv.PIN_HEAD_R * cv._pin_unit()
	var head: Vector2 = Pin.geometry(cv.pin_tip(k), pr, cv.pin_tilt(k))["head"]
	return cv._canvas.get_global_transform_with_canvas() * head


func _pin_index(cv, poi: bool, which: int = 0) -> int:
	var pins: Array = cv._pins()
	var seen := 0
	for k in pins.size():
		if (pins[k] as Dictionary).has("poi") == poi:
			if seen == which:
				return k
			seen += 1
	return -1


func _run() -> void:
	var si := _town()
	_ok(si >= 0, "a town with a layout exists")
	if si < 0:
		return
	var s: Dictionary = _bridge.settlements()[si]
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-popcard-%d" % Time.get_ticks_msec()
	DirAccess.make_dir_recursive_absolute(_root + "/Lore")
	var f := FileAccess.open(_root + "/Lore/Old.md", FileAccess.WRITE)
	f.store_string("# Old\n")
	f.close()
	_ok(bool(_bridge.vault_connect(_root, "Chronicle").get("ok", false)), "a throw-away vault binds")
	var pid: int = _bridge.poi_add(float(s["x"]) + 0.5 + 0.12, float(s["y"]) + 0.5 + 0.05, "tavern", "Popcard Tavern")
	_ok(pid >= 0, "a POI beside the market")
	_app.viewport.refresh_annotations()
	_app.arm_tool("inspect")
	var cv: CityViewerWindow = _app.city_viewer_window
	_app.open_city_viewer(si)
	await _settle(14)
	_ok(cv.visible and cv.is_embedded() and _nwin() == 1, "the viewer opens docked, one OS window")

	# ---- D0: docked, before anything pops out: the card under the app at the pin
	var k_d := _pin_index(cv, false)
	_ok(k_d >= 0, "the viewer shows a derived pin")
	var head_d: Vector2 = Pin.geometry(cv.pin_tip(k_d), cv.PIN_HEAD_R * cv._pin_unit(), cv.pin_tilt(k_d))["head"]
	await _close_card()
	cv._on_canvas_input(_mb(head_d, true))
	await _settle(2)
	cv._on_canvas_input(_mb(head_d, false))
	await _settle(10)
	var c = _card()
	_ok(c != null and c.visible, "D0: docked, a click on a pin opens the card")
	if c != null and c.visible:
		var anchor0: Vector2 = Vector2(cv.position) + _tip_in_window(cv, k_d)
		_ok(c.get_parent() == _app and c.is_embedded(), "D0: the card is a child of the app and embedded in the main window")
		_ok(_dist(c.panel_rect(), anchor0) <= 14.0, "D0: it stands at the pin's tip, at most 14 px off (panel %s, tip %s)" % [str(c.panel_rect()), str(anchor0)])
		_ok(Rect2(Vector2.ZERO, _app.get_viewport().get_visible_rect().size).encloses(c.panel_rect()), "D0: and wholly on the main window")
		await _shot(get_window(), "docked_card")
	await _close_card()

	# ---- P: popped out
	await PopoutHost.toggle(cv)
	await _settle(10)
	_ok(not cv.is_embedded() and cv.visible and _nwin() == 2, "the viewer popped out: native, two OS windows")
	var vis: Rect2 = cv.get_visible_rect()
	print("  popped window logical rect %s" % str(vis))
	var k := _pin_index(cv, false)
	var at := _head_in_window(cv, k)
	_ok(vis.has_point(at), "P0: the pin's head is inside the pop-out (%s)" % str(at))
	_app.select("settlement", -1)
	await _click_in(cv, at)
	c = _card()
	_ok(c != null and c.visible, "P1: a left click on a pin in the popped viewer opens the card")
	if c != null and c.visible:
		_ok(c.get_parent() == cv, "P1: the card is a child of the pop-out window")
		_ok(c.is_embedded(), "P1: embedded in the pop-out (its parent is the pop-out), not a window of its own")
		_ok(_nwin() == 2, "P1: no third OS window opened (%d)" % _nwin())
		var pr: Rect2 = c.panel_rect()
		_ok(vis.encloses(pr), "P1: the drawn card lies wholly inside the pop-out (card %s, window %s)" % [str(pr), str(vis)])
		var tip_p := _tip_in_window(cv, k)
		_ok(_dist(pr, tip_p) <= 14.0, "P1: and stands at the pin's tip, at most 14 px off (panel %s, tip %s)" % [str(pr), str(tip_p)])
		var hits: Array = _app.context_broker.last_request.get("hits", [])
		_ok(hits.size() == 1 and String(hits[0]["kind"]) == "settlement" and int(hits[0]["id"]) == si, "P1: for exactly this town (%s)" % str(hits))
		_ok(_row("NOTE", "band") != null, "P1: with a NOTE section")
		await _shot(cv, "popped_card")
	# ---- N: the Note section in the pop-out
	var cr = _row("Note…", "action")
	_ok(cr != null and _row("Create note", "action") == null and _row("Attach note", "action") == null, "N1: the card offers the one Note… row (Ruling FJ), none of the retired rows")
	if cr != null:
		var rc: Vector2 = (cr["node"] as Control).get_global_rect().get_center() + Vector2(c.position)
		await _click_in(cv, rc)
		await _settle(8)
		var vw = _app.vault_window
		var tid := int((_bridge.settlements()[si] as Dictionary).get("tid", si))
		_ok(vw.visible and String(vw._kind) == "settlement" and int(vw._entity_id) == tid, "N2: pressing it from the pop-out's card opens the vault on this town (%s %s)" % [str(vw._kind), str(vw._entity_id)])
		_ok(_nwin() == 2, "N2: no extra OS window opened (%d)" % _nwin())
		var cb: Button = _find_button(vw, "Create ")
		_ok(cb != null and vw.find_child("VaultPlaceNotes", true, false) != null, "N3: the vault's place panel offers Create")
		if cb != null:
			await _shot(get_window(), "popped_vault")
			cb.pressed.emit()
			await _settle(10)
			var links: Array = _bridge.vault_links_for("settlement", tid)
			_ok(links.size() == 1 and FileAccess.file_exists(_root + "/" + String((links[0] as Dictionary).get("path", ""))), "N3: the note is on disk and bound to the town (%s)" % str(links))
			var rs: Dictionary = _app.context_broker._note.resolve({"kind": "settlement", "id": si})
			_ok(String(rs.get("state", "")) == "note", "N3: the Note logic now reads state %s" % String(rs.get("state", "")))
		vw.hide()
	await _close_card()
	await _click_in(cv, at)
	_ok(_card_open() and _row("Open note") != null, "N4: the card now offers Open note")
	await _close_card()

	# ---- K: click-through in the pop-out
	var kp := _pin_index(cv, true)
	_ok(kp >= 0, "K0: the viewer shows the POI pin")
	if kp >= 0:
		await _click_in(cv, at)
		_ok(_card_open(), "K1: the town's card is open")
		var at_p := _head_in_window(cv, kp)
		await _click_in(cv, at_p)
		var h2: Array = _app.context_broker.last_request.get("hits", [])
		_ok(_card_open() and h2.size() == 1 and String(h2[0]["kind"]) == "poi" and int(h2[0]["id"]) == pid, "K2: a click on another pin with a card open opens THAT pin's card (%s)" % str(h2))
		_ok(_card().get_parent() == cv, "K2: still inside the pop-out")
	await _close_card()

	# ---- H: the pop-out's own click-through hook (`ContextCard._on_host_window_input`), driven on its signal because a
	# pushed event does not go through the embedded popup's own press-swallowing the way a real mouse press does
	await _click_in(cv, at)
	c = _card()
	_ok(_card_open() and c.get_parent() == cv, "H0: a card is open in the pop-out")
	if _card_open():
		var inside := _mb(Vector2(c.position) + Vector2(c.size) * 0.5, true)
		cv.window_input.emit(inside)
		_ok(_card_open(), "H1: a press inside the card's own rectangle leaves it open")
		var outside := _mb(Vector2(4.0, vis.size.y - 4.0), true)
		cv.window_input.emit(outside)
		_ok(not _card_open(), "H2: a press elsewhere in the pop-out closes it (and is not consumed here)")
	await _close_card()

	# ---- E: the pin in the window's bottom-right corner
	var cv_canvas_size: Vector2 = cv._canvas.size
	var tip: Vector2 = cv.pin_tip(k)
	cv._pan += (cv_canvas_size - Vector2(30.0, 24.0)) - tip
	cv._canvas.queue_redraw()
	await _settle(4)
	var at_e := _head_in_window(cv, k)
	_ok(vis.has_point(at_e) and at_e.x > vis.size.x * 0.6 and at_e.y > vis.size.y * 0.6, "E0: the pin is in the pop-out's lower right (%s of %s)" % [str(at_e), str(vis.size)])
	await _click_in(cv, at_e)
	c = _card()
	_ok(c != null and c.visible and c.get_parent() == cv, "E1: the card opens inside the pop-out")
	if c != null and c.visible:
		var pr2: Rect2 = c.panel_rect()
		_ok(vis.encloses(pr2), "E1: wholly inside it after the flip and clamp (card %s, window %s)" % [str(pr2), str(vis)])
		var tip_e := _tip_in_window(cv, k)
		_ok(_dist(pr2, tip_e) <= 14.0, "E1: and still beside the pin's tip, at most 14 px off (card %s, tip %s)" % [str(pr2), str(tip_e)])
		await _shot(cv, "popped_corner")
	await _close_card()
	cv._pan -= (cv_canvas_size - Vector2(30.0, 24.0)) - tip
	cv._canvas.queue_redraw()

	# ---- D: dock back; the same click is as it was; pop out again and it follows
	await PopoutHost.toggle(cv)
	await _settle(10)
	_ok(cv.is_embedded() and cv.visible and _nwin() == 1, "D1: docked again, one OS window")
	var head_d2: Vector2 = Pin.geometry(cv.pin_tip(k_d), cv.PIN_HEAD_R * cv._pin_unit(), cv.pin_tilt(k_d))["head"]
	await _close_card()
	cv._on_canvas_input(_mb(head_d2, true))
	await _settle(2)
	cv._on_canvas_input(_mb(head_d2, false))
	await _settle(10)
	c = _card()
	_ok(c != null and c.visible and c.get_parent() == _app and c.is_embedded(), "D2: docked, the card is back under the app, embedded in the main window")
	if c != null and c.visible:
		var anchor2: Vector2 = Vector2(cv.position) + _tip_in_window(cv, k_d)
		_ok(_dist(c.panel_rect(), anchor2) <= 14.0, "D2: at the pin's tip, at most 14 px off (panel %s, tip %s)" % [str(c.panel_rect()), str(anchor2)])
		_ok(_row("NOTE", "band") != null and _row("Open note") != null, "D2: with its NOTE section")
	await _close_card()
	await PopoutHost.toggle(cv)
	await _settle(10)
	await _click_in(cv, _head_in_window(cv, k))
	_ok(_card_open() and _card().get_parent() == cv, "D3: popped out again, the card follows it back into the pop-out")
	await _close_card()
	cv.hide()
	await _settle(6)
	_ok(_nwin() == 1, "the pop-out is gone with the viewer hidden (%d)" % _nwin())
	_reached_end = true


func _mb(local: Vector2, down: bool) -> InputEventMouseButton:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = down
	ev.position = local
	ev.global_position = local
	ev.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
	return ev
