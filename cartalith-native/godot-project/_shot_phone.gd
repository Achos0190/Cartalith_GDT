extends Node
## Phone-chrome capture harness, copied from `_shot.gd` rather than editing it
## in place (that file is git-tracked and other work may be mid-flight on it).
## The only real difference: `--force-touch` (read by `DccShell._ready()`)
## makes the phone/tablet composition reachable at all in this headless dev
## environment, which has no real touchscreen for `DisplayServer
## .is_touchscreen_available()` to find, and the output filename is distinct
## so a portrait and a landscape capture (or a desktop capture from `_shot.gd`
## itself) never clobber each other.
##
## Run:
##   godot --path . --resolution 393x852 _shot_phone.tscn -- --force-touch
##   godot --path . --resolution 852x393 _shot_phone.tscn -- --force-touch
## Add `--generate` too to capture a generated world rather than the empty
## state. One of `--drawer` / `--picker` / `--overflow` / `--leftsheet` /
## `--rightsheet` force-opens that phone overlay before the capture, since
## none of them are reachable by a script driving no real input.

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

	## `app.gd` opens the welcome dialog whenever no world exists, so without
	## this every capture is of that dialog rather than of the shell behind it.
	if "--nowelcome" in OS.get_cmdline_user_args():
		app.open_project_dialog.hide()
		await get_tree().process_frame

	## Simulates a device rotation the only way this environment can: resize the
	## real window to the transposed resolution and let `root.size_changed` fire
	## exactly as Android's own rotation makes it fire. This is what verifies
	## `_on_window_resized()` -> `_apply_phone_orientation()` actually re-lays the
	## shell, as opposed to `--resolution` merely *booting* into an orientation.
	if "--rotate" in OS.get_cmdline_user_args():
		var w := get_window()
		w.size = Vector2i(w.size.y, w.size.x)
		await get_tree().process_frame
		await get_tree().process_frame
		await get_tree().create_timer(0.4).timeout

	if "--drawer" in OS.get_cmdline_user_args():
		app._set_drawer_open(true)
	if "--picker" in OS.get_cmdline_user_args():
		app._set_panel_picker_open(true)
	if "--overflow" in OS.get_cmdline_user_args():
		app._set_overflow_open(true)
	if "--leftsheet" in OS.get_cmdline_user_args():
		app._set_sheet_open("left", true)
	if "--rightsheet" in OS.get_cmdline_user_args():
		app._set_sheet_open("right", true)
	await get_tree().process_frame
	await get_tree().process_frame

	var size := get_viewport().get_visible_rect().size
	var orientation := "landscape" if size.x > size.y else "portrait"
	var img := get_viewport().get_texture().get_image()
	var out := "user://shell_shot_phone_%s.png" % orientation
	img.save_png(out)
	print("saved ", ProjectSettings.globalize_path(out))
	get_tree().quit()
