extends SceneTree
## Ruling FS (owner, 2026-11-17): protects `urban_layout_draw.gd::_fill_ground_polygon`'s handling of a self-intersecting
## outline. A river town's `water_poly` is its centreline offset half the river's width each side
## (`cartalith_urban::site::build_site`); where the river turns tighter than that half-width the band crosses itself,
## the triangulator rejects it, and the old triangle fan from vertex 0 painted the inside of the bend as open water
## (246371 #11 at 384 x 288, unstretched: 877 ha drawn from a 22 ha band; evidence
## `cartalith-native/docs/evidence/river-stretch-2026-11-16/fs/`).
##
## Usage (no scene, no extension needed):
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . -s _bandfill_probe.gd
## Prints PASS/FAIL per check and `RESULT: PASS|FAIL`; exits 1 on any failure.
##
## Checks, each on a band built exactly as `build_site` builds one:
## - a U-shaped river with a hairpin spike in its base (the band crosses itself at the spike): the triangulator rejects
##   it (else the case tests nothing); the old fan covers a point in the U's dry middle (the positive control: the
##   defect reproduces); `_simple_pieces` covers no dry sample point and every on-band sample point, and every piece
##   triangulates;
## - a river that loops round and crosses its own first reach (a hole of dry land inside the loop), through
##   `_simple_pieces`: the loop's middle is not covered, the band is;
## - a keyhole outline whose union is a ring with a hole (the hole branch): the hole is not covered, the ring is;
## - a simple band still triangulates as it is, so `_fill_ground_polygon` takes its first branch and draws it unchanged;
## - a sub-pixel quad (the case the fan exists for) still draws.
## Tested through `_fill_parts`, the draw list `_fill_ground_polygon` draws, so every branch is reached.

const DRAW := preload("res://shell/urban_layout_draw.gd")

var _fails := 0
var _checks := 0


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print(("  PASS  " if cond else "  FAIL  ") + what)


## The band `build_site` draws for centreline `river` and width `w`: left offsets, then the right ones reversed. The
## normal at vertex i is the direction from i-1 to i+1 turned a quarter, as there.
static func band(river: PackedVector2Array, w: float) -> PackedVector2Array:
	var left := PackedVector2Array()
	var right := PackedVector2Array()
	var n := river.size()
	for i in n:
		var d := (river[mini(i + 1, n - 1)] - river[maxi(i - 1, 0)]).normalized()
		var nl := Vector2(-d.y, d.x)
		left.append(river[i] + nl * (w / 2.0))
		right.append(river[i] - nl * (w / 2.0))
	right.reverse()
	left.append_array(right)
	return left


## Whether `p` is inside any of `pieces`.
static func covered(pieces: Array, p: Vector2) -> bool:
	for q in pieces:
		if Geometry2D.is_point_in_polygon(p, q):
			return true
	return false


## The old fallback's triangles (a fan from vertex 0), as polygons, for the positive control.
static func fan(pts: PackedVector2Array) -> Array:
	var out: Array = []
	for i in range(1, pts.size() - 1):
		out.append(PackedVector2Array([pts[0], pts[i], pts[i + 1]]))
	return out


## Distance from `p` to polyline `pl`.
static func line_dist(p: Vector2, pl: PackedVector2Array) -> float:
	var best := INF
	for i in pl.size() - 1:
		best = minf(best, p.distance_to(Geometry2D.get_closest_point_to_segment(p, pl[i], pl[i + 1])))
	return best


