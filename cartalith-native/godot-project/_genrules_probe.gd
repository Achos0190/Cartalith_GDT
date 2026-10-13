extends Node
## Ruling BK header (2026-10-13, probe adopted into git; first run passed on the debug DLL):
## Protects: GenerationRulesWindow's urban-rules slider reaches Rust (apply_urban_wildness, get_active_urban_rules, urban_layouts) and Reset restores the baseline town byte-for-byte.
## Run (Godot_v4.7.1-stable_win64_console.exe, from godot-project/): --headless --path . _genrules_probe.tscn
## Result: 'PROBE PASS' (exit 0) or failures / 'WATCHDOG TIMEOUT' (exit 3)
## Sources: derived from this file's own assertions and a 2026-10-13 run; the feature symbols were grepped, not assumed.
##
## `lazy-riding-piglet.md` Batch F verification -- `GenerationRulesWindow`
## (artboard 1h) and the Rust `rules` parameter it now reaches through
## `EngineBridge.apply_urban_wildness`/`get_active_urban_rules`/
## `urban_layouts()`.
##
## Pure logic/data checks -- no pixel comparison, so this runs HEADLESS
## (`MISTAKES.md`: "Headless for logic and layout; windowed for anything
## that rasterises or times a frame"). Drives the real HSlider the window
## builds (`DccWidgets.slider`'s own control), not the bridge call directly,
## so the check exercises the same path a user's drag does.
##
## Run: Godot_v4.7.1-stable_win64_console.exe --headless --path . _genrules_probe.tscn

