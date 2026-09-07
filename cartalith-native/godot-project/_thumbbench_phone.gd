extends Node
## The phone half of `_thumbbench_shot.gd`. `app.gd` builds
## `PhoneProjectPicker` itself when the shell boots phone-sized, so this shoots
## the shell's own main viewport rather than parenting a picker by hand -- the
## first attempt did the latter and captured the desktop dialog instead.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 393x852 ##       --force-touch _thumbbench_phone.tscn

func _ready() -> void:
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(2.0).timeout
	for i in 8:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	var img: Image = get_viewport().get_texture().get_image()
	if img == null:
		print("PHONE  !! null framebuffer")
		get_tree().quit(1)
		return
	img.save_png("user://_thumbbench/phone.png")
	print("PHONE  %dx%d  is_phone=%s  picker=%s"
		% [img.get_width(), img.get_height(), app.is_phone(),
			app.phone_project_picker != null])
	get_tree().quit(0)
