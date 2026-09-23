extends Control
## Windowed proof for the faubourg's grain (`cartalith_urban::wallside`,
## Ruling H, owner 2026-09-23: *"the building outside the wall being arranged
## quite neatly, often this would follow the same half organic half chaotic
## style as within the city, often the poor district away from a gate and more
## rich people and buildings closer to the gate. (this isn't 100% a rule...)"*).
##
## Each town is written by `cargo run -p cartalith-urban --example town_json`
## TWICE -- on the tree before the change (`*_before.json`) and after it
## (`*_after.json`) -- and drawn here by the real `UrbanLayoutDraw.draw_layout`.
## Regenerate with (from `cartalith-native/`):
##
##   EX=target/debug/examples/town_json.exe
##   $EX 42   medieval 5000  inland godot-project/_faubgrain_town0_<phase>.json
##   $EX 1234 medieval 12000 inland godot-project/_faubgrain_town1_<phase>.json
##   $EX 99   medieval 9000  coast  godot-project/_faubgrain_town2_<phase>.json
##   $EX 7    medieval 1500  inland godot-project/_faubgrain_town3_<phase>.json
##
## Data half, per town, each measured on BOTH phases so the before is the
## control:
##  - WOBBLE: pairs of neighbouring faubourg lots in a row behind the first
##    (back-line midpoints under 10 m apart, back lines within 3 m of each
##    other off the wall face) whose back lines differ by more than 0.4 m. The
##    before rows are one offset curve, so this is near zero there.
##  - GRADIENT: built share of a built faubourg lot (building area over lot
##    area), in the third of lots nearest a land gate (straight-line, from the
##    lot's centroid) against the farthest third. After, the near third must be
##    the more built by a clear margin, and by more than before.
##  - The street-platted lots are an exact prefix of both lists (this change
##    moves only the faubourg).
##
## Pixel half, "flip the flag and diff the framebuffer": sample points inside
## the after faubourg's buildings must mostly change against the before
## drawing; points 60 m inside the wall must not (the control). Windowed,
## because `frame_post_draw` never delivers under `--headless`.
##
## godot4 --path . _faubgrain_probe.tscn

const TOWNS := 4
## The curtain's half-stroke (`WALL_W["curtain"]` 4.5 / 2), a literal.
const FACE := 2.25
const TILE := 200.0

var _fails := 0
var _layout: Dictionary = {}
var _to_screen: Callable
var _scale := 1.0


func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		_fails += 1
		print("  FAIL  %s" % what)


func _draw() -> void:
	draw_rect(Rect2(Vector2.ZERO, size), Color(0.761, 0.702, 0.580))
	if not _layout.is_empty():
		UrbanLayoutDraw.draw_layout(self, _layout, _to_screen, _scale, 1.0, 1.0, false)


func _settle() -> void:
	await RenderingServer.frame_post_draw
	await get_tree().process_frame
	await RenderingServer.frame_post_draw


func _pv(a: Array) -> PackedVector2Array:
	var out := PackedVector2Array()
	for p in a:
		out.append(Vector2(p[0], p[1]))
	return out


func _load(path: String) -> Dictionary:
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		_ok(false, "%s opens -- regenerate it (see the header)" % path)
		return {}
	var j: Dictionary = JSON.parse_string(f.get_as_text())
	var d := {}
	d["market"] = Vector2(j["market"][0], j["market"][1])
	var streets := {}
	for k in j["streets"]:
		streets[k] = _pv(j["streets"][k])
	d["streets"] = streets
	for k in ["blocks", "parcels", "buildings"]:
		var arr: Array = []
		for poly in j[k]:
			arr.append(_pv(poly))
		d[k] = arr
	d["block_plaza"] = PackedByteArray(j["block_plaza"])
	d["parcel_district"] = PackedStringArray(j["parcel_district"])
	d["parcel_tone"] = PackedFloat32Array(j["parcel_tone"])
	d["building_ridge"] = _pv(j["building_ridge"])
	d["building_district"] = PackedStringArray(j["building_district"])
	d["building_tone"] = PackedFloat32Array(j["building_tone"])
	for k in ["plaza", "wall_ring", "wall_gates", "river", "water_poly"]:
		if j.has(k):
			d[k] = _pv(j[k])
	if j.has("wall_style"):
		d["wall_style"] = j["wall_style"]
	if j.has("wall_centroid"):
		d["wall_centroid"] = Vector2(j["wall_centroid"][0], j["wall_centroid"][1])
	d["river_w"] = float(j["river_w"])
	return d


func _ring_dist(p: Vector2, ring: PackedVector2Array) -> float:
	var best := INF
	for i in ring.size():
		var q := Geometry2D.get_closest_point_to_segment(p, ring[i], ring[(i + 1) % ring.size()])
		best = minf(best, p.distance_to(q))
	return best


func _centroid(poly: PackedVector2Array) -> Vector2:
	var c := Vector2.ZERO
	for p in poly:
		c += p
	return c / maxf(1.0, poly.size())


