extends Node
## Edit-from-context probe (`EDIT_FROM_CONTEXT_AUDIT.md`, Rulings EI / EL / EO): the shared element rows
## (`GlobalTools.element_rows`) and the touch chips, driven by real pointer events against the real shell.
##
##   godot --path . --resolution 1600x1000 _edfc_probe.tscn -- --vp 1600x1000                    (desktop)
##   godot --path . --resolution 1600x1000 _edfc_probe.tscn -- --vp 1600x1000 --force-touch      (the touch tablet: window-capped, no --density)
##   ... -- --vp 1080x2340 --density 2.8125 --force-touch   (the phone: its WORLD and CIVIL legs use the DOMAIN tab, not MAP, which selects CARTO)
## A 2388x1668 @2 run classifies as a PHONE (the 1600x1000 window is 800 dp), so it is not the tablet form.
## Arguments read by THIS file: `--vp WxH`, `--shots DIR` (evidence frames, never asserted), `--only LM,TC,...` (a debugging aid; only a
## full run is a result). Accepted because the shell reads them: `--density D`, `--force-touch`.
## Anything else aborts with exit 2. Windowed, with a wiped user dir (`APPDATA`), one window lock at a time.
##
## Legs (each asserts with literals written by hand, never against the constant it came from):
##   LM  landmark: Rename, Note, Select present in WORLD, CIVIL and CARTO; Rename through the real dialog changes
##       `landmark_names()`; the name survives a save and reopen; Clear returns the generated name
##   ST  settlement: Rename in every domain; Edit, Delete (confirms) and Note in WORLD and CARTO
##   LB  label: Rename and Move in every domain, Delete (confirms) outside CARTO
##   IC  icon: Move in every domain, Delete (confirms) outside CARTO
##   RT  route: Rename in CIVIL
##   TC  touch: a tap shows the element's chips, a long press keeps the full card
##   WY  touch: a finger's way pick (22 px, not the mouse's 6) and the way's Delete chip, deleted and undone
##   NT  selection kinds: a click or tap selects a label or icon, and the Context panel's Note tab resolves its note
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument.

const SEED := 24601

var _fails := 0
var _checks := 0
var _vp: SubViewport
var _app: Node
var _ov: Control
var _touch := false
## The probe viewport's width in px (set by `_run`): the phone leg derives its 44-reference-px floor from it.
var _vw := 0
var _lm_key := ""


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fails += 1
		print("FAIL: ", what)
	else:
		print("ok:   ", what)


func _frames(n: int) -> void:
	for _k in n:
		await get_tree().process_frame


func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var k := args.find(name)
	if k >= 0 and k + 1 < args.size():
		return String(args[k + 1])
	return dflt


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 420.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--force-touch":
			_touch = true
			i += 1
		elif a in ["--vp", "--density", "--shots", "--only"]:
			i += 2
		else:
			print("ABORT: unknown argument ", a)
			get_tree().quit(2)
			return
	var parts := _arg("--vp", "1600x1000").split("x")
	await _run(int(parts[0]), int(parts[1]))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


# ---- pointer events pushed at the host viewport --------------------------------------------------------------------

func _mouse(p: Vector2, button: int, down: bool) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = button
	ev.pressed = down
	ev.position = p
	ev.global_position = p
	ev.button_mask = (MOUSE_BUTTON_MASK_LEFT if button == MOUSE_BUTTON_LEFT else MOUSE_BUTTON_MASK_RIGHT) if down else 0
	_vp.push_input(ev)


func _move_to(p: Vector2) -> void:
	var mv := InputEventMouseMotion.new()
	mv.position = p
	mv.global_position = p
	_vp.push_input(mv)
	await _frames(1)


func _click_at(p: Vector2) -> void:
	await _move_to(p)
	_mouse(p, MOUSE_BUTTON_LEFT, true)
	await _frames(1)
	_mouse(p, MOUSE_BUTTON_LEFT, false)
	await _frames(4)


func _rclick_at(p: Vector2) -> void:
	await _move_to(p)
	_mouse(p, MOUSE_BUTTON_RIGHT, true)
	await _frames(1)
	_mouse(p, MOUSE_BUTTON_RIGHT, false)
	await _frames(5)


func _finger(p: Vector2, down: bool) -> void:
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.pressed = down
	t.position = p
	_vp.push_input(t)
	var m := InputEventMouseButton.new()
	m.device = InputEvent.DEVICE_ID_EMULATION
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = down
	m.position = p
	m.global_position = p
	m.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
	_vp.push_input(m)


func _tap(p: Vector2) -> void:
	_finger(p, true)
	await _frames(3)
	_finger(p, false)
	await _frames(6)


## A held finger: down, wait past the hold (the tablet's 500 ms, the phone's 480 ms), then up.
func _hold(p: Vector2) -> void:
	_finger(p, true)
	await get_tree().create_timer(0.75).timeout
	await _frames(3)
	_finger(p, false)
	await _frames(6)


## Host-space event offset between where a pointer event is pushed and where the overlay says it received it
## (measured by `_calibrate`: the map sits inside the host with its own transform). Subtracted for MAP gestures only.
var _off := Vector2.ZERO


## Measures `_off`: pushes a motion at a probe point and reads the overlay's own `_last_mouse_local` back.
func _calibrate() -> void:
	var want_local := Vector2(150.0, 150.0)
	var p := _ov.get_global_rect().position + want_local
	await _move_to(p)
	await _frames(2)
	var got: Vector2 = _ov._last_mouse_local
	_off = got - want_local
	print("CAL off=", _off)


## The gesture on the map at `p`: a real right click on desktop, a real long press on touch.
func _context_at(p: Vector2) -> void:
	p -= _off
	await _close_card()
	if _touch:
		await _hold(p)
	else:
		await _rclick_at(p)


# ---- the card ------------------------------------------------------------------------------------------------------

func _card():
	var b = _app.get("context_broker")
	return b.card if b != null else null


func _card_open() -> bool:
	var c = _card()
	return c != null and c.visible


func _close_card() -> void:
	var c = _card()
	if c != null and c.visible:
		c.hide()
	var b = _app.get("context_broker")
	if b != null and b.ring_is_open():
		b.ring_close()
	_ov._touch_ring_active = false
	_ov._touch_swallow_up = false
	_ov._sample_pin = {}
	var pm = _app.get("_phone_menu")
	if pm != null and pm.peek_card_is_open():
		pm._dismiss_peek_card()
	await _frames(2)


## Ids of every row the broker merged for the last gesture (the data the card or the phone sheet was built from).
func _ids() -> Array:
	var out: Array = []
	for r in _app.context_broker.last_actions:
		out.append(String((r as Dictionary).get("id", "")))
	return out


## The merged row `id`, or `{}`.
func _row_of(id: String) -> Dictionary:
	for r in _app.context_broker.last_actions:
		if String((r as Dictionary).get("id", "")) == id:
			return r
	return {}


## Presses the card's row `id` with a real click when the desktop/tablet card is up; on the phone sheet runs it the way
## the sheet does (its row callable), since the sheet is a separate widget set.
func _press(id: String) -> bool:
	var c = _card()
	if c != null and c.visible:
		for r in c.drawn_rows():
			if String(r.get("id", "")) == id and String(r["kind"]) == "action":
				var n: Control = r["node"]
				var rect := n.get_global_rect()
				await _click_at(rect.get_center() + Vector2(c.position))
				return true
		return false
	var row := _row_of(id)
	if row.is_empty() or not row.has("callable"):
		return false
	## The phone sheet closes itself when a row is pressed (and releases its share of the world pause); running the
	## callable alone left it, and its scrim, over the map for every later leg. Closed first, as the sheet does.
	await _close_card()
	(row["callable"] as Callable).call()
	await _frames(3)
	return true


