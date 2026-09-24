extends SceneTree
## OUTSTANDING_WORK.md §2.11, "A reopened project still lacks road edges and a
## territory-paint editor": territory paint and GeoJSON border import on a
## reopened project, through the real `WorldGen` #[func]s.
##
## Each edit is made twice -- once on the generated world, once on the same
## world saved and reopened -- and every faction's `claimed_cells` must agree.
## The reopened editor's base is the restored claim grid, which for a world
## nobody painted is the generated one, so "base plus stroke" on the reopened
## side is exactly the generated side's answer. Before `project_open` built the
## editor the reopened dab staged nothing and the import refused.
##
## HEADLESS:
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . --script _reopentools_probe.gd

var _fail := 0

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("REOPENTOOLS %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

## Every faction's claimed-cell count, keyed by faction id.
func _claims(wg) -> Dictionary:
	var out := {}
	for f in wg.get_factions():
		out[int(f["id"])] = int(wg.civ_faction_territory_stats(int(f["id"])).get("claimed_cells", -1))
	return out

func _square_feature(wg, x0: float, y0: float, x1: float, y1: float, faction_name: String) -> String:
	var cell_km: float = 800.0 / float(wg.get_width())
	var gh := float(wg.get_height())
	var pts := [[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]]
	var coords := PackedStringArray()
	for p in pts:
		coords.append("[%f,%f]" % [p[0] * cell_km, (gh - p[1]) * cell_km])
	return '{"type":"FeatureCollection","features":[{"type":"Feature","geometry":{"type":"Polygon","coordinates":[[%s]]},"properties":{"layer":"territory","factionName":"%s"}}]}' % [",".join(coords), faction_name]

func _ring(a: float, b: float) -> PackedVector2Array:
	return PackedVector2Array([Vector2(a, a), Vector2(b, a), Vector2(b, b), Vector2(a, b)])

## A second session holding the saved project, as File > Open builds it.
func _reopen(path: String) -> WorldGen:
	var wg: WorldGen = WorldGen.new()
	var r: Dictionary = wg.project_open(path)
	_check("the project reopens complete", bool(r.get("ok", false)) and String(r.get("substrate", "")) == "complete", str(r.get("substrate", r.get("error", ""))))
	return wg

func _init() -> void:
	var gen: WorldGen = WorldGen.new()
	gen.generate(12345, 800.0, 128)
	var factions: Array = gen.get_factions()
	_check("world has factions", factions.size() >= 2, "%d" % factions.size())
	if factions.size() < 2:
		quit(1)
		return
	var f := int(factions[factions.size() - 1]["id"])
	var fname := String(factions[factions.size() - 1]["name"])
	var path := OS.get_user_data_dir().path_join("_reopentools_probe.zip")
	_check("the project was written", bool(gen.project_save(path).get("ok", false)), path)
	var base: Dictionary = _claims(gen)

	# --- territory paint: a lasso add, then a subtract over it ------------
	var re: WorldGen = _reopen(path)
	_check("the reopened claim grid is the generated one", _claims(re) == base, "%s vs %s" % [_claims(re), base])
	var staged_re: int = re.civ_territory_paint_polygon(_ring(30.0, 60.0), f, false)
	var staged_gen: int = gen.civ_territory_paint_polygon(_ring(30.0, 60.0), f, false)
	_check("a reopened project stages a lasso stroke", staged_re > 0 and staged_re == staged_gen, "%d vs %d cells" % [staged_re, staged_gen])
	re.civ_territory_commit()
	gen.civ_territory_commit()
	var painted: Dictionary = _claims(gen)
	_check("the stroke claimed cells for %s" % fname, painted[f] > base[f], "%d -> %d" % [base[f], painted[f]])
	_check("reopened commit = restored base plus the stroke (every faction as generated)", _claims(re) == painted,
		"%s vs %s" % [_claims(re), painted])
	re.civ_territory_paint_polygon(_ring(30.0, 60.0), f, true)
	re.civ_territory_commit()
	_check("a subtract over it restores the reopened base", _claims(re) == base, "%s vs %s" % [_claims(re), base])

	# --- GeoJSON border import --------------------------------------------
	var gen2: WorldGen = WorldGen.new()
	gen2.generate(12345, 800.0, 128)
	var re2: WorldGen = _reopen(path)
	var doc := _square_feature(gen2, 30.0, 30.0, 60.0, 60.0, fname)
	var r_re: Dictionary = re2.apply_geojson_document(doc)
	var r_gen: Dictionary = gen2.apply_geojson_document(doc)
	_check("a reopened project imports a territory polygon", bool(r_re.get("ok", false)) and int(r_re.get("territory_features_applied", 0)) == 1, str(r_re))
	_check("the import reports as on the generated world", JSON.stringify(r_re) == JSON.stringify(r_gen), "%s vs %s" % [r_re, r_gen])
	_check("the imported borders match the generated world's", _claims(re2) == _claims(gen2), "%s vs %s" % [_claims(re2), _claims(gen2)])
	_check("the import claimed cells", _claims(re2)[f] > base[f], "%d -> %d" % [base[f], _claims(re2)[f]])

	DirAccess.remove_absolute(path)
	print("REOPENTOOLS %s (%d failed)" % ["PASS" if _fail == 0 else "FAIL", _fail])
	quit(0 if _fail == 0 else 1)
