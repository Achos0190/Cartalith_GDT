extends Node
## `OUTSTANDING_WORK.md` §2.11, "`jp_claimed_at` may count every cell as
## claimed; missing grids still read as zeros elsewhere" -- the second and
## third halves. A project reopened from an archive without
## `rasters/territory.i32` (or `rasters/provinces.i32`) must report those
## figures as unknown, never as the zero an empty grid sums to.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _claimsabsent_probe.tscn
##
## A real generated world with its substrate, written through the app's own
## `_write_project`, then reopened three ways through `_load_project`:
##   1. control, whole: every claim-derived figure is present, and the Faction
##      Roster prints a claimed-cell total and no "Not known" reason;
##   2. without the claim grid: `get_factions()` omits `claimed_cells`; the
##      faction economy, military summary, relations and terrain fits carry
##      `absent` = "no_claim_grid" and omit what the claims feed; a garrison
##      reading says `no_claim_grid`; the regional population omits its
##      claimed share; the Roster prints dashes with the reason
##      and no "0 claimed cells"; Clear territory reports `{}` (count unknown)
##      and leaves a whole, empty, known grid behind;
##   3. without the province raster: `project_open` warns, there is no
##      boundary overlay (no panic), the province list is kept, and one
##      committed territory stroke brings the overlay back.

