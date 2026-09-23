extends Node
## Windowed proof for the place editor's Age control (`OUTSTANDING_WORK.md`
## §2.10, 2026-09-24). It used to be a bare number field showing `-1` whenever
## no age was set -- "no value" drawn as though it were an age.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _peage_probe.tscn
##
## On a REAL generated world, asserts:
##   1. `civ_settlement_details` carries `age_inferred`, and it equals the
##      reference formula `_umInferAge` over the layout's floored population;
##   2. an unedited settlement shows "Auto (~N yr)" and NO number field at -1;
##   3. choosing Set stores exactly N (so switching changes nothing yet) and
##      reveals a number field reading N;
##   4. choosing Auto again stores -1 and removes the field.

var _app: Node
var _bridge: Node
var _fails := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	print("PEAGE %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _find(n: Node, cls: String, out: Array) -> void:
	for c in n.get_children():
		if c.is_class(cls):
			out.append(c)
		_find(c, cls, out)

func _age_picker(pe: Node) -> OptionButton:
	var obs: Array = []
	_find(pe, "OptionButton", obs)
	for ob in obs:
		if (ob as OptionButton).item_count == 2 and (ob as OptionButton).get_item_text(1) == "Set" \
				and (ob as OptionButton).get_item_text(0).begins_with("Auto"):
			return ob
	return null

func _spin_values(pe: Node) -> Array:
	var sbs: Array = []
	_find(pe, "SpinBox", sbs)
	var out: Array = []
	for sb in sbs:
		out.append(int((sb as SpinBox).value))
	return out

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("PEAGE REFUSED: headless")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("PEAGE WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	_bridge.generate({"seed": 9137, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)
	var places: Array = _bridge.settlements()
	if places.is_empty():
		print("PEAGE ABORT: no settlement generated")
		get_tree().quit(2)
		return

	# 1. the engine value, against the reference formula computed here
	var d: Dictionary = _bridge.civ_settlement_details(0)
	var pop := maxf(20.0, float((places[0] as Dictionary).get("population", (places[0] as Dictionary).get("pop", 0))))
	var want := int(clampf(roundf(60.0 + 240.0 * (log(maxf(1.0, pop / 100.0)) / log(10.0))), 30.0, 1000.0))
	var inferred := int(d.get("age_inferred", -999))
	_check("details carry age_inferred = _umInferAge(pop)", inferred == want,
		"got %d, formula %d, pop %.0f" % [inferred, want, pop])
	_check("unedited settlement stores no age (-1)", int(d.get("age", 0)) == -1)

	# 2. the drawn control
	var pe = _app.place_editor_window
	pe.open_for(0)
	await _frames(8)
	var ob := _age_picker(pe)
	_check("Age is an Auto/Set picker", ob != null)
	if ob == null:
		get_tree().quit(1)
		return
	_check("Auto names the inferred age", ob.get_item_text(0) == "Auto (~%d yr)" % inferred, ob.get_item_text(0))
	_check("Auto is selected", ob.selected == 0)
	_check("no number field shows -1", not _spin_values(pe).has(-1), str(_spin_values(pe)))

	# 3. Set stores exactly the inferred age and shows it
	ob.select(1)
	ob.item_selected.emit(1)
	await _frames(8)
	var after_set := int(_bridge.civ_settlement_details(0).get("age", -2))
	_check("Set stores the inferred age", after_set == inferred, str(after_set))
	_check("a number field reads it", _spin_values(pe).has(inferred), str(_spin_values(pe)))

	# 4. Auto clears it again
	ob = _age_picker(pe)
	ob.select(0)
	ob.item_selected.emit(0)
	await _frames(8)
	_check("Auto stores -1 again", int(_bridge.civ_settlement_details(0).get("age", -2)) == -1)
	_check("and the number field is gone", not _spin_values(pe).has(inferred) or inferred < 0, str(_spin_values(pe)))

	print("PEAGE %s  (%d failures)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
