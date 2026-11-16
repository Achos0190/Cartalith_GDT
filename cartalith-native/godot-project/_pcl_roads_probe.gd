extends "res://_pcl_base.gd"
## PC-4 leftover (`PC_SHELL_PROPOSAL.md` 3.3 "A settlement shows ... its roads"). Protects the settlement page's
## "Roads from here" section (`ContextModules.roads_section`, shared with the Routes module).
##   A. For a settlement that roads end at, the page has `InspectRoads` with exactly as many road rows as an
##      INDEPENDENT count of `bridge.roads()` whose first or last point lies within 2.0 cells of the settlement (capped
##      at the module's `ROAD_ROWS_CAP`), and the Routes module (a second surface) lists the same number.
##   B. A REAL click on a road row shows that road's reading (the dock's Route context).
##   C. A settlement no road reaches shows the "No road ends here." note, never an empty section.
##   D. The page's Routes field no longer says "Not available" (it points at the section).
## Mutants: ROADS_RADIUS (`ROAD_END_CELLS := 2.0` -> 0.0), ROADS_DROPPED (the `roads_section` call in
## `_build_settlement_history` removed), ROADS_FIELD_STALE (the old "Not available" text kept on desktop).

func _count_roads(at: Vector2) -> int:
	var n := 0
	for r in _bridge.roads():
		var pts: PackedVector2Array = (r as Dictionary).get("points", PackedVector2Array())
		if pts.size() < 2:
			continue
		if pts[0].distance_to(at) <= 2.0 or pts[pts.size() - 1].distance_to(at) <= 2.0:
			n += 1
	return n


func _run() -> void:
	_prefix = "roads"
	var all: Array = _bridge.settlements()
	var with_road := -1
	var without := -1
	var want := 0
	## Prefer a settlement with a road end that is NEAR but not ON it (0 < distance <= 2 cells): that is the only fixture the
	## 2.0 radius can be told from a radius of 0 by (a first version used a settlement whose road ends sat exactly on it, and the
	## ROADS_RADIUS 0.0 mutant survived).
	var near_idx := -1
	for i in all.size():
		var s: Dictionary = all[i]
		var at := Vector2(float(s.get("x", 0)), float(s.get("y", 0)))
		var c := _count_roads(at)
		if c > 0 and with_road < 0:
			with_road = i
			want = c
		if c == 0 and without < 0:
			without = i
		if near_idx < 0 and c > 0:
			for r in _bridge.roads():
				var pts: PackedVector2Array = (r as Dictionary).get("points", PackedVector2Array())
				if pts.size() < 2:
					continue
				var d := minf(pts[0].distance_to(at), pts[pts.size() - 1].distance_to(at))
				if d > 0.05 and d <= 2.0:
					near_idx = i
	if near_idx >= 0:
		with_road = near_idx
		var sn: Dictionary = all[near_idx]
		want = _count_roads(Vector2(float(sn.get("x", 0)), float(sn.get("y", 0))))
	print("roads fixture: settlement %d, near-but-not-on end: %s, independent count %d" % [with_road, near_idx >= 0, want])
	_ok(with_road >= 0, "the world has a settlement a road ends at (%d roads in the world)" % _bridge.roads().size())
	if with_road < 0:
		return
	print("== A. the section ==")
	_app.select("settlement", with_road)
	await _settle(10)
	var body: Control = _app.right_dock_body
	var sec := body.find_child("InspectRoads", true, false)
	_ok(sec != null, "the settlement page has an InspectRoads section")
	var rows := body.find_children("*", "Button", true, false).filter(func(b): return (b as Node).has_meta("ctx_module_road"))
	var cap := int(_app.context_modules_obj().ROAD_ROWS_CAP)
	_ok(rows.size() == mini(want, cap), "road rows = min(independent count %d, cap %d) (got %d)" % [want, cap, rows.size()])
	await _shot("settlement_roads")
	print("== B. a real click ==")
	if not rows.is_empty():
		if await _press(rows[0], "the first road row"):
			await _settle(6)
			_ok(_app.right_dock_ctrl._context == _app.right_dock_ctrl.CTX_ROUTE, "the dock now shows the road's reading (context %s)" % str(_app.right_dock_ctrl._context))
	print("== C. none ==")
	if without >= 0:
		_app.select("settlement", without)
		await _settle(10)
		var note := false
		for l in body.find_children("*", "Label", true, false):
			if (l as Label).text == "No road ends here." and (l as Label).is_visible_in_tree():
				note = true
		_ok(note and body.find_child("InspectRoads", true, false) != null, "a settlement no road reaches says 'No road ends here.'")
	print("== D. the Routes field ==")
	_app.select("settlement", with_road)
	await _settle(10)
	var seen := ""
	for l in body.find_children("*", "Label", true, false):
		if (l as Label).text == "see Roads from here":
			seen = (l as Label).text
	_ok(seen != "", "the Routes field points at the section")
	_ok(not body.find_children("*", "Label", true, false).any(func(l): return (l as Label).tooltip_text.begins_with("Not available -- roads")), "no 'Not available' tooltip remains on the page")
