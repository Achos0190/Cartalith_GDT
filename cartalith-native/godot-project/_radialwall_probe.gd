extends Control
## Windowed proof for Ruling AA (owner, 2026-09-23): the radial (Venus) plan
## now carries the wall lots and the faubourg the organic plan got from
## `cartalith_urban::wallside` -- WITHOUT moving its wedge blocks.
##
## The shipping app cannot ask for a Venus town (the urban adapter passes no
## culture -- `urban_adapter.rs`' header), so each town is written by
## `cargo run -p cartalith-urban --example town_json` TWICE -- once on the tree
## before the change (`*_before.json`) and once after (`*_after.json`) -- and
## drawn here by the real `UrbanLayoutDraw.draw_layout`. Regenerate with:
##
##   EX=target/debug/examples/town_json.exe
##   $EX 31337 venus 8000 - godot-project/_radialwall_town0_<phase>.json   (golden venusRadial)
##   $EX 818   venus 1800 - godot-project/_radialwall_town1_<phase>.json   (golden venusSmall)
##   $EX 1414  venus 1000 - godot-project/_radialwall_town2_<phase>.json   (golden venusTinyCanal)
##   $EX 42    venus 12000 inland godot-project/_radialwall_town3_<phase>.json
##
## Data half, per town: the streets and the wedge blocks are identical before
## and after; the before lots are an exact prefix of the after lots; before has
## no faubourg, after has one wholly outside the wall; a strip 3 m outside the
## wall face is more covered after but under 60% (the gap stays open where no
## faubourg stands); and the faubourg thins off the wall -- its 10 m band
## against the wall is the fullest and its outermost holds under half of that.
##
## Pixel half, "flip the flag and diff the framebuffer": the after town drawn
## against the before town. Sample points 2.5 m inside every new lot's back
## line must change; points deep inside the town must not (the control).
## Windowed, because `frame_post_draw` never delivers under `--headless`.
##
## Godot_v4.7.1-stable_win64_console.exe --path . _radialwall_probe.tscn

const TOWNS := 4
## The curtain's half-stroke (`WALL_W["curtain"]` 4.5 / 2), a literal.
const FACE := 2.25
const TILE := 200.0
const BAND := 10.0

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


## JSON arrays into the packed types `draw_layout` reads.
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


func _bmap(par: PackedVector2Array, u: float, v: float) -> Vector2:
	return par[0].lerp(par[1], u).lerp(par[3].lerp(par[2], u), v)


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


## Fraction of points `off` metres from the wall's centreline (outward when
## `off > 0`), every 2 m along the ring, that fall inside any lot.
func _strip_cover(ring: PackedVector2Array, parcels: Array, off: float) -> float:
	var n := 0
	var hit := 0
	var c := _centroid(ring)
	for i in ring.size():
		var a := ring[i]
		var b := ring[(i + 1) % ring.size()]
		var steps := int(a.distance_to(b) / 2.0)
		var nrm := Vector2(-(b - a).y, (b - a).x).normalized()
		if Geometry2D.is_point_in_polygon(a.lerp(b, 0.5) + nrm * 3.0, ring):
			nrm = -nrm
		for s in steps:
			var p := a.lerp(b, (s + 0.5) / steps) + nrm * off
			n += 1
			for par in parcels:
				if Geometry2D.is_point_in_polygon(p, par):
					hit += 1
					break
	return float(hit) / maxf(1.0, n)


func _same_polys(a: Array, b: Array) -> bool:
	if a.size() != b.size():
		return false
	for i in a.size():
		if a[i] != b[i]:
			return false
	return true