## Types `text` into the visible one-field prompt and presses its OK. False when there is no prompt.
func _answer_prompt(text: String) -> bool:
	var dlg := _find_dialog()
	if dlg == null:
		return false
	var le := _find_line_edit(dlg)
	if le == null:
		return false
	le.text = text
	dlg.confirmed.emit()
	dlg.hide()  # the OK button hides its dialog before it emits; emitting alone left a phone-sized one on top of the map
	await _frames(4)
	return true


func _answer_confirm(yes: bool) -> bool:
	var dlg := _find_dialog()
	if dlg == null:
		return false
	if yes:
		dlg.confirmed.emit()
		dlg.hide()
	else:
		dlg.canceled.emit()
		dlg.hide()
	await _frames(4)
	return true


## How many `ConfirmationDialog`s are visible right now.
func _visible_dialogs() -> int:
	var n := 0
	var stack: Array = [_app]
	while not stack.is_empty():
		var nd: Node = stack.pop_back()
		if nd is ConfirmationDialog and (nd as ConfirmationDialog).visible and not nd.is_queued_for_deletion():
			n += 1
		for c in nd.get_children():
			stack.append(c)
	return n


func _find_dialog() -> ConfirmationDialog:
	var stack: Array = [_app]
	var found: ConfirmationDialog = null
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is ConfirmationDialog and (n as ConfirmationDialog).visible and not n.is_queued_for_deletion():
			found = n
		for c in n.get_children():
			stack.append(c)
	return found


func _find_line_edit(n: Node) -> LineEdit:
	if n is LineEdit:
		return n
	for c in n.get_children():
		var r := _find_line_edit(c)
		if r != null:
			return r
	return null


# ---- fixtures ------------------------------------------------------------------------------------------------------

## A screen position (host space) over `cell`, from the overlay's own transform, or (-1,-1) when it is off the plate.
## Whether host point `hp` is on the phone's own chrome rather than the map: outside the band between the app bar and the
## tool row, or on the layers button (top left). Always false off the phone.
func _phone_chrome(hp: Vector2) -> bool:
	return DccTheme.is_phone() and (not Rect2(20.0, 230.0, float(_vw) - 40.0, 1390.0).has_point(hp) or Rect2(0.0, 200.0, 160.0, 170.0).has_point(hp))


func _pos_of_cell(cell: Vector2) -> Vector2:
	var rect: Rect2 = _ov._displayed_rect()
	var p: Vector2 = _ov._cell_to_screen(cell, rect)
	var inter: Rect2 = _ov._interior_rect(rect)
	if not inter.grow(-24.0).has_point(p):
		return Vector2(-1, -1)
	var hp: Vector2 = _ov.get_global_transform() * p
	## On the phone the map host also carries the layers button (top left), the tool buttons and the control row; a
	## point beside them is the BUTTON's tap (`_on_map` counts any host descendant as the map). Taps stay in the
	## band between the app bar and the tool row, clear of the layers button (measured: a pin at (123, 237) sat under it and its tap never
	## reached the map).
	if _phone_chrome(hp):
		return Vector2(-1, -1)
	return hp if _app.viewport._on_map(hp) else Vector2(-1, -1)


func _local(p: Vector2) -> Vector2:
	return _ov.get_global_transform().affine_inverse() * p


## The first hit kind at host-space `p`, or "".
func _first_hit(p: Vector2) -> String:
	var hits: Array = _ov.hits_at(_local(p))
	return "" if hits.is_empty() else String((hits[0] as Dictionary)["kind"])


func _domain(d: String) -> void:
	_app.select_domain(d)
	await _frames(5)
	if DccTheme.is_phone():
		## The phone's MAP tab SELECTS CARTOGRAPHY (`_pick_phone_tab("map")`), which silently turned every WORLD and
		## CIVIL leg into a CARTO one; its DOMAIN tab keeps the domain just selected. The sheet is dropped to peek so
		## the map stays uncovered.
		_app._pick_phone_tab("domain", false)
		_app.set_phone_detent("peek")
	await get_tree().create_timer(0.4).timeout
	await _frames(3)



## Centres the map on grid `cell` (the docks of a touch form cover part of the map, so a fixture can sit under one).
func _centre(cell: Vector2) -> void:
	_app.viewport.move_view_to(cell.x, cell.y)
	await _frames(4)


func _wait_gen(bridge) -> void:
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout


# ---- the run -------------------------------------------------------------------------------------------------------

func _run(vw: int, vh: int) -> void:
	_vw = vw
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.6).timeout
	var bridge = _app.bridge
	bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.42})
	await _wait_gen(bridge)
	await get_tree().create_timer(1.0).timeout
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if "phone_project_picker" in _app and _app.phone_project_picker != null:
		_app.phone_project_picker.hide()
	await _frames(6)
	if DccTheme.is_phone():
		_app._pick_phone_tab("map")
		_app.set_phone_detent("peek")
	_ov = _app.viewport.overlay
	_app.viewport.reset_view()
	await _frames(6)
	_ok(bridge.has_world, "a world is up")
	_ok(_touch == _app.viewport._touch, "the form matches the flags (touch %s)" % str(_touch))
	await bridge.landmark_run()
	await _frames(4)
	_app.viewport.refresh_annotations()
	await _frames(4)

	if _runs("LM"):
		await _leg_landmark(bridge)
	if _runs("ST"):
		await _leg_settlement(bridge)
	if _runs("LB"):
		await _leg_label_icon(bridge)
	if _runs("RT"):
		await _leg_route(bridge)
	if _runs("TC"):
		await _leg_touch(bridge)
	if _runs("WY"):
		await _leg_way(bridge)
	if _runs("NT"):
		await _leg_note(bridge)
	if _runs("SAVE"):
		await _leg_save(bridge)


## Whether leg `name` runs: all of them by default, or only those listed in `--only LM,TC` (a debugging aid; the
## committed result is always a full run, which is what `RESULT:` reports).
func _runs(name: String) -> bool:
	var only := _arg("--only", "")
	return only == "" or only.split(",").has(name)


# ---- LM ------------------------------------------------------------------------------------------------------------

## The host position over landmark `i` whose first LANDMARK hit is `i` (a river or a settlement may be nearer, which the card
## handles per kind), recomputed from the overlay's current transform: a domain switch can resize the map.
func _lm_cell(i: int) -> Vector2:
	var lm: Dictionary = _app.bridge.landmarks()[i]
	return Vector2(float(lm.get("x", 0)) + 0.5, float(lm.get("y", 0)) + 0.5)


func _landmark_pos(i: int) -> Vector2:
	var lm: Dictionary = _app.bridge.landmarks()[i]
	var base := _pos_of_cell(Vector2(float(lm.get("x", 0)) + 0.5, float(lm.get("y", 0)) + 0.5))
	if base.x < 0.0:
		return base
	for r in [0, 2, 4, 6]:
		for a in 8:
			var p := base + Vector2.from_angle(a * PI / 4.0) * float(r)
			if not _app.viewport._on_map(p):
				continue
			for h in _ov.hits_at(_local(p)):
				if String((h as Dictionary)["kind"]) == "landmark":
					if int((h as Dictionary)["id"]) == i:
						return p
					break
	return Vector2(-1, -1)


