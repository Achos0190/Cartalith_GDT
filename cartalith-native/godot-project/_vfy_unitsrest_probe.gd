extends Node
## VERIFIER probe for the UNITS lane -- the rows the lane's own
## `_unitsflip_probe.gd` did NOT exercise live and reported as
## "symbol + parse-check only": Measure -- area (6 fields), River Catchment,
## Ecoregion area, Route Length (both arms: the engine `km` and the
## `_route_length_text()` fallback).
##
## Same method as the lane's probe: walk the real dock body for drawn Label
## text, compare against `DccUnits` re-evaluated live in whichever mode is
## set -- never a hand-typed conversion. Negative controls carried for the
## rows that must NOT move: Mean elevation (m), Centroid (grid cells),
## Source elevation / Fall (m), Discharge (a rate), Productivity, Ruggedness.

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("VUR %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
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

func _after(texts: Array, label: String) -> String:
	for i in texts.size() - 1:
		if String(texts[i]) == label:
			return String(texts[i + 1])
	return "<missing:%s>" % label

func _containing(texts: Array, needle: String) -> String:
	for t in texts:
		if String(t).find(needle) >= 0:
			return String(t)
	return "<no line containing:%s>" % needle

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
	print("VUR world generated: has_world=%s (%d frames)" % [app.bridge.has_world, waited])
	await _frames(8)
	if not app.bridge.has_world:
		print("VUR  !! generate failed -- nothing else here can run")
		get_tree().quit(2)
		return

	var rd = app.right_dock_ctrl
	var original_mode := DccSettings.units_mode()
	print("VUR original units_mode=%s (restored at the end)" % original_mode)

	# == A: Measure -- area ====================================================
	var ar := {
		"projected_km2": 12345.0, "true_surface_km2": 12500.0, "vertices": 7,
		"perimeter_km": 480.0, "water_km2": 2000.0, "land_km2": 10345.0,
		"centroid_x": 42.0, "centroid_y": 31.0,
		"bbox_w_km": 150.0, "bbox_h_km": 90.0, "mean_elev_m": 640.0,
		"water_from_civ": true, "stride": 1, "sampled_cells": 900,
	}
	var seen_area := {}
	for mode in ["km", "mi"]:
		DccSettings.set_units_mode(mode)
		rd._measure_result = ar
		rd._measure_mode = "area"
		rd._context = "measure"
		rd._rebuild()
		await _frames(2)
		var t := _texts()
		seen_area[mode] = t
		_check("A projected[%s]" % mode, _after(t, "Area · projected") == DccUnits.format_area(12345.0),
			"drawn=%s want=%s" % [_after(t, "Area · projected"), DccUnits.format_area(12345.0)])
		_check("A true-surface note[%s]" % mode,
			_containing(t, "true surface").begins_with("true surface %s ·" % DccUnits.format_area(12500.0)),
			_containing(t, "true surface"))
		_check("A perimeter[%s]" % mode, _after(t, "Perimeter") == DccUnits.format(480.0),
			"drawn=%s want=%s" % [_after(t, "Perimeter"), DccUnits.format(480.0)])
		_check("A water subtracted[%s]" % mode,
			_after(t, "Water subtracted") == "−%s" % DccUnits.format_area(2000.0),
			"drawn=%s want=-%s" % [_after(t, "Water subtracted"), DccUnits.format_area(2000.0)])
		_check("A land area[%s]" % mode, _after(t, "Land area") == DccUnits.format_area(10345.0),
			"drawn=%s want=%s" % [_after(t, "Land area"), DccUnits.format_area(10345.0)])
		_check("A bounding box[%s]" % mode,
			_after(t, "Bounding box") == "%s × %s" % [DccUnits.format(150.0), DccUnits.format(90.0)],
			"drawn=%s" % _after(t, "Bounding box"))
	_check("A NEG mean elevation unchanged by the flip",
		_after(seen_area["km"], "Mean elevation") == _after(seen_area["mi"], "Mean elevation"),
		"km=%s mi=%s" % [_after(seen_area["km"], "Mean elevation"), _after(seen_area["mi"], "Mean elevation")])
	_check("A NEG centroid (grid cells) unchanged by the flip",
		_after(seen_area["km"], "Centroid") == _after(seen_area["mi"], "Centroid"),
		"km=%s mi=%s" % [_after(seen_area["km"], "Centroid"), _after(seen_area["mi"], "Centroid")])
	_check("A projected DID move with the flip",
		_after(seen_area["km"], "Area · projected") != _after(seen_area["mi"], "Area · projected"),
		"km=%s mi=%s" % [_after(seen_area["km"], "Area · projected"), _after(seen_area["mi"], "Area · projected")])

	# == B: River Catchment ====================================================
	var riv := {
		"km": 88.0, "order": 4, "source_m": 1200.0, "drop_m": 950.0,
		"discharge": 4200.0, "mouth_discharge": 4000.0, "catchment_km2": 3500.0,
		"points": PackedVector2Array(),
	}
	var seen_riv := {}
	for mode in ["km", "mi"]:
		DccSettings.set_units_mode(mode)
		rd._river = riv
		rd._context = "river"
		rd._rebuild()
		await _frames(2)
		var t := _texts()
		seen_riv[mode] = t
		_check("B catchment[%s]" % mode, _after(t, "Catchment") == DccUnits.format_area(3500.0),
			"drawn=%s want=%s" % [_after(t, "Catchment"), DccUnits.format_area(3500.0)])
		_check("B river length[%s]" % mode, _after(t, "Length") == DccUnits.format_adaptive(88.0),
			"drawn=%s want=%s" % [_after(t, "Length"), DccUnits.format_adaptive(88.0)])
	_check("B NEG source elevation unchanged by the flip",
		_after(seen_riv["km"], "Source elevation") == _after(seen_riv["mi"], "Source elevation"),
		"km=%s mi=%s" % [_after(seen_riv["km"], "Source elevation"), _after(seen_riv["mi"], "Source elevation")])
	_check("B NEG fall unchanged by the flip",
		_after(seen_riv["km"], "Fall") == _after(seen_riv["mi"], "Fall"))
	_check("B NEG discharge (a rate, not a length) unchanged by the flip",
		_after(seen_riv["km"], "Discharge") == _after(seen_riv["mi"], "Discharge"),
		"km=%s mi=%s" % [_after(seen_riv["km"], "Discharge"), _after(seen_riv["mi"], "Discharge")])
	_check("B catchment DID move with the flip",
		_after(seen_riv["km"], "Catchment") != _after(seen_riv["mi"], "Catchment"),
		"km=%s mi=%s" % [_after(seen_riv["km"], "Catchment"), _after(seen_riv["mi"], "Catchment")])
	rd._river = {}

	# == C: Ecoregion area =====================================================
	var eco := {
		"id": 3, "cx": 40, "cy": 25, "biome_name": "Boreal Forest",
		"area_km2": 8800.0, "cells": 512, "richness": 41, "npp": 620.0,
		"tri": 0.123, "water": 0.44, "lat_abs": 58.0, "coastal": false,
		"rugged": true, "summary": "", "guilds": [],
	}
	var seen_eco := {}
	for mode in ["km", "mi"]:
		DccSettings.set_units_mode(mode)
		rd._wildlife_region = eco
		rd._context = "wildlife"
		rd._rebuild()
		await _frames(2)
		var t := _texts()
		seen_eco[mode] = t
		var want := "Boreal Forest · %s · 512 cells · neighbours —" % DccUnits.format_area(8800.0)
		_check("C ecoregion area line[%s]" % mode, _containing(t, "neighbours") == want,
			"drawn=%s want=%s" % [_containing(t, "neighbours"), want])
		_check("C ecoregion coord line ends in the live suffix[%s]" % mode,
			_containing(t, "· cell ").find(" %s · cell " % DccUnits.suffix()) >= 0,
			_containing(t, "· cell "))
	_check("C NEG productivity (g/m2/yr) unchanged by the flip",
		_after(seen_eco["km"], "Productivity") == _after(seen_eco["mi"], "Productivity"),
		"km=%s mi=%s" % [_after(seen_eco["km"], "Productivity"), _after(seen_eco["mi"], "Productivity")])
	_check("C NEG ruggedness (dimensionless) unchanged by the flip",
		_after(seen_eco["km"], "Ruggedness") == _after(seen_eco["mi"], "Ruggedness"))
	rd._wildlife_region = {}

	# == D: Route Length, BOTH arms ============================================
	var seen_rt := {}
	for mode in ["km", "mi"]:
		DccSettings.set_units_mode(mode)
		rd._route_kind = "road"
		rd._route_entry = {"name": "VUR way", "way_type": "trail", "km": 137.4,
			"points": PackedVector2Array([Vector2(0, 0), Vector2(10, 0)]), "manual": true}
		rd._context = "route"
		rd._rebuild()
		await _frames(2)
		var t := _texts()
		seen_rt[mode] = t
		_check("D route Length, engine-km arm[%s]" % mode,
			_after(t, "Length") == DccUnits.format(137.4, 1),
			"drawn=%s want=%s" % [_after(t, "Length"), DccUnits.format(137.4, 1)])

		## km == 0 forces `_route_length_text()`, the shared 3-call-site helper.
		var pts := PackedVector2Array([Vector2(0.0, 0.0), Vector2(60.0, 0.0)])
		rd._route_entry = {"name": "VUR fallback", "way_type": "trail", "km": 0.0,
			"points": pts, "manual": true}
		rd._rebuild()
		await _frames(2)
		var t2 := _texts()
		var gw: float = float(app.bridge.grid_size().x)
		var want_km: float = 60.0 * app.bridge.last_width_km / gw
		_check("D route Length, _route_length_text() fallback arm[%s]" % mode,
			_after(t2, "Length") == DccUnits.format(want_km),
			"drawn=%s want=%s (gw=%d width_km=%.2f)" % [
				_after(t2, "Length"), DccUnits.format(want_km), int(gw), app.bridge.last_width_km])
	_check("D route Length DID move with the flip",
		_after(seen_rt["km"], "Length") != _after(seen_rt["mi"], "Length"),
		"km=%s mi=%s" % [_after(seen_rt["km"], "Length"), _after(seen_rt["mi"], "Length")])
	rd._route_entry = {}

	DccSettings.set_units_mode(original_mode)
	print("VUR units_mode restored to %s" % DccSettings.units_mode())
	print("VUR TOTAL FAILURES: %d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
