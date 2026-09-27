extends Node
## Screenshot-only harness for the "planner fix lands on a sheet that covers
## the tab bar" row (Ruling BO / `OUTSTANDING_WORK.md`). `_navbarfix_probe.gd`
## already proves the RECT math (`sheet.get_global_rect().end.y <= bar_top`);
## this exists only to leave the owner a picture of it, since `--resolution`
## on the CLI was measured NOT to produce the requested window size in this
## environment (1080x2340 requested, 1080x1031 delivered) -- explicit
## `DisplayServer.window_set_size()` is the one path `_navbarfix_probe.gd`
## already established as reliable here, so this copies it rather than
## trusting the flag a second time.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _leftsheet_shot_probe.tscn -- --force-touch
##
## Protects nothing by itself (it has no assertion) -- it is evidence for a
## human, and `_navbarfix_probe.gd` is the thing that actually protects the
## rect math from regressing.

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 60.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var want := Vector2i(1080, 2340)
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	## A generated world, not the empty-project state -- otherwise the
	## screenshot shows an empty planner and proves nothing about the sheet
	## geometry a real session would see. Same seed/size `_shot_phone.gd`
	## already uses for its own captures.
	var bridge = app.bridge
	bridge.generate({
		"seed": 483920, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.6).timeout

	## The PLAN tab's own real entry point, not the generic `_set_sheet_open`
	## call `_navbarfix_probe.gd` uses -- this screenshot is meant to show the
	## actual reported scenario (tapping PLAN), and `open_journey_planner()`
	## is what every PLAN tap and every other entry point converges on
	## (`app.gd`'s own doc comment on that function).
	app.select_domain_mode("civilization", "planner")
	await _frames(2)
	app.journey_planner_view.open()
	await _frames(6)

	var img := get_viewport().get_texture().get_image()
	var out := "C:/Users/Vincent/AppData/Local/Temp/claude/C--Users-Vincent-Cartalith-GDT/00ffe296-8aa1-4e76-b7ec-e81351b1a7b7/scratchpad/uidesign/planner_sheet_tabbar.png"
	img.save_png(out)
	print("saved ", out)
	get_tree().quit()