func _init() -> void:
	var w := 30.0
	# 1. A U: down the left arm, along the base with a hairpin spike up into the U, up the right arm. Every 10 m.
	var u := PackedVector2Array()
	for y in range(0, 401, 10):
		u.append(Vector2(0, y))
	for x in range(10, 141, 10):
		u.append(Vector2(x, 400))
	u.append_array(PackedVector2Array([Vector2(146, 400), Vector2(149, 330), Vector2(150, 300), Vector2(151, 330), Vector2(154, 400), Vector2(160, 400)]))
	for x in range(170, 301, 10):
		u.append(Vector2(x, 400))
	for y in range(390, -1, -10):
		u.append(Vector2(300, y))
	var ub := band(u, w)
	_ok(Geometry2D.triangulate_polygon(ub).is_empty(), "the U band with a hairpin crosses itself: the triangulator rejects it (%d points)" % ub.size())
	var dry := [Vector2(150, 150), Vector2(80, 200), Vector2(220, 250), Vector2(150, 60)]
	var fan_hits := dry.filter(func(p: Vector2) -> bool: return covered(fan(ub), p)).size()
	_ok(fan_hits > 0, "positive control: the old fan covers %d of %d dry points in the U's middle" % [fan_hits, dry.size()])
	var ufill: Dictionary = DRAW._fill_parts(ub)
	_ok(ufill["fan"].is_empty(), "the U band is drawn without the fan")
	var pieces: Array = ufill["polygons"]
	for q in pieces:
		print("    piece %d points, area %.0f, triangulates %s" % [q.size(), DRAW._area(q), str(not Geometry2D.triangulate_polygon(q).is_empty())])
	_ok(not pieces.is_empty() and pieces.all(func(q: PackedVector2Array) -> bool: return not Geometry2D.triangulate_polygon(q).is_empty()), "the U band resolves into %d pieces, each triangulates" % pieces.size())
	_ok(dry.all(func(p: Vector2) -> bool: return not covered(pieces, p)), "no dry point in the U's middle is covered")
	# On-band samples: every fifth centreline point, off the spike (whose crossing the union may fold).
	var wet: Array = []
	for i in range(0, u.size(), 5):
		if u[i].distance_to(Vector2(150, 350)) > 70.0:
			wet.append(u[i])
	_ok(wet.all(func(p: Vector2) -> bool: return covered(pieces, p)), "every one of %d centreline samples is covered" % wet.size())
	# Nothing farther than the half-width (plus a metre of offset slack) from the centreline is covered: sample a grid.
	var stray := 0
	for gx in range(-40, 341, 8):
		for gy in range(-40, 441, 8):
			var p := Vector2(gx, gy)
			if covered(pieces, p) and line_dist(p, u) > w / 2.0 + 1.0:
				stray += 1
	_ok(stray == 0, "no grid point farther than half the width from the river is covered (%d)" % stray)

	# 2. A loop: a river runs east, turns north, west and south and crosses its own first reach, leaving a square of
	# dry land inside (x -50..100, y -150..0). Every 10 m.
	var lp := PackedVector2Array()
	for x in range(-200, 101, 10):
		lp.append(Vector2(x, 0))
	for y in range(-10, -151, -10):
		lp.append(Vector2(100, y))
	for x in range(90, -51, -10):
		lp.append(Vector2(x, -150))
	for y in range(-140, 101, 10):
		lp.append(Vector2(-50, y))
	var lb := band(lp, w)
	# Godot's triangulator accepts this band (an ear clip can succeed on a crossing outline), so it is resolved
	# through `_simple_pieces` itself, the function `_fill_parts` calls on a rejected outline.
	var lpieces: Array = DRAW._simple_pieces(lb)
	_ok(not covered(lpieces, Vector2(25, -75)) and not covered(lpieces, Vector2(-20, -40)) and not covered(lpieces, Vector2(70, -120)), "the dry middle of the loop is not covered")
	_ok(covered(lpieces, Vector2(-150, 0)) and covered(lpieces, Vector2(100, -80)) and covered(lpieces, Vector2(20, -150)) and covered(lpieces, Vector2(-50, 60)), "the band round the loop is covered")
	_ok(lpieces.all(func(q: PackedVector2Array) -> bool: return not Geometry2D.triangulate_polygon(q).is_empty()), "every loop piece triangulates (%d)" % lpieces.size())

	# 2b. A keyhole: one outline round a 100 m square, in along a zero-width cut and round a 50 m square inside it the
	# other way, back out. Its union is a ring with a hole, which reaches the hole branch: the middle must stay dry.
	var key := PackedVector2Array([Vector2(0, 0), Vector2(100, 0), Vector2(100, 100), Vector2(0, 100), Vector2(0, 0),
		Vector2(25, 25), Vector2(25, 75), Vector2(75, 75), Vector2(75, 25), Vector2(25, 25)])
	var kmerged: Array = Geometry2D.merge_polygons(key, PackedVector2Array())
	var kwind := kmerged.map(func(q: PackedVector2Array) -> bool: return Geometry2D.is_polygon_clockwise(q))
	_ok(kwind.has(true) and kwind.has(false), "the keyhole's union is a ring with a hole (%d rings), so the hole branch is reached" % kmerged.size())
	var kpieces: Array = DRAW._simple_pieces(key)
	_ok(not covered(kpieces, Vector2(50, 50)) and not covered(kpieces, Vector2(30, 70)) and not covered(kpieces, Vector2(70, 30)), "the keyhole's middle is not covered")
	_ok(covered(kpieces, Vector2(10, 50)) and covered(kpieces, Vector2(90, 50)) and covered(kpieces, Vector2(50, 10)) and covered(kpieces, Vector2(50, 90)), "the keyhole's ring is covered on all four sides")
	_ok(kpieces.all(func(q: PackedVector2Array) -> bool: return not Geometry2D.triangulate_polygon(q).is_empty()), "every keyhole piece triangulates (%d)" % kpieces.size())

	# 3. A gentle river: the band is simple and triangulates as it is (the first branch; nothing changes for it).
	var g := PackedVector2Array()
	for x in range(0, 401, 10):
		g.append(Vector2(x, 40.0 * sin(x / 80.0)))
	var gb := band(g, w)
	var gfill: Dictionary = DRAW._fill_parts(gb)
	_ok(gfill["fan"].is_empty() and gfill["polygons"].size() == 1 and gfill["polygons"][0] == gb, "a gentle river's band is drawn as it is, one polygon")
	# 4. The sub-pixel case the fan exists for (`_fill_ground_polygon`'s 2026-09 history): a quad 0.05 px across at an
	# offset near (276, 33) still draws. On Godot 4.7.2 the triangulator accepts it here, so this guards that it is not
	# dropped; no constructed input was found that reaches the fan itself (measured 2026-11-17: quads down to 1e-5 px
	# and a collinear outline all triangulate), so the fan branch stays as the September fallback, untested.
	var tiny := PackedVector2Array([Vector2(276.0, 33.0), Vector2(276.04, 33.0), Vector2(276.05, 33.03), Vector2(276.0, 33.02)])
	var tfill: Dictionary = DRAW._fill_parts(tiny)
	_ok(not (tfill["polygons"].is_empty() and tfill["fan"].is_empty()), "a sub-pixel quad still draws (%d polygons, %d fan triangles)" % [tfill["polygons"].size(), tfill["fan"].size()])

	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	quit(1 if _fails > 0 else 0)
