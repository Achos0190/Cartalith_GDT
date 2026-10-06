extends Node

## FH-R1 display half, verified in the real engine (not a deliverable beyond
## this probe). Protects: a faction holding a CUSTOM religion shows its own
## name on (1) the map hover card's "Ruler's faith" line and (2) the GeoJSON
## territory layer's `religionName`, while every engine reader (`get_factions`'
## `religion`, the GeoJSON `religion` key) still reads the BASE. Absent keys:
## a built-in ruler carries no `ruler_religion_name` and no `religionName`.
##
## Run WINDOWED from a scratch project beside a freshly built DLL; the Rust
## side is reached only through the DLL, so state its mtime when reporting.

const OVERLAY := preload("res://map_overlay.gd")

var fails := 0

func _chk(ok: bool, what: String) -> void:
	if ok:
		print("  PASS  ", what)
	else:
		print("  FAIL  PROBE-FAIL ", what)
		fails += 1

func _ready() -> void:
	var wg: WorldGen = WorldGen.new()
	wg.generate_sized(24601, 640.0, 96, 64)
	var places: Array = wg.get_settlements()
	_chk(not places.is_empty(), "the world has settlements")
	var any_named := false
	for p in places:
		if (p as Dictionary).has("ruler_religion_name"):
			any_named = true
	_chk(not any_named, "no custom religion yet: no settlement carries ruler_religion_name")
	_chk(not wg.export_geojson().contains("religionName"),
		"no custom religion yet: the GeoJSON writes no religionName (byte-identical to before)")

	var added: Dictionary = wg.civ_add_religion("Church of the Dawn", "sun_cult", 200, 150, 40, "")
	_chk(bool(added.get("ok", false)), "civ_add_religion ok: %s" % str(added))
	var key := String(added.get("key", ""))
	wg.civ_set_faction_field(1, "religion", key)
	wg.civ_set_faction_field(2, "religion", "old_gods")

	var f1: Dictionary = {}
	for r in wg.get_factions():
		if int((r as Dictionary).get("id", 0)) == 1:
			f1 = r
	_chk(String(f1.get("religion", "")) == "sun_cult",
		"engine reader get_factions still reads the BASE (sun_cult)")
	_chk(String(f1.get("religion_name", "")) == "Church of the Dawn", "and carries the custom name")

	places = wg.get_settlements()
	var s1: Dictionary = {}
	var s2: Dictionary = {}
	for p in places:
		var d: Dictionary = p
		if int(d["faction"]) == 1 and s1.is_empty():
			s1 = d
		if int(d["faction"]) == 2 and s2.is_empty():
			s2 = d
	_chk(not s1.is_empty() and not s2.is_empty(), "the world has settlements of factions 1 and 2")
	_chk(String(s1.get("ruler_religion_name", "")) == "Church of the Dawn",
		"a faction-1 settlement carries ruler_religion_name")
	_chk(not s2.has("ruler_religion_name"),
		"a built-in ruler's settlement carries NO ruler_religion_name (key omitted)")

	var col := PackedStringArray()
	for r in wg.get_factions():
		var fd: Dictionary = r
		var id := int(fd.get("id", 0))
		while col.size() < id:
			col.append("")
		if id > 0:
			col[id - 1] = String(fd.get("religion", ""))
	var ov: Control = OVERLAY.new()
	ov.set_faith_divergence_visible(true)
	ov.set_faction_religions(col)
	# Force a real divergence on the real dictionary: the settlement's plurality
	# is a different faith from its ruler's base.
	var moved: Dictionary = s1.duplicate()
	moved["religion"] = "old_gods"
	moved["adherents"] = {"old_gods": 10}
	moved["population"] = 10
	_chk(ov._faith_diverged(moved), "the forced settlement carries a ring")
	var lines: Array = ov._faith_lines(moved)
	print("  hover lines (custom ruler): ", lines)
	_chk(String(lines[lines.size() - 1]).begins_with("Ruler's faith Church of the Dawn (as Sun Cult)"),
		"the hover card names the ruler's custom religion, base in brackets")
	var plain: Dictionary = moved.duplicate()
	plain.erase("ruler_religion_name")
	var plain_lines: Array = ov._faith_lines(plain)
	_chk(String(plain_lines[plain_lines.size() - 1]).begins_with("Ruler's faith Sun Cult"),
		"control: without the key the line is the base name alone (unchanged behaviour)")

	var gj := wg.export_geojson()
	_chk(gj.contains('"religion":"sun_cult","religionName":"Church of the Dawn"'),
		"GeoJSON territory: religion stays the base key, religionName is the custom name")
	_chk(gj.contains('"religion":"old_gods"') and not gj.contains('"religion":"old_gods","religionName"'),
		"GeoJSON territory: a built-in faction writes no religionName")

	print("")
	print("RESULT: ", "ALL PASS" if fails == 0 else "%d FAILURES" % fails)
	get_tree().quit(0)
