extends Node
## `LANDMARK_GENERATION_SCOPE.md` M8 residual and M9 (2026-11-12), end to end through the real
## engine: what a Rust unit test cannot reach is `WorldGen::with_landmark_inputs`' assembly of the
## new inputs (biome, temperature, snowline, faiths, the present year) and the new bindings.
##
## Checks: (1) the twelve new kinds are listed buildable; (2) a landmark run on a generated, populated
## world reports a funnel for each and places Ancient forest and Delta; (3) the present-year split:
## a battle drawn 100-110 is a Historic battlefield at year 300 and a Battlefield at year 105, and
## a name typed for one carries to the other (the alias); (4) `landmark_phase` answers for a placed
## landmark with a valid phase and a reason, and is EMPTY for an unknown key (absence, not "natural").
## Positive and negative controls are the year change and the unknown key.
##
## Windowed, under C:/cw/.win_lock:
##   godot --path . _lmm9_probe.tscn
## Every line is prefixed `LM9  `; exit 0 on PASS, 1 on FAIL, 2 after `ABORT: ...`.

var _fail := 0
const NEW := ["delta", "ancient_forest", "salt_works", "glacial_feature", "battlefield_historic", "ruin",
	"destroyed_fortress", "historic_crossing", "shrine", "sacred_grove", "sacred_mountain", "ceremonial_site"]
const PHASES := ["natural", "utilised", "developed", "monumentalised", "abandoned", "ruined"]


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
	for m in ["conflict_add", "landmark_phase", "civ_add_year", "civ_goto_year"]:
		if not g.has_method(m):
			_p("ABORT: this binary has no %s" % m)
			get_tree().quit(2)
			return
	g.generate_sized(5521, 1600.0, 320, 240)
	var places: Array = g.get_settlements()
	if places.is_empty():
		g.civ_populate()
		places = g.get_settlements()
	var fids := []
	for f: Dictionary in g.get_factions():
		if int(f.get("id", 0)) > 0:
			fids.append(int(f["id"]))
	if places.size() < 3 or fids.size() < 2:
		_p("ABORT: world has %d settlements, %d factions" % [places.size(), fids.size()])
		get_tree().quit(2)
		return

	# -- 1. The table.
	var buildable := {}
	for k: Dictionary in g.landmark_kinds():
		buildable[String(k["key"])] = bool(k.get("buildable", false))
	var unbuilt := []
	for key in NEW:
		if not buildable.get(key, false):
			unbuilt.append(key)
	_check("the twelve new kinds are buildable", unbuilt.is_empty(), str(unbuilt))
	for key in ["temple", "pilgrimage_site", "tomb", "monument", "saddle", "river_crossing"]:
		_check("%s stays refused, with a reason" % key, not buildable.get(key, true))

	# -- 2. A run at the world's own year.
	_check("run", g.landmark_run())
	for key in NEW:
		var f := _funnel(g, key)
		_p("%-20s candidates %5d  placed %3d  %s" % [key, int(f.get("candidates", -1)), int(f.get("placed", -1)), String(f.get("limit", ""))])
		_check("%s has a funnel row" % key, not f.is_empty())
	_check("ancient forest placed on a real world", _of_kind(g, "ancient_forest").size() > 0, str(_funnel(g, "ancient_forest")))
	_check("delta placed on a real world", _of_kind(g, "delta").size() > 0, str(_funnel(g, "delta")))
	for key in ["ruin", "destroyed_fortress", "historic_crossing", "battlefield_historic"]:
		_check("%s has nothing to read yet (no collapse run, no ended battle): no_terrain" % key,
			String(_funnel(g, key).get("limit", "")) == "no_terrain" and int(_funnel(g, key).get("placed", -1)) == 0,
			str(_funnel(g, key)))

	# -- 3. The present-year split and the name alias.
	var site: Dictionary = places[0]
	var sx := int(site["x"])
	var sy := int(site["y"])
	var tid := int(site.get("tid", 0))
	var add: Dictionary = g.conflict_add({
		"name": "Battle of the Probe Ford", "kind": "battle", "start_year": 100, "end_year": 110,
		"sides": PackedInt32Array([fids[0], fids[1]]), "points": PackedVector2Array([Vector2(sx, sy)]),
		"anchor_kind": "settlement", "anchor_tid": tid})
	_check("conflict_add accepted the battle", bool(add.get("ok", false)), str(add))
	g.civ_add_year(300)
	_check("cursor at 300", g.get_civ_year() == 300, str(g.get_civ_year()))
	_check("run at 300", g.landmark_run())
	var hist := _of_kind(g, "battlefield_historic")
	_check("at year 300 the battle ended 190 years ago: a Historic battlefield, no Battlefield",
		hist.size() == 1 and _of_kind(g, "battlefield").is_empty(), "%d historic, %d current" % [hist.size(), _of_kind(g, "battlefield").size()])
	var hkey := ""
	if hist.size() == 1:
		hkey = "battlefield_historic@%d,%d" % [int(hist[0]["x"]), int(hist[0]["y"])]
		var causal: PackedStringArray = hist[0]["causal"]
		_check("the chain says how long ago", causal.size() > 0 and "190 years before year 300" in causal[0], str(causal))
		_check("a name typed for it is accepted", g.landmark_set_user_name(hkey, "The Ford Remembered"))
	g.civ_add_year(105)
	_check("cursor at 105", g.get_civ_year() == 105, str(g.get_civ_year()))
	_check("run at 105", g.landmark_run())
	var cur := _of_kind(g, "battlefield")
	_check("at year 105 the battle is still going: a Battlefield, no Historic battlefield",
		cur.size() == 1 and _of_kind(g, "battlefield_historic").is_empty(), "%d current" % cur.size())
	if cur.size() == 1 and hkey != "":
		var ckey := "battlefield@%d,%d" % [int(cur[0]["x"]), int(cur[0]["y"])]
		var named := ""
		for n: Dictionary in g.landmark_names():
			if String(n["key"]) == ckey:
				named = "%s/%s" % [String(n["name"]), String(n["source"])]
		_check("the typed name carried across the era change", named == "The Ford Remembered/user", named)
		_check("clearing it on the new key clears it for good", g.landmark_set_user_name(ckey, ""))
		for n: Dictionary in g.landmark_names():
			if String(n["key"]) == ckey:
				_check("after clearing the generated name is back", String(n["source"]) != "user", str(n))

	# -- 4. Phase.
	var lms: Array = g.landmarks()
	var seen := {}
	for l: Dictionary in lms:
		var key := "%s@%d,%d" % [String(l["kind"]), int(l["x"]), int(l["y"])]
		var ph: Dictionary = g.landmark_phase(key)
		if ph.has("phase"):
			seen[String(ph["phase"])] = true
			if not (String(ph["phase"]) in PHASES) or String(ph.get("because", "")) == "":
				_check("phase %s is valid with a reason" % key, false, str(ph))
	_p("phases seen: %s" % str(seen.keys()))
	_check("some landmark has a phase", not seen.is_empty())
	_check("at least two different phases (natural/utilised and developed or monumentalised)", seen.size() >= 2, str(seen.keys()))
	_check("an unknown key has no phase at all (absence, not 'natural')", g.landmark_phase("nope@1,1").is_empty())

	_p("%s  (%d failure(s))" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(0 if _fail == 0 else 1)
