extends Node
## Verifier probe, 2026-09-12 -- attacks Lane SEED's `request()` -> `seed_input.apply()`
## fix on the production path that writes `seed_input.value` and reads `request()`
## in the SAME frame: `world_workspace.gd::_pg_roll_seed()` is `app._new_seed()`
## (`randomise_seed()`, a direct `.value` write) followed by `_pg_rebuild()` ->
## `_pg_seed_row()` -> `app.new_world_dialog.request()`.
##
## Also measured: a typed value committed by a real Enter key event (the lane's
## "normally committed" control was a property write), and the frame timing of a
## bare `text_submitted` emission (the lane printed without yielding a frame).
##
## Desktop composition. `_pg_roll_seed()` is called directly and the dice's
## `pressed` is emitted -- neither is a finger.
##
## Run windowed: Godot_v4.7.1-stable_win64_console.exe --path . _vfy_seedroll_probe.tscn

var _app: Node
var _emits: Array = []
var _fail := 0

func _p(s: String) -> void:
	print("VFY ", s)

func _check(name: String, cond: bool, detail: String) -> void:
	print("VFY %s %s -- %s" % ["ok  " if cond else "FAIL", name, detail])
	if not cond:
		_fail += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _settle(dlg, v: float) -> void:
	dlg.seed_input.value = v
	await _frames(3)

func _built_seed(dlg) -> int:
	dlg.grid_w_input.set_value_no_signal(4)
	dlg.grid_h_input.set_value_no_signal(4)
	dlg.width_input.set_value_no_signal(40.0)
	dlg._on_create()
	var waited := 0.0
	while _app.bridge.generating and waited < 60.0:
		await get_tree().create_timer(0.1).timeout
		waited += 0.1
	return _app.bridge.world_gen.get_seed()

func _find_seed_label(host: Node) -> String:
	var stack: Array = [host]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is Label and (n as Label).text == "SEED":
			var row := n.get_parent()
			var idx := n.get_index()
			if idx + 1 < row.get_child_count() and row.get_child(idx + 1) is Label:
				return (row.get_child(idx + 1) as Label).text
		for c in n.get_children():
			stack.append(c)
	return "<no SEED row>"

