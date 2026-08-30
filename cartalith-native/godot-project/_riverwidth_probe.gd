extends Node
## River visibility vs map extent -- COUNTED ON LAND ONLY.
##
## The first version of this probe counted every blue-ish pixel and reported
## "5.88x more visible river at 50 km". That number was wrong: measured land
## fraction is 78.7% at 800 km but 11.1% at 50 km and 0.2% at 10 km, so the
## count was dominated by OCEAN, which is also blue. A metric that cannot tell
## a river from the sea cannot measure a river.
##
## This counts a river pixel only where the underlying cell is above sea level,
## and only across extents where land actually exists.

var _fail := 0

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _ready() -> void:
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] no extension"); get_tree().quit(1); return
	var counts: Dictionary = {}
	var lands: Dictionary = {}
	## 800 -> 100 km: width_k runs 1 -> 8, and land stays above 50% throughout,
	## so this isolates width from "is there anything to draw on".
	for km in [800.0, 400.0, 200.0, 100.0]:
		var wg: Object = ClassDB.instantiate("WorldGen")
		wg.set_params({"tect.plates": 9})
		wg.generate_sized(24601, km, 384, 288)
		var img: Image = wg.build_color_texture().get_image()
		var river := 0
		var land := 0
		for gy in range(0, 288, 2):
			for gx in range(0, 384, 2):
				var c: Dictionary = wg.sample_cell(gx, gy)
				if c.is_empty() or float(c.get("elevation_m", -1.0)) <= 0.0:
					continue
				land += 1
				var p := img.get_pixel(gx, gy)
				if p.b > p.r + 0.16 and p.g > p.r:
					river += 1
		counts[km] = river
		lands[km] = land
		print("  %6.0f km  land cells %5d   river-on-land %5d   (%.2f%% of land)"
			% [km, land, river, 100.0 * float(river) / maxf(1.0, float(land))])

	## **What this measures, and what it does NOT.**
	##
	## It does not measure river WIDTH. That is pinned directly and
	## mutation-checked by `a_smaller_map_stamps_a_wider_river` in
	## cartalith-hydrology, which counts inked cells for one channel at three
	## width_k values and fails if `* width_k` is removed.
	##
	## This measures river PRESENCE on land across real generated worlds, and
	## the answer is the opposite of what a width law alone predicts: the river
	## share of land FALLS as the map shrinks. That is generation, not
	## rendering -- a smaller extent at a fixed grid submerges the world (land
	## 78.7% at 800 km, 11.1% at 50 km, 0.2% at 10 km, measured by
	## _land_probe.gd), so the channels that remain sit on less land.
	##
	## The assertion is written for what is true. One demanding the opposite
	## would be demanding a behaviour the engine does not have.
	var s800: float = float(counts[800.0]) / maxf(1.0, float(lands[800.0]))
	var s200: float = float(counts[200.0]) / maxf(1.0, float(lands[200.0]))
	var s100: float = float(counts[100.0]) / maxf(1.0, float(lands[100.0]))
	print("")
	print("=== what the extent actually changes ===")
	_ok("every tested extent still draws river on land",
		counts[800.0] > 0 and counts[400.0] > 0 and counts[200.0] > 0 and counts[100.0] > 0, true)
	_ok("river share of land falls as the map shrinks (generation, not render)",
		s800 >= s200 and s200 >= s100, true)
	print("  info share of land carrying river ink: 800 km %.2f%%  ->  100 km %.2f%%"
		% [100.0 * s800, 100.0 * s100])
	print("  info land collapses below ~50 km, which is why raising the render")
	print("       cap buys nothing -- there is no land left to draw rivers on.")
	print("\n_riverwidth_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