## A landmark that can be picked on screen, as `[index, host_pos]`, or `[-1, ...]`.
func _find_landmark() -> Array:
	var n: int = _app.bridge.landmarks().size()
	for i in n:
		var p := _landmark_pos(i)
		if p.x >= 0.0:
			return [i, p]
	return [-1, Vector2.ZERO]


func _leg_landmark(bridge) -> void:
	print("== LM landmark ==")
	await _centre(_lm_cell(0))
	var f := _find_landmark()
	_ok(int(f[0]) >= 0, "LM a pickable landmark exists (landmarks placed: %d)" % bridge.landmarks().size())
	if int(f[0]) < 0:
		return
	var li: int = f[0]
	var pos: Vector2 = f[1]
	var generated := String((bridge.landmark_names()[li] as Dictionary)["name"])
	var key := String((bridge.landmark_names()[li] as Dictionary)["key"])
	for d in ["world", "civilization", "cartography"]:
		await _domain(d)
		await _centre(_lm_cell(li))
		pos = _landmark_pos(li)
		_ok(pos.x >= 0.0, "LM %s: the landmark is pickable there" % d)
		await _context_at(pos)
		_ok(not _ids().is_empty(), "LM %s: the context gesture produced rows" % d)
		var ids := _ids()
		for need in ["el.landmark_rename", "el.landmark_select"]:
			_ok(ids.has(need), "LM %s: row %s present (ids %s)" % [d, need, str(ids)])
		_ok(not ids.has("el.landmark_clear_name"), "LM %s: no Clear name while the name is generated" % d)
		await _close_card()
	# Rename through the real card + dialog, in WORLD.
	await _domain("world")
	await _centre(_lm_cell(li))
	pos = _landmark_pos(li)
	await _context_at(pos)
	_ok(await _press("el.landmark_rename"), "LM press Rename")
	_ok(await _answer_prompt("The Weeping Stair"), "LM the prompt took the name")
	var names: Array = bridge.landmark_names()
	_ok(String((names[li] as Dictionary)["name"]) == "The Weeping Stair", "LM the landmark now reads the typed name")
	_ok(String((names[li] as Dictionary)["source"]) == "user", "LM its source is user")
	# Clear returns the generated name.
	_app.viewport.refresh_annotations()
	await _frames(4)
	var f2 := [li, _landmark_pos(li)]
	_lm_key = key
	await _domain("cartography")
	await _centre(_lm_cell(li))
	f2[1] = _landmark_pos(li)
	if f2[1].x >= 0.0:
		await _context_at(f2[1])
		_ok(_ids().has("el.landmark_clear_name"), "LM Clear custom name is offered once named (names %s ids %s)" % [str(bridge.landmark_names()[li]), str(_ids())])
		_ok(await _press("el.landmark_clear_name"), "LM press Clear")
		var again: Array = bridge.landmark_names()
		var cleared := false
		for n2 in again:
			if String((n2 as Dictionary)["key"]) == key:
				cleared = String((n2 as Dictionary)["name"]) == generated and String((n2 as Dictionary)["source"]) != "user"
		_ok(cleared, "LM Clear brought back the generated name %s" % generated)
	await _close_card()


## A click on the map: a real left click on desktop, a real tap on touch.
func _map_click(p: Vector2) -> void:
	p -= _off
	if _touch:
		await _tap(p)
	else:
		await _click_at(p)


## The host position over settlement `i` whose first SETTLEMENT hit is `i`, recomputed from the current transform.
func _settlement_pos(i: int) -> Vector2:
	var s: Dictionary = _app.bridge.settlements()[i]
	var base := _pos_of_cell(Vector2(float(s["x"]), float(s["y"])))
	if base.x < 0.0:
		return base
	for r in [0, 2, 4]:
		for a in 8:
			var p := base + Vector2.from_angle(a * PI / 4.0) * float(r)
			if not _app.viewport._on_map(p):
				continue
			for h in _ov.hits_at(_local(p)):
				if String((h as Dictionary)["kind"]) == "settlement":
					if int((h as Dictionary)["id"]) == i:
						return p
					break
	return Vector2(-1, -1)


## The most populous settlement: a small place is hidden by the overlay's zoom rules on a phone-sized, zoomed view
## (measured: the one nearest the map centre had no pin and only a road under it), and a pin that is not drawn
## cannot be tapped. The biggest place is drawn at every zoom.
func _inland_settlement() -> int:
	var best := 0
	var best_pop := -1.0
	var all: Array = _app.bridge.settlements()
	for i in all.size():
		var pop := float((all[i] as Dictionary).get("population", 0.0))
		if pop > best_pop:
			best_pop = pop
			best = i
	return best



func _st_cell(i: int) -> Vector2:
	var s: Dictionary = _app.bridge.settlements()[i]
	return Vector2(float(s["x"]), float(s["y"]))


func _find_settlement() -> Array:
	var n: int = _app.bridge.settlements().size()
	for i in n:
		var p := _settlement_pos(i)
		if p.x >= 0.0:
			return [i, p]
	return [-1, Vector2.ZERO]


func _leg_settlement(bridge) -> void:
	print("== ST settlement ==")
	await _centre(_st_cell(0))
	var f := _find_settlement()
	_ok(int(f[0]) >= 0, "ST a pickable settlement exists")
	if int(f[0]) < 0:
		return
	var si: int = f[0]
	var pos: Vector2 = f[1]
	var name0 := String(bridge.settlements()[si]["name"])
	for d in ["world", "civilization", "cartography"]:
		await _domain(d)
		await _centre(_st_cell(si))
		pos = _settlement_pos(si)
		_ok(pos.x >= 0.0, "ST %s: the settlement is pickable" % d)
		await _context_at(pos)
		var ids := _ids()
		_ok(ids.has("el.settlement_rename"), "ST %s: Rename present (pos %s, hits %s, ids %s)" % [d, str(pos), str(_app.context_broker.last_request.get("hits")), str(ids)])
		if d == "civilization":
			for need in ["civ.edit", "civ.delete", "civ.vault_note"]:
				_ok(ids.has(need), "ST civilization: CIVIL's own %s kept" % need)
			_ok(not ids.has("el.settlement_delete") and not ids.has("el.settlement_note"), "ST civilization: no duplicate Delete or Note")
		else:
			for need in ["el.settlement_edit", "el.settlement_delete", "el.settlement_note"]:
				_ok(ids.has(need), "ST %s: %s present" % [d, need])
		await _close_card()
	# Rename in CARTO through the card and dialog.
	await _domain("cartography")
	await _centre(_st_cell(si))
	pos = _settlement_pos(si)
	await _context_at(pos)
	_ok(await _press("el.settlement_rename"), "ST press Rename in CARTO")
	_ok(await _answer_prompt("Probetown"), "ST the prompt took the name")
	_ok(String(bridge.settlements()[si]["name"]) == "Probetown", "ST the settlement is now Probetown (was %s)" % name0)
	# Delete in WORLD: cancel keeps it, confirm removes it.
	var n0: int = bridge.settlements().size()
	await _domain("world")
	await _centre(_st_cell(si))
	pos = _settlement_pos(si)
	await _context_at(pos)
	_ok(await _press("el.settlement_delete"), "ST press Delete in WORLD")
	_ok(await _answer_confirm(false), "ST the confirm appeared and was cancelled")
	_ok(bridge.settlements().size() == n0, "ST cancelling deleted nothing")
	await _context_at(pos)
	_ok(await _press("el.settlement_delete"), "ST press Delete again")
	_ok(await _answer_confirm(true), "ST the confirm was accepted")
	_ok(bridge.settlements().size() == n0 - 1, "ST confirming removed exactly one settlement (%d -> %d)" % [n0, bridge.settlements().size()])
	await _close_card()


