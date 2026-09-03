extends Node
## PH-16 verification probe: measures the Journey Planner centre panel's
## phone composition. `_build_center_panel()`'s structure does not depend on
## `bridge.has_world` -- only its row *content* does (`_bound`, set from
## `bridge.world_gen.has_method(...)`, is a capability check, not a
## has-a-world check) -- so no `_run_pipeline()` call is needed to exercise
## the bug this checks: PH-16 was a double-scale of the panel's own fixed-
## height rows, not a content problem.
##
## Run:
##   godot --headless --path . _ph16_probe.tscn -- --force-touch --h1080
##   godot --headless --path . _ph16_probe.tscn -- --force-touch
##
## Same window-sizing sequence as `_ph9_probe.gd`, for the same reason its own
## comment gives: `--resolution` is clamped to the monitor's usable rect on
## Windows, which silently turns a phone-sized run into a desktop one.

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	Input.set_emulate_touch_from_mouse(true)
	var want := Vector2i(1440, 3168)
	if OS.get_cmdline_user_args().has("--h1080"):
		want = Vector2i(1080, 2400)
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout

	var screen: Vector2 = app.get_viewport_rect().size
	print("=== phone=", app.is_phone(), " scale=", app.phone_scale(),
		" screen=", screen, " ===")
	if not app.is_phone():
		print("NOT PHONE -- window size did not register as a handset; nothing measured")
		get_tree().quit()
		return

	app.open_journey_planner()
	await get_tree().create_timer(0.8).timeout
	await _frames(4)

	var jp = app.journey_planner_view
	print("bound=", jp._bound, " active=", jp._active)
	var center: Control = jp._center_panel
	if center == null or not is_instance_valid(center) or center.get_child_count() == 0:
		print("NO CENTER PANEL CONTENT (jp_* not bound in this build?)")
		get_tree().quit()
		return
	print("center visible=", center.visible, " size=", center.size)

	var col: Control = center.get_child(0)
	print("col class=", col.get_class(), " children=", col.get_child_count())
	var expect_unscaled := [236, 150, 32]
	var labels := ["map_row_pad", "profile_wrap", "stops_wrap", "lower(expand)"]
	var running_y := 0.0
	for i in mini(labels.size(), col.get_child_count()):
		var c: Control = col.get_child(i)
		print(" [%d] %-14s class=%-16s custom_min=%-14s size=%-16s pos.y=%.1f" % [
			i, labels[i], c.get_class(), c.custom_minimum_size, c.size, c.position.y])
		if i < expect_unscaled.size():
			running_y += c.size.y

	var map_row_pad: Control = col.get_child(0)
	var single_scale: float = 236.0 * app.phone_scale()
	var double_scale: float = 236.0 * app.phone_scale() * app.phone_scale()
	print("map_row_pad.size.y=", map_row_pad.size.y,
		"  236*scale=", "%.1f" % single_scale,
		"  236*scale^2 (the PH-16 bug)=", "%.1f" % double_scale)
	print("sum of the three fixed rows' rendered height=", running_y,
		" of screen height=", screen.y,
		" (", "%.1f" % (100.0 * running_y / screen.y), "% )")

	# The actual PH-16 symptom: is there room left, on screen, for the
	# inspector/matrix area below the three fixed rows?
	var remaining := screen.y - running_y
	print("remaining height for inspector/matrix after the fixed rows=", remaining,
		" (must be positive and not tiny for PH-16 to be fixed)")

	print("=== done ===")
	get_tree().quit()
