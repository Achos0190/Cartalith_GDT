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

	fails += _foreign_round_trip(path)

	print("save-tree probe: ", "PASS" if fails == 0 else "%d FAILURE(S)" % fails)
	quit(1 if fails > 0 else 0)

## An entry this build does not recognise must survive open → re-save.
##
## `cartalith-io` proves its own half in Rust (`a_foreign_entry_survives_an_open
## _and_a_re_save`), but the **bridge** half is `WorldGen::carried_foreign` —
## filled by `project_open`, cloned into the write by
## `project_save_with_documents`, cleared by `release_world`/`load_save` — and
## a verifier measured that half pinned by nothing: replacing
## `std::mem::take(&mut data.foreign)` with `Default::default()` left the whole
## Rust suite green. `WorldGen` is a cdylib `GodotClass` with a
## `Base<RefCounted>`, so no unit test can construct one; this is the only level
## the contract is reachable from.
##
## The payload is deliberately neither valid JSON nor valid UTF-8: a carrier
## that parsed or transcoded an entry would pass a text fixture and corrupt a
## real tile.
func _foreign_round_trip(src: String) -> int:
	var bad := PackedByteArray([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0xFF, 0x00, 0xC3])
	var name := "vendor/newer_build.bin"

	var zr := ZIPReader.new()
	if zr.open(src) != OK:
		print("  FAIL: foreign: source archive did not reopen")
		return 1
	var entries := {}
	for n in zr.get_files():
		entries[n] = zr.read_file(n)
	zr.close()

	var mixed := OS.get_user_data_dir().path_join("_savetree_probe_foreign.zip")
	var zp := ZIPPacker.new()
	if zp.open(mixed) != OK:
		print("  FAIL: foreign: could not write the mixed archive")
		return 1
	for n in entries:
		zp.start_file(n)
		zp.write_file(entries[n])
		zp.close_file()
	zp.start_file(name)
	zp.write_file(bad)
	zp.close_file()
	zp.close()

	var wg3: WorldGen = WorldGen.new()
	var o: Dictionary = wg3.project_open(mixed)
	if not bool(o.get("ok", false)):
		print("  FAIL: foreign: open -> ", o.get("error", "?"))
		return 1

	var out := OS.get_user_data_dir().path_join("_savetree_probe_foreign_out.zip")
	var w: Dictionary = wg3.project_save(out)
	if not bool(w.get("ok", false)):
		print("  FAIL: foreign: re-save -> ", w.get("error", "?"))
		return 1

	var zr2 := ZIPReader.new()
	if zr2.open(out) != OK:
		print("  FAIL: foreign: re-saved archive did not open")
		return 1
	var back := zr2.read_file(name) if (name in zr2.get_files()) else PackedByteArray()
	zr2.close()

	if back.size() == 0:
		print("  FAIL: foreign entry '%s' was DROPPED by the re-save" % name)
		return 1
	if back != bad:
		print("  FAIL: foreign entry survived but changed: %d bytes -> %d" % [bad.size(), back.size()])
		return 1
	print("  foreign entry survived open+re-save byte-for-byte (", back.size(), " bytes)")
	return 0
