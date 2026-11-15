extends RefCounted
## Smooth sea and lake shorelines for the town layouts (owner, 2026-11-15: "Sea and lake should not draw as
## squares"; OUTSTANDING_WORK.md, City Viewer river-town sub-bullet). DRAWING ONLY.
##
## `layout["water_mask_runs"]` is the engine's own water mask (`cartalith_urban::site::WaterCtx::water_runs`): one
## axis-aligned quad per map-cell row run, in the layout's LOCAL box frame, about 22 m a cell. Drawn as rectangles
## it gives every shore a staircase. This file turns the SAME runs into smooth filled polygons:
##
##   1. the runs are laid back on their cell grid (cell size = the runs' own row height);
##   2. the grid's boundary is traced as closed rings of UNIT steps (outer rings and island rings alike);
##   3. each ring is relaxed by `SMOOTH_PASSES` neighbour-averaging passes (each vertex held within
##      `PULL_LIMIT_CELLS` of its traced position) and finished with `CHAIKIN_PASSES` of corner cutting, which turns
##      the staircase into a smooth curve;
##   4. outer rings are filled, islands subtracted (`_subtract` cuts a notch so no polygon carries a hole,
##      since `draw_colored_polygon` has none), and the bridge/ford release discs are subtracted the same way;
##   5. the pieces are triangulated once into an `ArrayMesh`, cached on the layout, and drawn with one
##      `draw_mesh`, so panning and zooming re-run no geometry.
##
## **It must never change what counts as water.** Building placement, `fit_with_water` and every "is this water"
## query read `water_mask_runs` itself (the engine decided placement before any of this ran); the polygons here are
## derived, drawn and cached, and nothing reads them back except the draw and `_shorepoly_probe.gd`. The smoothed
## shore must stay within `MAX_DEVIATION_CELLS` of the traced boundary (asserted by that probe).
##
## No `class_name`: preloaded by `urban_layout_draw.gd`, so no global-class registration (an import pass) is needed.

## Averaging passes per ring (a [1,2,1]/4 neighbour filter on the unit-step ring, i.e. a Gaussian that lets a
## staircase of steps several cells long relax into the straight line through it). LABELLED JUDGEMENT (2026-11-15):
## 8 passes spread a corner over about 4 cells each way, enough to flatten the 2-3 cell steps the real coasts show
## (measured on seed 483920 at 2 048: the Chaikin-only first version left them visibly stepped), while the clamp
## below keeps every vertex within `PULL_LIMIT_CELLS` of where the mask put it. Mutant `SMOOTH_PASSES = 0` must fail
## the probe (the literal staircase stays a staircase).
const SMOOTH_PASSES := 8

## How far a vertex may move from its traced position, in cells. LABELLED JUDGEMENT: 0.3. The owner's bound is half a cell
## (`MAX_DEVIATION_CELLS`) BOTH ways (smooth to traced, and a traced corner to the smooth shore); 0.4 measured 0.54 for
## the second on a 90-degree corner, 0.3 leaves margin for the Chaikin finish; a straight line through a staircase of any
## step length stays within half a cell of every corner, so a long run flattens fully.
const PULL_LIMIT_CELLS := 0.3

## Chaikin corner-cutting passes run AFTER the averaging, only to round what the filter left (the genuine corners
## of a rectangular lake, a box edge). LABELLED JUDGEMENT: 2, which adds under 0.1 cell of deviation to a ring the
## filter already flattened.
const CHAIKIN_PASSES := 2

## The bound the smoothed shore must keep from the traced boundary, in cells. LABELLED JUDGEMENT: half a cell is the
## owner-set limit for the task (the mask's own resolution; anything inside it is within the rasterisation error of
## where the engine put the shore). Chaikin on unit edges cannot exceed about 0.25 cell, so this has headroom.
const MAX_DEVIATION_CELLS := 0.5

## Largest grid (cells) this will trace; a bigger mask falls back to the old rectangles. LABELLED JUDGEMENT: a town
## box is a few hundred cells a side (a 4 M-cell grid is ~2000 x 2000, a size no layout reaches).
const MAX_GRID_CELLS := 4000000

