extends SceneTree
## Ruling BK header (2026-10-13, probe adopted into git; first run passed on the debug DLL):
## Protects: Optional LOD-tile export in saves (rulings 28/29): estimate equals the real tile bytes, default save writes no tiles, both archive shapes reopen clean.
## Run (Godot_v4.7.1-stable_win64_console.exe, from godot-project/): --headless --path . --script _lodsavepyramid_probe.gd (SceneTree script, no .tscn)
## Result: 'lod-save-pyramid probe: PASS'
## Sources: derived from this file's own assertions and a 2026-10-13 run; the feature symbols were grepped, not assumed.
##

## Owner rulings 28/29, `OUTSTANDING_WORK.md` §2.4: "LOD-tile export in the
## save file should be optional, default off, with its size shown at save
## time." This probe is the end-to-end check the Rust unit tests in
## `lod_worker.rs` cannot give -- `WorldGen` is a cdylib `GodotClass` and
## cannot be constructed there (`MISTAKES.md`), so the actual `#[func]`
## bindings (`lod_save_pyramid_estimate_bytes`, `project_save_with_documents`'s
## new `include_lod_tiles` argument) are only reachable from here.
##
## What it proves, in order:
##  1. `lod_save_pyramid_estimate_bytes()` answers a real, positive number for
##     a real generated world.
##  2. The DEFAULT save path (`include_lod_tiles` omitted, matching every
##     existing call site) writes NO `cartography/tiles/**` entries at all --
##     no regression to the pre-existing default.
##  3. `include_lod_tiles = true` writes real tile data whose total raw byte
##     count is EXACTLY the estimate from (1) -- not a placeholder, not an
##     approximation.
##  4. The archive with tiles reopens clean (`project_open`, `ok`, no
##     "dropped"/"stale" warning -- a fresh save's tiles must match their own
##     just-written world).
##  5. A pre-existing archive with no LOD entries at all (the shape of every
##     project saved before this feature existed) still opens clean too.

func _init() -> void:
	var fails := 0
	var wg: WorldGen = WorldGen.new()
	if not wg.has_method("lod_save_pyramid_estimate_bytes"):
		print("FAIL: lod_save_pyramid_estimate_bytes absent -- the binding is not built")
		quit(1)
		return
	if not wg.has_method("project_save_with_documents"):
		print("FAIL: project_save_with_documents absent")
		quit(1)
		return

	wg.generate_sized(24601, 400.0, 96, 64)

	# -- 1. the estimate ---------------------------------------------------
	var estimate: int = wg.lod_save_pyramid_estimate_bytes()
	print("  lod_save_pyramid_estimate_bytes() = ", estimate)
	if estimate <= 0:
		print("  FAIL: estimate must be positive for a real generated world")
		fails += 1

	# -- 2. default (no flag) writes nothing under cartography/tiles/ ------
	var path_off := OS.get_user_data_dir().path_join("_lodsavepyramid_probe_off.ctl")
	var r_off: Dictionary = wg.project_save_with_documents(path_off, {})
	if not bool(r_off.get("ok", false)):
		print("  FAIL: default save -> ", r_off.get("error", "?"))
		quit(1)
		return
	var zr_off := ZIPReader.new()
	if zr_off.open(path_off) != OK:
		print("  FAIL: default archive did not open")
		fails += 1
	else:
		var names_off := zr_off.get_files()
		zr_off.close()
		var tile_names_off: Array = Array(names_off).filter(func(n): return n.begins_with("cartography/tiles/"))
		if not tile_names_off.is_empty():
			print("  FAIL: the default (no include_lod_tiles) save wrote LOD entries: ", tile_names_off)
			fails += 1
		else:
			print("  default save: 0 cartography/tiles/ entries, as before this feature existed")

	# -- 3. include_lod_tiles=true writes real tile data -------------------
	var path_on := OS.get_user_data_dir().path_join("_lodsavepyramid_probe_on.ctl")
	var r_on: Dictionary = wg.project_save_with_documents(path_on, {}, PackedByteArray(), true)
	if not bool(r_on.get("ok", false)):
		print("  FAIL: include_lod_tiles=true save -> ", r_on.get("error", "?"))
		quit(1)
		return
	print("  include_lod_tiles=true: wrote ", r_on.get("bytes", 0), " bytes total")

	var zr_on := ZIPReader.new()
	if zr_on.open(path_on) != OK:
		print("  FAIL: on-archive did not open")
		quit(1)
		return
	var names_on: Array = Array(zr_on.get_files())
	if not ("cartography/tiles/index.json" in names_on):
		print("  FAIL: no cartography/tiles/index.json -- the producer did not run")
		fails += 1
	var tile_names_on: Array = names_on.filter(
		func(n): return n.begins_with("cartography/tiles/") and n.ends_with(".u8"))
	print("  ", tile_names_on.size(), " tile entries written")
	if tile_names_on.is_empty():
		print("  FAIL: no .u8 tile entries -- a real pyramid must carry real tiles")
		fails += 1
	var real_total := 0
	var any_nonzero := false
	for n in tile_names_on:
		var bytes: PackedByteArray = zr_on.read_file(n)
		real_total += bytes.size()
		for b in bytes:
			if b != 0:
				any_nonzero = true
				break
	zr_on.close()
	print("  real total tile bytes (raw, undeflated) = ", real_total)
	if real_total != estimate:
		print("  FAIL: real byte total (%d) does not match the save-time estimate (%d)"
			% [real_total, estimate])
		fails += 1
	if not any_nonzero:
		print("  FAIL: every tile byte was zero -- looks like a placeholder, not real colour data")
		fails += 1

	# -- 4. the fresh archive reopens clean ---------------------------------
	var wg_on := WorldGen.new()
	var o_on: Dictionary = wg_on.project_open(path_on)
	if not bool(o_on.get("ok", false)):
		print("  FAIL: reopening the tiled archive -> ", o_on.get("error", "?"))
		fails += 1
	else:
		var warnings_on: Array = o_on.get("warnings", [])
		var stale := warnings_on.filter(func(w): return "dropped" in String(w) or "stale" in String(w).to_lower())
		if not stale.is_empty():
			print("  FAIL: a just-written pyramid was reported stale on reopen: ", stale)
			fails += 1
		else:
			print("  tiled archive reopened clean, warnings=", warnings_on)

	# -- 5. a pre-existing (no-tiles) archive still opens clean -------------
	var wg_off := WorldGen.new()
	var o_off: Dictionary = wg_off.project_open(path_off)
	if not bool(o_off.get("ok", false)):
		print("  FAIL: reopening the untiled archive -> ", o_off.get("error", "?"))
		fails += 1
	else:
		var warnings_off: Array = o_off.get("warnings", [])
		if not warnings_off.is_empty():
			print("  FAIL: an archive with no LOD entries at all must not warn: ", warnings_off)
			fails += 1
		else:
			print("  untiled (pre-existing-shape) archive opens with no warnings, as before")

	print("lod-save-pyramid probe: ", "PASS" if fails == 0 else "%d FAILURE(S)" % fails)
	quit(1 if fails > 0 else 0)