## An empty land spot (host space) at least `away` px from `other`, scanning a lattice of the map interior.
func _empty_spot(other: Vector2, away: float) -> Vector2:
	## A phone's map is narrow and a WORLD pick also hits rivers, so the requested distance is relaxed (to 2/3, then
	## 1/3) before the probe gives up; the CALLER still gets the farthest spot the view allows.
	for k in [1.0, 0.66, 0.33]:
		var s := _empty_spot_at(other, away * k)
		if s.x >= 0.0:
			return s
	## A phone's WORLD map can be all river: take any spot that is clear of an ELEMENT, at the shortest distance.
	return _empty_spot_at(other, away * 0.33, false)


func _empty_spot_at(other: Vector2, away: float, strict := true) -> Vector2:
	var rect: Rect2 = _ov._displayed_rect()
	var inter: Rect2 = _ov._interior_rect(rect).grow(-60.0)
	var y := inter.position.y
	while y < inter.end.y:
		var x := inter.position.x
		while x < inter.end.x:
			var lp := Vector2(x, y)
			x += 17.0
			var hp: Vector2 = _ov.get_global_transform() * lp
			## 90 px off the screen edges: the phone opens its dock sheets on an edge swipe, so a finger that lands there
			## is not the map's (measured: a Move destination at x = 43 never completed its tap).
			if (other.x >= 0.0 and hp.distance_to(other) < away) or not _app.viewport._on_map(hp) \
					or hp.x < 90.0 or hp.x > float(_vw) - 90.0 or _phone_chrome(hp):
				continue
			## Never under the chip strip (a tap there is a chip press, not bare ground).
			if _app.element_chips != null and _app.element_chips.visible and _app.element_chips.get_global_rect().grow(24.0).has_point(hp):
				continue
			## On touch the pick is a finger's (a way within 22 px counts), so bare ground must be bare for THAT request.
			var spot_hits: Array = _ov.request_at(lp, "touch").get("hits", []) if _touch else _ov.hits_at(lp)
			if strict and not spot_hits.is_empty():
				continue
			## Loose: a river or road under the spot is allowed (a destination click does not mind), an element is not.
			if not strict and spot_hits.any(func(h): return String((h as Dictionary)["kind"]) in ["settlement", "landmark", "label", "icon"]):
				continue
			var g: Dictionary = _ov._grid_point(lp, rect, _ov._interior_rect(rect))
			var cell: Dictionary = _app.bridge.sample_cell(int(g["gx"]), int(g["gy"]))
			if String(cell.get("water", "")) == "land":
				return hp
		y += 17.0
	return Vector2(-1, -1)


func _cell_of(hp: Vector2) -> Vector2:
	var rect: Rect2 = _ov._displayed_rect()
	var g: Dictionary = _ov._grid_point(_local(hp), rect, _ov._interior_rect(rect))
	return Vector2(float(g["gx"]), float(g["gy"]))


