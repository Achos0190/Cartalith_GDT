extends Node
## Lane THUMBNAILS, the windowed half. `_thumbbench_probe.gd` asserts on the
## `Image` the renderer never sees; this one proves a tile reaches the screen,
## because a dummy rasteriser returns a null texture and every claim about a
## drawn pixel needs a real Vulkan context.
##
## The dialog is an `AcceptDialog`, i.e. its own `Window` and therefore its own
## `Viewport` -- the shell's main viewport does not contain it, so the grab is
## taken off the dialog rather than off `get_viewport()`.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 \
##       _thumbbench_shot.tscn

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	var dlg = app.open_project_dialog
	if dlg == null:
		print("SHOT  !! no dialog")
		get_tree().quit(1)
		return
	dlg.hide()
	await _frames(3)
	dlg.open()
	dlg._set_scope("all")
	await get_tree().create_timer(1.5).timeout
	await _frames(6)

	await RenderingServer.frame_post_draw
	var img: Image = dlg.get_texture().get_image()
	if img == null:
		print("SHOT  !! null framebuffer -- not a real renderer")
		get_tree().quit(1)
		return
	img.save_png("user://_thumbbench/gallery.png")
	print("SHOT  gallery %dx%d  theme=%s  tiles=%d"
		% [img.get_width(), img.get_height(), DccSettings.theme_mode(),
			dlg._grid.get_child_count() - 1])

	get_tree().quit(0)
