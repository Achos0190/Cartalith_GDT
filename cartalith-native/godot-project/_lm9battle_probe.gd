extends Node
## `LANDMARK_GENERATION_SCOPE.md` M9's conflict wiring -- Battlefield, end to
## end through the real engine: a generated, populated world; a landmark run
## with no conflicts (Battlefield must place nothing, limit `no_terrain`); a
## Battle drawn through `conflict_add` and anchored to a real settlement; a
## second run, where the battlefield must appear on exactly that cell with the
## battle's name at the head of its causal chain. Negative controls: a Front
## drawn elsewhere adds no battlefield, and deleting the battle takes it away.
##
## Windowed (this project's rule for every probe that boots the extension):
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _lm9battle_probe.tscn
##
## Exit 0 = all passed, 1 = a real failure, 2 = could not run.

var _fail := 0


func _p(s: String) -> void:
	print("LM9  %s" % s)


func _check(name: String, cond: bool, detail: String = "") -> void:
	_p("%s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1


func _funnel(g, key: String) -> Dictionary:
	for f: Dictionary in g.landmark_funnels():
		if String(f["kind"]) == key:
			return f
	return {}


func _of_kind(g, key: String) -> Array:
	var out := []
	for l: Dictionary in g.landmarks():
		if String(l["kind"]) == key:
			out.append(l)
	return out


func _ready() -> void:
	var g := WorldGen.new()
	if not g.has_method("conflict_add"):
		_p("ABORT: this binary has no conflict_add #[func]")
		get_tree().quit(2)
		return
	g.generate_sized(5521, 1600.0, 320, 240)
	var places: Array = g.get_settlements()
	if places.is_empty():
		g.civ_populate()
		places = g.get_settlements()
	var factions: Array = g.get_factions()
	var fids := []
	for f: Dictionary in factions:
		if int(f.get("id", 0)) > 0:
			fids.append(int(f["id"]))
	if places.is_empty() or fids.size() < 2:
		_p("ABORT: world has %d settlements, %d factions" % [places.size(), fids.size()])
		get_tree().quit(2)
		return
	g.landmark_set_armed("battlefield", true)

	# -- 1. No conflicts: Battlefield has no input and places nothing.
	_check("run without conflicts", g.landmark_run())
	var f0 := _funnel(g, "battlefield")
	_check("no battles -> limit no_terrain, 0 placed",
		String(f0.get("limit", "")) == "no_terrain" and int(f0.get("placed", -1)) == 0, str(f0))
	var before: Array = g.landmarks()
	_p("first run placed %d landmarks" % before.size())

	# -- 2. The site: the settlement farthest from its nearest first-run
	# landmark, so the question is the wiring, not whether §16's spacing (the
	# Cultural ring, 6 km = 1.2 cells here) lets it in.
	var site := {}
	var best := -1.0
	for s: Dictionary in places:
		var sp := Vector2(float(s["x"]), float(s["y"]))
		var near := INF
		for l: Dictionary in before:
			near = minf(near, Vector2(float(l["x"]), float(l["y"])).distance_to(sp))
		if near > best:
			best = near
			site = s
	_p("site's nearest first-run landmark: %.2f cells" % best)
	if best <= 1.2:
		_p("ABORT: every settlement has a landmark inside the Cultural ring")
		get_tree().quit(2)
		return
	var sx := int(site["x"])
	var sy := int(site["y"])
	var tid := int(site.get("tid", 0))
	_p("site: settlement tid=%d '%s' at (%d,%d)" % [tid, String(site.get("name", "")), sx, sy])
	_check("no battlefield on that cell before", _of_kind(g, "battlefield").is_empty())

	var add: Dictionary = g.conflict_add({
		"name": "Battle of the Probe Ford", "kind": "battle",
		"start_year": 1210, "end_year": 1211, "sides": PackedInt32Array([fids[0], fids[1]]),
		"points": PackedVector2Array([Vector2(sx, sy)]),
		"anchor_kind": "settlement", "anchor_tid": tid})
	_check("conflict_add accepted the battle", bool(add.get("ok", false)), str(add))
	var bid := int(add.get("id", 0))

	# -- 3. The same world, one battle drawn: the battlefield appears there.
	_check("run with the battle", g.landmark_run())
	var f1 := _funnel(g, "battlefield")
	var bf := _of_kind(g, "battlefield")
	_check("battlefield funnel: 1 candidate, 1 placed",
		int(f1.get("candidates", -1)) == 1 and int(f1.get("placed", -1)) == 1, str(f1))
	_check("exactly one battlefield", bf.size() == 1, str(bf.size()))
	if bf.size() == 1:
		var l: Dictionary = bf[0]
		_check("on the drawn cell", int(l["x"]) == sx and int(l["y"]) == sy, "%d,%d" % [int(l["x"]), int(l["y"])])
		var causal: PackedStringArray = l["causal"]
		_check("causal chain names the battle and its years",
			causal.size() > 0 and causal[0] == "site of Battle of the Probe Ford (1210–1211)", str(causal))
		_check("causal chain ends in the type label", causal.size() > 0 and causal[causal.size() - 1] == "Battlefield")
		_p("battlefield: %s" % str(l))
	var after: Array = g.landmarks()
	_p("second run placed %d landmarks (%d before)" % [after.size(), before.size()])

	# -- 4. Negative control: a front is a line through country, not a field.
	var far := Vector2((sx + 40) % 320, sy)
	var fr: Dictionary = g.conflict_add({
		"name": "Probe Front", "kind": "front", "start_year": 1210, "end_year": 1211,
		"sides": PackedInt32Array([fids[0], fids[1]]),
		"points": PackedVector2Array([far, far + Vector2(0, 10)])})
	_check("conflict_add accepted the front", bool(fr.get("ok", false)), str(fr))
	g.landmark_run()
	_check("a front adds no battlefield", _of_kind(g, "battlefield").size() == 1,
		str(_of_kind(g, "battlefield").size()))

	# -- 5. Delete the battle: the battlefield goes with it.
	_check("battle deleted", g.conflict_delete(bid))
	g.landmark_run()
	var f2 := _funnel(g, "battlefield")
	_check("no battle left -> 0 placed, no_terrain",
		int(f2.get("placed", -1)) == 0 and String(f2.get("limit", "")) == "no_terrain", str(f2))

	_p("%s  (%d failure(s))" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(0 if _fail == 0 else 1)