## Release discs are 24-gons, LABELLED JUDGEMENT: the old code released an exact circle's chord per row; 24 sides
## stay within 0.9% of the radius, far inside the bridge approach's own slack.
const RELEASE_SIDES := 24

const CACHE_KEY := "_shore_cache"


## The traced shoreline of `runs`, before smoothing: `{"cell": float, "rings": Array[PackedVector2Array]}` in the
## layout's local frame, every ring a closed loop (first vertex not repeated) of unit-cell steps. Empty `rings` when
## there is nothing to trace or the grid is over `MAX_GRID_CELLS`. Pure; the probe measures the smooth shore against it.
static func trace_rings(runs: Array) -> Dictionary:
	var cell := INF
	var ox := INF
	var oy := INF
	var ex := -INF
	var ey := -INF
	for run in runs:
		var q: PackedVector2Array = run
		if q.size() < 4:
			continue
		cell = minf(cell, q[2].y - q[0].y)
		ox = minf(ox, q[0].x)
		oy = minf(oy, q[0].y)
		ex = maxf(ex, q[2].x)
		ey = maxf(ey, q[2].y)
	if cell == INF or cell <= 0.0:
		return {"cell": 1.0, "rings": []}
	var gw := int(round((ex - ox) / cell))
	var gh := int(round((ey - oy) / cell))
	if gw <= 0 or gh <= 0 or gw * gh > MAX_GRID_CELLS:
		return {"cell": cell, "rings": []}
	var grid := PackedByteArray()
	grid.resize(gw * gh)
	for run in runs:
		var q: PackedVector2Array = run
		if q.size() < 4:
			continue
		var i0 := int(round((q[0].x - ox) / cell))
		var i1 := int(round((q[2].x - ox) / cell))
		var j0 := int(round((q[0].y - oy) / cell))
		var j1 := maxi(int(round((q[2].y - oy) / cell)), j0 + 1)
		for j in range(j0, j1):
			for i in range(i0, i1):
				grid[j * gw + i] = 1
	## Directed unit edges, water on the right-hand side walking in y-down screen space (clockwise round a lone cell).
	## Keyed by start corner; a corner touched by two diagonal cells has two outgoing edges, taken in turn.
	var out_edges := {}
	var stride := gw + 1
	for j in gh:
		for i in gw:
			if grid[j * gw + i] == 0:
				continue
			if j == 0 or grid[(j - 1) * gw + i] == 0:
				_add_edge(out_edges, j * stride + i, j * stride + i + 1)
			if i == gw - 1 or grid[j * gw + i + 1] == 0:
				_add_edge(out_edges, j * stride + i + 1, (j + 1) * stride + i + 1)
			if j == gh - 1 or grid[(j + 1) * gw + i] == 0:
				_add_edge(out_edges, (j + 1) * stride + i + 1, (j + 1) * stride + i)
			if i == 0 or grid[j * gw + i - 1] == 0:
				_add_edge(out_edges, (j + 1) * stride + i, j * stride + i)
	var rings: Array = []
	for start in out_edges.keys():
		while not (out_edges[start] as Array).is_empty():
			var ring := PackedVector2Array()
			var cur: int = start
			while true:
				ring.append(Vector2(ox + float(cur % stride) * cell, oy + float(cur / stride) * cell))
				var nexts: Array = out_edges[cur]
				if nexts.is_empty():
					break
				var nxt: int = nexts.pop_back()
				cur = nxt
				if cur == start:
					break
			if ring.size() >= 4:
				rings.append(ring)
	return {"cell": cell, "rings": rings}


static func _add_edge(edges: Dictionary, a: int, b: int) -> void:
	if not edges.has(a):
		edges[a] = []
	(edges[a] as Array).append(b)
	if not edges.has(b):
		edges[b] = []


