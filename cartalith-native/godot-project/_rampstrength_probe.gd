extends SceneTree

# What `TerrainAppearance::ramp_strength` actually does when it is moved off the
# shipped `0.0` -- measured, not reasoned from `LayerStack::composite`.
#
# The question this exists to answer is a judgement, not a bug: the right dock's
# Layers section and the left dock's Terrain raster stack both draw a Colour
# relief row whose dot, opacity, blend and order are live over a layer that
# `composite` skips entirely while the ramp contributes nothing. Either the ramp
# gets a non-zero default, or the row folds away until it has one. Both answers
# need a number for "what does the layer look like when it is on".
#
# Three seeds, because one world is one sample. Run WINDOWED -- these are pixels
# (`MISTAKES.md`: `ImageTexture.update()` is a no-op under `--headless`). This
# path uses `ImageTexture::create_from_image`, so it would in fact survive
# headless; running it windowed anyway costs nothing and keeps the claim honest.
#
#   godot --path <godot-project> --script _rampstrength_probe.gd

const SEEDS := [1234, 483920, 4242]
const GRID_W := 192
const GRID_H := 121
const WIDTH_KM := 400.0

var fails := 0

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

func _bytes(wg) -> PackedByteArray:
	var tex: Texture2D = wg.build_color_texture()
	if tex == null:
		return PackedByteArray()
	var img: Image = tex.get_image()
	if img == null:
		return PackedByteArray()
	return img.get_data()

## Percentage of channel bytes that differ by more than `tol`, the mean absolute
## difference over every channel byte, and the largest single difference.
func _diff(a: PackedByteArray, b: PackedByteArray, tol: int) -> Dictionary:
	if a.size() == 0 or a.size() != b.size():
		return {"pct": -1.0, "mean": -1.0, "max": -1}
	var moved := 0
	var total := 0
	var peak := 0
	for i in a.size():
		var d: int = absi(a[i] - b[i])
		total += d
		if d > peak:
			peak = d
		if d > tol:
			moved += 1
	return {
		"pct": 100.0 * float(moved) / float(a.size()),
		"mean": float(total) / float(a.size()),
		"max": peak,
	}

func _initialize() -> void:
	if not ClassDB.class_exists("WorldGen"):
		print("FAIL  WorldGen is not registered -- the GDExtension did not load")
		quit(1)
		return
	var wg = ClassDB.instantiate("WorldGen")
	for m in ["generate_sized", "build_color_texture", "set_appearance", "get_appearance",
			"reset_appearance", "get_color_ramp", "set_layer_stack", "get_layer_stack"]:
		_ok(wg.has_method(m), "WorldGen exposes %s()" % m)

	# ---- The shipped default, and that the ramp behind it is populated -----
	var app0: Dictionary = wg.get_appearance()
	_ok(float(app0.get("ramp_strength", -1.0)) == 0.0,
		"ramp_strength ships at 0.0 (read back: %s)" % str(app0.get("ramp_strength", "<absent>")))
	var stops: Array = wg.get_color_ramp()
	_ok(stops.size() > 0, "the default ramp carries %d stops -- the strength slider has something to reveal" % stops.size())

	print("")
	for seed in SEEDS:
		print("seed %d, %dx%d grid over %.0f km:" % [seed, GRID_W, GRID_H, WIDTH_KM])
		wg.call("generate_sized", seed, WIDTH_KM, GRID_W, GRID_H)
		wg.reset_appearance()
		var base := _bytes(wg)
		_ok(base.size() > 0, "  the default world renders (%d bytes)" % base.size())

		for s in [0.15, 0.35, 0.60, 1.00]:
			wg.set_appearance({"ramp_strength": s})
			var d := _diff(base, _bytes(wg), 2)
			print("  strength %.2f -> %.1f%% of channel bytes move by >2, mean |d| %.2f levels, max %d"
				% [s, d["pct"], d["mean"], d["max"]])
		wg.reset_appearance()
		_ok(_bytes(wg) == base, "  reset_appearance() returns the default image byte-identically")

		# ---- The defect itself, both ways round --------------------------
		# The four controls the Colour relief row carries. At the shipped
		# strength they must move nothing; at 0.35 they must move something.
		# A one-sided measurement cannot tell a dead control from a dead layer.
		#
		# `set_layer_stack` takes **exactly three rows, top-first**, and refuses
		# anything else with a 0 -- a one-row array would be a no-op that read
		# as "the control moved nothing". Every write below is checked for its 3.
		var swaps := [
			["hidden", [{"id": "hillshade"}, {"id": "colour_relief", "visible": false}, {"id": "terrain"}]],
			["opacity 0.25", [{"id": "hillshade"}, {"id": "colour_relief", "opacity": 0.25}, {"id": "terrain"}]],
			["blend Multiply", [{"id": "hillshade"}, {"id": "colour_relief", "blend": "Multiply"}, {"id": "terrain"}]],
			["moved to the top", [{"id": "colour_relief"}, {"id": "hillshade"}, {"id": "terrain"}]],
		]
		for sw in swaps:
			var what := String(sw[0])
			var rows: Array = sw[1]
			wg.reset_appearance()
			_ok(wg.set_layer_stack(rows) == 3, "  the engine accepted \"%s\"" % what)
			var d_off := _diff(base, _bytes(wg), 2)
			_ok(d_off["pct"] == 0.0 and d_off["max"] == 0,
				"  strength 0.00: colour relief %s moves ZERO bytes (max delta %d)" % [what, d_off["max"]])

			wg.reset_appearance()
			wg.set_appearance({"ramp_strength": 0.35})
			var lit := _bytes(wg)
			wg.set_layer_stack(rows)
			var d_on := _diff(lit, _bytes(wg), 2)
			print("    strength 0.35: %s moves %.1f%% of bytes, mean |d| %.2f, max %d"
				% [what, d_on["pct"], d_on["mean"], d_on["max"]])
			_ok(d_on["pct"] > 0.0, "  strength 0.35: colour relief %s moves pixels" % what)
		wg.reset_appearance()
		print("")

	print("%s -- %d failure(s)" % ["ALL PASS" if fails == 0 else "FAILURES", fails])
	quit(1 if fails > 0 else 0)