func _find_dice(host: Node) -> Button:
	var stack: Array = [host]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is Button and (n as Button).text == "↻":
			return n
		for c in n.get_children():
			stack.append(c)
	return null

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("VFY WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	await _frames(3)

	var dlg: NewWorldDialog = _app.new_world_dialog
	var le: LineEdit = dlg.seed_input.get_line_edit()
	dlg.seed_input.value_changed.connect(func(v: float): _emits.append(int(v)))

	# -- S0: does LineEdit.text follow a .value write while the dialog is hidden?
	_p("S0 start: value=%d le.text='%s' dlg.visible=%s" % [int(dlg.seed_input.value), le.text, str(dlg.visible)])
	dlg.seed_input.value = 77777
	await _frames(30)
	await get_tree().create_timer(0.5).timeout
	_p("S0 hidden: wrote 77777, +30f+0.5s value=%d le.text='%s'" % [int(dlg.seed_input.value), le.text])
	dlg.popup_centered()
	await _frames(3)
	_p("S0 popped: +3f value=%d le.text='%s'" % [int(dlg.seed_input.value), le.text])
	dlg.hide()
	await _frames(2)
	dlg.seed_input.value = 88888
	await _frames(30)
	_p("S0 hidden again: wrote 88888, +30f value=%d le.text='%s'" % [int(dlg.seed_input.value), le.text])

	# -- S1: bare same-frame roll + request() (dialog closed) -------------------
	await _settle(dlg, 11111)
	_emits.clear()
	dlg.randomise_seed()
	var rolled1 := int(dlg.seed_input.value)
	var text_before_req := le.text
	var req1 := int(dlg.request()["seed"])
	var after1 := int(dlg.seed_input.value)
	await _frames(3)
	_p("S1 same-frame randomise_seed()+request(): rolled=%d le.text_at_request='%s' request.seed=%d value_after=%d emits=%s +3f value=%d text='%s'"
		% [rolled1, text_before_req, req1, after1, str(_emits), int(dlg.seed_input.value), le.text])

	# -- S2: control, one frame between roll and request() ----------------------
	await _settle(dlg, 11111)
	_emits.clear()
	dlg.randomise_seed()
	var rolled2 := int(dlg.seed_input.value)
	await _frames(1)
	var req2 := int(dlg.request()["seed"])
	_p("S2 one frame apart: rolled=%d request.seed=%d emits=%s" % [rolled2, req2, str(_emits)])
	_check("S2 control: roll then request() one frame later keeps the roll", req2 == rolled2,
		"rolled=%d request.seed=%d" % [rolled2, req2])

	# -- S7: desktop user flow -- "New seed", a human pause, the WORLD dock's Generate
	var ws7 = _app._world_workspace()
	await _settle(dlg, 33333)
	dlg.grid_w_input.set_value_no_signal(4)
	dlg.grid_h_input.set_value_no_signal(4)
	dlg.width_input.set_value_no_signal(40.0)
	_app._new_seed()
	var rolled7 := int(dlg.seed_input.value)
	await get_tree().create_timer(0.5).timeout
	ws7._regenerate_now()
	var waited7 := 0.0
	while _app.bridge.generating and waited7 < 60.0:
		await get_tree().create_timer(0.1).timeout
		waited7 += 0.1
	var built7: int = _app.bridge.world_gen.get_seed()
	_check("S7 New seed, 0.5 s, WORLD dock _regenerate_now() builds the rolled seed", built7 == rolled7,
		"rolled=%d built=%d le.text='%s'" % [rolled7, built7, le.text])

	# -- S3: the real production function, _pg_roll_seed() ----------------------
	var ws = _app._world_workspace()
	var host := VBoxContainer.new()
	add_child(host)
	ws.build_phone_generate(host)
	await _frames(3)
	await _settle(dlg, 11111)
	ws._pg_rebuild()
	await _frames(2)
	_p("S3 pre-roll: value=%d seed_row_label='%s'" % [int(dlg.seed_input.value), _find_seed_label(host)])
	_emits.clear()
	ws._pg_roll_seed()
	var emits3 := _emits.duplicate()
	var now3 := int(dlg.seed_input.value)
	await _frames(3)
	var label3 := _find_seed_label(host)
	_p("S3 _pg_roll_seed(): emits=%s value_same_frame=%d +3f value=%d text='%s' seed_row_label='%s'"
		% [str(emits3), now3, int(dlg.seed_input.value), le.text, label3])
	var rolled3: int = emits3[0] if emits3.size() > 0 else -1
	_check("S3 _pg_roll_seed(): the rolled seed survives its own row rebuild",
		rolled3 != -1 and rolled3 != 11111 and int(dlg.seed_input.value) == rolled3,
		"rolled=%d final=%d label='%s'" % [rolled3, int(dlg.seed_input.value), label3])
	var built3: int = await _built_seed(dlg)
	_check("S3 next Create builds the rolled seed", built3 == rolled3,
		"rolled=%d built=%d" % [rolled3, built3])

	# -- S4: the dice's own pressed signal (not a finger) -----------------------
	ws._pg_rebuild()
	await _frames(2)
	await _settle(dlg, 22222)
	ws._pg_rebuild()
	await _frames(2)
	var dice := _find_dice(host)
	_p("S4 dice found=%s" % str(dice != null))
	if dice != null:
		_emits.clear()
		dice.pressed.emit()
		var emits4 := _emits.duplicate()
		await _frames(3)
		var rolled4: int = emits4[0] if emits4.size() > 0 else -1
		_check("S4 dice pressed.emit(): the rolled seed survives",
			rolled4 != -1 and rolled4 != 22222 and int(dlg.seed_input.value) == rolled4,
			"emits=%s final=%d label='%s'" % [str(emits4), int(dlg.seed_input.value), _find_seed_label(host)])

	# -- S5: typed + a real Enter key event, dialog popped ----------------------
	dlg.popup_centered()
	await _frames(3)
	await _settle(dlg, 11111)
	le.grab_focus()
	await _frames(1)
	le.text = "424242"
	le.caret_column = le.text.length()
	var down := InputEventKey.new()
	down.keycode = KEY_ENTER
	down.physical_keycode = KEY_ENTER
	down.pressed = true
	Input.parse_input_event(down)
	var up := down.duplicate() as InputEventKey
	up.pressed = false
	Input.parse_input_event(up)
	Input.flush_buffered_events()
	await _frames(3)
	var v5 := int(dlg.seed_input.value)
	_p("S5 typed 424242 + Enter key event: value=%d text='%s' le_has_focus=%s" % [v5, le.text, str(le.has_focus())])
	var built5: int = await _built_seed(dlg)
	_check("S5 typed + Enter (committed) builds the typed seed", built5 == 424242,
		"value_before_create=%d built=%d" % [v5, built5])

	# -- S6: bare text_submitted emission, no focus change, frame by frame -------
	le.release_focus()
	await _settle(dlg, 12345)
	le.text = "222222"
	le.text_submitted.emit("222222")
	var imm := int(dlg.seed_input.value)
	await _frames(1)
	var f1 := int(dlg.seed_input.value)
	await _frames(1)
	var f2 := int(dlg.seed_input.value)
	_p("S6 bare text_submitted.emit, no focus change: immediate=%d +1f=%d +2f=%d focus_owner=%s"
		% [imm, f1, f2, str(get_viewport().gui_get_focus_owner())])
	await _settle(dlg, 12345)
	le.text = "555555"
	await _frames(3)
	_p("S6b control, no emit, no focus change: +3f value=%d text='%s'" % [int(dlg.seed_input.value), le.text])

	_p("RESULT failed=%d" % _fail)
	get_tree().quit(0)
