extends SceneTree

func _init() -> void:
	var ok := true

	var wg := WorldGen.new()
	wg.generate(24601, 400.0, 8)
	var name1: String = wg.get_world_name()
	print("generated name: '", name1, "' seed: ", wg.get_seed())
	if name1 == "":
		print("FAIL: freshly generated world has no name")
		ok = false

	# Regenerating with the SAME seed must produce the SAME name
	# (determinism -- world_name is a pure function of the seed).
	var wg2 := WorldGen.new()
	wg2.generate(24601, 400.0, 8)
	var name2: String = wg2.get_world_name()
	if name2 != name1:
		print("FAIL: same seed produced a different name: '", name1, "' vs '", name2, "'")
		ok = false
	else:
		print("OK: determinism holds ('", name1, "' == '", name2, "')")

	# A different seed should (almost certainly) produce a different name.
	var wg3 := WorldGen.new()
	wg3.generate(483920, 400.0, 8)
	var name3: String = wg3.get_world_name()
	print("second world name: '", name3, "' seed: ", wg3.get_seed())
	if name3 == name1:
		print("WARN: different seeds produced the same name (statistically rare, not necessarily a bug)")

	# --- Tree-format (.ctl) round trip ---------------------------------
	# The Rust side opens paths with `std::fs`/`ZIPReader::open` directly, so
	# it needs a real OS path -- `user://` is a Godot-only URI the engine
	# resolves, not something the extension's own file I/O understands.
	var tmp_dir := ProjectSettings.globalize_path("user://")
	var tree_path := tmp_dir + "worldname_probe_tree.ctl"
	var save_ok: bool = wg.project_save(tree_path).get("ok", false)
	print("tree save ok: ", save_ok)
	if not save_ok:
		print("FAIL: tree save reported not ok")
		ok = false

	var wg_loaded := WorldGen.new()
	var load_ok: bool = wg_loaded.load_save(tree_path)
	print("tree load ok: ", load_ok, " loaded name: '", wg_loaded.get_world_name(),
		"' loaded seed: ", wg_loaded.get_seed())
	if not load_ok:
		print("FAIL: tree load reported not ok")
		ok = false
	elif wg_loaded.get_world_name() != name1:
		print("FAIL: tree round trip did not carry the name: expected '", name1,
			"' got '", wg_loaded.get_world_name(), "'")
		ok = false
	elif wg_loaded.get_seed() != 24601:
		print("FAIL: tree round trip did not carry the seed")
		ok = false
	else:
		print("OK: tree round trip carried the name and seed")

	# --- Strip "world.name" from the archive's project.json to simulate an
	# --- archive saved BEFORE this field existed, and confirm it still opens
	# --- with no name (never invented) rather than refusing. ---
	var stripped_path := tmp_dir + "worldname_probe_stripped.ctl"
	var strip_ok := _strip_world_name(tree_path, stripped_path)
	if not strip_ok:
		print("FAIL: could not build the pre-name fixture archive")
		ok = false
	else:
		var wg_old := WorldGen.new()
		var old_load_ok: bool = wg_old.load_save(stripped_path)
		print("pre-name archive load ok: ", old_load_ok, " name: '", wg_old.get_world_name(),
			"' seed: ", wg_old.get_seed())
		if not old_load_ok:
			print("FAIL: an archive with no world.name must still open")
			ok = false
		elif wg_old.get_world_name() != "":
			print("FAIL: a pre-name archive must not have an invented name, got '",
				wg_old.get_world_name(), "'")
			ok = false
		elif wg_old.get_seed() != 24601:
			print("FAIL: pre-name archive lost its seed")
			ok = false
		else:
			print("OK: pre-name archive opened with no name and its real seed")

	# --- Flat legacy save_project() round trip --------------------------
	var flat_path := tmp_dir + "worldname_probe_flat.zip"
	var flat_save_ok: bool = wg.save_project(flat_path)
	print("flat save ok: ", flat_save_ok)
	var wg_flat := WorldGen.new()
	var flat_load_ok: bool = wg_flat.load_save(flat_path)
	print("flat load ok: ", flat_load_ok, " name: '", wg_flat.get_world_name(), "'")
	if not flat_load_ok or wg_flat.get_world_name() != name1:
		print("FAIL: flat round trip did not carry the name")
		ok = false
	else:
		print("OK: flat round trip carried the name")

	print("PROBE_RESULT: ", "PASS" if ok else "FAIL")
	quit(0 if ok else 1)

## Rewrites a real tree-format archive's project.json with its "world.name"
## member removed, everything else byte-identical -- the shape a save written
## before this field existed genuinely has. Uses `ZIPReader`/`ZIPPacker`
## rather than hand-building a fixture, so every other member (grid, seed,
## rasters, documents) is the real writer's own output.
func _strip_world_name(src_path: String, dst_path: String) -> bool:
	var reader := ZIPReader.new()
	if reader.open(src_path) != OK:
		return false
	var packer := ZIPPacker.new()
	if packer.open(dst_path) != OK:
		return false
	for entry in reader.get_files():
		var data: PackedByteArray = reader.read_file(entry)
		if entry == "project.json":
			var parsed = JSON.parse_string(data.get_string_from_utf8())
			if parsed is Dictionary and (parsed as Dictionary).has("world"):
				var world = (parsed as Dictionary)["world"]
				if world is Dictionary and (world as Dictionary).has("name"):
					(world as Dictionary).erase("name")
					print("stripped world.name from the fixture project.json")
			data = JSON.stringify(parsed).to_utf8_buffer()
		packer.start_file(entry)
		packer.write_file(data)
		packer.close_file()
	packer.close()
	reader.close()
	return true
