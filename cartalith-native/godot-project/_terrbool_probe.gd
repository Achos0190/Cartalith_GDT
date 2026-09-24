extends SceneTree

## `OUTSTANDING_WORK.md` §2.11, "Go to year bypasses the territory paint
## model", through the real `WorldGen` funcs:
##   1. `EngineBridge.civ_territory_paint_at` returns the engine's `bool`, and
##      marks the project dirty only when a dab was staged;
##   2. `civ_territory_paint_polygon` still returns `-1` for a refused faction,
##      and does not mark the project dirty then;
##   2b. `civ_territory_commit` returns the engine's `bool` and marks dirty
##      only after a commit that happened;
##   3. Ruling AT (2026-09-24), measured: Go to year at an unrecorded year
##      leaves territory -- and a pending stroke -- exactly as it was; at a
##      recorded year the snapshot comes back and becomes the paint base, so a
##      stroke, a subtract and Add year all act on that year.
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

	## 2b. The commit wrapper (§2.11 "Three small reopen/territory defects"):
	## returns the engine's `bool`, marks dirty only when something was
	## committed, and only after the engine call -- a listener on
	## `dirty_changed` reads the committed grid, not the one before it.
	bridge._set_dirty(false)
	var none = bridge.civ_territory_commit()
	_check("commit: nothing pending returns false", typeof(none) == TYPE_BOOL and none == false, "%s (type %d)" % [none, typeof(none)])
	_check("commit: nothing pending leaves the project clean", bridge.world_dirty == false)
	var before_commit := int(wg.civ_faction_territory_stats(f).get("claimed_cells", -1))
	bridge.civ_territory_paint_polygon(PackedVector2Array([Vector2(100, 100), Vector2(120, 100), Vector2(120, 120), Vector2(100, 120)]), f, false)
	bridge._set_dirty(false)
	var seen := [-1]
	var listener := func(v: bool) -> void:
		if v:
			seen[0] = int(wg.civ_faction_territory_stats(f).get("claimed_cells", -1))
	bridge.dirty_changed.connect(listener)
	var did = bridge.civ_territory_commit()
	bridge.dirty_changed.disconnect(listener)
	var after_commit := int(wg.civ_faction_territory_stats(f).get("claimed_cells", -1))
	_check("commit: a pending stroke returns true", typeof(did) == TYPE_BOOL and did == true, "%s" % did)
	_check("commit: a real commit marks the project dirty", bridge.world_dirty == true)
	_check("premise: the commit claimed cells", after_commit > before_commit, "%d -> %d" % [before_commit, after_commit])
	_check("commit: the dirty listener reads the committed grid", seen[0] == after_commit, "listener saw %d, committed %d, before %d" % [seen[0], after_commit, before_commit])
	bridge._set_dirty(false)

	## 3. Go to year (Ruling AT), measured.
	var live := _total(wg, ids)
	_check("premise: the world has claimed cells", live > 0, "%d" % live)
	## Year 100 is the computed map; year 200 is it plus a stroke of faction
	## `g` over a 40x40 square, so the two years differ in a known place.
	var g := int(ids[ids.size() - 1])
	var sq := PackedVector2Array([Vector2(20, 20), Vector2(60, 20), Vector2(60, 60), Vector2(20, 60)])
	wg.civ_add_year(100)
	var at100 := _total(wg, ids)
	var g100 := int(wg.civ_faction_territory_stats(g).get("claimed_cells", -1))
	wg.civ_add_year(200)
	wg.civ_territory_paint_polygon(sq, g, false)
	wg.civ_territory_commit()
	wg.civ_add_year(200)   # records the stroke under 200; the cursor stays
	var g200 := int(wg.civ_faction_territory_stats(g).get("claimed_cells", -1))
	_check("premise: year 200 gives faction %d more cells than 100" % g, g200 > g100, "%d vs %d" % [g200, g100])

	## Unrecorded: nothing but the cursor moves, a pending stroke included.
	wg.civ_territory_paint_polygon(PackedVector2Array([Vector2(70, 70), Vector2(90, 70), Vector2(90, 90), Vector2(70, 90)]), g, false)
	wg.civ_goto_year(150)
	var g150 := int(wg.civ_faction_territory_stats(g).get("claimed_cells", -1))
	_check("an unrecorded year leaves territory untouched", g150 == g200 and _total(wg, ids) > 0, "faction %d: %d at 150, %d before" % [g, g150, g200])
	_check("an unrecorded year keeps naming the year the claims came from", wg.get_civ_territory_year() == {"year": 200}, str(wg.get_civ_territory_year()))
	wg.civ_territory_commit()
	var g150c := int(wg.civ_faction_territory_stats(g).get("claimed_cells", -1))
	_check("the pending stroke survived the move and commits", g150c > g200, "%d -> %d" % [g200, g150c])

	## Recorded: the snapshot comes back, paint and the pending stroke go.
	wg.civ_territory_paint_polygon(sq, g, false)   # pending again, must be dropped
	wg.civ_goto_year(100)
	var back := _total(wg, ids)
	var g_back := int(wg.civ_faction_territory_stats(g).get("claimed_cells", -1))
	_check("the recorded year's snapshot comes back", back == at100 and g_back == g100, "%d vs %d, faction %d %d vs %d" % [back, at100, g, g_back, g100])
	_check("the territory year is 100", wg.get_civ_territory_year() == {"year": 100}, str(wg.get_civ_territory_year()))
	## A one-cell subtract dab where year 100 and 200 differ must restore year
	## 100's owner. Before the ruling the base was the first computed map and
	## the paint still held year 200's square, so the first commit here redrew it.
	wg.civ_territory_paint_at(90.0, 20.0, g, 0.0, false)
	wg.civ_territory_commit()
	var g_dab := int(wg.civ_faction_territory_stats(g).get("claimed_cells", -1))
	_check("a stroke at year 100 edits year 100 only", g_dab <= g100 + 1 and g_dab >= g100, "%d vs %d" % [g_dab, g100])
	wg.civ_territory_paint_at(90.0, 20.0, 0, 0.0, true)
	wg.civ_territory_commit()
	var g_sub := int(wg.civ_faction_territory_stats(g).get("claimed_cells", -1))
	_check("subtract restores year 100's owner", g_sub == g100, "%d vs %d" % [g_sub, g100])
	## Add year re-records 100 from the live grid: still year 100's map.
	wg.civ_add_year(300)
	wg.civ_goto_year(100)
	var g_rec := int(wg.civ_faction_territory_stats(g).get("claimed_cells", -1))
	_check("Add year recorded year 100, not the pre-jump map", g_rec == g100, "%d vs %d (year 200 had %d)" % [g_rec, g100, g200])
	wg.civ_goto_year(200)
	var g_200 := int(wg.civ_faction_territory_stats(g).get("claimed_cells", -1))
	_check("year 200 is untouched", g_200 == g200, "%d vs %d" % [g_200, g200])

	print("RESULT %s  checks=%d fails=%d" % ["PASS" if _fails == 0 else "FAIL", _checks, _fails])
	quit(1 if _fails > 0 else 0)
