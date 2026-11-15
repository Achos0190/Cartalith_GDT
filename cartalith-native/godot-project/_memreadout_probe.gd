## Protects the top bar's memory readout (owner, 2026-11-15: it read 0 on
## Android): `EngineBridge.memory_bytes()` must be the process's resident bytes
## from the extension, larger than Godot's own allocator count, never 0.
extends SceneTree
func _initialize() -> void:
	var eb = load("res://shell/engine_bridge.gd").new()
	root.add_child(eb)
	await process_frame
	var r := int(eb.memory_bytes())
	var st := OS.get_static_memory_usage()
	var ok := r > st and r > (1 << 20)
	print("MEMREADOUT resident=%d static=%d %s" % [r, st, "PASS" if ok else "FAIL"])
	quit(0 if ok else 1)
