extends Node
## Screenshots of the two handset route panes named in `OUTSTANDING_WORK.md`
## ("Two handset route panes still overflow -- and it is the BODY, not the
## footer"), for the owner to look at (Ruling AZ: screenshots go to the
## owner). The bounds assertion this row also asks for -- "for every child
## control, assert its global rect lies inside the window body and the
## viewport" -- lives in `_panemin_probe.gd`'s `_check_controls_in_bounds()`,
## which already opens every `DataManagerWindow` route and had the room to
## grow a per-control leg; this file exists only for the pixels that leg
## cannot produce (`_panemin_probe.gd`'s own header: no pixel is read by that
## file except in its one dedicated ink-control leg).
##
## **Windowed, not `--headless`.** `ImageTexture.update()` and
## `RenderingServer.frame_post_draw` both go vacuous under the dummy display
## driver (`MISTAKES.md`'s pixel-probe row) -- this file would produce two
## identical blank PNGs and report success.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _exportpanes_probe.tscn -- --vp 1080x2340 --force-touch

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _grab_and_save(path: String) -> void:
	await RenderingServer.frame_post_draw
	var t := get_viewport().get_texture()
	var img := t.get_image()
	img.save_png(path)
	print("[exportpanes] saved %s (%dx%d)" % [path, img.get_width(), img.get_height()])

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("[exportpanes] ABORT --headless: needs a real framebuffer. Run windowed.")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	var vp_i := args.find("--vp")
	var wh := "500x1080" if vp_i < 0 else String(args[vp_i + 1])
	var parts: PackedStringArray = wh.split("x")
	DisplayServer.window_set_size(Vector2i(int(parts[0]), int(parts[1])))
	await _frames(6)

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	var w: Node = app.data_manager_window
	app.open_data_manager()
	await _frames(8)

	w.open_route("export_maps")
	await _frames(10)
	await _grab_and_save("res://_exportpanes_maps.png")

	w.open_route("export_gis")
	await _frames(10)
	await _grab_and_save("res://_exportpanes_gis.png")

	print("[exportpanes] RESULT done")
	get_tree().quit(0)