func _area(poly: PackedVector2Array) -> float:
	var a := 0.0
	for i in poly.size():
		a += poly[i].cross(poly[(i + 1) % poly.size()])
	return absf(a) * 0.5


func _render(layout: Dictionary, box: Rect2) -> Image:
	var vp: Vector2 = get_viewport_rect().size
	var fit: float = minf((vp.x - 32.0) / box.size.x, (vp.y - 32.0) / box.size.y)
	var origin: Vector2 = (vp - box.size * fit) * 0.5 - box.position * fit
	_layout = layout
	_scale = fit
	_to_screen = func(p: Vector2) -> Vector2: return origin + p * fit
	queue_redraw()
	await _settle()
	return get_viewport().get_texture().get_image()


func _px(img: Image, p: Vector2) -> Color:
	var s: Vector2 = _to_screen.call(p)
	var x := int(round(s.x))
	var y := int(round(s.y))
	if x < 0 or y < 0 or x >= img.get_width() or y >= img.get_height():
		_ok(false, "sample (%d, %d) is on screen" % [x, y])
		return Color(0, 0, 0, 0)
	return img.get_pixel(x, y)


## Indices of the faubourg lots.
func _faub(d: Dictionary) -> Array:
	var out: Array = []
	var ds: PackedStringArray = d["parcel_district"]
	for i in ds.size():
		if ds[i] == "faubourg":
			out.append(i)
	return out


## [pairs, wobbling pairs] over neighbouring off-face faubourg lots.
func _wobble(d: Dictionary, ring: PackedVector2Array) -> Array:
	var ps: Array = d["parcels"]
	var backs: Array = []
	for i in _faub(d):
		var par: PackedVector2Array = ps[i]
		var m := par[3].lerp(par[2], 0.5)
		var off := _ring_dist(m, ring) - FACE
		if off > 1.5:
			backs.append([m, off])
	var pairs := 0
	var wob := 0
	for a in backs.size():
		for b in range(a + 1, backs.size()):
			if (backs[a][0] as Vector2).distance_to(backs[b][0]) >= 10.0:
				continue
			var dd := absf(float(backs[a][1]) - float(backs[b][1]))
			if dd >= 3.0:
				continue
			pairs += 1
			if dd > 0.4:
				wob += 1
	return [pairs, wob]


## [near-third built share, far-third built share, lots] by distance to the
## nearest land gate.
func _gradient(d: Dictionary) -> Array:
	var gates: PackedVector2Array = d.get("wall_gates", PackedVector2Array())
	var ps: Array = d["parcels"]
	var bs: Array = d["buildings"]
	var bc: Array = []
	for b in bs:
		bc.append([_centroid(b), _area(b)])
	var rows: Array = []
	for i in _faub(d):
		var par: PackedVector2Array = ps[i]
		var c := _centroid(par)
		var g := INF
		for q in gates:
			g = minf(g, c.distance_to(q))
		var built := 0.0
		for b in bc:
			if Geometry2D.is_point_in_polygon(b[0], par):
				built += float(b[1])
		# A lot with no building at all is left out: near a gate that is
		# almost always one `build_markets` cleared for a gate market (measured
		# 2026-09-23, town1: 17 cleared lots in the near third, 0 in the far),
		# which says nothing about the grammar under test.
		if built > 0.0:
			rows.append([g, built / maxf(1.0, _area(par))])
	rows.sort_custom(func(x, y): return x[0] < y[0])
	var n := rows.size() / 3
	if n == 0:
		return [0.0, 0.0, rows.size()]
	var near := 0.0
	var far := 0.0
	for k in n:
		near += float(rows[k][1])
		far += float(rows[rows.size() - 1 - k][1])
	return [near / n, far / n, rows.size()]