## `passes` rounds of [1,2,1]/4 neighbour averaging on a closed ring, each vertex then pulled back to within `limit`
## (model metres) of its ORIGINAL position, so the relaxed ring never strays more than `limit` from the traced
## boundary whatever the pass count. With `closed` false the two end points stay put (an open path such as a shore
## polyline). Pure; fewer than 4 vertices comes back unchanged.
static func relax(ring: PackedVector2Array, limit: float, passes: int, closed: bool = true) -> PackedVector2Array:
	var n := ring.size()
	if n < 4 or passes <= 0:
		return ring
	var cur := ring
	for _p in passes:
		var nxt := PackedVector2Array()
		nxt.resize(n)
		for k in n:
			if not closed and (k == 0 or k == n - 1):
				nxt[k] = cur[k]
				continue
			var v := (cur[(k + n - 1) % n] + cur[k] * 2.0 + cur[(k + 1) % n]) * 0.25
			var off := v - ring[k]
			if off.length() > limit:
				v = ring[k] + off.normalized() * limit
			nxt[k] = v
		cur = nxt
	return cur


## One Chaikin pass: each edge P->P' yields the points a quarter and three quarters along it. A closed ring of n
## vertices becomes 2n; an open path keeps its end points. Pure.
static func chaikin(ring: PackedVector2Array, passes: int, closed: bool = true) -> PackedVector2Array:
	var cur := ring
	for _p in passes:
		var nxt := PackedVector2Array()
		var n := cur.size()
		if not closed and n > 0:
			nxt.append(cur[0])
		for k in (n if closed else n - 1):
			var a := cur[k]
			var b := cur[(k + 1) % n]
			nxt.append(a.lerp(b, 0.25))
			nxt.append(a.lerp(b, 0.75))
		if not closed and n > 0:
			nxt.append(cur[n - 1])
		cur = nxt
	return cur


## The smoothed rings of `runs`, outer rings and islands both: `{"cell", "raw", "smooth"}` with `raw` the traced
## rings and `smooth` the Chaikin'd ones in the same order. `passes` is a parameter so the probe can mutate it.
static func smooth_rings(runs: Array, passes: int = CHAIKIN_PASSES, smooth_passes: int = SMOOTH_PASSES) -> Dictionary:
	var t := trace_rings(runs)
	var smooth: Array = []
	for r in t["rings"]:
		smooth.append(chaikin(relax(r, float(t["cell"]) * PULL_LIMIT_CELLS, smooth_passes), passes))
	return {"cell": t["cell"], "raw": t["rings"], "smooth": smooth}


## Fillable pieces (no polygon holds a hole) covering the water the smoothed rings enclose, minus the discs of
## `release` (centre, radius pairs). A ring's nesting depth (how many other rings contain it) says whether it is an
## outer (even) or an island (odd); an island is subtracted from the outer one level up that contains it.
static func fill_pieces(smooth: Array, release: Array) -> Array:
	var depth: Array[int] = []
	for a in smooth.size():
		var d := 0
		var probe: Vector2 = (smooth[a] as PackedVector2Array)[0]
		for b in smooth.size():
			if a != b and Geometry2D.is_point_in_polygon(probe, smooth[b]):
				d += 1
		depth.append(d)
	var pieces: Array = []
	for a in smooth.size():
		if depth[a] % 2 != 0:
			continue
		var outer: PackedVector2Array = smooth[a]
		if Geometry2D.is_polygon_clockwise(outer):
			outer = _reversed(outer)
		var cuts: Array = []
		for b in smooth.size():
			if depth[b] == depth[a] + 1 and Geometry2D.is_point_in_polygon((smooth[b] as PackedVector2Array)[0], outer):
				cuts.append(smooth[b])
		var mine: Array = [outer]
		for cut in cuts:
			mine = _subtract(mine, cut, 0)
		for rel in release:
			mine = _subtract(mine, _disc(rel[0], float(rel[1])), 0)
		pieces.append_array(mine)
	return pieces


static func _reversed(p: PackedVector2Array) -> PackedVector2Array:
	var out := PackedVector2Array()
	for k in range(p.size() - 1, -1, -1):
		out.append(p[k])
	return out


static func _disc(c: Vector2, r: float) -> PackedVector2Array:
	var out := PackedVector2Array()
	for k in RELEASE_SIDES:
		var a := TAU * float(k) / float(RELEASE_SIDES)
		out.append(c + Vector2(cos(a), sin(a)) * r)
	return out


