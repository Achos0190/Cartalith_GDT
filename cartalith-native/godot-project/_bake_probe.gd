extends Node
## TEMPORARY verification harness for the bake / atlas / finalize system.
## Not committed. Drives the whole thing through EngineBridge exactly as the
## shell does -- generate, bake, read a chunk back, finalize, prove the lock
## actually refuses, un-finalize, export/import, clear.
##
##   godot --headless --path . _bake_probe.tscn

var bridge: Node
var fails := 0

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

func _sea() -> float:
	var p: Dictionary = bridge.world_gen.get_params()
	return float(p.get("sea_level", -1.0))

func _ready() -> void:
	## A GDScript error aborts _ready() *without* reaching the quit() at the
	## bottom, and headless Godot then idles forever rather than failing. This
	## watchdog is what turns that into a visible failure with an exit code.
	get_tree().create_timer(180.0).timeout.connect(func() -> void:
		push_error("bake probe watchdog: _ready never finished")
		print("\n==== WATCHDOG: the probe did not finish ====\n")
		get_tree().quit(2))
	bridge = load("res://shell/engine_bridge.gd").new()
	add_child(bridge)
	await get_tree().process_frame

	print("\n== 1. generate ==")
	bridge.world_gen.generate_sized(20260824, 800.0, 384, 256)
	bridge.has_world = true
	_ok(bridge.world_gen.get_width() == 384, "world is 384 wide")
	## Captured rather than hardcoded: section 12 has to restore whatever the
	## default actually is to get back to the namespace section 5 baked.
	var sea0: float = _sea()
	print("  sea_level starts at %.4f" % sea0)
	_ok(abs(sea0 - 0.55) > 0.01, "the default differs from the probe's test value")

	print("\n== 2. atlas root + world key ==")
	_ok(bridge.atlas_ready(), "atlas root resolved (%s)" % DccSettings.storage_root("atlas_cache"))
	var wk: String = String(bridge.world_gen.atlas_world_key())
	print("  world key: %s" % wk)
	_ok(wk.length() > 0, "world key is non-empty")
	bridge.set_atlas_tile_size(256)
	_ok(bridge.atlas_tile_size() == 256, "tile size set to 256")
	bridge.atlas_clear()

	print("\n== 3. status before any bake ==")
	var st0: Dictionary = bridge.atlas_status()
	print("  %s" % st0)
	_ok(int(st0.get("chunks", -1)) == 0, "empty atlas reports 0 chunks")
	_ok(not bool(st0.get("finalized", true)), "a fresh world is not finalized")

	print("\n== 4. estimate ==")
	var est: Dictionary = bridge.bake_estimate(2)
	print("  %s" % est)
	_ok(int(est.get("tiles", 0)) == 21, "depth 2 is 21 tiles")
	_ok(int(est.get("bytes", 0)) > 0, "byte estimate is non-zero (%s)" % est.get("bytes_text", "?"))

	print("\n== 5. bake ==")
	var r: Dictionary = bridge.bake_all(2)
	print("  %s" % r)
	_ok(bool(r.get("ok", false)), "bake reported ok")
	_ok(int(r.get("baked", 0)) == 21, "21 chunks baked")
	_ok(int(r.get("failed", 1)) == 0, "nothing failed")

	print("\n== 6. status after the bake ==")
	var st: Dictionary = bridge.atlas_status()
	print("  %s" % String(st.get("text", "")))
	print("  bytes: %s   deepest: %d" % [st.get("bytes_text", "?"), int(st.get("deepest_level", -9))])
	_ok(int(st.get("chunks", 0)) == 21, "21 chunks on disk")
	_ok(int(st.get("deepest_level", -1)) == 2, "deepest level is 2")
	_ok(int(st.get("bytes", 0)) > 21 * 1024, "the atlas occupies real bytes")

	print("\n== 7. the atlas is READ back ==")
	var png: PackedByteArray = bridge.atlas_tile_png(2, 1, 1)
	print("  chunk (2,1,1) visual: %d bytes" % png.size())
	_ok(png.size() > 1000, "a baked chunk's PNG comes back")
	var img := Image.new()
	_ok(img.load_png_from_buffer(png) == OK, "and it is a real decodable PNG")
	print("  decoded %dx%d" % [img.get_width(), img.get_height()])
	_ok(bridge.world_gen.atlas_is_covered(4, 4, 4), "a descendant of a baked chunk is covered")
	_ok(bridge.atlas_tile_png(2, 99, 99).is_empty(), "an unbaked chunk comes back empty")

	print("\n== 8. finalize LOCKS ==")
	_ok(bridge.set_finalized(true), "finalize accepted (the atlas is non-empty)")
	_ok(bridge.is_finalized(), "is_finalized() agrees")
	var msg: String = String(bridge.finalize_check("generation"))
	print("  refusal: %s" % msg)
	_ok(msg != "" and msg.contains("Un-finalize"), "generation is refused, and names the escape hatch")
	_ok(bridge.finalize_check("height_edit") != "", "height edits are refused")
	_ok(bridge.finalize_check("presentation") == "", "presentation is NOT refused")

	print("\n== 9. the guards really bite ==")
	var before: String = String(bridge.world_gen.atlas_world_key())
	bridge.world_gen.generate_sized(999, 800.0, 384, 256)
	_ok(String(bridge.world_gen.atlas_world_key()) == before, "generate_sized did nothing while finalized")
	var res: Dictionary = bridge.world_gen.set_params({"sea_level": 0.55})
	print("  set_params -> %s" % res)
	_ok(PackedStringArray(res.get("rejected", [])).size() == 1, "set_params rejected the write")
	_ok(abs(_sea() - 0.55) > 0.01, "and sea_level did not move")

	print("\n== 10. un-finalize releases ==")
	_ok(bridge.set_finalized(false), "un-finalize always succeeds")
	var res2: Dictionary = bridge.world_gen.set_params({"sea_level": 0.55})
	_ok(PackedStringArray(res2.get("rejected", [])).size() == 0, "set_params works again")
	_ok(abs(_sea() - 0.55) < 0.01, "and sea_level moved")

	## NOTE: sea_level stays at 0.55 across section 11 on purpose. Restoring it
	## here (as this probe first did) puts the world back in its ORIGINAL atlas
	## namespace, so section 11's "the key moved" compares wk against itself and
	## fails against a perfectly correct engine. The restore belongs after the
	## namespace assertions, not before them.
	print("\n== 11. an empty atlas refuses to finalize ==")
	var wk2: String = String(bridge.world_gen.atlas_world_key())
	print("  world key after the parameter change: %s (was %s)" % [wk2, wk])
	_ok(wk2 != wk, "changing a parameter changed the atlas namespace")
	_ok(int(bridge.atlas_status().get("chunks", -1)) == 0, "the new namespace has no chunks")
	_ok(not bridge.set_finalized(true), "finalizing a world with no bake is refused")

	print("\n== 12. archive round trip ==")
	## Back to the namespace section 5 actually baked.
	bridge.world_gen.set_params({"sea_level": sea0})
	_ok(String(bridge.world_gen.atlas_world_key()) == wk, "restoring the parameter restores the namespace")
	var zip: PackedByteArray = bridge.atlas_export_zip(true)
	print("  archive: %d bytes" % zip.size())
	_ok(zip.size() > 10000, "the archive has real content")
	bridge.atlas_clear()
	_ok(int(bridge.atlas_status().get("chunks", -1)) == 0, "cleared")
	var imp: Dictionary = bridge.atlas_import_zip(zip)
	print("  import -> %s" % imp)
	_ok(bool(imp.get("ok", false)) and int(imp.get("chunks", 0)) == 21, "21 chunks re-imported")
	_ok(bool(imp.get("matches_current", false)), "and they belong to the current world")
	_ok(int(bridge.atlas_status().get("chunks", 0)) == 21, "status agrees after the import")

	print("\n== 13. clear ==")
	_ok(bridge.atlas_clear() == 21, "clear removed 21 chunks")
	_ok(int(bridge.atlas_status().get("chunks", -1)) == 0, "and the atlas is empty")
	_ok(not bridge.is_finalized(), "clearing un-finalized too")

	print("\n==== %s (%d failures) ====\n" % ["ALL PASS" if fails == 0 else "FAILURES", fails])
	get_tree().quit(1 if fails > 0 else 0)
