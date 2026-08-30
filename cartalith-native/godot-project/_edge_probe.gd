extends Node
## Lead #2, measured: does a sub-Window with CONTENT_SCALE_MODE_CANVAS_ITEMS and
## a fractional content_scale_factor rasterise its fonts at the scaled size, or
## rasterise at nominal and let the canvas transform smear them?
##
## Two windows, same physical glyph height:
##   A: content_scale_factor = 3.664, font_size 12   (the shipped phone case)
##   B: content_scale_factor = 1.0,   font_size 44   (the native-raster control)
## The discriminator is the edge gradient: a natively-rasterised glyph steps
## from ground to ink in ~1 px, a 3.664x resample spreads that ramp over ~4 px,
## so max |dLum| between horizontally adjacent pixels caps at about 1/3.664.

const SCALE := 1440.0 / 393.0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _make(factor: float, fs: int, oversample: bool, override_os: float = 0.0) -> Window:
	var w := Window.new()
	w.borderless = true
	w.transparent = false
	w.size = Vector2i(560, 120)
	w.position = Vector2i(4, 4)
	w.content_scale_mode = Window.CONTENT_SCALE_MODE_CANVAS_ITEMS
	w.content_scale_aspect = Window.CONTENT_SCALE_ASPECT_IGNORE
	w.content_scale_factor = factor
	w.oversampling = oversample
	if override_os > 0.0:
		w.oversampling_override = override_os
	var bg := ColorRect.new()
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	bg.color = Color.BLACK
	w.add_child(bg)
	var l := Label.new()
	l.text = "HImnE 018"
	l.position = Vector2(4, 4)
	l.add_theme_color_override("font_color", Color.WHITE)
	l.add_theme_font_size_override("font_size", fs)
	l.add_theme_font_override("font", load("res://fonts/IBMPlexMono-Regular.ttf"))
	w.add_child(l)
	add_child(w)
	return w

## Max and mean of the top 1 % of horizontally adjacent luminance deltas, plus
## how many pixels sit at a "hard" edge (|d| > 0.5). Resampled bitmaps cannot
## produce a hard edge; natively-rasterised text produces thousands.
func _edges(img: Image, tag: String) -> Dictionary:
	var w := img.get_width()
	var h := img.get_height()
	var deltas: Array[float] = []
	var hard := 0
	var ink := 0
	for y in range(0, h):
		var prev := -1.0
		for x in range(0, w):
			var c := img.get_pixel(x, y)
			var lum := c.r * 0.299 + c.g * 0.587 + c.b * 0.114
			if lum > 0.5:
				ink += 1
			if prev >= 0.0:
				var d: float = absf(lum - prev)
				if d > 0.02:
					deltas.append(d)
				if d > 0.5:
					hard += 1
			prev = lum
	deltas.sort()
	var mx: float = deltas[deltas.size() - 1] if deltas.size() > 0 else 0.0
	var p99: float = deltas[int(deltas.size() * 0.99)] if deltas.size() > 0 else 0.0
	var res := {"tag": tag, "max": mx, "p99": p99, "hard": hard, "ink": ink,
		"n": deltas.size(), "w": w, "h": h}
	print("[edge] %-28s max=%.4f p99=%.4f hard(>0.5)=%d inkpx=%d n=%d %dx%d"
		% [tag, mx, p99, hard, ink, deltas.size(), w, h])
	return res

func _ready() -> void:
	print("=== engine ", Engine.get_version_info()["string"], " scale=", SCALE, " ===")
	var root := get_tree().root
	print("root.oversampling=", root.oversampling, " override=", root.oversampling_override,
		" content_scale_factor=", root.content_scale_factor)

	var cases := [
		["A scaled 3.664 / fs12 oversample=on", SCALE, 12, true, 0.0],
		["B native  1.000 / fs44 oversample=on", 1.0, int(round(12.0 * SCALE)), true, 0.0],
		["C scaled 3.664 / fs12 oversample=OFF", SCALE, 12, false, 0.0],
		["D scaled 3.664 / fs12 override=3.664", SCALE, 12, true, SCALE],
		["E scaled 2.750 / fs12 override=2.750", 2.75, 12, true, 2.75],
		["F scaled 2.750 / fs12 override=0", 2.75, 12, true, 0.0],
	]
	for cse in cases:
		var win := _make(cse[1], cse[2], cse[3], 0.0)
		await _frames(4)
		if float(cse[4]) > 0.0:
			## Set only once the window is in the tree and sized -- setting it in
			## the constructor is what the first cut did, and it read back 0.
			win.oversampling_override = float(cse[4])
			print("    post-tree override set -> ", win.oversampling_override,
				" get_oversampling()=", win.get_oversampling())
		await _frames(6)
		print("  ", cse[0], " -> factor stored=", win.content_scale_factor,
			" oversampling=", win.oversampling,
			" get_oversampling()=", (win.get_oversampling() if win.has_method("get_oversampling") else "n/a"),
			" visible_rect=", win.get_visible_rect())
		var img := win.get_texture().get_image()
		img.save_png("user://edge_%s.png" % String(cse[0]).substr(0, 1))
		_edges(img, cse[0])
		win.queue_free()
		await _frames(2)
	print("=== done ===")
	get_tree().quit()
