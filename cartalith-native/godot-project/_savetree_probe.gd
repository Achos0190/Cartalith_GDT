extends SceneTree

## Proves File ▸ Save now writes the documented tree and that the civ layer
## survives a reopen. Before 2026-08-25 `load_save` cleared civ/sculpt/icons/
## paint/labels/infra on every load, so a saved settlement could not come back.

func _init() -> void:
	var fails := 0
	var wg: WorldGen = WorldGen.new()
	if not wg.has_method("project_save"):
		print("FAIL: project_save absent -- the shell probe would disable Save")
		quit(1)
		return

	wg.set_params({"tect.plates": 9, "climate.lat_n": 62.0})
	wg.generate_sized(24601, 640.0, 96, 64)
	if wg.has_method("compute_civilisation"):
		wg.compute_civilisation()
	var before: Array = wg.get_settlements()
	print("  settlements before save: ", before.size())
	if before.is_empty():
		print("  FAIL: no settlements to test with")
		fails += 1

	var path := OS.get_user_data_dir().path_join("_savetree_probe.zip")
	var r: Dictionary = wg.project_save(path)
	if not bool(r.get("ok", false)):
		print("  FAIL: save -> ", r.get("error", "?"))
		quit(1)
		return
	print("  wrote ", r.get("bytes", 0), " bytes, ", r.get("entries", 0), " entries")

	## The layout test the spec defines: project.json present means the tree.
	var zr := ZIPReader.new()
	if zr.open(path) != OK:
		print("  FAIL: archive did not open")
		quit(1)
		return
	var names := zr.get_files()
	zr.close()
	if not ("project.json" in names):
		print("  FAIL: no project.json -- this is the FLAT export, not the tree")
		fails += 1
	var dirs := {}
	for n in names:
		if "/" in n:
			dirs[n.get_slice("/", 0)] = true
	print("  entries=", names.size(), " top-level folders: ", dirs.keys())

	## The regression that matters: reopen and see whether civ came back.
	var wg2: WorldGen = WorldGen.new()
	var o: Dictionary = wg2.project_open(path)
	if not bool(o.get("ok", false)):
		print("  FAIL: open -> ", o.get("error", "?"))
		quit(1)
		return
	print("  layout=", o.get("layout", "?"), " restored=", o.get("restored", []))
	var after: Array = wg2.get_settlements()
	print("  settlements after reopen: ", after.size())
	if after.size() != before.size():
		print("  FAIL: civ layer did not survive (%d -> %d)" % [before.size(), after.size()])
		fails += 1
	elif not before.is_empty():
		var a: Dictionary = before[0]
		var b: Dictionary = after[0]
		for k in ["name", "x", "y"]:
			if a.get(k) != b.get(k):
				print("  FAIL: settlement 0 %s changed: %s -> %s" % [k, a.get(k), b.get(k)])
				fails += 1
		if fails == 0:
			print("  settlement 0 identical: ", a.get("name"), " @ (", a.get("x"), ",", a.get("y"), ")")

	print("save-tree probe: ", "PASS" if fails == 0 else "%d FAILURE(S)" % fails)
	quit(1 if fails > 0 else 0)