static func _bounds(p: PackedVector2Array) -> Rect2:
	var r := Rect2(p[0], Vector2.ZERO)
	for v in p:
		r = r.expand(v)
	return r


## Every piece of `pieces` minus polygon `s`, as simple polygons. When `s` lies wholly inside a piece the clip would
## return a hole, which cannot be drawn; the piece is first cut in two along the vertical line through `s`'s centre,
## so `s` straddles the cut and each half loses a notch instead. `depth` bounds that recursion (one cut suffices).
static func _subtract(pieces: Array, s: PackedVector2Array, depth: int) -> Array:
	var sb := _bounds(s)
	var out: Array = []
	for p in pieces:
		var pp: PackedVector2Array = p
		if not _bounds(pp).intersects(sb):
			out.append(pp)
			continue
		var holes := 0
		var solid: Array = []
		for r in Geometry2D.clip_polygons(pp, s):
			if Geometry2D.is_polygon_clockwise(r):
				holes += 1
			else:
				solid.append(r)
		if holes == 0 or depth >= 2:
			out.append_array(solid)
			continue
		var cx := sb.get_center().x
		var pb := _bounds(pp).grow(1.0)
		var left := PackedVector2Array([pb.position, Vector2(cx, pb.position.y), Vector2(cx, pb.end.y), Vector2(pb.position.x, pb.end.y)])
		var right := PackedVector2Array([Vector2(cx, pb.position.y), Vector2(pb.end.x, pb.position.y), pb.end, Vector2(cx, pb.end.y)])
		for half in [left, right]:
			var halves: Array = []
			for h in Geometry2D.intersect_polygons(pp, half):
				if not Geometry2D.is_polygon_clockwise(h):
					halves.append(h)
			out.append_array(_subtract(halves, s, depth + 1))
	return out


## The layout's cached shore, built on first use and kept on the layout dictionary under `CACHE_KEY` (a derived,
## underscore-named key nothing else reads). `{"mesh": ArrayMesh, "pieces": Array}`, or an empty dictionary when the
## mask could not be traced (the caller then draws the old rectangles). Rebuilt when the run count or the release
## points differ from the cached build. `passes` is a parameter so the probe can mutate it.
static func cached(layout: Dictionary, release: Array, passes: int = CHAIKIN_PASSES) -> Dictionary:
	var runs: Array = layout.get("water_mask_runs", [])
	var key := [runs.size(), release.hash(), passes]
	var hit: Dictionary = layout.get(CACHE_KEY, {})
	if not hit.is_empty() and hit["key"] == key:
		return hit
	var s := smooth_rings(runs, passes)
	var pieces := fill_pieces(s["smooth"], release)
	var verts := PackedVector2Array()
	var idx := PackedInt32Array()
	for p in pieces:
		var tri := Geometry2D.triangulate_polygon(p)
		if tri.is_empty():
			continue
		var base := verts.size()
		verts.append_array(p)
		for t in tri:
			idx.append(base + t)
	var built := {"key": key, "pieces": pieces, "rings": s, "mesh": null}
	if not idx.is_empty():
		var arrays := []
		arrays.resize(Mesh.ARRAY_MAX)
		arrays[Mesh.ARRAY_VERTEX] = verts
		arrays[Mesh.ARRAY_INDEX] = idx
		var mesh := ArrayMesh.new()
		mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
		built["mesh"] = mesh
	layout[CACHE_KEY] = built
	return built


## Draws the cached shore filled with `color`. `project` is the layout's local-to-screen Callable (it takes a
## PackedVector2Array); it is affine for every caller (scale, rotation, pan), so the mesh is drawn with the
## transform recovered from the images of the origin and the two unit vectors. Returns false when there is nothing
## to draw (no mesh), so the caller can fall back to the rectangles.
static func draw(ci: CanvasItem, layout: Dictionary, release: Array, project: Callable, color: Color) -> bool:
	var b := cached(layout, release)
	var mesh: ArrayMesh = b.get("mesh")
	if mesh == null:
		return false
	var o: PackedVector2Array = project.call(PackedVector2Array([Vector2.ZERO, Vector2.RIGHT, Vector2.DOWN]))
	ci.draw_mesh(mesh, null, Transform2D(o[1] - o[0], o[2] - o[0], o[0]), color)
	return true


