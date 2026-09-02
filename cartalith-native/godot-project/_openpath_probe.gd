extends SceneTree

## Does the shell's File ▸ Open actually read what File ▸ Save writes?
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . --script _openpath_probe.gd
##
## Save goes through `project_save`, which writes the TREE layout. When this
## probe was first written (2026-08-26) Open went through
## `EngineBridge.load_save` -> `WorldGen::load_save` -> `cartalith_io::load_save`,
## the FLAT HTML-format reader, and silently returned a world with **0** of the
## 8 settlements it had saved. That measurement is what
## `engine_bridge.gd::load_save()`'s own header cites, and the fix landed there:
## `EngineBridge.load_save()` now calls `project_open` and keeps
## `world_gen.load_save` only as the
## fallback for a binary too old to have it.
##
## **So the header this probe used to carry is no longer true, and the sentence
## it printed -- "the shell's Open loses the civ layer" -- named a path the
## shell had stopped taking.** The probe now asserts the contract that replaced
## it, and keeps the raw-binding measurement as the standing reason the fallback
## is only a fallback:
##
##   1. `EngineBridge.load_save()` -- what File ▸ Open actually calls -- round
##      trips the civilisation layer. This is the assertion; it fails the run.
##   2. `WorldGen.project_open()` restores it too, and reports the layout.
##   3. `WorldGen.load_save()` (the raw flat reader, reached only on an old
##      binary) still drops it. Reported as an OBSERVATION, not a failure:
##      losing the civ layer is what the flat format *is*, and the shell no
##      longer routes through it. It becomes a failure only if the bridge
##      stops preferring `project_open` -- which check 1 is what catches.

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

	var fails := 0

	# -- 1. The path the SHELL takes on File > Open, through the real bridge ---
	## `EngineBridge`, not `WorldGen`, because the routing decision under test
	## lives in the bridge (`shell/engine_bridge.gd::load_save`), not in the
	## engine. Calling `WorldGen.load_save` here -- which is what this probe
	## used to do -- measures the fallback and calls it "the shell's Open".
	var bridge: EngineBridge = EngineBridge.new()
	get_root().add_child(bridge)
	var shell_ok: bool = bridge.load_save(path)
	var shell_settlements: int = (bridge.world_gen.get_settlements() as Array).size() if shell_ok else -1
	print("  EngineBridge.load_save (what File ▸ Open calls): ok=%s settlements=%d layout=%s"
		% [shell_ok, shell_settlements, bridge.last_open_layout])
	if not shell_ok:
		print("  FAIL: the shell's Open cannot read its own Save at all")
		fails += 1
	elif shell_settlements != before:
		print("  FAIL: the shell's Open loses the civ layer (%d -> %d) -- the bridge "
			% [before, shell_settlements]
			+ "has stopped preferring project_open, or project_open has regressed")
		fails += 1
	## The layout it reports is the second half of the same contract: a tree
	## archive read as "flat" is the original defect wearing a passing number.
	if shell_ok and bridge.last_open_layout != "tree":
		print("  FAIL: project_save wrote the tree layout; Open read it as '%s'"
			% bridge.last_open_layout)
		fails += 1

	# -- 2. The engine call underneath it -------------------------------------
	var b: WorldGen = WorldGen.new()
	var o: Dictionary = b.project_open(path)
	var tree_settlements: int = (b.get_settlements() as Array).size()
	print("  project_open (what the bridge now calls): ok=%s restored=%s settlements=%d"
		% [o.get("ok"), o.get("restored", []), tree_settlements])
	if tree_settlements != before:
		print("  FAIL: project_open did not restore the civ layer (%d -> %d)" % [before, tree_settlements])
		fails += 1

	# -- 3. The raw flat reader, as an observation -----------------------------
	## Not a failure. This is the pre-2026-08-26 path, reachable now only on a
	## binary too old to export `project_open`. It is measured every run so the
	## comment in `engine_bridge.gd::load_save()` keeps its number.
	var a: WorldGen = WorldGen.new()
	var flat_ok: bool = a.load_save(path)
	var flat_settlements: int = (a.get_settlements() as Array).size() if flat_ok else -1
	print("  WorldGen.load_save (the flat fallback, not the shell path): ok=%s settlements=%d"
		% [flat_ok, flat_settlements])
	if flat_ok and flat_settlements < before:
		print("  OBSERVED (not a failure): the flat reader still drops the civ layer "
			+ "(%d -> %d) -- this is why the bridge prefers project_open"
			% [before, flat_settlements])

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

	## Was `quit(0)`, unconditionally -- so the run that first measured the
	## civ-layer loss reported it in prose and exited green. The count is the
	## verdict now.
	# -- 4. The staleness fingerprint ------------------------------------------
	## `EngineBridge._has()` (`shell/engine_bridge.gd`) records every binding
	## the shell asked for and this build does not export; `missing_bindings()`
	## returns them, and nothing in this probe suite read it until
	## 2026-09-01. It matters most here: every result above is a claim about
	## which reader `load_save()` chose, and `load_save()` chooses by asking
	## `_has("project_open")`. On a `.dll` that predates `project_open` the
	## bridge silently falls back to the flat reader, check 1 fails with a
	## civ-layer message, and the real cause -- a stale library -- is named
	## nowhere. Read last, after `load_save()` has run its guards.
	var missing := bridge.missing_bindings()
	if not missing.is_empty():
		print("  FAIL stale extension: the shell asked for %d binding(s) this build "
			% missing.size()
			+ "does not export (%s). " % ", ".join(missing)
			+ "Rebuild the crates before reading anything above.")
		fails += 1

	## Freed before the exit, not left parented. An `EngineBridge` still in the
	## tree holds the process open past `quit()` -- measured here: with it
	## attached every assertion printed and the run then hung until the
	## caller's timeout, which is a green result nobody ever sees.
	get_root().remove_child(bridge)
	bridge.free()

	print("open-path probe: ", "no gap" if fails == 0 else "%d FAILURE(S)" % fails)
	quit(1 if fails > 0 else 0)
