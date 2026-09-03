extends Node
## Pixel evidence for the append, run **windowed** because `--headless` renders
## nothing to diff (`_phonechrome_probe.gd`'s own note: structural assertions
## only there). `_rdappend_probe.gd` asserts on the dock's `Label` tree, which
## proves the nodes exist and not that anything reached the screen -- MISTAKES.md
## line 42: "reasoning from the scene graph proves nothing under an opaque
## overlay. Flip the flag and diff the framebuffer."
##
## So: select a settlement, capture, arm Territory, capture, and count differing
## pixels **inside the right dock's own global rect**. A count of 0 would mean
## the appended section is inert whatever the node tree says. The count is a
## palette-agnostic measure (differing/total), not a brightness threshold, so it
## is not the palette-bound trap MISTAKES.md line 41 records -- the palette is
## printed alongside it anyway so the run is identifiable.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 \
##       _rdappend_shot.tscn

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _grab() -> Image:
	await RenderingServer.frame_post_draw
	return get_viewport().get_texture().get_image()

func _ready() -> void:
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	app.bridge.generate({
		"seed": 483920, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while app.bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _frames(12)

	var settlements: Array = app.bridge.settlements()
	if settlements.is_empty():
		print("RDS  !! no settlement -- cannot run")
		get_tree().quit(1)
		return
	app.right_dock_ctrl.on_settlement_selected(settlements[0], 0)
	await _frames(6)
	var rect: Rect2 = app.right_dock_body.get_global_rect()
	var before := await _grab()
	print("RDS dock rect=%s  title=%s" % [rect, app.right_dock_ctrl._current_title()])

	app.arm_tool("territory")
	await _frames(8)
	var after := await _grab()

	if rect.size.x < 2.0 or rect.size.y < 2.0:
		print("RDS  !! right dock body has no rect to sample -- %s" % rect)
		get_tree().quit(1)
		return

	var x0 := int(maxf(0.0, rect.position.x))
	var y0 := int(maxf(0.0, rect.position.y))
	var x1 := int(minf(float(before.get_width()), rect.position.x + rect.size.x))
	var y1 := int(minf(float(before.get_height()), rect.position.y + rect.size.y))
	var diff := 0
	var total := 0
	var acc := Color(0, 0, 0)
	for y in range(y0, y1):
		for x in range(x0, x1):
			total += 1
			acc += before.get_pixel(x, y)
			if before.get_pixel(x, y) != after.get_pixel(x, y):
				diff += 1
	var mean := acc / maxf(1.0, float(total))
	print("RDS palette: mean dock pixel before = (%d,%d,%d) -- %s theme" % [
		int(mean.r * 255.0), int(mean.g * 255.0), int(mean.b * 255.0),
		"light" if mean.get_luminance() > 0.5 else "dark"])
	print("RDS title after arm = %s" % app.right_dock_ctrl._current_title())
	print("RDS pixels changed inside the dock: %d of %d (%.1f%%)" % [
		diff, total, 100.0 * float(diff) / maxf(1.0, float(total))])
	print("RDS verdict: %s" % ("the appended section reached the screen"
		if diff > 0 else "INERT -- 0 pixels moved"))
	get_tree().quit(0 if diff > 0 else 1)
