extends Node
## Development harness: boot the shell, optionally generate a small world, and
## capture one frame for visual review against `design/Cartalith DCC Shell.dc.html`.
## Run: godot --path . --resolution 1920x1080 _shot.tscn [-- --generate]

func _ready() -> void:
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(0.8).timeout

	if "--generate" in OS.get_cmdline_user_args():
		var bridge = app.bridge
		bridge.generate({
			"seed": 483920, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
			"archetype": "", "villages": true, "sea_level": 0.42,
		})
		while bridge.generating:
			await get_tree().create_timer(0.25).timeout
		await get_tree().create_timer(0.6).timeout

	var img := get_viewport().get_texture().get_image()
	img.save_png("user://shell_shot.png")
	print("saved ", ProjectSettings.globalize_path("user://shell_shot.png"))
	get_tree().quit()