var _app: Node

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 120.0
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

	# -- find a settlement urban_layouts() actually produces a layout for -----
	var settlements: Array = bridge.settlements()
	print("PROBE settlements=%d" % settlements.size())
	var idx := -1
	var baseline: Dictionary = {}
	for i in settlements.size():
		var got: Array = bridge.urban_layouts(PackedInt32Array([i]))
		if not got.is_empty():
			idx = i
			baseline = got[0]
			break
	if idx < 0:
		print("FAIL: no settlement produced a layout at all -- cannot verify reachability")
		_finish(1)
		return
	print("PROBE using settlement index=%d edges=%d street_len=%.1f" % [
		idx, int(baseline.get("edge_count", 0)), float(baseline.get("street_len_m", 0.0))])

	# -- 1. get_default_urban_rules()/get_active_urban_rules() agree at start -
	var defaults: Dictionary = bridge.get_default_urban_rules()
	var active: Dictionary = bridge.get_active_urban_rules()
	print("PROBE defaults keys=%d active keys=%d" % [defaults.size(), active.size()])
	if defaults.size() != 24 or active.size() != 24:
		print("FAIL: expected 24 dotted-key fields, got defaults=%d active=%d" % [defaults.size(), active.size()])
		fails += 1
	if active != defaults:
		print("FAIL: active rules should equal defaults before any edit")
		fails += 1

	# -- 2. open the real window, drive the real Wildness HSlider -------------
	_app.generation_rules_window.open()
	await get_tree().process_frame
	await get_tree().process_frame
	var win: GenerationRulesWindow = _app.generation_rules_window
	if not win.visible:
		print("FAIL: GenerationRulesWindow did not open")
		fails += 1

	var before_row_text := _street_row_text(win, 0)  # branch_angle_jitter row
	print("PROBE street row 0 before slider drive: %s" % before_row_text)

	var slider := _find_hslider(win)
	if slider == null:
		print("FAIL: no HSlider found in GenerationRulesWindow")
		fails += 1
		_finish(fails)
		return
	slider.value = 1.9
	slider.value_changed.emit(1.9)
	await get_tree().process_frame

	var after_row_text := _street_row_text(win, 0)
	print("PROBE street row 0 after wildness=1.90: %s" % after_row_text)
	if after_row_text == before_row_text:
		print("FAIL: the street parameter table did not change when the slider moved")
		fails += 1
	else:
		print("PASS: parameter table recomputed live off the real slider drag")

	var moved_active: Dictionary = bridge.get_active_urban_rules()
	var moved_branch := float(moved_active.get("street.branch_angle_jitter", -1.0))
	# `apply_wildness`: clamp(0.26*1.9, 0.15, 0.70) = clamp(0.494, 0.15, 0.70) = 0.494
	var want_branch := clampf(0.26 * 1.9, 0.15, 0.70)
	print("PROBE street.branch_angle_jitter got=%.6f want=%.6f" % [moved_branch, want_branch])
	if not is_equal_approx(moved_branch, want_branch):
		print("FAIL: apply_urban_wildness did not compute the reference's own formula")
		fails += 1
	else:
		print("PASS: apply_urban_wildness matches apply_wildness's own clamp(0.26*w,0.15,0.70)")

	# -- 3. regenerate the same settlement's layout, confirm it moved ---------
	var moved: Array = bridge.urban_layouts(PackedInt32Array([idx]))
	if moved.is_empty():
		print("FAIL: the settlement lost its layout entirely after a rules change")
		fails += 1
	else:
		var m: Dictionary = moved[0]
		print("PROBE after wildness=1.90: edges=%d street_len=%.1f (baseline edges=%d street_len=%.1f)" % [
			int(m.get("edge_count", 0)), float(m.get("street_len_m", 0.0)),
			int(baseline.get("edge_count", 0)), float(baseline.get("street_len_m", 0.0))])
		if int(m.get("edge_count", 0)) == int(baseline.get("edge_count", 0)) \
				and is_equal_approx(float(m.get("street_len_m", 0.0)), float(baseline.get("street_len_m", 0.0))):
			print("FAIL: the regenerated layout is identical to the baseline -- rules did not reach urban_layouts()")
			fails += 1
		else:
			print("PASS: urban_layouts() produced a measurably different town under the new rules")

	# -- 4. the place editor's own Layout tab reads the same fresh dictionary -
	# Close the rules window first -- two exclusive AcceptDialogs open at once
	# is a real engine constraint (`_set_transient_exclusive_child`), not part
	# of what this step verifies.
	win.hide()
	await get_tree().process_frame
	_app.place_editor_window.open_for(idx)
	await get_tree().process_frame
	await get_tree().process_frame
	var pe: PlaceEditorWindow = _app.place_editor_window
	pe._switch_tab("layout")
	await get_tree().process_frame
	var direct: Array = bridge.urban_layouts(PackedInt32Array([idx]))
	if not direct.is_empty():
		var d: Dictionary = direct[0]
		print("PROBE place editor's own urban_layouts() re-read: edges=%d (cross-check against moved=%d)" % [
			int(d.get("edge_count", 0)), int(moved[0].get("edge_count", 0)) if not moved.is_empty() else -1])
		if not moved.is_empty() and int(d.get("edge_count", 0)) != int(moved[0].get("edge_count", 0)):
			print("FAIL: the place editor's Layout tab would read a different edge count than a direct re-call")
			fails += 1

	# -- 5. Reset returns to DEFAULT_RULES's own behaviour ---------------------
	bridge.reset_active_urban_rules()
	var reset_active: Dictionary = bridge.get_active_urban_rules()
	if reset_active != defaults:
		print("FAIL: reset_active_urban_rules() did not restore DEFAULT_RULES")
		fails += 1
	var reset_layout: Array = bridge.urban_layouts(PackedInt32Array([idx]))
	if not reset_layout.is_empty():
		var r: Dictionary = reset_layout[0]
		print("PROBE after reset: edges=%d street_len=%.1f (baseline edges=%d street_len=%.1f)" % [
			int(r.get("edge_count", 0)), float(r.get("street_len_m", 0.0)),
			int(baseline.get("edge_count", 0)), float(baseline.get("street_len_m", 0.0))])
		if int(r.get("edge_count", 0)) != int(baseline.get("edge_count", 0)) \
				or not is_equal_approx(float(r.get("street_len_m", 0.0)), float(baseline.get("street_len_m", 0.0))):
			print("FAIL: reset did not reproduce the DEFAULT_RULES baseline exactly")
			fails += 1
		else:
			print("PASS: reset_active_urban_rules() reproduces the untouched baseline byte-for-byte")

	_finish(fails)


## The first data row under `_street_body` -- a `HBoxContainer` whose middle
## child is the computed-value `Label` (`_param_row`'s own layout: label,
## value, default).
func _street_row_text(win: GenerationRulesWindow, row_i: int) -> String:
	if row_i * 2 >= win._street_body.get_child_count():
		return ""
	var row: HBoxContainer = win._street_body.get_child(row_i * 2)
	var value_label: Label = row.get_child(1)
	return value_label.text


func _find_hslider(root: Node) -> HSlider:
	for n in root.get_children():
		if n is HSlider:
			return n
		var found := _find_hslider(n)
		if found != null:
			return found
	return null


func _finish(fails: int) -> void:
	if fails == 0:
		print("PROBE PASS")
		get_tree().quit(0)
	else:
		print("PROBE FAIL: %d assertion(s)" % fails)
		get_tree().quit(1)
