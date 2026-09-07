extends Node
## Does Godot 4.7's `CheckBox` MODULATE its `checked`/`unchecked` icons with
## `checkbox_checked_color` / `checkbox_unchecked_color`? If it does, the
## switch textures installed on 2026-09-07 are multiplied by
## `dark_theme.tres`'s amber/grey before they reach the screen, and a probe
## that reads the `ImageTexture` cannot see it.
##
## Method: render the same `CheckBox` twice, changing ONLY that colour, and
## diff the two frames. Must be run WINDOWED -- the headless dummy renderer
## returns a blank viewport texture.
##
##   Godot_v4.7.1 --path . _vfy_cbmod_probe.tscn

func shot() -> Image:
	for _i in 5:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	return get_viewport().get_texture().get_image()

func _ready() -> void:
	DccTheme.set_phone(false)
	DccTheme.set_touch(false)
	DccTheme.apply_theme(true)
	var bg := ColorRect.new()
	bg.color = Color(0, 0, 0)
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(bg)
	var col := VBoxContainer.new()
	col.position = Vector2(20, 20)
	col.size = Vector2(200, 60)
	add_child(col)
	var cb := DccWidgets.toggle(col, "", true, func(_v): pass)

	cb.add_theme_color_override("checkbox_checked_color", Color(1, 1, 1))
	var a := await shot()
	cb.add_theme_color_override("checkbox_checked_color", Color(1, 0, 0))
	var b := await shot()

	var diff := 0
	var amax := 0.0
	for y in a.get_height():
		for x in a.get_width():
			var d: float = (a.get_pixel(x, y) - b.get_pixel(x, y)).g
			if absf(d) > 2.0 / 255.0:
				diff += 1
				amax = maxf(amax, absf(d))
	var lit := 0
	for y in a.get_height():
		for x in a.get_width():
			if a.get_pixel(x, y).get_luminance() > 0.02:
				lit += 1
	print("=== _vfy_cbmod_probe ===")
	print("lit pixels in frame A (proves the switch actually drew): %d" % lit)
	print("pixels that changed when checkbox_checked_color moved white->red: %d (max dg=%.3f)"
		% [diff, amax])
	if lit < 20:
		print("INCONCLUSIVE: nothing drew -- run WINDOWED, not --headless")
	elif diff > 0:
		print("MODULATED: dark_theme.tres's checkbox_checked_color TINTS the switch")
	else:
		print("NOT MODULATED: the icon reaches the screen untinted")
	get_tree().quit(0)
