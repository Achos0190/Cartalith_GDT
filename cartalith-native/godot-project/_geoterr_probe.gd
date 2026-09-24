extends SceneTree
## OUTSTANDING_WORK.md §2.11, both rows, through the real `WorldGen` #[func]s:
##   1. imported GeoJSON territory survives Recompute civilisation, and a
##      subtract stroke over it restores the computed owner;
##   2. under Generate Roads a later subtract restores the computed owner,
##      not the paint the rebuild kept.
## Counts are `civ_faction_territory_stats(f).claimed_cells`.
##
## HEADLESS:
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . --script _geoterr_probe.gd

var _fail := 0

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("GEOTERR %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _claimed(wg, f: int) -> int:
	return int(wg.civ_faction_territory_stats(f).get("claimed_cells", -1))

func _square_feature(wg, x0: float, y0: float, x1: float, y1: float, faction_name: String) -> String:
	var cell_km: float = 800.0 / float(wg.get_width())
	var gh := float(wg.get_height())
	var pts := [[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]]
	var coords := PackedStringArray()
	for p in pts:
		coords.append("[%f,%f]" % [p[0] * cell_km, (gh - p[1]) * cell_km])
	var props := '"layer":"territory"'
	if faction_name != "":
		props += ',"factionName":"%s"' % faction_name
	return '{"type":"Feature","geometry":{"type":"Polygon","coordinates":[[%s]]},"properties":{%s}}' % [",".join(coords), props]

func _doc(features: String) -> String:
	return '{"type":"FeatureCollection","features":[%s]}' % features

func _ring(a: float, b: float) -> PackedVector2Array:
	return PackedVector2Array([Vector2(a, a), Vector2(b, a), Vector2(b, b), Vector2(a, b)])

func _init() -> void:
	var wg: WorldGen = WorldGen.new()
	wg.generate(12345, 800.0, 128)
	var factions: Array = wg.get_factions()
	_check("world has factions", factions.size() >= 2, "%d" % factions.size())
	if factions.size() < 2:
		quit(1)
		return
	var f := int(factions[factions.size() - 1]["id"])
	var fname := String(factions[factions.size() - 1]["name"])
	var f0 := _claimed(wg, f)

	# --- row 2: import, recompute, subtract -----------------------------
	var r: Dictionary = wg.apply_geojson_document(_doc(_square_feature(wg, 30.0, 30.0, 60.0, 60.0, fname)))
	_check("import applied one territory polygon", bool(r.get("ok", false)) and int(r.get("territory_features_applied", 0)) == 1, str(r))
	var f1 := _claimed(wg, f)
	_check("import claimed cells for %s" % fname, f1 > f0, "%d -> %d" % [f0, f1])
	var rc: Dictionary = wg.recompute_civilisation()
	_check("recompute ran", bool(rc.get("ok", false)), str(rc.get("reason", "")))
	var f2 := _claimed(wg, f)
	_check("imported borders survive a recompute", f2 == f1, "after import %d, after recompute %d (unpainted %d)" % [f1, f2, f0])
	wg.civ_territory_paint_polygon(_ring(26.0, 64.0), f, true)
	wg.civ_territory_commit()
	var f3 := _claimed(wg, f)
	_check("subtract over the import restores the computed owner", f3 == f0, "%d, computed %d" % [f3, f0])

	var r2: Dictionary = wg.apply_geojson_document(_doc(_square_feature(wg, 30.0, 30.0, 60.0, 60.0, "")))
	_check("a no-faction polygon is reported skipped", int(r2.get("territory_features_skipped", 0)) == 1
		and int(r2.get("territory_features_applied", -1)) == 0, str(r2))
	_check("a no-faction polygon changes nothing", _claimed(wg, f) == f0)

	# --- row 1: paint, Generate Roads, subtract --------------------------
	wg.civ_territory_paint_polygon(_ring(30.0, 60.0), f, false)
	wg.civ_territory_commit()
	var f4 := _claimed(wg, f)
	_check("paint claimed cells", f4 > f0, "%d -> %d" % [f0, f4])
	var rr: Dictionary = wg.civ_auto_routes()
	_check("generate roads ran", bool(rr.get("ok", false)), str(rr.get("reason", "")))
	_check("generate roads kept the paint", _claimed(wg, f) == f4)
	wg.civ_territory_paint_polygon(_ring(30.0, 60.0), f, true)
	wg.civ_territory_commit()
	var f5 := _claimed(wg, f)
	_check("subtract after Generate Roads restores the computed owner", f5 == f0, "%d, computed %d, painted %d" % [f5, f0, f4])

	print("GEOTERR %s (%d failed)" % ["PASS" if _fail == 0 else "FAIL", _fail])
	quit(0 if _fail == 0 else 1)
