extends SceneTree

## The save/reopen round trip through the REAL `EngineBridge`, not through
## `WorldGen` directly — because the bug this probe exists for was in the
## shell, not the engine. `project_save` wrote the tree and `EngineBridge.
## load_save` read it with the flat HTML reader, so File ▸ Save stored 8
## settlements and File ▸ Open produced 0, with `ok == true` and no warning.
##
## Also covers the caller-owned document channel in the same pass, since it
## rides the same two calls.

func _init() -> void:
	var fails := 0
	var bridge: EngineBridge = EngineBridge.new()
	get_root().add_child(bridge)

	bridge.world_gen.set_params({"tect.plates": 9})
	bridge.world_gen.generate_sized(24601, 640.0, 96, 64)
	bridge.world_gen.recompute_civilisation()
	bridge.has_world = true
	bridge.save_api = bridge.world_gen.has_method("project_save")

	var before: int = (bridge.world_gen.get_settlements() as Array).size()
	print("  settlements before save: ", before)
	if before == 0:
		print("  FAIL: fixture has no settlements to lose")
		quit(1)
		return

	# A caller-owned document with an integer above 2^31, to prove the shell's
	# half of §14.1 as well as the engine's.
	var doc := JSON.stringify({"next_id": 4294967297, "journeys": [
		{"name": "Ærik's road — 城壁", "route": 0, "trim": [0.25, 0.75]}]})
	var path := OS.get_user_data_dir().path_join("_savereopen_probe.zip")

	if not bridge.save_project(path, {"entities/journeys.json": doc}):
		print("  FAIL: save_project refused")
		quit(1)
		return
	print("  saved with 1 caller-owned document")

	var b2: EngineBridge = EngineBridge.new()
	get_root().add_child(b2)
	if not b2.load_save(path):
		print("  FAIL: load_save refused")
		fails += 1
	var after: int = (b2.world_gen.get_settlements() as Array).size()
	print("  settlements after reopen: ", after)
	if after != before:
		print("  FAIL: the civ layer did not survive File Save -> File Open (%d -> %d)" % [before, after])
		fails += 1

	var got := String(b2.last_documents.get("entities/journeys.json", ""))
	print("  document returned: %d bytes" % got.length())
	if got != doc:
		print("  FAIL: the document did not come back byte-identical")
		print("    wrote: ", doc)
		print("    read:  ", got)
		fails += 1
	else:
		print("  document is byte-identical, big integer and all")

	# And the old format still opens, which is the owner's read-both rule.
	var flat := OS.get_user_data_dir().path_join("_savereopen_flat.zip")
	bridge.world_gen.save_project(flat)
	var b3: EngineBridge = EngineBridge.new()
	get_root().add_child(b3)
	var flat_ok: bool = b3.load_save(flat)
	print("  flat (old HTML) archive still opens: %s, %d cells wide" % [flat_ok, b3.world_gen.get_width()])
	if not flat_ok or b3.world_gen.get_width() <= 0:
		print("  FAIL: the old format stopped opening")
		fails += 1

	print("save/reopen probe: ", "PASS" if fails == 0 else "%d FAILURE(S)" % fails)
	quit(1 if fails > 0 else 0)
