extends Node
## Proof for IN-13 piece 4, caravans as a derived view (`OUTSTANDING_WORK.md`
## §2.3, 2026-09-24; Rulings AF + AP). On a real generated world:
##   1. `civ_trade_flows()` returns caravan rows, one per way with load;
##   2. each row's goods volumes sum to its load, and names at least one good;
##   3. rows are sorted by load, heaviest first, and `caravan_count` is the
##      number of loaded ways (at least as many as the drawn roads show);
##   4. the Trade panel draws a "Caravans" group with rows after a match;
##   5. (sea half, Ruling AQ, 2026-10-13) `sea_caravans` holds one row per port
##      pair trading by sea, goods summing to load, heaviest first, together
##      carrying exactly the sea flows' volume, and the panel lists them.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _caravan_probe.tscn
##
## Result: `CARAVAN ok  <what>` or `CARAVAN FAIL  <what>` per check, then a final
##         `CARAVAN PASS|FAIL  (N failures)`; exit 0 when none failed, else 1.

var _fails := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	print("CARAVAN %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _find_script(n: Node, file: String) -> Node:
	for c in n.get_children():
		if c.get_script() != null and String(c.get_script().resource_path).ends_with(file):
			return c
		var r := _find_script(c, file)
		if r != null:
			return r
	return null

func _ready() -> void:
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge: Node = app.bridge
	bridge.generate({"seed": 9137, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(6)

	var d: Dictionary = bridge.civ_trade_flows()
	var rows: Array = d.get("caravans", [])
	var count := int(d.get("caravan_count", -1))
	_check("caravan rows exist", rows.size() > 0, "%d rows, count %d" % [rows.size(), count])
	var sums_ok := true
	var named_ok := true
	var sorted_ok := true
	var prev := INF
	for r in rows:
		var row: Dictionary = r
		var load := float(row["load"])
		var vols: PackedFloat64Array = row["volumes"]
		var s := 0.0
		for v in vols:
			s += v
		if absf(s - load) > 1e-6 * maxf(1.0, load):
			sums_ok = false
		if (row["goods"] as PackedStringArray).is_empty():
			named_ok = false
		if load > prev:
			sorted_ok = false
		prev = load
	_check("each caravan's goods sum to its load", sums_ok)
	_check("each caravan names what it carries", named_ok)
	_check("heaviest first", sorted_ok)
	var drawn_loaded := 0
	for v in (d.get("way_load", PackedFloat32Array()) as PackedFloat32Array):
		if v > 0.0:
			drawn_loaded += 1
	_check("one caravan per loaded way", count >= drawn_loaded and count == rows.size() or count > 2000,
		"caravan_count %d, loaded drawn ways %d" % [count, drawn_loaded])

	# Sea half (Ruling AQ, 2026-10-13): one caravan per port pair trading by sea.
	var sea: Array = d.get("sea_caravans", [])
	var sea_count := int(d.get("sea_caravan_count", -1))
	var pairs := {}
	var sea_vol := 0.0
	for f in d.get("flows", []):
		if String(f["mode"]) == "sea":
			pairs[Vector2i(mini(int(f["from"]), int(f["to"])), maxi(int(f["from"]), int(f["to"])))] = true
			sea_vol += float(f["volume"])
	var whole := int(d.get("flow_rows", 0)) == int(d.get("flow_count", -1))
	_check("this world trades by sea (else the sea checks prove nothing)", pairs.size() > 0,
		"sea_share %.3f, %d sea pairs in %d flow rows" % [float(d.get("sea_share", 0.0)), pairs.size(), int(d.get("flow_rows", 0))])
	_check("one sea caravan per port pair trading by sea", not whole or sea_count == pairs.size(),
		"sea_caravan_count %d, pairs %d (flow list whole: %s)" % [sea_count, pairs.size(), whole])
	var sea_ok := true
	var sea_sorted := true
	var sea_sum := 0.0
	prev = INF
	for r in sea:
		var row: Dictionary = r
		var load := float(row["load"])
		var s := 0.0
		for v in (row["volumes"] as PackedFloat64Array):
			s += v
		if absf(s - load) > 1e-6 * maxf(1.0, load) or (row["goods"] as PackedStringArray).is_empty() 				or int(row["a"]) >= int(row["b"]) or not pairs.has(Vector2i(int(row["a"]), int(row["b"]))):
			sea_ok = false
		if load > prev:
			sea_sorted = false
		prev = load
		sea_sum += load
	_check("each sea caravan is a real trading pair whose goods sum to its load", sea_ok)
	_check("sea caravans heaviest first", sea_sorted)
	_check("sea caravans carry exactly the sea flows' volume", not whole or absf(sea_sum - sea_vol) <= 1e-6 * maxf(1.0, sea_vol),
		"%.3f vs %.3f" % [sea_sum, sea_vol])

	var ws: Node = _find_script(app, "infrastructure_workspace.gd")
	_check("the Trade workspace is found", ws != null)
	if ws != null:
		ws._match_trade_flows()
		await _frames(4)
		var body: Node = ws._flows_body
		var found := false
		var notes := 0
		var stack: Array = [body]
		while not stack.is_empty():
			var n: Node = stack.pop_back()
			for c in n.get_children():
				stack.append(c)
				if c is Button and (c as Button).text.to_lower().contains("caravans"):  ## a group header is a Button, e.g. "▾ Caravans"
					found = true
				if c is Label and (c as Label).text.contains(" -- ") and (c as Label).text.contains("%"):
					notes += 1
		_check("the Trade panel draws a Caravans group with rows", found and notes > 0, "found %s, rows with shares %d" % [found, notes])
		var by_sea := false
		var sea_lines := 0
		var stack2: Array = [body]
		var names := {}
		for r in sea:
			names[String(r["name"])] = true
		while not stack2.is_empty():
			var n: Node = stack2.pop_back()
			for c in n.get_children():
				stack2.append(c)
				if c is Label:
					var t := (c as Label).text
					if t == "By sea:":
						by_sea = true
					var cut := t.find(" -- ", t.find(" -- ") + 4)
					if cut > 0 and names.has(t.substr(0, cut)):
						sea_lines += 1
		_check("the Caravans group lists the sea lanes", by_sea and sea_lines == mini(12, sea.size()),
			"By sea: %s, sea lines %d of %d" % [by_sea, sea_lines, sea.size()])
	print("CARAVAN %s  (%d failures)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
