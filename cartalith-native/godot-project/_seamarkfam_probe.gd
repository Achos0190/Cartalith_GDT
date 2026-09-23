extends Node
## Windowed proof that the Asset library window reaches the port's ninth
## family, `seamarks` (`OUTSTANDING_WORK.md` §2.10, 2026-09-24).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 \
##       _seamarkfam_probe.tscn -- --nowelcome
##
## Before the fix `FAMILIES` listed eight keys and no `seamarks`, so a pack
## could fill those slots but the window could not show them. Asserts, in order:
##   1. the ENGINE has the family: `as_family_slots("seamarks")` returns 8 rows
##      (`slots.rs::PACK_SEAMARK_SLOTS`) -- the source of truth, read not assumed;
##   2. the window's `FAMILIES` carries `seamarks`, with the engine's slot ids;
##   3. opening the window on it draws one grid cell per engine slot, with a
##      control: the same window on `poi` draws that family's own count, so the
##      grid demonstrably follows the family rather than a fixed size.

var _fails := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _ok(what: String, cond: bool, detail: String = "") -> void:
	print("SEAFAM %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("SEAFAM REFUSED: headless (the grid is only built windowed)")
		get_tree().quit(2)
		return
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	if "--nowelcome" in OS.get_cmdline_user_args():
		app.open_project_dialog.hide()
		await _frames(2)

	app.open_asset_library("seamarks")
	await _frames(20)
	var win: Node = app.asset_library_window
	_ok("the Asset library window is open", win != null and (win as Window).visible)
	if win == null:
		get_tree().quit(1)
		return
	var bridge: Node = win.get("_bridge")

	# 1. the engine side, read rather than assumed
	var eng: Array = bridge.as_family_slots("seamarks")
	var eng_ids: Array = []
	for s in eng:
		eng_ids.append(String((s as Dictionary).get("id", "")))
	_ok("engine has 8 sea-mark slots", eng.size() == 8, str(eng_ids))

	# 2. the window's family list carries it, with the engine's own ids
	var fam: Dictionary = {}
	for f in AssetLibraryWindow.FAMILIES:
		if String(f["key"]) == "seamarks":
			fam = f
	_ok("FAMILIES has seamarks", not fam.is_empty())
	var win_ids: Array = fam.get("slots", [])
	var same := win_ids.size() == eng_ids.size()
	for id in eng_ids:
		same = same and win_ids.has(id)
	_ok("window slot ids match the engine's", same, "%s vs %s" % [win_ids, eng_ids])

	# 3. the grid follows the family: seamarks, then poi as the control
	var sea_cells := (win.get("_grid") as GridContainer).get_child_count()
	_ok("grid draws one cell per sea-mark slot", sea_cells == eng.size(),
		"%d cells" % sea_cells)
	app.open_asset_library("poi")
	await _frames(20)
	var poi_n: int = (bridge.as_family_slots("poi") as Array).size()
	var poi_cells := (win.get("_grid") as GridContainer).get_child_count()
	_ok("control: poi draws its own count, not a fixed size",
		poi_cells == poi_n and poi_n != sea_cells, "poi %d cells / %d slots" % [poi_cells, poi_n])

	print("SEAFAM %s  (%d failures)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