func _probe_town(tag: String) -> void:
	var before := _load("res://_faubgrain_%s_before.json" % tag)
	var after := _load("res://_faubgrain_%s_after.json" % tag)
	if before.is_empty() or after.is_empty():
		return
	var ring: PackedVector2Array = after.get("wall_ring", PackedVector2Array())
	_ok(ring.size() >= 6 and (after.get("wall_gates", PackedVector2Array()) as PackedVector2Array).size() > 0,
		"%s: walled, with a land gate" % tag)
	if ring.size() < 6:
		return
	var bp: Array = before["parcels"]
	var ap: Array = after["parcels"]
	var fb := _faub(before)
	var fa := _faub(after)
	print("FAUBGRAIN %s: faubourg %d -> %d lots, %d -> %d buildings" % [
		tag, fb.size(), fa.size(), (before["buildings"] as Array).size(), (after["buildings"] as Array).size()])
	# The street-platted lots (everything before the first faubourg lot) do not move.
	var first_b: int = fb[0] if not fb.is_empty() else bp.size()
	var first_a: int = fa[0] if not fa.is_empty() else ap.size()
	var same := first_a == first_b
	if same:
		for i in first_a:
			if bp[i] != ap[i]:
				same = false
				break
	_ok(same, "%s: the %d lots platted before the faubourg are identical before and after" % [tag, first_a])

	var wb := _wobble(before, ring)
	var wa := _wobble(after, ring)
	var rb := float(wb[1]) / maxf(1.0, wb[0])
	var ra := float(wa[1]) / maxf(1.0, wa[0])
	print("FAUBGRAIN %s: WOBBLE neighbouring back lines differing > 0.4 m: before %d/%d (%.0f%%), after %d/%d (%.0f%%)" % [
		tag, wb[1], wb[0], rb * 100.0, wa[1], wa[0], ra * 100.0])
	_ok(wa[0] >= 5 and ra > rb + 0.25, "%s: the back lines of a row wobble after and did not before" % tag)

	var gb := _gradient(before)
	var ga := _gradient(after)
	print("FAUBGRAIN %s: GRADIENT built share near-gate third / far third: before %.3f / %.3f, after %.3f / %.3f (%d lots)" % [
		tag, gb[0], gb[1], ga[0], ga[1], ga[2]])
	_ok(ga[0] > ga[1] * 1.2, "%s: near the gate the faubourg is more built than far from it (after, x1.2)" % tag)
	_ok(ga[0] - ga[1] > gb[0] - gb[1], "%s: that gap is wider than before (the before is the control)" % tag)

	# -- pixels ------------------------------------------------------------------
	var lo := Vector2(INF, INF)
	var hi := Vector2(-INF, -INF)
	for p in ring:
		lo = lo.min(p)
		hi = hi.max(p)
	var pad := (hi - lo) * 0.15
	var box := Rect2(lo - pad, hi - lo + pad * 2.0)
	(await _render(before, box)).save_png("res://_faubgrain_%s_before.png" % tag)
	(await _render(after, box)).save_png("res://_faubgrain_%s_after.png" % tag)
	var samples: Array = []
	var bds: PackedStringArray = after["building_district"]
	var abs_: Array = after["buildings"]
	for i in abs_.size():
		if bds[i] == "faubourg":
			samples.append(_centroid(abs_[i]))
	var ctrls: Array = []
	for i in first_a:
		var c := _centroid(ap[i])
		if _ring_dist(c, ring) >= 60.0 and Geometry2D.is_point_in_polygon(c, ring):
			ctrls.append(c)
	var moved := 0
	var ctrl_moved := 0
	var tx := box.position.x
	while tx < box.end.x:
		var ty := box.position.y
		while ty < box.end.y:
			var tile := Rect2(tx, ty, TILE, TILE)
			var mine: Array = samples.filter(func(q): return tile.has_point(q))
			var mine_c: Array = ctrls.filter(func(q): return tile.has_point(q))
			if not mine.is_empty() or not mine_c.is_empty():
				var a: Image = await _render(after, tile)
				var b: Image = await _render(before, tile)
				for q in mine:
					if _px(a, q) != _px(b, q):
						moved += 1
				for q in mine_c:
					if _px(a, q) != _px(b, q):
						ctrl_moved += 1
			ty += TILE
		tx += TILE
	print("FAUBGRAIN %s: faubourg building centroids changed %d/%d; interior control points changed %d/%d" % [
		tag, moved, samples.size(), ctrl_moved, ctrls.size()])
	_ok(float(moved) / maxf(1.0, samples.size()) > 0.5,
		"%s: the faubourg is redrawn -- >50%% of its building centroids change" % tag)
	_ok(ctrls.size() > 0 and ctrl_moved == 0,
		"%s: CONTROL -- no interior pixel 60 m in from the wall moves (%d points)" % [tag, ctrls.size()])

	# Close-ups for a human: the densest faubourg cluster, before and after.
	if fa.is_empty():
		return
	var best := -1
	var best_n := -1
	for i in fa:
		var c0 := _centroid(ap[i])
		var n := 0
		for k in fa:
			if _centroid(ap[k]).distance_to(c0) < 80.0:
				n += 1
		if n > best_n:
			best_n = n
			best = i
	var c1 := _centroid(ap[best])
	var clo := Vector2(INF, INF)
	var chi := Vector2(-INF, -INF)
	for k in fa:
		if _centroid(ap[k]).distance_to(c1) > 140.0:
			continue
		for q in (ap[k] as PackedVector2Array):
			clo = clo.min(q)
			chi = chi.max(q)
	var cbox := Rect2(clo - Vector2(30, 30), chi - clo + Vector2(60, 60))
	var side := maxf(cbox.size.x, cbox.size.y)
	cbox = Rect2(cbox.get_center() - Vector2(side, side) * 0.5, Vector2(side, side))
	(await _render(before, cbox)).save_png("res://_faubgrain_%s_cluster_before.png" % tag)
	(await _render(after, cbox)).save_png("res://_faubgrain_%s_cluster_after.png" % tag)


func _ready() -> void:
	size = get_viewport_rect().size
	for k in TOWNS:
		await _probe_town("town%d" % k)
	print("FAUBGRAIN %s (%d failed)" % ["ALL PASS" if _fails == 0 else "SOME FAILED", _fails])
	get_tree().quit(1 if _fails > 0 else 0)
