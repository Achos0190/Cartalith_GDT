extends Node
## The phone half of `_thumbbench_shot.gd`. `app.gd` builds
## `PhoneProjectPicker` itself when the shell boots phone-sized, so this shoots
## the shell's own main viewport rather than parenting a picker by hand -- the
## first attempt did the latter and captured the desktop dialog instead.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 393x852 _thumbbench_phone.tscn -- --force-touch
## Result: a capture tool that asserts nothing. Command per the header: `Godot_v4.7.1-stable_win64_console.exe --path .
## --resolution 393x852 --force-touch _thumbbench_phone.tscn` (`--force-touch` goes after `--`, because
## `dcc_shell.gd` reads it from `OS.get_cmdline_user_args()`; the line used to run it together with a stray `##` and put it
## before the scene name, where the shell never sees it -- corrected in the Ruling BK probe pass). Windowed, since it
## grabs the main viewport. Prints `PHONE  <w>x<h>  is_phone=<b>  picker=<b>` and saves `user://_thumbbench/phone.png`;
## `PHONE  !! null framebuffer` exits 1, otherwise 0.

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
