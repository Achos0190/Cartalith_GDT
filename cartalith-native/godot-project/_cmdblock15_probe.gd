extends Node
## **Re-opening "15 menu commands unavailable, each carrying a true reason"**
## (`OUTSTANDING_WORK.md`, owned by `STATUS.md`, re-cut 2026-09-03).
##
## Eleven of the fifteen `_todo()` rows in `shell/menus.gd` are gated on
## `_engine_has(name)` / `_bridge.has_method(name)` -- i.e. on whether the
## CURRENTLY LOADED `cartalith_godot` extension exports that gdext `#[func]`,
## not on whether the shell knows how to draw the row. This probe asks the
## real, already-loaded `WorldGen` the same question `menus.gd` asks at boot,
## so "is the reason still true today" is answered from the running artifact
## rather than from reading source (`MISTAKES.md`: re-open a backlog row at
## its cited symbol; a Rust source grep alone cannot say what a possibly-
## stale `.dll` actually registered in ClassDB).
##
## Two rows are gated on runtime DATA rather than a method
## (`_layout_names.is_empty()` for "Forget layout...", `_docs_dir() == ""` for
## "Documentation") and are reported separately below. Two more
## ("Anti-aliasing - anisotropy", "3D viewport defaults") are architecture-
## level -- no 3D viewport exists at all (`DECISIONS.md` section 4 defers it
## to Phase 3) -- and are not worth a method check.
##
## Loads NO scene beyond a bare `EngineBridge`+`WorldGen` pair -- `app.tscn`'s
## full boot is not needed to ask `has_method()`, and skipping it avoids the
## concurrent CIVIL-rail workflow's in-flight files entirely.
##
##   godot --headless _cmdblock15_probe.tscn

func _log(s: String) -> void:
	print("[cmdblock15] %s" % s)

func _ready() -> void:
	var bridge: EngineBridge = EngineBridge.new()
	add_child(bridge)
	await get_tree().process_frame

	var wg: WorldGen = bridge.world_gen
	_log("WorldGen present: %s" % (wg != null))

	var methods := [
		"redo_available",              # Redo (menus.gd:902)
		"landmark_kinds",              # No landmark types (menus.gd:1895)
		"gpu_enumerate_devices",       # Devices / VRAM budget / Fallback (menus.gd:2951/2962/2963)
		"gpu_set_multi_mode",          # Multi-GPU mode, second half of gpu_api (menus.gd:2952)
		"set_cpu_thread_count",        # CPU worker threads (menus.gd:2956)
		"cpu_logical_core_count",      # CPU worker threads, other half (menus.gd:2956)
		"gpu_clear_readback_failures", # Try the GPU again (menus.gd:3197)
		"gpu_readback_failed",         # Try the GPU again, other half (menus.gd:3197)
		"atlas_export_zip",            # Export atlas... (menus.gd:4232)
		"atlas_import_zip",            # Import atlas... (menus.gd:4239)
		"atlas_evict_to",              # Size cap - GB (menus.gd:4272)
	]
	var have := 0
	for m in methods:
		var ok: bool = wg != null and wg.has_method(m)
		if ok:
			have += 1
		_log("  has_method(%-28s) = %s" % [m, ok])

	_log("gpu_api (cached, = enumerate_devices AND set_multi_mode): %s" % bridge.gpu_api)

	if wg != null and wg.has_method("landmark_kinds"):
		var kinds = wg.landmark_kinds()
		_log("landmark_kinds().is_empty() = %s (size %d)" % [
			(kinds as Array).is_empty(), (kinds as Array).size()])

	## The two non-DLL gates: "Forget layout..." (menus.gd:4915) and
	## "Documentation" (menus.gd:5403). Both are static, so no boot is needed.
	_log("DccMenus._docs_dir() = '%s' (Documentation _todo fires only when empty)"
		% DccMenus._docs_dir())
	var names: Array = DccSettings.layout_names()
	_log("DccSettings.layout_names() = %s (Forget layout _todo fires only when empty)"
		% [names])

	_log("RESULT %d of %d checked methods are present on the loaded WorldGen" %
		[have, methods.size()])
	get_tree().quit(0)
