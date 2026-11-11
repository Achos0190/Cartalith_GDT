extends Node
## Edit-from-context probe (`EDIT_FROM_CONTEXT_AUDIT.md`, Rulings EI / EL / EO): the shared element rows
## (`GlobalTools.element_rows`) and the touch chips, driven by real pointer events against the real shell.
##
##   godot --path . --resolution 1600x1000 _edfc_probe.tscn -- --vp 1600x1000                    (desktop)
##   godot --path . --resolution 1600x1000 _edfc_probe.tscn -- --vp 1600x1000 --force-touch      (the touch tablet: window-capped, no --density)
##   ... -- --vp 1080x2340 --density 2.8125 --force-touch   (phone: NOT green, 25-34 failures in the probe's phone hold handling; do not cite)
## A 2388x1668 @2 run classifies as a PHONE (the 1600x1000 window is 800 dp), so it is not the tablet form.
## Arguments read by THIS file: `--vp WxH`. Accepted because the shell reads them: `--density D`, `--force-touch`.
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
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument.

const SEED := 24601

var _fails := 0
var _checks := 0
var _vp: SubViewport
var _app: Node
var _ov: Control
var _touch := false
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
		elif a in ["--vp", "--density"]:
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
	await _frames(4)
	return true


func _answer_confirm(yes: bool) -> bool:
	var dlg := _find_dialog()
	if dlg == null:
		return false
	if yes:
		dlg.confirmed.emit()
	else:
		dlg.canceled.emit()
		dlg.hide()
	await _frames(4)
	return true


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
func _pos_of_cell(cell: Vector2) -> Vector2:
	var rect: Rect2 = _ov._displayed_rect()
	var p: Vector2 = _ov._cell_to_screen(cell, rect)
	var inter: Rect2 = _ov._interior_rect(rect)
	if not inter.grow(-24.0).has_point(p):
		return Vector2(-1, -1)
	var hp: Vector2 = _ov.get_global_transform() * p
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
		_app._pick_phone_tab("map")
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

	await _leg_landmark(bridge)
	await _leg_settlement(bridge)
	await _leg_label_icon(bridge)
	await _leg_route(bridge)
	await _leg_touch(bridge)
	await _leg_save(bridge)


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
	var rect: Rect2 = _ov._displayed_rect()
	var inter: Rect2 = _ov._interior_rect(rect).grow(-60.0)
	var y := inter.position.y
	while y < inter.end.y:
		var x := inter.position.x
		while x < inter.end.x:
			var lp := Vector2(x, y)
			x += 17.0
			var hp: Vector2 = _ov.get_global_transform() * lp
			if (other.x >= 0.0 and hp.distance_to(other) < away) or not _app.viewport._on_map(hp):
				continue
			if not _ov.hits_at(lp).is_empty():
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
		await _close_card()
	await _domain("world")
	pa = await _label_hit_pos(li)
	await _context_at(pa)
	_ok(await _press("el.label_rename"), "LB press Rename")
	_ok(await _answer_prompt("Renamed label"), "LB prompt answered")
	_ok(String(bridge.label_get(li).get("text", "")) == "Renamed label", "LB the label text changed")
	var dest := _empty_spot(pa, 120.0)
	_ok(dest.x >= 0.0, "LB a destination spot exists")
	await _domain("civilization")
	pa = await _label_hit_pos(li)
	await _context_at(pa)
	_ok(await _press("el.label_move"), "LB press Move")
	_ok(_app.armed_tool == "move_element", "LB Move armed the destination click (armed %s)" % _app.armed_tool)
	await _close_card()
	await _map_click(dest)
	var cd := _cell_of(dest)
	var lb: Dictionary = bridge.label_get(li)
	_ok(absf(float(lb.get("x", -99)) - cd.x) < 1.0 and absf(float(lb.get("y", -99)) - cd.y) < 1.0, "LB the label moved to the clicked cell (%s,%s vs %s)" % [str(lb.get("x")), str(lb.get("y")), str(cd)])
	_ok(_app.armed_tool == "inspect", "LB the map is handed back to Inspect")
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
		_ok(ids2.has("el.icon_delete") == (d2 != "cartography"), "IC %s: Delete from the shared provider only outside CARTO" % d2)
		await _close_card()
	await _domain("world")
	pb = await _icon_hit_pos(ii)
	var dest2 := _empty_spot(pb, 120.0)
	await _context_at(pb)
	_ok(await _press("el.icon_move"), "IC press Move")
	await _close_card()
	await _map_click(dest2)
	var cd2 := _cell_of(dest2)
	var ic: Dictionary = bridge.icon_get(ii)
	_ok(absf(float(ic.get("x", -99)) - cd2.x) < 1.0 and absf(float(ic.get("y", -99)) - cd2.y) < 1.0, "IC the icon moved to the clicked cell")
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


func _label_hit_pos(li: int) -> Vector2:
	var lb: Dictionary = _app.bridge.label_get(li)
	var cell := Vector2(float(lb.get("x", 0)), float(lb.get("y", 0)))
	await _centre(cell)
	return _scan_for("label", li, _pos_of_cell(cell))


func _icon_hit_pos(ii: int) -> Vector2:
	var ic: Dictionary = _app.bridge.icon_get(ii)
	var cell := Vector2(float(ic.get("x", 0)) + 0.5, float(ic.get("y", 0)) + 0.5)
	await _centre(cell)
	return _scan_for("icon", ii, _pos_of_cell(cell))


## The host position near `base` whose first hit of `kind` has `id`, scanning a small square, or (-1,-1).
func _scan_for(kind: String, id: int, base: Vector2) -> Vector2:
	if base.x < 0.0:
		return base
	for dy in range(-14, 15, 2):
		for dx in range(-14, 15, 2):
			var p := base + Vector2(dx, dy)
			if not _app.viewport._on_map(p):
				continue
			for h in _ov.hits_at(_local(p)):
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
	# Press Rename through the real chip, answer the prompt.
	var cc := _chip_center("Rename")
	await _move_to(cc)
	await _tap(cc)
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
	await _centre(_st_cell(0))
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


func _desc(n: Node) -> String:
	var out := ""
	var d := 0
	while n != null and d < 8:
		out += "%s(%s)%s < " % [n.name, n.get_class(), (" mf=" + str((n as Control).mouse_filter)) if n is Control else ""]
		n = n.get_parent()
		d += 1
	return out
