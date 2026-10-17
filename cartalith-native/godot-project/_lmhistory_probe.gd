extends Node
## `LANDMARK_GENERATION_SCOPE.md` M8 residual (2026-10-17) -- Abandoned
## settlement and Ancient road, end to end through the real engine. What a
## unit test cannot reach is `WorldGen::with_landmark_inputs`' assembly of
## `LandmarkInputs::history` from the live `CivData` (`timeline`, `year`,
## `settlements`, `ways`); this drives it through the shell's own bindings.
##
## Steps: a generated, populated world; one recorded year (100) -- nothing is
## history yet, so both kinds report `no_terrain`; a second year (200), then
## one settlement deleted from the live roster -- Abandoned settlement must now
## have exactly that one candidate, and if it places, on that settlement's
## cell with its name in the causal chain. Negative control: going back to
## year 100, whose recording still holds the settlement and is not before
## itself, leaves Abandoned settlement with no candidate again. Portage, Ancient
## road and the rest are reported, not asserted (world-dependent).
##
## Windowed (this project's rule for every probe that boots the extension):
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _lmhistory_probe.tscn
##
## Result: an assertion. Every line is prefixed `LMH  `: `ok  |FAIL  <check>`
## per check and a final `LMH  PASS|FAIL  (<n> failure(s))`. Exit 0 on PASS,
## 1 on FAIL, 2 after `ABORT: ...` (no settlements, or a binary without the
## Timeline bindings).

var _fail := 0


func _p(s: String) -> void:
	print("LMH  %s" % s)


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
	if not g.has_method("civ_add_year") or not g.has_method("civ_delete_settlement"):
		_p("ABORT: this binary has no Timeline/roster bindings")
		get_tree().quit(2)
		return
	g.generate_sized(5521, 1600.0, 320, 240)
	var places: Array = g.get_settlements()
	if places.is_empty():
		g.civ_populate()
		places = g.get_settlements()
	if places.size() < 3:
		_p("ABORT: world has %d settlements" % places.size())
		get_tree().quit(2)
		return

	# -- 1. One recorded year: nothing is before it, so no history.
	g.civ_add_year(100)
	_check("cursor at 100", g.get_civ_year() == 100, str(g.get_civ_year()))
	_check("run at 100", g.landmark_run())
	for key in ["abandoned_settlement", "ancient_road"]:
		var f := _funnel(g, key)
		_check("%s at the first year: no_terrain, 0 candidates" % key,
			String(f.get("limit", "")) == "no_terrain" and int(f.get("candidates", -1)) == 0, str(f))

	# -- 2. Year 200, then delete the settlement farthest from every other one
	# (so the 5 km clearance cannot be what refuses it).
	g.civ_add_year(200)
	_check("cursor at 200", g.get_civ_year() == 200, str(g.get_civ_year()))
	places = g.get_settlements()
	var pick := -1
	var best := -1.0
	for i in places.size():
		var sp := Vector2(float(places[i]["x"]), float(places[i]["y"]))
		var near := INF
		for j in places.size():
			if j != i:
				near = minf(near, sp.distance_to(Vector2(float(places[j]["x"]), float(places[j]["y"]))))
		if near > best:
			best = near
			pick = i
	var gone: Dictionary = places[pick]
	_p("deleting %s at (%d, %d), %.1f cells from its nearest neighbour" % [
		String(gone["name"]), int(gone["x"]), int(gone["y"]), best])
	_check("delete", g.civ_delete_settlement(pick))
	_check("run at 200", g.landmark_run())
	var fa := _funnel(g, "abandoned_settlement")
	_check("one former settlement enters the funnel", int(fa.get("candidates", -1)) == 1, str(fa))
	for key in ["ancient_road", "portage"]:
		_p("info: %s %s" % [key, str(_funnel(g, key))])
	# With every other kind armed, another landmark may already hold the cell
	# (the funnel then says `spacing`), so the placement itself is checked
	# with Abandoned settlement armed alone.
	for k: Dictionary in g.landmark_kinds():
		g.landmark_set_armed(String(k["key"]), String(k["key"]) == "abandoned_settlement")
	_check("run with Abandoned settlement alone", g.landmark_run())
	var ab := _of_kind(g, "abandoned_settlement")
	_check("it places", ab.size() == 1, str(_funnel(g, "abandoned_settlement")))
	if ab.size() == 1:
		var l: Dictionary = ab[0]
		_check("on the deleted settlement's cell",
			int(l["x"]) == int(gone["x"]) and int(l["y"]) == int(gone["y"]), "%d,%d" % [int(l["x"]), int(l["y"])])
		var chain := " | ".join(PackedStringArray(l.get("causal", [])))
		_check("the chain names it and its last year",
			chain.find("%s stood here until 100" % String(gone["name"])) >= 0, chain)

	# -- 3. Back to 100: its own recording is the present, and holds the place.
	g.civ_goto_year(100)
	_check("cursor back at 100", g.get_civ_year() == 100, str(g.get_civ_year()))
	g.landmark_run()
	var fb := _funnel(g, "abandoned_settlement")
	_check("at 100 nothing is abandoned", int(fb.get("candidates", -1)) == 0, str(fb))

	_p("%s  (%d failure(s))" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(0 if _fail == 0 else 1)