## Share of a `water_poly`'s or `river`'s edges that must be axis-aligned or exactly 45 degrees for it to count as a
## traced mask staircase. LABELLED JUDGEMENT: 0.8; a traced cell path is all such edges bar a box clip, while a river
## channel or a synthetic body is almost none (its edges run at arbitrary angles), so the line sits well clear of both (measured by the probe on real bay towns).
const STAIRCASE_EDGE_SHARE := 0.8


## The layout's `water_poly` for DRAWING: see `_smooth_outline`. The stored polygon is never changed.
static func smooth_water_poly(layout: Dictionary, passes: int = CHAIKIN_PASSES, smooth_passes: int = SMOOTH_PASSES) -> PackedVector2Array:
	return _smooth_outline(layout, "water_poly", true, passes, smooth_passes)


## The layout's `river` centreline for DRAWING: on a coastal or lake site the engine's `river` polyline is the traced
## shoreline (stroked as the ink plan's water edge), a staircase at the mask's cell; a real river's is not. See
## `_smooth_outline`. The stored polyline is never changed.
static func smooth_river(layout: Dictionary, passes: int = CHAIKIN_PASSES, smooth_passes: int = SMOOTH_PASSES) -> PackedVector2Array:
	return _smooth_outline(layout, "river", false, passes, smooth_passes)


## `layout[key]` for drawing: when the layout has mask runs (so a cell size is known) and the polygon or polyline is
## the engine's traced staircase (more than `STAIRCASE_EDGE_SHARE` of its edges axis-aligned or exactly 45 degrees, which is how a traced
## cell path runs, 4- or 8-connected; a river channel or a
## synthetic body has almost none), the same relax-then-Chaikin smoothing as `smooth_rings` is applied; otherwise it
## comes back untouched. Edges are first cut to at most a cell so the filter has unit steps; every vertex stays within
## `PULL_LIMIT_CELLS` of the original outline. Cached on the layout under `"_shore_cache_" + key`.
## `passes`/`smooth_passes` are parameters so the probe can mutate them.
static func _smooth_outline(layout: Dictionary, key_name: String, closed: bool, passes: int, smooth_passes: int) -> PackedVector2Array:
	var poly: PackedVector2Array = layout.get(key_name, PackedVector2Array())
	var runs: Array = layout.get("water_mask_runs", [])
	if poly.size() < 4 or runs.is_empty():
		return poly
	var key := [poly.size(), runs.size(), passes, smooth_passes]
	var ck := "_shore_cache_" + key_name
	var hit: Dictionary = layout.get(ck, {})
	if not hit.is_empty() and hit["key"] == key:
		return hit["poly"]
	var cell := INF
	for run in runs:
		var q: PackedVector2Array = run
		if q.size() >= 4:
			cell = minf(cell, q[2].y - q[0].y)
	var out := poly
	if cell != INF and cell > 0.0:
		var n := poly.size()
		var edges := n if closed else n - 1
		var axis := 0
		for k in edges:
			var d := poly[(k + 1) % n] - poly[k]
			if absf(d.x) < 0.001 or absf(d.y) < 0.001 or absf(absf(d.x) - absf(d.y)) < 0.001:
				axis += 1
		if float(axis) >= STAIRCASE_EDGE_SHARE * float(edges):
			var fine := PackedVector2Array()
			for k in edges:
				var a := poly[k]
				var b := poly[(k + 1) % n]
				var steps := maxi(1, int(ceil(a.distance_to(b) / cell - 0.001)))
				for t in steps:
					fine.append(a.lerp(b, float(t) / float(steps)))
			if not closed:
				fine.append(poly[n - 1])
			out = chaikin(relax(fine, cell * PULL_LIMIT_CELLS, smooth_passes, closed), passes, closed)
	layout[ck] = {"key": key, "poly": out}
	return out
