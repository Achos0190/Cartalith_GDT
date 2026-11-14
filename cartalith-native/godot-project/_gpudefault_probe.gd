extends SceneTree
## Protects Ruling EY: use_gpu boot default is off on touch-class, on on desktop, and a stored explicit choice wins.
## godot --headless --path . --script res://_gpudefault_probe.gd [-- --force-touch] [-- --store=0|1]
func _init() -> void:
	var args := OS.get_cmdline_user_args()
	for a in args:
		if a.begins_with("--store="):
			DccSettings.set_gpu_enabled(a.ends_with("1"))
	var b = load("res://shell/engine_bridge.gd").new()
	root.add_child(b)
	await process_frame
	print("PROBE params_available=", b._params_available, " use_gpu=", b.param_get("use_gpu"), " stored=", DccSettings.gpu_enabled())
	quit(0)
