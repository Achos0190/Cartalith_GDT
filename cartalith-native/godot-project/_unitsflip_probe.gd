extends Node
## Verification harness for the UNITS lane row -- "the right dock still prints
## raw kilometres." Boots the real app, generates a world, then drives
## `right_dock.gd`'s own state directly (the same privacy-agnostic pattern
## `_rdappend_probe.gd` already uses on `_context`/`_current_title()`) to put
## Measure -- radius and Measure -- section on screen with known numbers,
## flips `DccSettings.units_mode()` km -> mi, and asserts every fixed row's
## drawn text equals what `DccUnits` itself prints for that same input in
## whichever mode is live -- computed by calling `DccUnits`, never hand-typed,
## so nothing here pins a conversion constant against itself.
##
## The Sample panel's cursor "Position" row is checked through its own real
## public path (`on_cursor_sampled`), not synthetic state, because it is the
## row this lane is named for.
##
## Two exceptions get an explicit NEGATIVE control -- elevation (vertical,
## stays metres) and the profile's "min · max" pair -- so a flip that
## over-converted them would fail exactly as loudly as one that under-converts
## a real distance.
##
## `DccSettings` persists to `user://cartalith_settings.cfg`: the original
## mode is read first and restored before quitting, the same courtesy a
## theme-flip probe owes the file it pokes.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _unitsflip_probe.tscn

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	var tag := "ok  " if cond else "FAIL"
	print("UFP %s  %s%s" % [tag, name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _collect(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append((c as Label).text)
		_collect(c, out)

func _texts() -> Array:
	var out: Array = []
	_collect(app.right_dock_body, out)
	return out

## The value Label immediately after the row whose own Label reads exactly
## `label` -- `_field()`/`_accent_readout()` both add caption then value as
## consecutive children, so this is exact-match position, not substring.
func _after(texts: Array, label: String) -> String:
	for i in texts.size() - 1:
		if String(texts[i]) == label:
			return String(texts[i + 1])
	return "<missing:%s>" % label

## The Label immediately BEFORE one whose text reads exactly `value` -- for
## the one row here that prints a distance as its LABEL (a crossing's km
## position) rather than as its value.
func _before(texts: Array, value: String) -> String:
	for i in range(1, texts.size()):
		if String(texts[i]) == value:
			return String(texts[i - 1])
	return "<missing before:%s>" % value

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 1800:
		await get_tree().process_frame
		waited += 1
	print("UFP world generated: has_world=%s (%d frames)" % [app.bridge.has_world, waited])
	await _frames(8)
	if not app.bridge.has_world:
		print("UFP  !! generate failed -- nothing else here can run")
		get_tree().quit(1)
		return

	var rd = app.right_dock_ctrl
	var original_mode := DccSettings.units_mode()
	print("UFP original units_mode=%s (restored at the end)" % original_mode)

	# == A: Sample panel "Position" -- the row this lane is named for =========
	DccSettings.set_units_mode("km")
	rd._context = "sample"
	rd._rebuild()
	await _frames(2)
	rd.on_cursor_sampled(40.0, 25.0, true)
	await _frames(2)
	var pos_km := String(rd._sample_pos.text)
	var elev_km := String(rd._sample_elev.text)
	_check("A1 Position is km-suffixed in km mode", pos_km.ends_with(" km"), pos_km)

	DccSettings.set_units_mode("mi")
	rd.on_cursor_sampled(40.0, 25.0, true)
	await _frames(2)
	var pos_mi := String(rd._sample_pos.text)
	var elev_mi := String(rd._sample_elev.text)
	_check("A2 Position is mi-suffixed in mi mode", pos_mi.ends_with(" mi"), pos_mi)
	_check("A3 Position's own drawn string changed with the flip",
		pos_mi != pos_km, "km=%s mi=%s" % [pos_km, pos_mi])

	## **A3b/A3c: the NUMBER, not just the suffix.** A1/A2/A3 together were
	## shown to pass a mutation that kept the kilometre value and appended a
	## mile suffix -- " 15.6 · 9.8 mi", a km number wearing a mi label. Suffix
	## plus inequality is satisfied by relabelling alone, so the three checks
	## above could not fail on the exact defect this probe exists for.
	##
	## Reconstructed from `DccUnits.to_unit()` on the raw km, never from a
	## typed constant, so the assertion moves with the converter rather than
	## pinning a number that a rounding change would break.
	var _cell_km: float = rd._cell_km()  if rd.has_method("_cell_km") else \
		(float(rd.bridge.map_width_km()) / maxf(1.0, float(rd.bridge.grid_width())))
	var _want_x := DccUnits.to_unit(40.0 * _cell_km)
	var _want_y := DccUnits.to_unit(25.0 * _cell_km)
	_check("A3b Position's mi NUMBERS match DccUnits.to_unit, not just its suffix",
		_num_near(pos_mi, _want_x) and _num_near(pos_mi, _want_y),
		"drawn=%s want=%.2f,%.2f" % [pos_mi, _want_x, _want_y])
	DccSettings.set_units_mode("km")
	rd.on_cursor_sampled(40.0, 25.0, true)
	await _frames(2)
	var _pos_km2 := String(rd._sample_pos.text)
	var _wx_km := DccUnits.to_unit(40.0 * _cell_km)
	_check("A3c and the km NUMBERS match too, so neither mode is relabelled",
		_num_near(_pos_km2, _wx_km), "drawn=%s want=%.2f" % [_pos_km2, _wx_km])
	DccSettings.set_units_mode("mi")
	rd.on_cursor_sampled(40.0, 25.0, true)
	await _frames(2)
	_check("A4 Elevation (the written vertical exception) is untouched by the flip",
		elev_mi == elev_km, "km=%s mi=%s" % [elev_km, elev_mi])

	# == B: Measure -- radius, synthetic and exact =============================
	var mr := {
		"radius_km": 100.0, "diameter_km": 200.0, "circumference_km": 314.159,
		"area_km2": 7854.0,
	}
	for mode in ["km", "mi"]:
		DccSettings.set_units_mode(mode)
		rd._measure_result = mr
		rd._measure_mode = "radius"
		rd._context = "measure"
		rd._rebuild()
		await _frames(2)
		var t := _texts()
		_check("B radius[%s]" % mode, _after(t, "Radius") == DccUnits.format(100.0),
			"drawn=%s want=%s" % [_after(t, "Radius"), DccUnits.format(100.0)])
		_check("B diameter[%s]" % mode, _after(t, "Diameter") == DccUnits.format(200.0),
			"drawn=%s want=%s" % [_after(t, "Diameter"), DccUnits.format(200.0)])
		_check("B circumference[%s]" % mode,
			_after(t, "Circumference") == DccUnits.format(314.159),
			"drawn=%s want=%s" % [_after(t, "Circumference"), DccUnits.format(314.159)])
		_check("B enclosed area[%s]" % mode,
			_after(t, "Enclosed area") == DccUnits.format_area(7854.0),
			"drawn=%s want=%s" % [_after(t, "Enclosed area"), DccUnits.format_area(7854.0)])

	# == C: Measure -- section, synthetic, plus the vertical negative control ==
	var ms := {
		"length_km": 50.0, "length_3d_km": 51.5, "bearing_deg": 45.0, "spacing_m": 425.0,
		"samples": [1, 2, 3],
		"stats": {
			"min_m": 10.0, "max_m": 800.0, "mean_m": 300.0, "ascent_m": 500.0,
			"descent_m": -200.0, "net_m": 300.0, "mean_slope_deg": 5.0,
			"max_slope_deg": 20.0, "above_2000m_km": 12.5,
			"river_crossings": 1, "ridge_crossings": 0, "shore_crossings": 0,
		},
		"crossings": [{"km": 25.0, "label": "UFP crossing", "elev_m": 300.0}],
	}
	var section_texts := {}
	for mode in ["km", "mi"]:
		DccSettings.set_units_mode(mode)
		rd._measure_result = ms
		rd._measure_mode = "section"
		rd._context = "measure"
		rd._rebuild()
		await _frames(2)
		var t := _texts()
		section_texts[mode] = t
		_check("C length[%s]" % mode, _after(t, "Length") == DccUnits.format(50.0),
			"drawn=%s want=%s" % [_after(t, "Length"), DccUnits.format(50.0)])
		_check("C 3D length[%s]" % mode, _after(t, "3D length") == DccUnits.format(51.5),
			"drawn=%s want=%s" % [_after(t, "3D length"), DccUnits.format(51.5)])
		var want_spacing := "%d · %s" % [3, DccUnits.format(425.0 / 1000.0, 1)]
		_check("C samples-spacing[%s]" % mode,
			_after(t, "Samples · spacing") == want_spacing,
			"drawn=%s want=%s" % [_after(t, "Samples · spacing"), want_spacing])
		_check("C above-2000m[%s]" % mode,
			_after(t, "above 2 000 m") == DccUnits.format(12.5),
			"drawn=%s want=%s" % [_after(t, "above 2 000 m"), DccUnits.format(12.5)])
		var want_crossing_label := DccUnits.format(25.0)
		_check("C crossing label[%s]" % mode,
			_before(t, "UFP crossing") == want_crossing_label,
			"drawn=%s want=%s" % [_before(t, "UFP crossing"), want_crossing_label])

	_check("C length changed with the flip",
		_after(section_texts["km"], "Length") != _after(section_texts["mi"], "Length"),
		"km=%s mi=%s" % [_after(section_texts["km"], "Length"), _after(section_texts["mi"], "Length")])
	_check("C negative control: 'min · max' (metres, no unit toggle) is UNCHANGED by the flip",
		_after(section_texts["km"], "min · max") == _after(section_texts["mi"], "min · max"),
		"km=%s mi=%s" % [_after(section_texts["km"], "min · max"), _after(section_texts["mi"], "min · max")])

	# -- Restore the persisted preference before anything else touches it ------
	DccSettings.set_units_mode(original_mode)
	print("UFP units_mode restored to %s" % DccSettings.units_mode())

	print("UFP TOTAL FAILURES: %d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

## Is `want` present as a number in `s`, to the precision the row draws?
##
## Compares parsed numbers rather than formatted strings, because the row
## pads to a column width and a string compare would fail on whitespace while
## passing on a wrong unit -- the opposite of what is wanted here.
func _num_near(s: String, want: float) -> bool:
	for tok in s.replace("·", " ").split(" ", false):
		var t := tok.strip_edges().replace(",", "")
		if t.is_valid_float() and absf(t.to_float() - want) <= 0.06:
			return true
	return false
