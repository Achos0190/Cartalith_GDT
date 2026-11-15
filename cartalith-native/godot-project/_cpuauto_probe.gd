## Protects `engine_bridge.gd::_restore_cpu_pref` pushing "auto" (0) at boot:
## with no user setting, the pool must be built from `auto_thread_count`
## (all allowed cores) before any render can build it at Rayon's default.
extends SceneTree
func _initialize() -> void:
	var eb = load("res://shell/engine_bridge.gd").new()
	root.add_child(eb)
	await process_frame
	var active := int(eb.cpu_thread_count_active()) if eb else -1
	var cores := OS.get_processor_count()
	print("CPUAUTO active=%d cores=%d %s" % [active, cores, "PASS" if active == cores else "FAIL"])
	quit(0 if active == cores else 1)