func _probe_town(tag: String) -> void:
	var before := _load("res://_radialwall_%s_before.json" % tag)
	var after := _load("res://_radialwall_%s_after.json" % tag)
	if before.is_empty() or after.is_empty():
		return
	var ring: PackedVector2Array = after.get("wall_ring", PackedVector2Array())
	_ok(ring.size() >= 6, "%s: walled" % tag)
	if ring.size() < 6:
		return
	var bp: Array = before["parcels"]
	var ap: Array = after["parcels"]
	var ad: PackedStringArray = after["parcel_district"]
	print("RADIALWALL %s: %d -> %d lots, %d -> %d buildings, %d blocks, %d land gates" % [
		tag, bp.size(), ap.size(), (before["buildings"] as Array).size(),
		(after["buildings"] as Array).size(), (after["blocks"] as Array).size(),
		(after.get("wall_gates", PackedVector2Array()) as PackedVector2Array).size()])

	# -- the wedge blocks do not move ------------------------------------------
	var streets_same := true
	for k in after["streets"]:
		if not before["streets"].has(k) or before["streets"][k] != after["streets"][k]:
			streets_same = false
	_ok(streets_same and before["streets"].size() == after["streets"].size(),
		"%s: every street segment is identical before and after" % tag)
	_ok(_same_polys(before["blocks"], after["blocks"]),
		"%s: the %d wedge blocks are identical before and after" % [tag, (after["blocks"] as Array).size()])
	_ok(_same_polys(bp, ap.slice(0, bp.size())),
		"%s: the %d lots the reference platted are an exact prefix of the new list" % [tag, bp.size()])

	# -- a faubourg outside, thinning off the wall ------------------------------
	var faub_before := (before["parcel_district"] as PackedStringArray).count("faubourg")
	var faub := 0
	var faub_out := 0
	var bands := {}
	for i in range(bp.size(), ap.size()):
		if ad[i] != "faubourg":
			continue
		faub += 1
		var par: PackedVector2Array = ap[i]
		var all_out := true
		for q in par:
			if Geometry2D.is_point_in_polygon(q, ring):
				all_out = false
		if all_out:
			faub_out += 1
		var d := _ring_dist(par[3].lerp(par[2], 0.5), ring) - FACE
		var band := int(floor((d + 1.0) / BAND))
		bands[band] = int(bands.get(band, 0)) + 1
	var keys := bands.keys()
	keys.sort()
	var txt := PackedStringArray()
	# Not strictly monotone: a town has up to three clusters of 3-5 rows each,
	# so the far bands sum clusters of different depths (measured 2026-09-23,
	# town3: 30-40 m holds 4 lots and 40-50 m holds 6). What `wallside`
	# promises is the shape: fullest against the wall, and the outermost band
	# well under half of that.
	var thinning := keys.size() >= 2
	for k in keys.size():
		txt.append("%d-%dm:%d" % [keys[k] * BAND, (keys[k] + 1) * BAND, bands[keys[k]]])
		if int(bands[keys[k]]) > int(bands[keys[0]]):
			thinning = false
	if keys.size() >= 2 and int(bands[keys[-1]]) * 2 >= int(bands[keys[0]]):
		thinning = false
	print("RADIALWALL %s: faubourg %d -> %d lots; by distance off the wall face: %s" % [
		tag, faub_before, faub, ", ".join(txt)])
	_ok(faub_before == 0 and faub > 0, "%s: no faubourg before, one after" % tag)
	_ok(faub_out == faub, "%s: every faubourg lot is wholly outside the wall" % tag)
	_ok(thinning, "%s: the faubourg thins off the wall -- >= 2 bands, the first the fullest, the last under half of it" % tag)

	# -- the gap: the strip outside the wall stays mostly open --------------------
	var out_b := _strip_cover(ring, bp, FACE + 3.0)
	var out_a := _strip_cover(ring, ap, FACE + 3.0)
	var in_b := _strip_cover(ring, bp, -(FACE + 3.0))
	var in_a := _strip_cover(ring, ap, -(FACE + 3.0))
	print("RADIALWALL %s: strip 3 m outside the wall face covered %.1f%% -> %.1f%%; 3 m inside %.1f%% -> %.1f%%" % [
		tag, out_b * 100.0, out_a * 100.0, in_b * 100.0, in_a * 100.0])
	# Not `out_b == 0`: on a river site the ring follows the bank, and the
	# harbour's quay lots already stand past it (town2: 4.7% before).
	_ok(out_a > out_b and out_a < 0.6,
		"%s: outside the wall, built against in places and a gap kept elsewhere" % tag)
	_ok(in_a > in_b, "%s: more of the wall's inner face has a lot against it" % tag)

	# -- pixels ------------------------------------------------------------------
	var lo := Vector2(INF, INF)
	var hi := Vector2(-INF, -INF)
	for p in ring:
		lo = lo.min(p)
		hi = hi.max(p)
	var pad := (hi - lo) * 0.12
	var box := Rect2(lo - pad, hi - lo + pad * 2.0)
	(await _render(before, box)).save_png("res://_radialwall_%s_before.png" % tag)
	(await _render(after, box)).save_png("res://_radialwall_%s_after.png" % tag)
	var samples: Array = []
	for i in range(bp.size(), ap.size()):
		var par: PackedVector2Array = ap[i]
		var depth := (par[0].distance_to(par[3]) + par[1].distance_to(par[2])) * 0.5
		samples.append(_bmap(par, 0.5, 1.0 - minf(0.5, 2.5 / maxf(depth, 1.0))))
	var ctrls: Array = []
	for i in bp.size():
		var c := _centroid(bp[i])
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
	print("RADIALWALL %s: at %.1f px/m, new-lot samples changed %d/%d; interior control points changed %d/%d" % [
		tag, _scale, moved, samples.size(), ctrl_moved, ctrls.size()])
	_ok(float(moved) / maxf(1.0, samples.size()) > 0.8,
		"%s: the new lots are drawn -- >80%% of their samples change" % tag)
	_ok(ctrls.size() > 0 and ctrl_moved == 0,
		"%s: CONTROL -- no interior pixel 60 m in from the wall moves (%d points)" % [tag, ctrls.size()])

	# A close-up for a human: the first faubourg cluster, before and after.
	var first := -1
	for i in range(bp.size(), ap.size()):
		if ad[i] == "faubourg":
			first = i
			break
	if first >= 0:
		var c0 := _centroid(ap[first])
		var clo := Vector2(INF, INF)
		var chi := Vector2(-INF, -INF)
		for i in range(bp.size(), ap.size()):
			if ad[i] != "faubourg" or _centroid(ap[i]).distance_to(c0) > 120.0:
				continue
			for q in (ap[i] as PackedVector2Array):
				clo = clo.min(q)
				chi = chi.max(q)
		var cbox := Rect2(clo - Vector2(40, 40), chi - clo + Vector2(80, 80))
		var side := maxf(cbox.size.x, cbox.size.y)
		cbox = Rect2(cbox.get_center() - Vector2(side, side) * 0.5, Vector2(side, side))
		(await _render(before, cbox)).save_png("res://_radialwall_%s_cluster_before.png" % tag)
		(await _render(after, cbox)).save_png("res://_radialwall_%s_cluster_after.png" % tag)


func _ready() -> void:
	size = get_viewport_rect().size
	for k in TOWNS:
		await _probe_town("town%d" % k)
	print("RADIALWALL %s (%d failed)" % ["ALL PASS" if _fails == 0 else "SOME FAILED", _fails])
	get_tree().quit(1 if _fails > 0 else 0)
