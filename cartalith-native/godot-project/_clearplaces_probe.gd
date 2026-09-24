extends SceneTree
## OUTSTANDING_WORK.md §2.11 "Clear places leaves a stale territory base, and
## Go to year bypasses the paint model", through the real `WorldGen` #[func]s:
##   1. after Clear places, a paint commit and a GeoJSON import draw only what
##      they wrote -- no computed or painted claim from before the clear;
##   2. a faction id past 255 is refused by the brush and the lasso, and
##      nothing is painted.
## Counts are `civ_faction_territory_stats(f).claimed_cells`, summed.
##
## HEADLESS:
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . --script _clearplaces_probe.gd

var _fail := 0

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("CLEARPL %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _claimed(wg, f: int) -> int:
	return int(wg.civ_faction_territory_stats(f).get("claimed_cells", -1))

func _total(wg, ids: Array) -> int:
	var t := 0
	for f in ids:
		t += _claimed(wg, int(f))
	return t

func _ring(a: float, b: float) -> PackedVector2Array:
	return PackedVector2Array([Vector2(a, a), Vector2(b, a), Vector2(b, b), Vector2(a, b)])

func _square_feature(wg, x0: float, y0: float, x1: float, y1: float, faction_name: String) -> String:
	var cell_km: float = 800.0 / float(wg.get_width())
	var gh := float(wg.get_height())
	var pts := [[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]]
	var coords := PackedStringArray()
	for p in pts:
		coords.append("[%f,%f]" % [p[0] * cell_km, (gh - p[1]) * cell_km])
	return '{"type":"FeatureCollection","features":[{"type":"Feature","geometry":{"type":"Polygon","coordinates":[[%s]]},"properties":{"layer":"territory","factionName":"%s"}}]}' % [",".join(coords), faction_name]

func _init() -> void:
	var wg: WorldGen = WorldGen.new()
	wg.generate(12345, 800.0, 128)
	var factions: Array = wg.get_factions()
	_check("world has factions", factions.size() >= 2, "%d" % factions.size())
	if factions.size() < 2:
		quit(1)
		return
	var ids: Array = []
	for d in factions:
		ids.append(int(d["id"]))
	var f := int(factions[0]["id"])
	var g := int(factions[factions.size() - 1]["id"])
	var gname := String(factions[factions.size() - 1]["name"])

	# --- case 3: refusal ---------------------------------------------------
	var before := _total(wg, ids)
	_check("premise: computed borders claim cells", before > 0, "%d" % before)
	var dab: bool = wg.civ_territory_paint_at(40.0, 40.0, 256, 6.0, false)
	_check("brush refuses faction 256", dab == false, str(dab))
	var lasso: int = wg.civ_territory_paint_polygon(_ring(30.0, 60.0), 256, false)
	_check("lasso refuses faction 256 with -1", lasso == -1, "%d" % lasso)
	wg.civ_territory_commit()
	_check("a refused stroke paints nothing", _total(wg, ids) == before, "%d -> %d" % [before, _total(wg, ids)])
	_check("brush accepts a real faction", wg.civ_territory_paint_at(40.0, 40.0, f, 0.0, false) == true)
	wg.civ_territory_discard()

	# --- case 1: paint, clear places, paint again ---------------------------
	var staged: int = wg.civ_territory_paint_polygon(_ring(10.0, 50.0), f, false)
	wg.civ_territory_commit()
	_check("premise: paint staged and committed", staged > 0 and _claimed(wg, f) > 0, "staged %d, f claims %d" % [staged, _claimed(wg, f)])
	var r: Dictionary = wg.civ_clear_places()
	_check("clear places removed settlements", int(r.get("settlements", 0)) > 0, str(r))
	_check("clear places leaves no claim", _total(wg, ids) == 0, "%d" % _total(wg, ids))

	var n2: int = wg.civ_territory_paint_polygon(_ring(70.0, 80.0), g, false)
	wg.civ_territory_commit()
	var after := _total(wg, ids)
	_check("the next commit draws only the new stroke", after == n2 and _claimed(wg, g) == n2,
		"total %d, g %d, staged %d (old computed %d)" % [after, _claimed(wg, g), n2, before])
	if f != g:
		_check("the old painted faction does not come back", _claimed(wg, f) == 0, "%d" % _claimed(wg, f))

	var ri: Dictionary = wg.apply_geojson_document(_square_feature(wg, 20.0, 20.0, 30.0, 30.0, gname))
	_check("import applied", int(ri.get("territory_features_applied", 0)) == 1, str(ri))
	var imp := _total(wg, ids)
	_check("an import after the clear adds only its polygon", imp > after and imp - after <= 11 * 11 and _claimed(wg, f) == (0 if f != g else _claimed(wg, f)),
		"total %d -> %d" % [after, imp])

	print("CLEARPL %s (%d failed)" % ["PASS" if _fail == 0 else "FAIL", _fail])
	quit(0 if _fail == 0 else 1)
