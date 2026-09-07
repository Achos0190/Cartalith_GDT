extends Node
## What the switch ACTUALLY looks like on screen, versus the `ImageTexture`
## `_switch()` built. Run WINDOWED.
func frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw

func hx(c: Color) -> String:
	return "%02x%02x%02x" % [roundi(c.r * 255.0), roundi(c.g * 255.0), roundi(c.b * 255.0)]

func _ready() -> void:
	DccTheme.set_phone(false); DccTheme.set_touch(false); DccTheme.apply_theme(true)
	var bg := ColorRect.new(); bg.color = Color(0, 0, 0)
	bg.set_anchors_preset(Control.PRESET_FULL_RECT); add_child(bg)
	var col := VBoxContainer.new(); col.position = Vector2(20, 20)
	col.size = Vector2(300, 100); add_child(col)
	var on := DccWidgets.toggle(col, "", true, func(_v): pass)
	var off := DccWidgets.toggle(col, "", false, func(_v): pass)
	await frames(6)
	var img := get_viewport().get_texture().get_image()
	print("=== _vfy_cbdraw_probe ===")
	for pair in [[on, "checked", "ON "], [off, "unchecked", "OFF"]]:
		var cb: CheckBox = pair[0]
		var tex: Texture2D = cb.get_theme_icon(pair[1])
		var raw := tex.get_image()
		var r := cb.get_global_rect()
		## The icon is drawn left-aligned inside the CheckBox; find it by
		## scanning the row through the switch's vertical centre.
		var ty := int(r.position.y + r.size.y * 0.5)
		var best := Color(0, 0, 0)
		var bx := -1
		for x in range(int(r.position.x), int(r.position.x + r.size.x)):
			var p := img.get_pixel(x, ty)
			if p.get_luminance() > best.get_luminance():
				best = p; bx = x
		## Track colour: the raw texture's own leftmost track pixel, at the
		## same vertical centre, versus what is on screen at that column.
		var trk_raw := raw.get_pixel(1, raw.get_height() / 2)
		print("%s knob raw=%s  brightest-on-screen=%s (x=%d)  track raw=%s"
			% [pair[2], hx(raw.get_pixel(raw.get_width() / 2, raw.get_height() / 2)),
				hx(best), bx, hx(trk_raw)])
		print("    modulate in force: %s" % hx(cb.get_theme_color(
			"checkbox_checked_color" if pair[1] == "checked" else "checkbox_unchecked_color")))
	get_tree().quit(0)
