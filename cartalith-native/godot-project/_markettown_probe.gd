extends Node
## Ruling BK header (2026-10-13, probe adopted into git; first run passed on the debug DLL):
## Protects: The Walled Market Town rule-set chip in GenerationRulesWindow is selectable, yields a different town than DEFAULT_RULES, and Reset restores the default town; unknown preset ids are rejected.
## Run (Godot_v4.7.1-stable_win64_console.exe, from godot-project/): --headless --path . _markettown_probe.tscn
## Result: 'PROBE PASS' (exit 0) / 'PROBE FAIL: N assertion(s)'
## Sources: derived from this file's own assertions and a 2026-10-13 run; the feature symbols were grepped, not assumed.
##
## `OUTSTANDING_WORK.md` §2.1, the third profile: checks that the "Walled Market
## Town" rule set (`cartalith_urban::MARKET_TOWN_RULES`) can be selected in the
## real app. It presses the real chip in `GenerationRulesWindow`, reads the
## active rules back, confirms `urban_layouts()` produces a different town,
## and confirms Reset restores the DEFAULT_RULES town byte-for-byte.
##
## Logic only, so headless is fine.
## Run: Godot_v4.7.1-stable_win64_console.exe --headless --path . _markettown_probe.tscn

var _app: Node


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 180.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_app.open_project_dialog.hide()
	await get_tree().process_frame

	var bridge = _app.bridge
	bridge.generate({
		"seed": 90210, "width_km": 1600.0, "grid_w": 192, "grid_h": 144,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout

	var fails := 0
	# The largest settlement that lays out, since the preset matters most there.
	var settlements: Array = bridge.settlements()
	var idx := -1
	var baseline: Dictionary = {}
	var best := -1
	for i in settlements.size():
		var got: Array = bridge.urban_layouts(PackedInt32Array([i]))
		if got.is_empty():
			continue
		var n := int(got[0].get("edge_count", 0))
		if n > best:
			best = n
			idx = i
			baseline = got[0]
	if idx < 0:
		print("FAIL: no settlement produced a layout")
		_finish(1)
		return
	print("PROBE settlement %d baseline edges=%d street_len=%.1f" % [
		idx, int(baseline.get("edge_count", 0)), float(baseline.get("street_len_m", 0.0))])

	# -- press the real chip --------------------------------------------------
	_app.generation_rules_window.open()
	await get_tree().process_frame
	await get_tree().process_frame
	var win: GenerationRulesWindow = _app.generation_rules_window
	var chip: BaseButton = _find_button(win._presets_row, "Walled Market Town")
	if chip == null:
		print("FAIL: no 'Walled Market Town' chip in the presets row")
		_finish(fails + 1)
		return
	chip.pressed.emit()
	await get_tree().process_frame

	var active: Dictionary = bridge.get_active_urban_rules()
	# Literals from MARKET_TOWN_RULES's own definition, not read from Rust.
	var want := {
		"street.parallel_street_spacing": 50.0, "street.segment_length_median": 80.0,
		"street.pierce_chance": 0.03, "street.branch_angle_jitter": 0.18,
		"parcels.plot_depth_variance": 0.15, "parcels.subdivision_cap": 1.0,
		"parcels.frontage_width_variance": 0.22, "settlement.max_wall_generations": 1.0,
	}
	for k in want:
		if not is_equal_approx(float(active.get(k, -1.0)), float(want[k])):
			print("FAIL: %s = %s, want %s" % [k, str(active.get(k)), str(want[k])])
			fails += 1
	if win._active_preset != "Walled Market Town":
		print("FAIL: chip not shown active (%s)" % win._active_preset)
		fails += 1
	# The settlement table must show the active value (1.00), not the default (3.00).
	var shown := ""
	for row in win._settlement_body.get_children():
		if row is HBoxContainer and (row.get_child(0) as Label).text == "Max wall generations":
			shown = (row.get_child(1) as Label).text
	print("PROBE settlement table 'Max wall generations' shows %s" % shown)
	if shown != "1.00":
		print("FAIL: settlement table shows %s, want 1.00" % shown)
		fails += 1

	var mt: Array = bridge.urban_layouts(PackedInt32Array([idx]))
	if mt.is_empty():
		print("FAIL: no layout under the preset")
		fails += 1
	else:
		var m: Dictionary = mt[0]
		print("PROBE market_town edges=%d street_len=%.1f" % [
			int(m.get("edge_count", 0)), float(m.get("street_len_m", 0.0))])
		if int(m.get("edge_count", 0)) == int(baseline.get("edge_count", 0)) \
				and is_equal_approx(float(m.get("street_len_m", 0.0)), float(baseline.get("street_len_m", 0.0))):
			print("FAIL: the preset did not reach urban_layouts()")
			fails += 1

	# -- Reset restores the DEFAULT_RULES town ---------------------------------
	bridge.reset_active_urban_rules()
	var back: Array = bridge.urban_layouts(PackedInt32Array([idx]))
	if back.is_empty() or int(back[0].get("edge_count", 0)) != int(baseline.get("edge_count", 0)) \
			or float(back[0].get("street_len_m", 0.0)) != float(baseline.get("street_len_m", 0.0)):
		print("FAIL: reset did not reproduce the baseline")
		fails += 1

	# An unknown id is refused and leaves the rules alone.
	if not bridge.apply_urban_rules_preset("no_such_town").is_empty():
		print("FAIL: unknown preset id returned rules")
		fails += 1
	if bridge.get_active_urban_rules() != bridge.get_default_urban_rules():
		print("FAIL: unknown preset id changed the active rules")
		fails += 1
	_finish(fails)


func _find_button(root: Node, text: String) -> BaseButton:
	for n in root.get_children():
		if n is Button and (n as Button).text == text:
			return n
		var f := _find_button(n, text)
		if f != null:
			return f
	return null


func _finish(fails: int) -> void:
	if fails == 0:
		print("PROBE PASS")
		get_tree().quit(0)
	else:
		print("PROBE FAIL: %d assertion(s)" % fails)
		get_tree().quit(1)
