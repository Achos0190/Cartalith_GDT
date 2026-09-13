extends SceneTree
## Lane URBANDRAW: loads the HEAD-vs-tree PNG pairs `_towertint_probe.gd`
## saved and reports a full-canvas pixel diff count per settlement -- the
## Gates' "pixel confinement" measurement. Pure image loading (`Image.load`),
## no rendering, so headless is correct here (MISTAKES.md: windowed is only
## for capturing a NEW frame; reading an already-saved PNG is data).
##
##   Godot_v4.7.1-stable_win64.exe --headless --path . --script _towertint_diff_probe.gd -- --dir <shared out dir>

func _init() -> void:
	var args := OS.get_cmdline_user_args()
	var dir := ""
	for i in args.size():
		if args[i] == "--dir" and i + 1 < args.size():
			dir = args[i + 1]
	if dir == "":
		print("DIFFPROBE ABORT -- no --dir given")
		quit(2)
		return

	for idx in range(5):
		var head_path := "%s/head_s%d.png" % [dir, idx]
		var tree_path := "%s/tree_s%d.png" % [dir, idx]
		var a := Image.new()
		var b := Image.new()
		var ea := a.load(head_path)
		var eb := b.load(tree_path)
		if ea != OK or eb != OK:
			print("DIFFPROBE settlement #%d: load error head=%s tree=%s" % [idx, ea, eb])
			continue
		if a.get_size() != b.get_size():
			print("DIFFPROBE settlement #%d: SIZE MISMATCH %s vs %s" % [idx, a.get_size(), b.get_size()])
			continue
		var w := a.get_width()
		var h := a.get_height()
		var diff := 0
		var minx := w
		var miny := h
		var maxx := -1
		var maxy := -1
		for y in h:
			for x in w:
				if a.get_pixel(x, y) != b.get_pixel(x, y):
					diff += 1
					if x < minx: minx = x
					if y < miny: miny = y
					if x > maxx: maxx = x
					if y > maxy: maxy = y
		var total := w * h
		var pct := 100.0 * float(diff) / float(total)
		if diff > 0:
			print("DIFFPROBE settlement #%d: %dx%d=%d px, diff=%d (%.2f%%), bbox=(%d,%d)-(%d,%d)" % [
				idx, w, h, total, diff, pct, minx, miny, maxx, maxy])
		else:
			print("DIFFPROBE settlement #%d: %dx%d=%d px, diff=0 (byte-identical)" % [idx, w, h, total])

	quit(0)
