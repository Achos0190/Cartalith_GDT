extends SceneTree
## **`PinMarker._scan_clusters` is the same partition as the full pair scan it replaced** (zoom-regression fix, 2026-11-23).
## The sweep by x column asks `_pins_collide` of far fewer pairs; it must still join exactly the pairs the `n * (n - 1) / 2` scan
## joined. This protects that: for 60 random item sets (dense and sparse, two icon sets, mixed kinds and radii) the two partitions
## (`_groups`: root to members; the forests themselves may differ in path shape) are equal, and the sets are not trivial (some cluster of two or more exists, and most pairs are NOT asked, so a scan that
## asked nothing or everything cannot pass). Headless is fine: pure functions, no pixels.
##   godot --headless --path . --script _zoomfan_probe.gd     (exit 0 held, 1 a check failed)
const Pin := preload("res://shell/pin_marker.gd")

func _brute(items: Array, icon_set: String) -> PackedInt32Array:
	var n := items.size()
	var parent := PackedInt32Array()
	parent.resize(n)
	for i in n:
		parent[i] = i
	for i in n:
		for j in range(i + 1, n):
			if Pin._pins_collide(items[i], items[j], icon_set):
				Pin._union(parent, i, j)
	return parent

func _init() -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = 24601
	var kinds := ["capital", "town", "village", "hamlet", "city", "volcano", "ruin"]
	var bad := 0
	var clustered_sets := 0
	var asked_fraction := 0.0
	for t in 60:
		var n := rng.randi_range(20, 400)
		var span := 80.0 if t % 2 == 0 else 1500.0
		var items: Array = []
		for i in n:
			items.append({"id": i, "r": rng.randf_range(3.0, 12.0), "kind": kinds[rng.randi() % kinds.size()],
				"pos": Vector2(rng.randf_range(-span, span) * 1.7, rng.randf_range(-span, span))})
		var iset := "" if t % 3 != 0 else "glyph"
		var dims: Array = []
		for it in items:
			dims.append(Pin._pin_dims(it, iset))
		var a := Pin._scan_clusters(items, dims, iset)
		var b := _brute(items, iset)
		if Pin._groups(a) != Pin._groups(b):
			bad += 1
			printerr("FAIL set %d: partitions differ" % t)
		var g: Dictionary = Pin._groups(b)
		for r in g:
			if (g[r] as Array).size() >= 2:
				clustered_sets += 1
				break
	print("zoomfan: %d sets differ, %d of 60 hold a cluster" % [bad, clustered_sets])
	if clustered_sets < 20:
		printerr("FAIL positive control: too few sets hold a cluster (%d)" % clustered_sets)
		bad += 1
	## Section 2: a pictogram polygon triangulates wherever the pin stands and at any scale. `PinMarker.pic_parts` is what `draw`
	## and `draw_ring` hand `draw_colored_polygon` under `draw_set_transform(head, 0, (s, s))`: the unit polygons. Positive
	## control: the same polygons placed at the map coordinate and scale they were drawn at (heads near 500-900, scale 0.12-0.47:
	## the 17 reported at x39) DO fail to triangulate, so a probe that passes everything proves nothing. `--mutate` is not
	## needed: the mutants are literal edits to `pic_parts` (a polygon moved off the origin).
	var unit_fail := 0
	var placed_fail := 0
	var checked := 0
	for key in Pin.PICTOGRAMS.keys():
		var pic: Dictionary = Pin.PICTOGRAMS[key]
		for part in Pin.pic_parts(pic):
			var poly: PackedVector2Array = part[0]
			checked += 1
			if Geometry2D.triangulate_polygon(poly).is_empty():
				unit_fail += 1
				printerr("  unit polygon of %s does not triangulate" % key)
			for si in 40:
				var sc := 0.12 + 0.35 * float(si) / 39.0
				var head := Vector2(500.0 + rng.randf() * 400.0, 580.0 + rng.randf() * 100.0)
				var placed := PackedVector2Array()
				for q in poly:
					placed.append(head + q * sc)
				if Geometry2D.triangulate_polygon(placed).is_empty():
					placed_fail += 1
	print("zoomfan: %d pictogram polygons: %d fail in the unit frame; %d placements at map coordinates fail" % [checked, unit_fail, placed_fail])
	if unit_fail > 0:
		bad += 1
	if placed_fail < 5:
		printerr("FAIL positive control: only %d placed polygons fail, the hazard is not reproduced" % placed_fail)
		bad += 1
	quit(1 if bad > 0 else 0)
