extends Node
## Diagnostic: on a REAL generated world, what does `detectRiverCrossings`
## actually resolve to? Lane A measured 1 bridge on a 64x64 synthetic fixture
## with hand-built ways; the brief asked for "a real seed".

var _app: Node

func _ready() -> void:
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	for _i in 3:
		await get_tree().process_frame
	DccTheme.apply_theme(false)

	var b = _app.bridge
	b.generate({"seed": 40417, "width_km": 2000.0, "grid_w": 256,
		"grid_h": 192, "archetype": "", "villages": true, "sea_level": 0.45})
	var waited := 0
	while b.generating and waited < 6000:
		await get_tree().process_frame
		waited += 1
	for _j in 8:
		await get_tree().process_frame

	var sets: Array = b.settlements()
	print("settlements=%d" % sets.size())
	var all_idx := PackedInt32Array()
	for i in sets.size():
		all_idx.append(i)
	var diags: Array = b.settlement_diagnostics(all_idx)

	# The 12 biggest settlements plus every riverthrough one, capped.
	var by_pop := []
	for i in sets.size():
		by_pop.append({"i": i, "pop": int((sets[i] as Dictionary).get("pop", 0))})
	by_pop.sort_custom(func(a, c): return a["pop"] > c["pop"])
	var idx := PackedInt32Array()
	for e in by_pop:
		if idx.size() >= 12:
			break
		idx.append(int(e["i"]))
	var through := 0
	for j in diags.size():
		if idx.size() >= 20:
			break
		if String((diags[j] as Dictionary).get("site_kind", "?")) == "riverthrough" and not idx.has(j):
			idx.append(j)
			through += 1
	print("asking for %d layouts (%d of them riverthrough-by-diagnostics)" % [idx.size(), through])

	var layouts: Array = b.urban_layouts(idx)
	var tot_b := 0
	var tot_f := 0
	var with_bpt := 0
	var real_water := 0
	var river_pts := 0
	for d in layouts:
		var dd: Dictionary = d
		var nb: int = (dd.get("bridges", PackedVector2Array()) as PackedVector2Array).size()
		tot_b += nb
		if dd.has("ford"):
			tot_f += 1
		if dd.has("bridge_pt"):
			with_bpt += 1
		if bool(dd.get("uses_real_water", false)):
			real_water += 1
		var rv: PackedVector2Array = dd.get("river", PackedVector2Array())
		river_pts += rv.size()
		print("  i=%3d kind=%-13s pop=%-6d realwater=%s river_pts=%-4d bridge_pt=%s bridges=%d ford=%s  | %s"
			% [int(dd.get("index", -1)), String(dd.get("site_kind", "?")), int(dd.get("pop", 0)),
			   str(dd.get("uses_real_water", false)), rv.size(), str(dd.has("bridge_pt")), nb,
			   str(dd.has("ford")), _stage(dd)])
	print("TOTAL bridges=%d fords=%d  towns=%d  with_bridge_pt=%d  uses_real_water=%d  river_pts_sum=%d"
		% [tot_b, tot_f, layouts.size(), with_bpt, real_water, river_pts])
	get_tree().quit(0)

func _stage(dd: Dictionary) -> String:
	for s in dd.get("stages", PackedStringArray()):
		if String(s).begins_with("detectRiverCrossings"):
			return String(s)
	return "<no stage line>"
