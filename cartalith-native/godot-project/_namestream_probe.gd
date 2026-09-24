extends SceneTree

## `OUTSTANDING_WORK.md` §2.11, "Three small reopen/territory defects", the
## third: the manual-placement name stream (`CivTools::name_rng`) restarted
## from the seed on reopen, so a reopened session's first blank-named drop
## drew the same name as the first drop before the save. Now the save records
## the stream's position (`entities/settlements.json` `name_stream`,
## `SAVEFILE_COMPAT.md` §9.1) and `project_open` resumes it.
##
## Through the real `WorldGen`: generate, place six blank-named settlements,
## `project_save`, `project_open` into a fresh `WorldGen`, place six more; no
## name placed after the reopen may repeat one placed before it.
##
## HEADLESS:
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . --script res://_namestream_probe.gd

var _checks := 0
var _fails := 0

func _check(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	print("NAMESTREAM %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

## Blank-named drops over a coarse lattice until `want` new settlements exist;
## a drop that selects an existing settlement or lands in water places none.
func _place(wg: WorldGen, faction: int, want: int, offset: int) -> Array:
	var names: Array = []
	var gw := 128
	var step := 9
	var y := 6 + offset
	while y < gw - 4 and names.size() < want:
		var x := 5 + offset
		while x < gw - 4 and names.size() < want:
			var before := wg.get_settlements().size()
			var idx := wg.civ_drop_settlement(float(x), float(y), "town", faction, "", false)
			if idx >= 0 and wg.get_settlements().size() == before + 1:
				names.append(String(wg.get_settlements()[idx]["name"]))
			x += step
		y += step
	return names

func _initialize() -> void:
	var wg := WorldGen.new()
	wg.generate(12345, 800.0, 128)
	var factions: Array = wg.get_factions()
	var f := int(factions[0]["id"]) if not factions.is_empty() else 1
	var before := _place(wg, f, 6, 0)
	_check("six blank-named settlements placed before the save", before.size() == 6, str(before))

	var path := ProjectSettings.globalize_path("user://_namestream_probe.zip")
	var saved: Dictionary = wg.project_save(path)
	_check("the project saved", bool(saved.get("ok", false)), str(saved.get("error", "")))

	var re := WorldGen.new()
	var opened: Dictionary = re.project_open(path)
	_check("the project reopened", bool(opened.get("ok", false)), str(opened.get("error", "")))
	_check("premise: reopened as the full world, so drops are allowed", String(opened.get("substrate", "")) == "complete", String(opened.get("substrate", "")))
	## Offset lattice, so these clicks are not the pre-save ones re-selecting.
	var after := _place(re, f, 6, 4)
	_check("six more placed after the reopen", after.size() == 6, str(after))
	var repeats: Array = []
	for n in after:
		if before.has(n):
			repeats.append(n)
	_check("no name placed after the reopen repeats one placed before it", repeats.is_empty(), "repeats %s; before %s; after %s" % [repeats, before, after])

	DirAccess.remove_absolute(path)
	print("RESULT %s  checks=%d fails=%d" % ["PASS" if _fails == 0 else "FAIL", _checks, _fails])
	quit(1 if _fails > 0 else 0)
