extends SceneTree

## `OUTSTANDING_WORK.md` §2.11, "Three small reopen/territory defects", the
## second: `project_bridge.rs::civ_from_project` read an archive without
## `rasters/territory.i32` as a grid of unowned cells. Now it is no territory
## (`CivData::territory` empty), `project_open` warns, and the Territory tool's
## base is "nothing claimed" so a stroke still commits.
##
## Built from the real pre-substrate archive
## (`crates/cartalith-godot/tests/fixtures/project_pre_substrate_2026-09-24.zip`)
## with that one entry taken out, through the real `WorldGen` funcs:
##   1. control: the whole archive has claims, a wash and no territory warning;
##   2. stripped: `warnings` names the absent raster; the Territory stats are
##      `{}` (unknown) rather than zeros; there is no wash;
##   3. a re-save and reopen keeps it absent -- the unknown is not written back
##      as zeros;
##   4. a stroke commits onto a whole grid: stats and wash come back.
##
## HEADLESS:
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . --script res://_terrmissing_probe.gd

const FIXTURE := "res://../crates/cartalith-godot/tests/fixtures/project_pre_substrate_2026-09-24.zip"
const ENTRY := "rasters/territory.shuffled.i32"

var _checks := 0
var _fails := 0

func _check(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	print("TERRMISSING %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _territory_warnings(r: Dictionary) -> Array:
	var out: Array = []
	for w in r.get("warnings", PackedStringArray()):
		if String(w).contains("territory.i32"):
			out.append(String(w))
	return out

func _strip(src: String, dst: String) -> int:
	var reader := ZIPReader.new()
	if reader.open(src) != OK:
		return -1
	var packer := ZIPPacker.new()
	if packer.open(dst) != OK:
		return -1
	var dropped := 0
	for name in reader.get_files():
		if name == ENTRY:
			dropped += 1
			continue
		packer.start_file(name)
		packer.write_file(reader.read_file(name))
		packer.close_file()
	packer.close()
	reader.close()
	return dropped

func _initialize() -> void:
	var src := ProjectSettings.globalize_path(FIXTURE)
	var stripped := ProjectSettings.globalize_path("user://_terrmissing_probe.zip")
	var resaved := ProjectSettings.globalize_path("user://_terrmissing_probe_resaved.zip")

	## 1. Control.
	var whole := WorldGen.new()
	var r0: Dictionary = whole.project_open(src)
	_check("control: the whole archive opens", bool(r0.get("ok", false)), str(r0.get("error", "")))
	var fs: Array = whole.get_factions()
	var f := int(fs[0]["id"]) if not fs.is_empty() else 1
	var s0: Dictionary = whole.civ_faction_territory_stats(f)
	_check("control: faction %d's claims are known" % f, int(s0.get("claimed_cells", -1)) > 0, str(s0))
	_check("control: there is a territory wash", whole.build_territory_texture() != null)
	_check("control: no territory warning", _territory_warnings(r0).is_empty(), str(r0.get("warnings", [])))

	## 2. The same archive without its claim grid.
	_check("premise: exactly the one entry was taken out", _strip(src, stripped) == 1)
	var wg := WorldGen.new()
	var r: Dictionary = wg.project_open(stripped)
	_check("stripped: it opens", bool(r.get("ok", false)), str(r.get("error", "")))
	_check("stripped: the civ layer still restores", Array(r.get("restored", [])).has("civ") and wg.get_settlements().size() > 0, str(r.get("restored", [])))
	var tw := _territory_warnings(r)
	_check("stripped: project_open says the claim grid is absent", tw.size() == 1 and String(tw[0]).begins_with("rasters/territory.i32: absent"), str(tw))
	var s1: Dictionary = wg.civ_faction_territory_stats(f)
	_check("stripped: the Territory stats are unknown, not zeros", s1.is_empty(), str(s1))
	_check("stripped: there is no territory wash", wg.build_territory_texture() == null)

	## 3. Re-saved untouched, it is still absent -- not all-unclaimed.
	var saved: Dictionary = wg.project_save(resaved)
	_check("re-save: saved", bool(saved.get("ok", false)), str(saved.get("error", "")))
	var again := WorldGen.new()
	var r2: Dictionary = again.project_open(resaved)
	_check("re-save: reopened, the claim grid is still reported absent", _territory_warnings(r2).size() == 1, str(r2.get("warnings", [])))

	## 4. A stroke commits onto a whole grid.
	_check("paint: a dab is staged", wg.civ_territory_paint_at(20.0, 20.0, f, 3.0, false) == true)
	_check("paint: the commit reports it", wg.civ_territory_commit() == true)
	var s2: Dictionary = wg.civ_faction_territory_stats(f)
	_check("paint: the stats are back and count the stroke", int(s2.get("claimed_cells", -1)) > 0, str(s2))
	_check("paint: the wash is back", wg.build_territory_texture() != null)

	DirAccess.remove_absolute(stripped)
	DirAccess.remove_absolute(resaved)
	print("RESULT %s  checks=%d fails=%d" % ["PASS" if _fails == 0 else "FAIL", _checks, _fails])
	quit(1 if _fails > 0 else 0)
