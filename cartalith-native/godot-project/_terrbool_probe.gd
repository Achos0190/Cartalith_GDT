extends SceneTree

## `OUTSTANDING_WORK.md` §2.11, "Go to year bypasses the territory paint
## model" -- its "Also" clause only (Go to year's behaviour is an owner
## question and is not changed):
##   1. `EngineBridge.civ_territory_paint_at` returns the engine's `bool`, and
##      marks the project dirty only when a dab was staged;
##   2. `civ_territory_paint_polygon` still returns `-1` for a refused faction,
##      and does not mark the project dirty then;
##   3. the corrected `dcc_shell.gd` §10a claim, measured: Go to year at an
##      unrecorded year leaves no claimed cell; at a recorded year its
##      snapshot comes back.
##
## HEADLESS:
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . --script res://_terrbool_probe.gd

var _checks := 0
var _fails := 0

func _check(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	print("TERRBOOL %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _total(wg: WorldGen, ids: Array) -> int:
	var t := 0
	for f in ids:
		t += int(wg.civ_faction_territory_stats(int(f)).get("claimed_cells", 0))
	return t

func _initialize() -> void:
	var bridge = load("res://shell/engine_bridge.gd").new()
	root.add_child(bridge)
	var wg: WorldGen = bridge.world_gen
	wg.generate(12345, 800.0, 128)
	bridge.has_world = true
	bridge._set_dirty(false)
	var factions: Array = wg.get_factions()
	_check("world has factions", factions.size() >= 1, "%d" % factions.size())
	if factions.is_empty():
		print("RESULT FAIL  checks=%d fails=%d (cannot run)" % [_checks, _fails + 1])
		quit(3)
		return
	var ids: Array = []
	for d in factions:
		ids.append(int(d["id"]))
	var f := int(ids[0])

	## 1. The brush wrapper.
	var refused = bridge.civ_territory_paint_at(40.0, 40.0, 256, 6.0, false)
	_check("brush: faction 256 returns false", typeof(refused) == TYPE_BOOL and refused == false, "%s (type %d)" % [refused, typeof(refused)])
	_check("brush: a refused dab leaves the project clean", bridge.world_dirty == false)
	var staged = bridge.civ_territory_paint_at(40.0, 40.0, f, 6.0, false)
	_check("brush: faction %d returns true" % f, typeof(staged) == TYPE_BOOL and staged == true, "%s" % staged)
	_check("brush: a staged dab marks the project dirty", bridge.world_dirty == true)
	bridge.civ_territory_discard()
	bridge._set_dirty(false)

	## 2. The lasso wrapper.
	var ring := PackedVector2Array([Vector2(30, 30), Vector2(50, 30), Vector2(50, 50), Vector2(30, 50)])
	var n_ref: int = bridge.civ_territory_paint_polygon(ring, 256, false)
	_check("lasso: faction 256 returns -1", n_ref == -1, "%d" % n_ref)
	_check("lasso: a refused ring leaves the project clean", bridge.world_dirty == false)
	var n_ok: int = bridge.civ_territory_paint_polygon(ring, f, false)
	_check("lasso: a real ring stages cells", n_ok > 0, "%d" % n_ok)
	_check("lasso: staged cells mark the project dirty", bridge.world_dirty == true)
	bridge.civ_territory_discard()

	## 3. Go to year, measured (the §10a correction's claim).
	var live := _total(wg, ids)
	_check("premise: the world has claimed cells", live > 0, "%d" % live)
	wg.civ_add_year(100)
	var at100 := _total(wg, ids)
	wg.civ_goto_year(150)
	var at150 := _total(wg, ids)
	_check("an unrecorded year leaves no claimed cell", at150 == 0, "%d at 150 (was %d)" % [at150, at100])
	wg.civ_goto_year(100)
	var back := _total(wg, ids)
	_check("the recorded year's snapshot comes back", back == at100 and back > 0, "%d vs %d" % [back, at100])

	print("RESULT %s  checks=%d fails=%d" % ["PASS" if _fails == 0 else "FAIL", _checks, _fails])
	quit(1 if _fails > 0 else 0)
