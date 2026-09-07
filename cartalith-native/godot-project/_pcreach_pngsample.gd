extends Node
## Arbiter for the PC-REACH light-theme leg. Re-samples the saved menu
## screenshots from disk (no viewport in the loop) and writes a crop of the
## open popup, so the popup's own ground can be looked at rather than inferred
## from a downscaled full-frame render.
##
##   Godot_v4.7.1 --headless --path . _pcreach_pngsample.tscn -- --dir <dir>

func _hex(c: Color) -> String:
	return "#%02x%02x%02x" % [int(round(c.r * 255)), int(round(c.g * 255)),
		int(round(c.b * 255))]

func _top(img: Image, r: Rect2i, n: int) -> String:
	var tally: Dictionary = {}
	var x := r.position.x
	while x < r.position.x + r.size.x:
		var y := r.position.y
		while y < r.position.y + r.size.y:
			if x >= 0 and y >= 0 and x < img.get_width() and y < img.get_height():
				var k := _hex(img.get_pixel(x, y))
				tally[k] = int(tally.get(k, 0)) + 1
			y += 2
		x += 2
	var keys: Array = tally.keys()
	keys.sort_custom(func(a, b): return int(tally[a]) > int(tally[b]))
	var out: Array = []
	for i in mini(n, keys.size()):
		out.append("%s x%d" % [keys[i], int(tally[keys[i]])])
	return ", ".join(out)

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var dir := ""
	var i := args.find("--dir")
	if i >= 0 and i + 1 < args.size():
		dir = String(args[i + 1])
	for e in [["light_menu_data.png", Rect2i(240, 40, 340, 300)],
			["light_menu_preferences.png", Rect2i(290, 40, 350, 300)]]:
		var path: String = dir + "/" + String(e[0])
		var img := Image.new()
		if img.load(path) != OK:
			print("  %s LOAD FAILED" % e[0])
			continue
		var r: Rect2i = e[1]
		print("  %-30s top5 in %s: %s" % [e[0], r, _top(img, r, 5)])
		var crop := img.get_region(r)
		crop.save_png("%s/CROP_%s" % [dir, String(e[0])])
		print("     crop -> CROP_%s  %dx%d" % [e[0], crop.get_width(), crop.get_height()])
	print("### DONE ###")
	get_tree().quit()
