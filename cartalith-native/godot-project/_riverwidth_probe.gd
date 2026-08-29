extends Node
## Does a smaller map actually draw wider, more visible rivers?
##
##   godot --headless --path . _riverwidth_probe.tscn
##
## Owner: "As soon as the map width/size becomes lower the size width and
## length of a river should become bigger and more visible."
##
## The unit test pins the stamp's own arithmetic. This measures the thing the
## owner can actually see: river-tinted pixels in the real colour texture, on
## real generated worlds, at the same seed and grid with only the km extent
## changed. A river-tinted cell is one whose blue channel is pushed up and
## whose red is pushed down relative to what the terrain alone would give, so
## it is counted by comparing against the same world's own land/sea palette
## rather than by matching an absolute colour.

var _fail := 0

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

## Cells whose colour carries the river tint: the tint is
## `(r*0.5, g*0.5+0.3, b*0.5+0.45)`, so an inked cell is markedly bluer than
## it is red. Ocean is blue too, so require a real blue-over-red margin AND
## exclude the deep-water range the tint never reaches.
func _river_pixels(tex: Texture2D) -> int:
	var img: Image = tex.get_image()
	var n := 0
	for y in img.get_height():
		for x in img.get_width():
			var c := img.get_pixel(x, y)
			if c.b > c.r + 0.22 and c.b > 0.45 and c.g > c.r:
				n += 1
	return n

func _ready() -> void:
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var wg: Object = ClassDB.instantiate("WorldGen")

	## Same seed, same grid, ONLY the km extent changes -- so any difference in
	## the count is the extent and nothing else.
	var counts: Dictionary = {}
	for km in [3200.0, 800.0, 200.0, 50.0]:
		wg.set_params({"tect.plates": 9})
		wg.generate_sized(24601, km, 256, 192)
		var tex: Texture2D = wg.build_color_texture()
		counts[km] = _river_pixels(tex)
		print("  %8.0f km -> %6d river-tinted pixels" % [km, counts[km]])

	print("\n=== the owner's requirement ===")
	_ok("200 km draws more river than 800 km", counts[200.0] > counts[800.0], true)
	_ok("50 km draws more river than 200 km", counts[50.0] > counts[200.0], true)
	## **Deliberately not asserted monotone across the whole range**, and this
	## is a measured finding rather than a slack test. `river_flow_thresh`
	## divides by `terrain_detail_k(...) * river_coarse_ease(...)`, and that
	## product is **minimised at exactly 800 km** -- so 800 km is the sparsest
	## river network in BOTH directions, and a 3200 km world channelizes more
	## cells than an 800 km one. Measured here: 3200 km -> 5455 tinted pixels
	## against 800 km's 4476.
	##
	## That is faithful to the reference (`riverFlowThresh` 4493,
	## `terrainDetailK` 2643, `riverCoarseEase` 2674 all behave identically),
	## so it is reported, not patched. The width law this probe exists to check
	## is monotone; the channel-COUNT law underneath it is not, and the two
	## compose.
	print("  info 3200 km draws %d, 800 km draws %d -- the 800 km trough is"
		% [counts[3200.0], counts[800.0]])
	print("       expected and faithful to the reference, not a regression.")
	if counts[800.0] > 0:
		print("  info 50 km / 800 km ratio: %.2fx" % (float(counts[50.0]) / float(counts[800.0])))

	print("\n_riverwidth_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
