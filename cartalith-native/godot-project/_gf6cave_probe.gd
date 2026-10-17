extends Node
## `GEOLOGY_FIRST_SCOPE.md` GF-6 (2026-10-17): Cave and Spring read GF-5's
## karst model through `WorldGen::with_landmark_inputs`' assembly of
## `LandmarkInputs::karst` (`KarstView::for_world`). A unit test cannot reach
## that assembly; this drives it through the shell's own bindings.
##
## Steps: (1) the app's parameters (`geology_karst` off), seed 483920 at
## 512 x 328 and 800 km: Cave must report `no_terrain` with 0 candidates and
## place nothing. (2) The same world with `geology_processes` and
## `geology_karst` set on through `set_params`: Cave must have candidates and
## place at least one, and every cave's chain must name the dissolving rock.
## Spring is reported, not asserted.
##
## Windowed (this project's rule for every probe that boots the extension):
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _gf6cave_probe.tscn
##
## Result: an assertion. Every line is prefixed `GF6  `: `ok  |FAIL  <check>`
## per check and a final `GF6  PASS|FAIL  (<n> failure(s))`. Exit 0 on PASS,
## 1 on FAIL, 2 after `ABORT: ...` (a `set_params` key rejected).

var _fail := 0


func _p(s: String) -> void:
	print("GF6  %s" % s)


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
	# -- 1. The app's own parameters: the karst model is off.
	g.generate_sized(483920, 800.0, 512, 328)
	_check("run, karst off", g.landmark_run())
	var f := _funnel(g, "cave")
	_check("cave is no_terrain with karst off", String(f.get("limit", "")) == "no_terrain", str(f))
	_check("cave has no candidates with karst off", int(f.get("candidates", -1)) == 0, str(f))
	_check("no cave placed with karst off", _of_kind(g, "cave").is_empty())
	_p("info spring, karst off: %s" % str(_funnel(g, "spring")))

	# -- 2. The same world with the karst model on.
	var rep: Dictionary = g.set_params({"geology_processes": true, "geology_karst": true})
	if not (rep.get("rejected", PackedStringArray()) as PackedStringArray).is_empty():
		_p("ABORT: set_params rejected %s" % str(rep))
		get_tree().quit(2)
		return
	g.generate_sized(483920, 800.0, 512, 328)
	_check("run, karst on", g.landmark_run())
	f = _funnel(g, "cave")
	_check("cave has candidates with karst on", int(f.get("candidates", 0)) > 0, str(f))
	var caves := _of_kind(g, "cave")
	_check("caves placed with karst on", not caves.is_empty(), str(f))
	for c: Dictionary in caves:
		var chain := str(c.get("causal", []))
		_check("cave chain names dissolving rock", chain.find("the karst model dissolves") >= 0, chain)
	_p("info spring, karst on: %s" % str(_funnel(g, "spring")))
	_p("PASS  (0 failure(s))" if _fail == 0 else "FAIL  (%d failure(s))" % _fail)
	get_tree().quit(0 if _fail == 0 else 1)