func _leg_label_icon(bridge) -> void:
	print("== LB/IC label and icon ==")
	await _domain("world")
	await _centre(Vector2(128, 96))
	var a := _empty_spot(Vector2(-1, -1), 0.0)
	_ok(a.x >= 0.0, "LB an empty land spot for the label")
	if a.x < 0.0:
		return
	var ca := _cell_of(a)
	var li: int = bridge.label_create(ca.x, ca.y, "Probe label")
	_ok(li >= 0, "LB a label was created")
	_app.viewport.refresh_annotations()
	await _frames(4)
	var pa := await _label_hit_pos(li)
	_ok(pa.x >= 0.0, "LB the label is pickable on screen")
	if pa.x < 0.0:
		return
	for d in ["world", "civilization", "cartography"]:
		await _domain(d)
		pa = await _label_hit_pos(li)
		await _context_at(pa)
		var ids := _ids()
		for need in ["el.label_rename", "el.label_move"]:
			_ok(ids.has(need), "LB %s: %s present (pa %s hits %s ids %s)" % [d, need, str(pa), str(_app.context_broker.last_request.get("hits")), str(ids)])
		_ok(ids.has("el.label_delete") == (d != "cartography"), "LB %s: Delete from the shared provider only outside CARTO" % d)
		_ok(ids.has("el.label_note"), "LBN %s: the Note row is offered on a label (Ruling EQ item 2)" % d)
		await _close_card()
	## Ruling EQ item 2: right-click > Note on a label. A real vault folder is connected so the note can exist.
	var root := ProjectSettings.globalize_path("user://") + "eq2_vault"
	DirAccess.make_dir_recursive_absolute(root + "/Labels")
	var wf := FileAccess.open(root + "/Labels/Probe label.md", FileAccess.WRITE)
	wf.store_string("# Probe label

A note for a label.
")
	wf.close()
	_ok(bool(bridge.vault_connect(root, "EQ").get("ok", false)), "LBN a vault folder is connected")
	await _domain("world")
	pa = await _label_hit_pos(li)
	await _context_at(pa)
	_ok(String(_row_of("el.label_note").get("label", "")) == "Attach note to Probe label…", "LBN unlinked: the row says Attach note to Probe label… (%s)" % str(_row_of("el.label_note").get("label")))
	await _close_card()
	var gid: Dictionary = bridge.vault_label_entity_id(li)
	_ok(bool(gid.get("ok", false)) and gid.has("entity_id") and String(gid.get("label", "")) == "Probe label", "LBN the engine names the label's vault id (%s)" % str(gid))
	var eid := int(gid.get("entity_id", 0))
	_ok(bool(bridge.vault_attach("label", eid, "Probe label", "Labels/Probe label.md", "").get("ok", false)), "LBN a note is attached to the label")
	_ok(bridge.vault_links_for("label", eid).size() == 1 and bridge.vault_links_for("settlement", eid).is_empty(), "LBN filed under kind label only")
	await _context_at(pa)
	_ok(String(_row_of("el.label_note").get("label", "")) == "Open note for Probe label", "LBN linked: the row says Open note for Probe label (%s)" % str(_row_of("el.label_note").get("label")))
	_ok(await _press("el.label_note"), "LBN press Open note")
	await _frames(6)
	var vwin = _app.vault_window
	_ok(vwin != null and vwin.visible and String(vwin._kind) == "label" and int(vwin._entity_id) == eid, "LBN the vault window opened for kind label at that entity")
	var rez: Dictionary = vwin._resolve_entity("label", eid)
	_ok(String(rez.get("name", "")) == "Probe label" and rez.has("x") and rez.has("y"), "LBN the window resolves the label's name and cell (%s)" % str(rez))
	vwin.hide()
	await _frames(3)
	await _domain("world")
	pa = await _label_hit_pos(li)
	await _context_at(pa)
	_ok(await _press("el.label_rename"), "LB press Rename")
	_ok(await _answer_prompt("Renamed label"), "LB prompt answered")
	_ok(String(bridge.label_get(li).get("text", "")) == "Renamed label", "LB the label text changed")
	await _domain("civilization")
	pa = await _label_hit_pos(li)
	await _context_at(pa)
	_ok(await _press("el.label_move"), "LB press Move")
	_ok(_app.armed_tool == "move_element", "LB Move armed the destination click (armed %s)" % _app.armed_tool)
	await _close_card()  # closing the phone sheet re-lays the map out: choose the destination only after it
	## The destination is chosen NOW, in the view the click lands in: a domain switch and the label re-find above
	## re-centre the view, which made a spot picked earlier point at other ground on a phone-sized map.
	var dest := _empty_spot(pa, 120.0)
	_ok(dest.x >= 0.0, "LB a destination spot exists")
	## The expected cell is read BEFORE the click: a move can re-fit the view (it did, by 36 cells), and a cell read
	## afterwards names other ground than the tap landed on.
	var cd := _cell_of(dest)
	await _map_click(dest)
	var lb: Dictionary = bridge.label_get(li)
	_ok(absf(float(lb.get("x", -99)) - cd.x) < 1.0 and absf(float(lb.get("y", -99)) - cd.y) < 1.0, "LB the label moved to the clicked cell (%s,%s vs %s)" % [str(lb.get("x")), str(lb.get("y")), str(cd)])
	_ok(_app.armed_tool == "inspect", "LB the map is handed back to Inspect")
	var eid2 := int(bridge.vault_label_entity_id(li).get("entity_id", 0))
	_ok(eid2 != 0 and eid2 != eid, "LBN the moved label has a new cell id")
	_ok(bridge.vault_links_for("label", eid2).size() == 1 and bridge.vault_links_for("label", eid).is_empty(), "LBN the note followed the label to its new cell (Ruling EQ item 2)")
	var nl: int = bridge.label_list().size()
	await _domain("world")
	pa = await _label_hit_pos(li)
	await _context_at(pa)
	_ok(await _press("el.label_delete"), "LB press Delete")
	_ok(await _answer_confirm(false), "LB confirm shown, cancelled")
	_ok(bridge.label_list().size() == nl, "LB cancel deleted nothing")
	await _context_at(pa)
	_ok(await _press("el.label_delete"), "LB press Delete again")
	_ok(await _answer_confirm(true), "LB confirmed")
	_ok(bridge.label_list().size() == nl - 1, "LB exactly one label removed")
	await _close_card()

	var pack: String = ProjectSettings.globalize_path("res://") + "../crates/cartalith-assets/tests/fixtures/reference_pack.zip"
	var ok_pack: bool = bridge.world_gen.load_asset_pack(pack)
	_ok(ok_pack, "IC the fixture pack loaded")
	if not ok_pack:
		return
	var b := _empty_spot(Vector2(-1, -1), 0.0)
	var cb := _cell_of(b)
	_ok(bridge.icon_arm("feature", 0, 1.0, 0.0, 0.0), "IC an icon armed")
	var ii: int = bridge.icon_place(cb.x, cb.y)
	bridge.icon_disarm()
	_ok(ii >= 0, "IC an icon placed")
	_app.viewport.refresh_annotations()
	await _frames(4)
	var ieid := int(bridge.vault_icon_entity_id(ii).get("entity_id", 0))
	_ok(ieid != 0, "ICN the engine names the icon's vault id")
	var wf2 := FileAccess.open(root + "/Labels/Icon note.md", FileAccess.WRITE)
	wf2.store_string("# Icon note
")
	wf2.close()
	_ok(bool(bridge.vault_attach("icon", ieid, "Icon", "Labels/Icon note.md", "").get("ok", false)), "ICN a note is attached to the icon")
	var pb := await _icon_hit_pos(ii)
	_ok(pb.x >= 0.0, "IC the icon is pickable")
	if pb.x < 0.0:
		return
	for d2 in ["world", "civilization", "cartography"]:
		await _domain(d2)
		pb = await _icon_hit_pos(ii)
		await _context_at(pb)
		var ids2 := _ids()
		_ok(ids2.has("el.icon_move"), "IC %s: Move present" % d2)
		_ok(ids2.has("el.icon_note"), "ICN %s: the Note row is offered on an icon (Ruling EQ item 2)" % d2)
		_ok(ids2.has("el.icon_delete") == (d2 != "cartography"), "IC %s: Delete from the shared provider only outside CARTO" % d2)
		await _close_card()
	await _domain("world")
	pb = await _icon_hit_pos(ii)
	var dest2 := _empty_spot(pb, 120.0)
	await _context_at(pb)
	_ok(await _press("el.icon_move"), "IC press Move")
	await _close_card()
	var cd2 := _cell_of(dest2)
	await _map_click(dest2)
	var ic: Dictionary = bridge.icon_get(ii)
	_ok(absf(float(ic.get("x", -99)) - cd2.x) < 1.0 and absf(float(ic.get("y", -99)) - cd2.y) < 1.0, "IC the icon moved to the clicked cell")
	var ieid2 := int(bridge.vault_icon_entity_id(ii).get("entity_id", 0))
	_ok(ieid2 != 0 and ieid2 != ieid and bridge.vault_links_for("icon", ieid2).size() == 1 and bridge.vault_links_for("icon", ieid).is_empty(), "ICN the note followed the icon to its new cell")
	var ni: int = bridge.icon_list().size()
	pb = await _icon_hit_pos(ii)
	await _context_at(pb)
	_ok(await _press("el.icon_delete"), "IC press Delete")
	_ok(await _answer_confirm(false), "IC confirm cancelled")
	_ok(bridge.icon_list().size() == ni, "IC cancel deleted nothing")
	await _context_at(pb)
	_ok(await _press("el.icon_delete"), "IC press Delete again")
	_ok(await _answer_confirm(true), "IC confirmed")
	_ok(bridge.icon_list().size() == ni - 1, "IC exactly one icon removed")
	await _close_card()


func _label_hit_pos(li: int, only := false) -> Vector2:
	var lb: Dictionary = _app.bridge.label_get(li)
	var cell := Vector2(float(lb.get("x", 0)), float(lb.get("y", 0)))
	await _centre(cell)
	return _scan_for("label", li, _pos_of_cell(cell), only)


func _icon_hit_pos(ii: int, only := false) -> Vector2:
	var ic: Dictionary = _app.bridge.icon_get(ii)
	var cell := Vector2(float(ic.get("x", 0)) + 0.5, float(ic.get("y", 0)) + 0.5)
	await _centre(cell)
	return _scan_for("icon", ii, _pos_of_cell(cell), only)


## The host position near `base` whose first hit of `kind` has `id`, scanning a small square, or (-1,-1). With `only`,
## the position must also be one where a pick SELECTS that element: it is the nearest hit of the request a click (or
## a tap, on touch) would make, and no settlement or landmark is under it (those win the selection).
func _scan_for(kind: String, id: int, base: Vector2, only := false) -> Vector2:
	if base.x < 0.0:
		return base
	for dy in range(-14, 15, 2):
		for dx in range(-14, 15, 2):
			var p := base + Vector2(dx, dy)
			if not _app.viewport._on_map(p):
				continue
			var hits: Array = _ov.request_at(_local(p), "touch" if _touch else "mouse").get("hits", []) if only else _ov.hits_at(_local(p))
			if only:
				var clean := not hits.is_empty() and String((hits[0] as Dictionary)["kind"]) == kind and int((hits[0] as Dictionary)["id"]) == id
				for h in hits:
					if String((h as Dictionary)["kind"]) in ["settlement", "landmark"]:
						clean = false
				if clean:
					return p
				continue
			for h in hits:
				if String((h as Dictionary)["kind"]) == kind:
					if int((h as Dictionary)["id"]) == id:
						return p
					break
	return Vector2(-1, -1)


func _leg_route(bridge) -> void:
	print("== RT route ==")
	bridge.route_begin("mixed")
	bridge.route_append_stop(256 * 0.25, 192 * 0.25)
	bridge.route_append_stop(256 * 0.35, 192 * 0.30)
	var ri: int = bridge.route_commit()
	_ok(ri >= 0, "RT a route committed")
	if ri < 0:
		return
	var pts: PackedVector2Array = bridge.route_get(ri).get("points", PackedVector2Array())
	await _domain("civilization")
	await _centre(pts[pts.size() / 2])
	var p := _pos_of_cell(pts[pts.size() / 2])
	_ok(p.x >= 0.0, "RT the route midpoint is on screen")
	_app.arm_tool("inspect")
	await _context_at(p)
	_ok(_ids().has("civ.route_rename"), "RT CIVIL offers Rename")
	_ok(await _press("civ.route_rename"), "RT press Rename")
	_ok(await _answer_prompt("Amber way"), "RT prompt answered")
	_ok(String(bridge.route_get(ri).get("name", "")) == "Amber way", "RT the route is named Amber way")
	await _close_card()


func _chip_center(w: String) -> Vector2:
	var b: Button = _app.element_chips.chip_button(w)
	if b == null:
		return Vector2(-1, -1)
	return b.get_global_transform() * (b.size * 0.5)


func _leg_touch(bridge) -> void:
	print("== TC touch chips ==")
	var chips = _app.element_chips
	_ok(chips != null, "TC the chip strip exists")
	if not _touch:
		# Desktop: a mouse click on an element must never show chips.
		await _domain("civilization")
		var fd := _find_landmark()
		if int(fd[0]) >= 0:
			await _click_at(fd[1] - _off)
			_ok(not chips.visible, "TC desktop: a click shows no chips")
		return
	await _domain("civilization")
	_app.arm_tool("inspect")
	# Landmark: tap shows Rename and Note, not Move or Delete.
	await _centre(_lm_cell(0))
	var f := _find_landmark()
	_ok(int(f[0]) >= 0, "TC a landmark is pickable")
	var li: int = f[0]
	var lp: Vector2 = f[1]
	await _tap(lp)
	_ok(chips.visible, "TC a tap on a landmark shows the chips")
	var words: Array = chips.chip_words()
	_ok(words.has("Rename") and words.has("Note") and not words.has("Delete") and not words.has("Move"), "TC landmark chips are Rename and Note (got %s)" % str(words))
	if DccTheme.is_phone():
		## The tap's selection opens the phone's right sheet over the map, which would cover the chips (found by this
		## probe: a finger on Rename pressed the sheet's "Why it is here"). With chips up the sheet must be closed.
		_ok(not _app.right_dock.visible, "TC phone: the right sheet is closed while the chips are up")
	# Press Rename through the real chip, answer the prompt.
	var cc := _chip_center("Rename")
	await _move_to(cc)
	await _tap(cc)
	_ok(_visible_dialogs() == 1, "TC one tap on a chip opens ONE prompt, not one per delivered event (%d)" % _visible_dialogs())
	_ok(await _answer_prompt("Chip name"), "TC the Rename chip opened the prompt")
	var nm := String((bridge.landmark_names()[li] as Dictionary)["name"])
	_ok(nm == "Chip name", "TC the landmark is now Chip name (got %s)" % nm)
	_ok(not chips.visible, "TC pressing a chip hides the strip")
	# Long press keeps the full card.
	await _centre(_lm_cell(li))
	lp = _landmark_pos(li)
	await _context_at(lp)
	var ids := _ids()
	_ok(ids.has("global.centre") and ids.has("el.landmark_rename"), "TC a long press keeps the full card (ids %s)" % str(ids))
	await _close_card()
	# Settlement in CIVIL: Rename, Note, Delete chips; Delete confirms.
	await _centre(_st_cell(_inland_settlement()))
	var fs := _find_settlement()
	_ok(int(fs[0]) >= 0, "TC a settlement is pickable")
	var si: int = fs[0]
	await _tap(fs[1])
	words = chips.chip_words()
	_ok(words.has("Rename") and words.has("Note") and words.has("Delete"), "TC settlement chips are Rename, Note, Delete (got %s)" % str(words))
	var n0: int = bridge.settlements().size()
	await _tap(_chip_center("Delete"))
	_ok(await _answer_confirm(false), "TC the Delete chip asked first")
	_ok(bridge.settlements().size() == n0, "TC cancelling the chip's confirm deleted nothing")
	# A tap on bare ground hides the strip.
	await _tap(fs[1])
	_ok(chips.visible, "TC chips are back after tapping the settlement again")
	await _centre(_st_cell(si))
	var empty := _empty_spot(Vector2(-1, -1), 0.0)
	await _tap(empty)
	_ok(not chips.visible, "TC a tap on bare ground hides the chips")
	# Label: Move, Rename, Delete (not CARTO).
	await _centre(Vector2(128, 96))
	var a := _empty_spot(Vector2(-1, -1), 0.0)
	var ca := _cell_of(a)
	var lbi: int = bridge.label_create(ca.x, ca.y, "Chip label")
	_app.viewport.refresh_annotations()
	await _frames(4)
	var pl := await _label_hit_pos(lbi)
	_ok(pl.x >= 0.0, "TC the label is pickable")
	if pl.x >= 0.0:
		await _tap(pl)
		words = chips.chip_words()
		_ok(words.has("Move") and words.has("Rename") and words.has("Delete"), "TC label chips are Move, Rename, Delete (got %s)" % str(words))
		await _tap(_chip_center("Move"))
		_ok(_app.armed_tool == "move_element", "TC the Move chip armed the destination click")
		var dest := _empty_spot(a, 120.0)
		await _tap(dest - _off + _off)
		var cd := _cell_of(dest)
		var lb: Dictionary = bridge.label_get(lbi)
		_ok(absf(float(lb.get("x", -99)) - cd.x) < 1.0 and absf(float(lb.get("y", -99)) - cd.y) < 1.0, "TC the tapped destination moved the label")
		_ok(_app.armed_tool == "inspect", "TC the map is back on Inspect")


## SAVE: a user landmark name is saved with the world and comes back after a reopen (Ruling EL). Last, because a reopened
## world is a loaded one (no label or route fixtures after it).
func _leg_save(bridge) -> void:
	print("== SAVE ==")
	var key := _lm_key
	_ok(bridge.landmark_set_user_name(key, "The Weeping Stair"), "SAVE a name is set")
	var path := OS.get_user_data_dir().path_join("edfc_lm.cartalith")
	_ok(bridge.save_project(path), "SAVE the project saved")
	_ok(bridge.load_save(path), "SAVE the project reopened")
	await _frames(6)
	var found := false
	for n in bridge.landmark_names():
		if String((n as Dictionary)["key"]) == key:
			found = String((n as Dictionary)["name"]) == "The Weeping Stair" and String((n as Dictionary)["source"]) == "user"
	_ok(found, "SAVE the user name survived save and reopen")


# ---- NT: selection kinds and the Note tab -----------------------------------------------------------------------------

## Saves the probe's root window to `<--shots dir>/<name>.png` when `--shots` was given. Evidence only: never asserted.
func _shot(name: String) -> void:
	var dir := _arg("--shots", "")
	if dir == "":
		return
	DirAccess.make_dir_recursive_absolute(dir)
	await _frames(2)
	_vp.get_texture().get_image().save_png(dir.path_join(name + ".png"))


## The Note tab's resolver: the dock's own when it built one (desktop, tablet), else a fresh instance (the phone's
## sheet has no tab strip, so the dock never builds the tab; the resolver is the same class either way).
func _resolver(dock):
	return dock._note if dock._note != null else load("res://shell/context_note.gd").new(_app)


## A label and an icon become SELECTION subjects (a click on desktop, a tap with chips on touch) and the Context
## panel's Note tab resolves the bound note for them (Ruling EQ item 2, "Note tab" follow-up). Legs, each with literals:
##   NTL  label: the pick selects {label, index}; the header names it; Note resolves no_note, then note after an attach
##   NTI  icon: the same
##   NTD  deleting a label that sits before the selected one drops the selection (indices shift)
##   NTP  the phone's equivalent (no tabs): the sheet's page offers the Note row; a chip press leaves the world pause level
func _leg_note(bridge) -> void:
	print("== NT selection and the Note tab ==")
	await _domain("world")
	await _centre(Vector2(128, 96))
	_app.arm_tool("inspect")
	var root := ProjectSettings.globalize_path("user://") + "nt_vault"
	DirAccess.make_dir_recursive_absolute(root + "/Notes")
	var wf := FileAccess.open(root + "/Notes/Note label.md", FileAccess.WRITE)
	wf.store_string("# Note label\n\nbody\n")
	wf.close()
	_ok(bool(bridge.vault_connect(root, "NT").get("ok", false)), "NT a vault folder is connected")
	var a := _empty_spot(Vector2(-1, -1), 0.0)
	var b := _empty_spot(a, 150.0)
	_ok(a.x >= 0.0 and b.x >= 0.0, "NT two empty land spots")
	if a.x < 0.0 or b.x < 0.0:
		return
	var ca := _cell_of(a)
	var cb := _cell_of(b)
	var early: int = bridge.label_create(cb.x, cb.y, "Early label")
	var li: int = bridge.label_create(ca.x, ca.y, "Note label")
	## An earlier leg may have left a note linked to a cell id (a deleted label's link stays), and a label created on
	## that cell would inherit it: step along until this label's own id has no link, so "unlinked" is true by
	## construction and not by luck.
	var tries := 0
	while tries < 8 and not bridge.vault_links_for("label", int(bridge.vault_label_entity_id(li).get("entity_id", 0))).is_empty():
		bridge.label_delete(li)
		ca.x += 3.0
		li = bridge.label_create(ca.x, ca.y, "Note label")
		tries += 1
	_ok(tries < 8, "NT a fresh cell with no inherited link (%d tries)" % tries)
	_ok(early >= 0 and li > early, "NT two labels, the one under test after the other (%d, %d)" % [early, li])
	_app.viewport.refresh_annotations()
	await _frames(4)
	var dock = _app.right_dock_ctrl
	var depth0: int = _app._context_surface_depth
	# --- NTL: pick the label ---
	var pl := await _label_hit_pos(li, true)
	_ok(pl.x >= 0.0, "NTL the label is pickable")
	if pl.x < 0.0:
		return
	await _map_click(pl)
	await _frames(4)
	_ok(_app.selection == {"kind": "label", "id": li}, "NTL a %s on a label selects it (%s)" % ["tap" if _touch else "click", str(_app.selection)])
	_ok(dock.shown_subject() == {"kind": "label", "id": li}, "NTL the Context panel shows that subject (%s)" % str(dock.shown_subject()))
	_ok(dock.subject_name(_app.selection) == "Note label", "NTL the header names the label (%s)" % dock.subject_name(_app.selection))
	await _shot("nt_label_selected")
	var r: Dictionary = _resolver(dock).resolve(_app.selection)
	_ok(String(r.get("state", "")) == "no_note" and String(r.get("vk", "")) == "label", "NTL unlinked: the Note tab resolves no_note for kind label (%s)" % str(r))
	var eid := int(bridge.vault_label_entity_id(li).get("entity_id", 0))
	_ok(eid != 0 and int(r.get("vid", -1)) == eid, "NTL the tab's entity id is the engine's cell id, not the index")
	_ok(bool(bridge.vault_attach("label", eid, "Note label", "Notes/Note label.md", "").get("ok", false)), "NTL a note is attached")
	r = _resolver(dock).resolve(_app.selection)
	_ok(String(r.get("state", "")) == "note" and String(r.get("rel", "")) == "Notes/Note label.md", "NTL linked: the tab resolves the note (%s)" % str(r))
	if not DccTheme.is_phone():
		_ok(dock.show_tab("note"), "NTL the Note tab opens")
		await _frames(6)
		var vw = _app.vault_window
		_ok(vw.is_docked() and vw.docked_path().ends_with("Note label.md"), "NTL the tab docks the label's note editor (%s)" % vw.docked_path())
		dock.show_tab("inspect")
		await _frames(3)
	else:
		# The phone has no tabs: its sheet's Inspect page carries the Note row, and the chip opens the vault.
		dock.refresh_shown()  # the page was built before the note was attached
		_app.phone_show_right_sheet()
		await _frames(4)
		var act = _app.right_dock_body.find_child("Act_el_label_note", true, false)
		_ok(act != null, "NTP the phone sheet's label page offers the Note row")
		if act != null:
			_ok(String((act as Button).text).to_lower() == "open note for note label", "NTP the row reads Open note for Note label (%s)" % (act as Button).text)
			## 44 reference px of a 412-wide canvas, in this viewport's physical px, less 5 % for rounding: derived from
			## the viewport width the probe was started with, not from the shell's own scale.
			var floor_px := 44.0 * float(_vw) / 412.0 * 0.95
			_ok((act as Button).size.y >= floor_px, "NTP the Note row is at least 44 dp tall: %.0f px >= %.0f (%s)" % [(act as Button).size.y, floor_px, str((act as Button).size)])
	if DccTheme.is_phone():
		await _shot("nt_phone_sheet_open")
		_app._close_all_phone_overlays()
		await _frames(3)
	# --- NTD: an earlier label's deletion drops the selection ---
	_ok(bridge.label_delete(early), "NTD the earlier label is deleted")
	await _frames(3)
	_ok(_app.selection.is_empty(), "NTD the selection is dropped, not left on a neighbour (%s)" % str(_app.selection))
	li -= 1
	# --- NTI: an icon ---
	var pack: String = ProjectSettings.globalize_path("res://") + "../crates/cartalith-assets/tests/fixtures/reference_pack.zip"
	_ok(bridge.world_gen.load_asset_pack(pack), "NTI the fixture pack loaded")
	var c := _empty_spot(a, 150.0)
	var cc := _cell_of(c)
	_ok(bridge.icon_arm("feature", 0, 1.0, 0.0, 0.0), "NTI an icon armed")
	var ii: int = bridge.icon_place(cc.x, cc.y)
	bridge.icon_disarm()
	_ok(ii >= 0, "NTI an icon placed")
	_app.viewport.refresh_annotations()
	await _frames(4)
	var pi := await _icon_hit_pos(ii, true)
	_ok(pi.x >= 0.0, "NTI the icon is pickable")
	if pi.x < 0.0:
		return
	await _map_click(pi)
	await _frames(4)
	_ok(_app.selection == {"kind": "icon", "id": ii}, "NTI a %s on an icon selects it (%s)" % ["tap" if _touch else "click", str(_app.selection)])
	_ok(dock.shown_subject() == {"kind": "icon", "id": ii}, "NTI the panel shows the icon as the subject")
	await _shot("nt_icon_tapped")
	r = _resolver(dock).resolve(_app.selection)
	_ok(String(r.get("state", "")) == "no_note" and String(r.get("vk", "")) == "icon", "NTI unlinked: the Note tab resolves no_note for kind icon (%s)" % str(r))
	var ieid := int(bridge.vault_icon_entity_id(ii).get("entity_id", 0))
	var wf2 := FileAccess.open(root + "/Notes/Icon note.md", FileAccess.WRITE)
	wf2.store_string("# Icon note\n")
	wf2.close()
	_ok(bool(bridge.vault_attach("icon", ieid, "Icon", "Notes/Icon note.md", "").get("ok", false)), "NTI a note is attached to the icon")
	r = _resolver(dock).resolve(_app.selection)
	_ok(String(r.get("state", "")) == "note" and String(r.get("rel", "")) == "Notes/Icon note.md", "NTI linked: the tab resolves the icon's note (%s)" % str(r))
	if _touch:
		# The chip strip is up after a tap; its Note chip opens the vault at the icon, and the world pause is level.
		var chips = _app.element_chips
		_ok(chips.visible and chips.chip_words().has("Note"), "NTP the icon's chips include Note (%s)" % str(chips.chip_words()))
		var nb: Button = chips.chip_button("Note")
		if nb != null:
			var chip_floor := 44.0 * float(_vw) / 412.0 * 0.95 if DccTheme.is_phone() else 44.0
			_ok(nb.size.x >= chip_floor and nb.size.y >= chip_floor, "NTP the Note chip is at least 44 dp square: %s >= %.0f" % [str(nb.size), chip_floor])
			await _tap(_chip_center("Note"))
			await _frames(6)
			_ok(_app.vault_window.visible and String(_app.vault_window._kind) == "icon", "NTP the Note chip opened the vault at kind icon")
			_app.vault_window.hide()
			await _frames(3)
	_ok(_app._context_surface_depth == depth0, "NT the world pause is level after every pick, chip and sheet (%d vs %d)" % [_app._context_surface_depth, depth0])
	# --- a click on bare ground clears it ---
	var empty := _empty_spot(pi, 80.0)
	await _map_click(empty)
	await _frames(3)
	_ok(_app.selection.is_empty(), "NT a pick of bare ground clears the label/icon selection")


# ---- WY: a finger's way pick and the way's Delete chip -----------------------------------------------------------------

## The pick tolerance of a LINE (a way) is a finger's on touch (`ContextBroker.PICK_LINE_TOUCH_PX`, 22 px) and the mouse's
## 6 px otherwise, and a way's Delete chip is offered only when the way is the nearest hit. Literals, not the constants:
##   WYA  a touch request finds a way 10..22 px off the line that the mouse request does not (kills 22 -> 6)
##   WYB  no touch request reports a way farther than 22.5 px, and no mouse request one farther than 6.5 (kills 22 -> 44)
##   WYC  tapping such a spot shows exactly the Delete chip; pressing it deletes the way, and Undo puts it back
func _leg_way(bridge) -> void:
	print("== WY way pick and chip ==")
	if not _touch:
		return
	await _domain("civilization")
	_app.arm_tool("inspect")
	await _centre(_st_cell(_inland_settlement()))
	var rect: Rect2 = _ov._displayed_rect()
	var inter: Rect2 = _ov._interior_rect(rect)
	var found_lp := Vector2(-1, -1)
	var found_hit := {}
	var max_touch := 0.0
	var max_mouse := 0.0
	var seen_way := 0
	var y := inter.position.y + 10.0
	while y < inter.end.y - 10.0:
		var x := inter.position.x + 10.0
		while x < inter.end.x - 10.0:
			var lp := Vector2(x, y)
			x += 5.0
			var hp: Vector2 = _ov.get_global_transform() * lp
			if not _app.viewport._on_map(hp) or _phone_chrome(hp):
				continue
			var th: Array = _ov.request_at(lp, "touch").get("hits", [])
			var mh: Array = _ov.hits_at(lp)
			for h in mh:
				if String((h as Dictionary)["kind"]) == "way":
					max_mouse = maxf(max_mouse, float((h as Dictionary)["dist"]))
			var touch_way := {}
			var only_ways := not th.is_empty()
			for h in th:
				var k := String((h as Dictionary)["kind"])
				if k == "way":
					seen_way += 1
					max_touch = maxf(max_touch, float((h as Dictionary)["dist"]))
					if touch_way.is_empty():
						touch_way = h
				else:
					only_ways = false
			var mouse_has_way := false
			for h in mh:
				if String((h as Dictionary)["kind"]) == "way":
					mouse_has_way = true
			if found_lp.x < 0.0 and only_ways and not touch_way.is_empty() and not mouse_has_way \
					and float(touch_way["dist"]) > 10.0 and float(touch_way["dist"]) <= 22.0:
				found_lp = lp
				found_hit = touch_way
		y += 5.0
	_ok(seen_way > 0, "WY the view holds ways to pick (%d touch way hits)" % seen_way)
	_ok(max_touch <= 22.5, "WYB no touch way hit farther than 22.5 px (max %.1f)" % max_touch)
	_ok(max_mouse <= 6.5, "WYB no mouse way hit farther than 6.5 px (max %.1f)" % max_mouse)
	_ok(found_lp.x >= 0.0, "WYA a touch request finds a way 10..22 px off the line that the mouse request does not")
	if found_lp.x < 0.0:
		return
	var fhp: Vector2 = _ov.get_global_transform() * found_lp
	var store := String(found_hit["store"])
	var idx := int(found_hit["index"])
	var before: Dictionary = bridge.way_get(store, idx)
	_ok(not before.is_empty(), "WYC the picked way exists before the tap")
	await _tap(fhp)
	var chips = _app.element_chips
	_ok(chips.visible and chips.chip_words() == ["Delete"], "WYC tapping beside a way shows exactly the Delete chip (%s)" % str(chips.chip_words()))
	if not chips.visible:
		return
	await _tap(_chip_center("Delete"))
	await _frames(4)
	## Later indices shift down on a delete, so "gone" is "the way at that index is no longer this one".
	_ok(bridge.way_get(store, idx) != before, "WYC the Delete chip deleted the way (index %d of %s now holds another)" % [idx, store])
	_ok(not chips.visible, "WYC pressing the chip hides the strip")
	var undone: String = bridge.undo_last()
	await _frames(4)
	_ok(undone != "" and bridge.way_get(store, idx) == before, "WYC Edit > Undo put the same way back at the same index (%s)" % undone)


func _desc(n: Node) -> String:
	var out := ""
	var d := 0
	while n != null and d < 8:
		out += "%s(%s)%s < " % [n.name, n.get_class(), (" mf=" + str((n as Control).mouse_filter)) if n is Control else ""]
		n = n.get_parent()
		d += 1
	return out
