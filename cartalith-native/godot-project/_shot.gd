extends Node
## Temporary: capture one frame of the shell for visual review, then quit.
func _ready() -> void:
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	var img := get_viewport().get_texture().get_image()
	img.save_png("user://shell_shot.png")
	print("saved ", ProjectSettings.globalize_path("user://shell_shot.png"))
	get_tree().quit()
