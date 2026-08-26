extends SceneTree

## Does the shell's File ▸ Open actually read what File ▸ Save writes?
##
## Save goes through `project_save` (the tree). Open goes through
## `EngineBridge.load_save` -> `WorldGen::load_save` -> `cartalith_io::load_save`,
## which is the FLAT HTML-format reader. This measures what that costs.

func _init() -> void:
	var wg: WorldGen = WorldGen.new()
	wg.set_params({"tect.plates": 9})
	wg.generate_sized(24601, 640.0, 96, 64)
	wg.recompute_civilisation()
	var before: int = (wg.get_settlements() as Array).size()
	print("  settlements before save: ", before)

	var path := OS.get_user_data_dir().path_join("_openpath_probe.zip")
	var w: Dictionary = wg.project_save(path)
	print("  project_save: ok=%s entries=%s" % [w.get("ok"), w.get("entries")])

	# The path the SHELL takes on File > Open.
	var a: WorldGen = WorldGen.new()
	var flat_ok: bool = a.load_save(path)
	var flat_settlements: int = (a.get_settlements() as Array).size() if flat_ok else -1
	print("  load_save (what the shell calls): ok=%s settlements=%d" % [flat_ok, flat_settlements])

	# The path the engine provides and nothing calls.
	var b: WorldGen = WorldGen.new()
	var o: Dictionary = b.project_open(path)
	var tree_settlements: int = (b.get_settlements() as Array).size()
	print("  project_open (never called by the shell): ok=%s restored=%s settlements=%d"
		% [o.get("ok"), o.get("restored", []), tree_settlements])

	var fails := 0
	if not flat_ok:
		print("  FINDING: the shell's Open cannot read its own Save at all")
		fails += 1
	elif flat_settlements < before:
		print("  FINDING: the shell's Open loses the civ layer (%d -> %d)" % [before, flat_settlements])
		fails += 1
	if tree_settlements != before:
		print("  FAIL: project_open did not restore the civ layer either (%d -> %d)" % [before, tree_settlements])
		fails += 1

	# Second question: can `project_open` also read the OLD flat format? The
	# owner's rule is read both formats and write only the new one, so if it
	# can, the shell's Open becomes one call rather than a format sniff.
	var flat_path := OS.get_user_data_dir().path_join("_openpath_flat.zip")
	var fw: bool = wg.save_project(flat_path)
	print("  save_project (flat export): ok=", fw)
	var c: WorldGen = WorldGen.new()
	var fo: Dictionary = c.project_open(flat_path)
	print("  project_open on a FLAT archive: ok=%s layout=%s width=%d"
		% [fo.get("ok"), fo.get("layout", "?"), c.get_width()])
	if not bool(fo.get("ok", false)) or c.get_width() <= 0:
		print("  NOTE: project_open cannot read the flat format -- Open needs both readers")
	else:
		print("  project_open reads both layouts -- Open can be one call")

	print("open-path probe: ", "no gap" if fails == 0 else "%d FINDING(S)" % fails)
	quit(0)