var _fails := 0
var _checks := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	print("CLAIMSABSENT %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

## Copy `src` to `dst` without every entry whose name starts with `prefix`;
## returns how many entries were dropped.
func _strip(src: String, dst: String, prefix: String) -> int:
	var reader := ZIPReader.new()
	if reader.open(src) != OK:
		return -1
	var packer := ZIPPacker.new()
	if packer.open(dst) != OK:
		return -1
	var dropped := 0
	for name in reader.get_files():
		if name.begins_with(prefix):
			dropped += 1
			continue
		packer.start_file(name)
		packer.write_file(reader.read_file(name))
		packer.close_file()
	packer.close()
	reader.close()
	return dropped

func _texts(node: Node, out: PackedStringArray) -> void:
	if node is Label:
		out.append((node as Label).text)
	elif node is RichTextLabel:
		out.append((node as RichTextLabel).get_parsed_text())
	elif node is Button:
		out.append((node as Button).text)
	for c in node.get_children():
		_texts(c, out)

func _roster_text(app: Node) -> String:
	app.open_faction_roster()
	await _frames(4)
	var out := PackedStringArray()
	_texts(app.faction_roster_window, out)
	app.faction_roster_window.hide()
	return "\n".join(out)

func _all_rows_have(rows: Array, key: String) -> bool:
	if rows.is_empty():
		return false
	for r in rows:
		if not (r as Dictionary).has(key):
			return false
	return true

func _no_row_has(rows: Array, key: String) -> bool:
	for r in rows:
		if (r as Dictionary).has(key):
			return false
	return true

func _all_absent(rows: Array) -> bool:
	if rows.is_empty():
		return false
	for r in rows:
		if String((r as Dictionary).get("absent", "")) != "no_claim_grid":
			return false
	return true

func _open(app: Node, path: String) -> bool:
	var ok: bool = app._load_project(path)
	await _frames(10)
	return ok

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("CLAIMSABSENT REFUSED: headless")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("CLAIMSABSENT WATCHDOG"); get_tree().quit(3))
	wd.start()

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge: Node = app.bridge
	bridge.generate({"seed": 9137, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(6)

	var path := OS.get_user_data_dir().path_join("_claimsabsent_probe.zip")
	var no_terr := OS.get_user_data_dir().path_join("_claimsabsent_probe_noterr.zip")
	var no_prov := OS.get_user_data_dir().path_join("_claimsabsent_probe_noprov.zip")
	var done := [false]
	app._write_project(path, func(): done[0] = true)
	for _i in 200:
		if done[0]:
			break
		await _frames(2)
	_check("setup: the project was written", FileAccess.file_exists(path), path)

	# 1. control
	_check("control: it opens", await _open(app, path))
	var wg: Object = bridge.world_gen
	var factions: Array = wg.get_factions()
	var f := int((factions[0] as Dictionary)["id"]) if not factions.is_empty() else 1
	_check("control: get_factions carries claimed_cells", _all_rows_have(factions, "claimed_cells"), str(factions.size()))
	var econ: Array = wg.civ_faction_economy()
	_check("control: the economy has territory_km2", _all_rows_have(econ, "territory_km2") and _no_row_has(econ, "absent"), str(econ.size()))
	var mil: Array = (wg.civ_military_summary() as Dictionary).get("factions", [])
	_check("control: military rows have overall and a manpower model",
		_all_rows_have(mil, "overall") and _no_row_has(mil, "absent")
		and not ((mil[0] as Dictionary).get("manpower", {}) as Dictionary).is_empty() if not mil.is_empty() else false, str(mil.size()))
	var rel: Array = wg.civ_faction_relations()
	_check("control: relations have values", _all_rows_have(rel, "value") and _no_row_has(rel, "absent"), str(rel.size()))
	var fits: Array = wg.civ_faction_terrain_fits()
	_check("control: terrain fits have a mix", _all_rows_have(fits, "mix") and _no_row_has(fits, "absent"), str(fits.size()))
	_check("control: a province overlay draws", wg.build_province_boundary_texture() != null)
	_check("control: regional population reports the claimed share", (wg.civ_regional_population() as Dictionary).has("claimed"))
	var t0: String = await _roster_text(app)
	_check("control: the roster prints a claimed-cell total", t0.contains(" claimed cells") and not t0.contains("claimed cells —"), "")
	_check("control: the roster gives no 'not known' reason", not t0.contains(FactionRosterWindow.NO_CLAIM_GRID))

	# 2. without the claim grid
	_check("premise: the claim grid entry was taken out", _strip(path, no_terr, "rasters/territory.") == 1)
	_check("no claims: it opens", await _open(app, no_terr))
	wg = bridge.world_gen
	factions = wg.get_factions()
	_check("no claims: get_factions still lists the factions", not factions.is_empty(), str(factions.size()))
	_check("no claims: get_factions omits claimed_cells", _no_row_has(factions, "claimed_cells"))
	_check("no claims: territory stats are unknown", (wg.civ_faction_territory_stats(f) as Dictionary).is_empty())
	var rp: Dictionary = wg.civ_regional_population()
	_check("no claims: regional population omits the claimed share", rp.has("total") and not rp.has("claimed"), str(rp))
	econ = wg.civ_faction_economy()
	_check("no claims: economy rows say no_claim_grid", _all_absent(econ), str(econ.slice(0, 1)))
	_check("no claims: economy omits every claimed-cell figure",
		_no_row_has(econ, "territory_km2") and _no_row_has(econ, "food_capacity") and _no_row_has(econ, "food_surplus")
		and _no_row_has(econ, "resources") and _no_row_has(econ, "exports"))
	_check("no claims: economy keeps pop", _all_rows_have(econ, "pop"))
	var msum: Dictionary = wg.civ_military_summary()
	mil = msum.get("factions", [])
	_check("no claims: military rows say no_claim_grid", _all_absent(mil), str(mil.size()))
	var empty_manpower := true
	for r in mil:
		if not ((r as Dictionary).get("manpower", {}) as Dictionary).is_empty():
			empty_manpower = false
	_check("no claims: military omits overall and the manpower model", _no_row_has(mil, "overall") and empty_manpower)
	_check("no claims: military keeps the walls-and-population axis", _all_rows_have(mil, "military"))
	rel = wg.civ_faction_relations()
	_check("no claims: relations say no_claim_grid", _all_absent(rel), str(rel.size()))
	_check("no claims: relations omit value, stance and border",
		_no_row_has(rel, "value") and _no_row_has(rel, "stance") and _no_row_has(rel, "border_cells") and _all_rows_have(rel, "culture_term"))
	fits = wg.civ_faction_terrain_fits()
	_check("no claims: terrain fits say no_claim_grid and carry no mix", _all_absent(fits) and _no_row_has(fits, "mix"))
	var towns: Array = wg.get_settlements()
	var tid := 0
	for s in towns:
		if int((s as Dictionary).get("faction", 0)) > 0:
			tid = int((s as Dictionary).get("tid", 0))
			break
	var g: Dictionary = wg.civ_settlement_garrison(tid, 0)
	_check("no claims: a claimed town's garrison reading says no_claim_grid", tid > 0
		and String(g.get("absent", "")) == "no_claim_grid", "tid %d: %s" % [tid, g])
	var t1: String = await _roster_text(app)
	_check("no claims: the roster dashes the claimed-cell total", t1.contains("claimed cells —"))
	_check("no claims: the roster gives the reason", t1.contains(FactionRosterWindow.NO_CLAIM_GRID))
	_check("no claims: the roster dashes Territory", t1.contains("Territory: —"))
	_check("no claims: the roster dashes the terrain composition", t1.contains("Composition: —"))
	_check("no claims: the roster dashes the manpower model", t1.contains("Standing army, field army and levy: —"))
	_check("no claims: the roster prints no '0 claimed cells'", not t1.contains("  0 claimed cells"))
	var cleared: Dictionary = wg.civ_clear_territory()
	_check("no claims: Clear territory reports the count as unknown", cleared.is_empty(), str(cleared))
	var after: Array = wg.get_factions()
	_check("no claims: after the clear the claims are known and empty",
		_all_rows_have(after, "claimed_cells") and int((after[0] as Dictionary)["claimed_cells"]) == 0, str(after.slice(0, 1)))

	# 3. without the province raster
	_check("premise: the province entry was taken out", _strip(path, no_prov, "rasters/provinces.") == 1)
	var direct: Object = WorldGen.new()
	var r: Dictionary = direct.project_open(no_prov)
	var pw: Array = []
	for w in r.get("warnings", PackedStringArray()):
		if String(w).contains("provinces.i32"):
			pw.append(String(w))
	_check("no provinces: project_open says the raster is absent", pw.size() == 1 and String(pw[0]).begins_with("rasters/provinces.i32: absent"), str(pw))
	_check("no provinces: no boundary overlay, and no panic", direct.build_province_boundary_texture() == null)
	_check("no provinces: the province list is kept", (direct.get_provinces() as Array).size() > 0)
	_check("no provinces: a stroke is staged", direct.civ_territory_paint_at(20.0, 20.0, f, 3.0, false) == true)
	_check("no provinces: the commit reports it", direct.civ_territory_commit() == true)
	_check("no provinces: the overlay is back after a territory commit", direct.build_province_boundary_texture() != null)

	DirAccess.remove_absolute(path)
	DirAccess.remove_absolute(no_terr)
	DirAccess.remove_absolute(no_prov)
	print("CLAIMSABSENT RESULT %s  checks=%d fails=%d" % ["PASS" if _fails == 0 else "FAIL", _checks, _fails])
	get_tree().quit(1 if _fails > 0 else 0)
