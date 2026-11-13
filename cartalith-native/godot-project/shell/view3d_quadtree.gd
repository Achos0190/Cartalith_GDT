extends Node3D
## No `class_name`, deliberately: `view3d.gd` preloads this file (3D-1's reason).

## **3D-3's terrain: the Standard and Full tiers' quadtree**
## (`3D_VIEWPORT_SCOPE.md` §5, milestone 3D-3; §3.2, §3.3, §3.5, §3.12, §3.13).
##
## What it draws: the pyramid's own tiles as a CDLOD quadtree. A node is a
## pyramid tile `(z, col, row)`; every drawn node is one instance of one shared
## `N x N` patch in ONE `MultiMesh` (one draw call), N from the gate's
## workload table (`View3DGate.WORKLOADS`: 33 phone, 65 desktop Standard, 129
## Full). Each node's tile colour, relief and vertex grid come from the LOD
## worker (`WorldGen.view3d_request_tile` / `view3d_take_ready`, §3.13: "requests
## go to `lod_worker`"), so a level change never synthesises on this thread.
##
## How a node is chosen (§3.3, §3.13): from the root down, a node is skipped
## when its box (footprint x the relief range of its own and its two parents'
## tiles) is outside the camera's frustum; it is split when the largest `L_3d`
## anywhere in it reaches `z + 0.5` (CDLOD's range condition, with EF-9's
## boost bounded by the boost field's maximum over the node) and all four
## children are resident; otherwise it is drawn, and its missing children are
## asked for. Children are asked for already when `L_3d` comes within
## `PREFETCH_LEVELS` of the split (§3.13 step 2). A node is never drawn
## without its tile, and its parent and grandparent tiles are resident by
## construction (they are on its path from the root): that is LOD-D3's parent
## fallback, carried to 3D.
##
## Zoom gains detail at every level (Ruling CI item 1): `L_3d` rises as the
## camera comes down, so nodes split level by level down to the deepest level
## the engine and the quality tier allow (`view3d_world()`'s `max_level`, the
## two caps 2D applies).
##
## Memory (§3.12): one `Texture2DArray` of colour tiles and one of vertex grids,
## `View3DGate.LAYERS[...]` layers each, allocated once; a tile out of use for
## `PARK_FRAMES` is the first evicted, and a tile in use this frame (every
## drawn node and its ancestors) never is.
##
## What it must never do:
## - change anything 2D draws: it reads tiles through the 3D-only worker queue
##   and writes no world state;
## - call the engine while a generate holds `WorldGen` (`readable`);
## - draw a node whose tile is not resident, or evict a tile a drawn node reads.

const QShader := preload("res://shell/view3d_quadtree.gdshader")
const View3DGate := preload("res://shell/view3d_gate.gd")

## How far below a split `L_3d` asks for the children already (§3.13 step 2,
## §8.5's `PREFETCH_LEVELS`), in levels.
const PREFETCH_LEVELS := 0.25
## The share of a level's band over which the CDLOD morph runs (shader uniform
## `morph_band`). A labelled judgement: an edge facing a coarser neighbour is
## fully morphed by construction (weight 0 there); a coarser node is unmorphed
## along that edge only when `L_3d` falls by at most `1 - morph_band` across a
## finer node, which at the desktop's focal length is about 0.35 level
## (`log2(1 + 226 / f_px)` at f_px 813), so 0.5 leaves room. Where it does not
## hold, the skirt covers the gap.
const MORPH_BAND := 0.5
## Frames a tile may sit unused before it is the first evicted (§3.13 step 3's
## park time). A labelled judgement: about two seconds at 60 Hz.
const PARK_FRAMES := 120
## Finished tiles uploaded per frame, at most, and the upload time after which
## no further one is started this frame (labelled judgements: one tile's
## mipmaps and two layer uploads cost about a millisecond here).
const TAKE_PER_FRAME := 4
## Frames after which a tile asked for and never landed is forgotten, so it can
## be asked again and stops holding room (the worker drops a job silently when
## the engine refuses a tile or the generation moved). A labelled judgement:
## about five seconds at 60 Hz, far beyond one tile's synthesis here.
const ASK_EXPIRE_FRAMES := 300
const UPLOAD_BUDGET_MS := 4.0
## Skirt depth, as a fraction of the node's ground width, added to the node's
## own relief range (labelled judgement: §3.5 sizes the skirt by the node's
## relief bound; the width term keeps a fin on flat ground).
const SKIRT_WIDTH_FRACTION := 0.02
## The drape's mip bias (shader `drape_bias`). A labelled judgement: a tile is
## drawn between 0.71 and 1.41 screen px per texel inside its band, so an
## unbiased trilinear lookup would blur the half where it is minified; -0.5
## keeps top-down at mip 0, as 2D's unmipped sprites draw.
const DRAPE_BIAS := -0.5

var _wg: Object = null
var _readable := Callable()
## The tier's workload key in `View3DGate.WORKLOADS` / `LAYERS`.
var _tier_key := "standard_desktop"
var _n := 65
var _cap_nodes := 512
var _cap_layers := 256

## The world, from `view3d_world()` ({} until a snapshot is live).
var _world := {}
var _gw := 0
var _gh := 0
var _cell_m := 0.0
var _sea := 0.42
var _peak := 4000.0
var _max_level := 0
var _exag := 1.0
## Tile dimensions, from the first tile (every tile of one world has them).
var _tw := 0
var _th := 0

## Resident tiles: Vector3i(z, c, r) -> {layer, geo: PackedFloat32Array,
## gmin, gmax, used}.
var _tiles := {}
var _free_layers: Array[int] = []
var _geo_arr: Texture2DArray = null
var _col_arr: Texture2DArray = null
## Tiles asked of the worker and not yet landed.
var _asked := {}
## The level-0 relief (the distance's height reference) and its min/max
## pyramid: level m holds, per block of 2^m texels, [min, max].
var _ref_img: Image = null
var _ref_pyr: Array = []
## EF-9.4's boost: on, its texture, its max pyramid and the entry px-per-cell.
var _boost_on := false
var _boost_tex: Texture2D = null
var _boost_n := 0
var _boost_key := ""
var _boost_pyr: Array = []
var _ppc_entry := 0.0

var _mm: MultiMesh
var _mmi: MultiMeshInstance3D
var _mat: ShaderMaterial
var _buf := PackedFloat32Array()
var _frame := 0

## The last selection, for the card, the clamp and the probe: drawn node keys
## (Vector3i -> true), how many, and the levels they span.
var drawn := {}
var drawn_count := 0
var level_min := -1
var level_max := -1
## Tiles uploaded so far, and the last frame's selection time in ms.
var tiles_uploaded := 0
var last_select_ms := 0.0
## Why no terrain is drawn, when none is ("" while it is drawn).
var reason := "no world yet"
## The node-level debug view (§5): 0 off, 1 tint by level, 2 level index,
## 3 skirt fragments in pure green (the probe's morph measure).
var debug_levels := 0

func _ready() -> void:
	_mat = ShaderMaterial.new()
	_mat.shader = QShader
	_mm = MultiMesh.new()
	_mm.transform_format = MultiMesh.TRANSFORM_3D
	_mm.use_custom_data = true
	_mmi = MultiMeshInstance3D.new()
	_mmi.multimesh = _mm
	_mmi.material_override = _mat
	_mmi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	## Displacement happens in the shader and the camera stays at the origin;
	## a box this large keeps Godot from culling the MultiMesh. The nodes are
	## culled by this script (§3.13).
	_mmi.custom_aabb = AABB(Vector3(-1e9, -1e6, -1e9), Vector3(2e9, 2e6, 2e9))
	_mmi.visible = false
	add_child(_mmi)

## Binds the engine for the tier `tier_key` (a `View3DGate.WORKLOADS` key
## other than "basic").
func bind(wg: Object, readable: Callable, tier_key: String) -> void:
	_wg = wg
	_readable = readable
	if tier_key != _tier_key or _mm.mesh == null:
		_tier_key = tier_key
		_n = int(View3DGate.WORKLOADS[tier_key]["n"])
		_cap_nodes = int(View3DGate.WORKLOADS[tier_key]["instances"])
		_cap_layers = int(View3DGate.LAYERS[tier_key])
		_mm.instance_count = 0
		_mm.mesh = make_patch(_n)
		_mm.instance_count = _cap_nodes
		_buf.resize(_cap_nodes * 16)
		_geo_arr = null
		_col_arr = null
	clear("waiting for the first tile")

## Forgets every tile (a new world, a look change, a re-entry), so nothing of
## an old picture is drawn over a new one. The arrays are kept and re-filled.
func clear(why: String) -> void:
	_tiles = {}
	_asked = {}
	_world = {}
	_gw = 0
	_gh = 0
	_cell_m = 0.0
	_ref_img = null
	_ref_pyr = []
	_boost_key = ""
	_boost_pyr = []
	_free_layers.clear()
	for i in _cap_layers:
		_free_layers.append(i)
	drawn = {}
	drawn_count = 0
	level_min = -1
	level_max = -1
	reason = why
	if _mmi != null:
		_mmi.visible = false
		_mm.visible_instance_count = 0

func set_exaggeration(x: float) -> void:
	_exag = x
	_mat.set_shader_parameter("exag", _exag)

func exaggeration() -> float:
	return _exag

func tier_key() -> String:
	return _tier_key

func patch_verts() -> int:
	return _n

func node_cap() -> int:
	return _cap_nodes

func layer_cap() -> int:
	return _cap_layers

func resident() -> int:
	return _tiles.size()

func world_known() -> bool:
	return _gw > 0 and _cell_m > 0.0 and _ref_img != null

func grid() -> Vector2i:
	return Vector2i(_gw, _gh)

func cell_m() -> float:
	return _cell_m

func max_level() -> int:
	return _max_level

## EF-9.4's boost, as the 2D view runs it (`ViewportHost._ef9_enabled()` and
## the 2D LOD entry's px per cell, `view_for_3d()`): the boost field's texture
## comes from the engine, and `ppc_entry` is the 2D view's own entry gate, so
## `L_3d` adds exactly 2D's boost (§3.3).
func set_boost(on: bool, ppc_entry: float) -> void:
	_boost_on = on
	_ppc_entry = ppc_entry

## One N x N unit patch on [0, 1]^2 in x/z (§3.2) plus a skirt (§3.5): every
## border vertex again with y = -1, joined to the border by a strip of quads.
static func make_patch(n: int) -> ArrayMesh:
	var verts := PackedVector3Array()
	for j in n:
		for i in n:
			verts.append(Vector3(float(i) / (n - 1), 0.0, float(j) / (n - 1)))
	var idx := PackedInt32Array()
	for j in n - 1:
		for i in n - 1:
			var a := j * n + i
			idx.append_array(PackedInt32Array([a, a + 1, a + n, a + 1, a + n + 1, a + n]))
	## The border, walked once around, as surface vertex indices.
	var ring: Array[int] = []
	for i in n - 1:
		ring.append(i)
	for j in n - 1:
		ring.append(j * n + n - 1)
	for i in range(n - 1, 0, -1):
		ring.append((n - 1) * n + i)
	for j in range(n - 1, 0, -1):
		ring.append(j * n)
	var base := verts.size()
	for v in ring:
		verts.append(Vector3(verts[v].x, -1.0, verts[v].z))
	var m := ring.size()
	for q in m:
		var a: int = ring[q]
		var b: int = ring[(q + 1) % m]
		var sa := base + q
		var sb := base + (q + 1) % m
		idx.append_array(PackedInt32Array([a, b, sa, b, sb, sa]))
	var arr := []
	arr.resize(Mesh.ARRAY_MAX)
	arr[Mesh.ARRAY_VERTEX] = verts
	arr[Mesh.ARRAY_INDEX] = idx
	var mesh := ArrayMesh.new()
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arr)
	return mesh

## Triangles per patch: the surface's and the skirt's (the probe pins it).
static func patch_triangles(n: int) -> int:
	return 2 * (n - 1) * (n - 1) + 2 * 4 * (n - 1)

func _can_read() -> bool:
	return _wg != null and _readable.is_valid() and bool(_readable.call()) and _wg.has_method("view3d_request_tile")

## Normalised height -> drawn metres, the shader's `metres()`.
func metres(h: float) -> float:
	return maxf((h - _sea) / maxf(1.0 - _sea, 1e-6) * _peak, 0.0) * _exag

# -- the world, the root and the pyramids --------------------------------------

func _ensure_world() -> bool:
	if not _world.is_empty():
		return true
	if not _can_read():
		reason = "the engine is busy" if _wg != null else "no engine"
		if _wg != null and not _wg.has_method("view3d_request_tile"):
			reason = "this engine build has no WorldGen.view3d_request_tile() (rebuild the extension)"
		return false
	## The tile context is built on a worker (LOD-D6); 2 means "building".
	var st: int = _wg.lod_prepare()
	if st == 0:
		reason = "the engine has no tile context for this world"
		return false
	if st == 2:
		reason = "preparing tiles"
		return false
	var w: Dictionary = _wg.view3d_world()
	if w.is_empty():
		reason = "preparing tiles"
		return false
	_world = w
	_gw = int(w["gw"])
	_gh = int(w["gh"])
	_cell_m = float(w["cell_m"])
	_sea = float(w["sea_level"])
	_peak = float(w["peak_m"])
	_max_level = int(w["max_level"])
	_mat.set_shader_parameter("sea_level", _sea)
	_mat.set_shader_parameter("peak_m", _peak)
	_mat.set_shader_parameter("exag", _exag)
	_mat.set_shader_parameter("cell_m", _cell_m)
	_mat.set_shader_parameter("gw", float(_gw))
	_mat.set_shader_parameter("grid_n", float(_n))
	_mat.set_shader_parameter("morph_band", MORPH_BAND)
	_mat.set_shader_parameter("drape_bias", DRAPE_BIAS)
	_mat.set_shader_parameter("world_m", Vector2((_gw - 1) * _cell_m, (_gh - 1) * _cell_m))
	return true

## A min/max pyramid over a `w x h` float grid: level 0 is [v, v] per texel,
## level m+1 the min and max of 2 x 2 blocks of level m (edges clamped). Used
## for the boost field's maximum and the relief reference's range over a node.
static func build_pyramid(data: PackedFloat32Array, w: int, h: int) -> Array:
	var lv: Array = [{"w": w, "h": h, "lo": data, "hi": data}]
	while w > 1 or h > 1:
		var prev: Dictionary = lv[-1]
		var nw := maxi(1, (w + 1) / 2)
		var nh := maxi(1, (h + 1) / 2)
		var lo := PackedFloat32Array()
		var hi := PackedFloat32Array()
		lo.resize(nw * nh)
		hi.resize(nw * nh)
		var plo: PackedFloat32Array = prev["lo"]
		var phi: PackedFloat32Array = prev["hi"]
		for y in nh:
			var y0 := mini(2 * y, h - 1)
			var y1 := mini(2 * y + 1, h - 1)
			for x in nw:
				var x0 := mini(2 * x, w - 1)
				var x1 := mini(2 * x + 1, w - 1)
				lo[y * nw + x] = minf(minf(plo[y0 * w + x0], plo[y0 * w + x1]), minf(plo[y1 * w + x0], plo[y1 * w + x1]))
				hi[y * nw + x] = maxf(maxf(phi[y0 * w + x0], phi[y0 * w + x1]), maxf(phi[y1 * w + x0], phi[y1 * w + x1]))
		lv.append({"w": nw, "h": nh, "lo": lo, "hi": hi})
		w = nw
		h = nh
	return lv

## [min, max] of a pyramid over the texel rectangle [x0, x1] x [y0, y1]
## (inclusive, clamped): read at the level where the rectangle spans at most
## two blocks a side, so a query costs at most nine reads.
static func pyramid_range(pyr: Array, x0: int, y0: int, x1: int, y1: int) -> Vector2:
	if pyr.is_empty():
		return Vector2(0.0, 0.0)
	var w0: int = pyr[0]["w"]
	var h0: int = pyr[0]["h"]
	x0 = clampi(x0, 0, w0 - 1)
	x1 = clampi(x1, x0, w0 - 1)
	y0 = clampi(y0, 0, h0 - 1)
	y1 = clampi(y1, y0, h0 - 1)
	var span := maxi(x1 - x0, y1 - y0) + 1
	var m := 0
	while (1 << (m + 1)) <= span and m + 1 < pyr.size():
		m += 1
	var l: Dictionary = pyr[m]
	var lw: int = l["w"]
	var lh: int = l["h"]
	var lo: PackedFloat32Array = l["lo"]
	var hi: PackedFloat32Array = l["hi"]
	var r := Vector2(INF, -INF)
	for y in range(mini(y0 >> m, lh - 1), mini(y1 >> m, lh - 1) + 1):
		for x in range(mini(x0 >> m, lw - 1), mini(x1 >> m, lw - 1) + 1):
			r.x = minf(r.x, lo[y * lw + x])
			r.y = maxf(r.y, hi[y * lw + x])
	return r

## The boost field's texture and max pyramid, rebuilt when its key moves.
func _sync_boost(bridge: Object) -> void:
	if not _boost_on or bridge == null or not bridge.has_method("lod_ef9_boost"):
		_mat.set_shader_parameter("boost_on", 0.0)
		return
	var info: Dictionary = bridge.lod_ef9_boost()
	if info.is_empty() or bool(info.get("pending", false)) or not bool(info.get("any", false)):
		_mat.set_shader_parameter("boost_on", 0.0)
		return
	var key := String(info.get("key", ""))
	if key != _boost_key or _boost_tex == null:
		_boost_tex = bridge.lod_ef9_boost_texture()
		_boost_n = int(info.get("n", 0))
		_boost_key = key
		_boost_pyr = []
		if _boost_tex != null and _boost_n > 0:
			var img := _boost_tex.get_image()
			if img != null:
				_boost_pyr = build_pyramid(img.get_data().to_float32_array(), img.get_width(), img.get_height())
	var ok := _boost_tex != null and _boost_n > 0 and not _boost_pyr.is_empty() and _ppc_entry > 0.0
	_mat.set_shader_parameter("boost_on", 1.0 if ok else 0.0)
	_mat.set_shader_parameter("boost_tex", _boost_tex)
	_mat.set_shader_parameter("boost_n", _boost_n)
	_mat.set_shader_parameter("ppc_entry", _ppc_entry)

func _boost_live() -> bool:
	return _boost_on and not _boost_pyr.is_empty() and _ppc_entry > 0.0

## Allocates the two layer arrays at the tier's capacity, once per tile size
## (kept across `clear()`, so re-entering 3D does not reallocate them).
func _ensure_arrays(cw: int, ch: int) -> void:
	if _col_arr != null and _tw == cw and _th == ch:
		return
	_tw = cw
	_th = ch
	var c := Image.create_empty(cw, ch, true, Image.FORMAT_RGBA8)
	var g := Image.create_empty(_n, _n, false, Image.FORMAT_RF)
	var cs: Array[Image] = []
	var gs: Array[Image] = []
	for i in _cap_layers:
		cs.append(c)
		gs.append(g)
	_col_arr = Texture2DArray.new()
	_col_arr.create_from_images(cs)
	_geo_arr = Texture2DArray.new()
	_geo_arr.create_from_images(gs)
	_mat.set_shader_parameter("col", _col_arr)
	_mat.set_shader_parameter("geo", _geo_arr)
	_mat.set_shader_parameter("col_px", Vector2(cw, ch))

# -- streaming -------------------------------------------------------------------

## Uploads finished tiles within the frame's budget.
func _drain() -> bool:
	var t0 := Time.get_ticks_usec()
	var got := false
	var ready: Array = _wg.view3d_take_ready(TAKE_PER_FRAME)
	for d in ready:
		var key := Vector3i(int(d["z"]), int(d["col"]), int(d["row"]))
		_asked.erase(key)
		if int(d["grid"]) != _n or _tiles.has(key):
			continue
		var rgba: Image = d["rgba"]
		## Every tile of one world has one size; a different size is only
		## legitimate while nothing is resident (a new world), when the arrays
		## are reallocated for it.
		if not _tiles.is_empty() and (rgba.get_width() != _tw or rgba.get_height() != _th):
			reason = "tile %s is %d x %d, not %d x %d" % [key, rgba.get_width(), rgba.get_height(), _tw, _th]
			continue
		_ensure_arrays(rgba.get_width(), rgba.get_height())
		var layer := _alloc_layer()
		if layer < 0:
			continue
		rgba.generate_mipmaps()
		_col_arr.update_layer(rgba, layer)
		var geo: Image = d["geo"]
		_geo_arr.update_layer(geo, layer)
		_tiles[key] = {"layer": layer, "geo": geo.get_data().to_float32_array(),
			"gmin": float(d["geo_min"]), "gmax": float(d["geo_max"]), "used": _frame}
		tiles_uploaded += 1
		got = true
		if key == Vector3i.ZERO:
			_set_ref(d["height"])
		if (Time.get_ticks_usec() - t0) / 1000.0 > UPLOAD_BUDGET_MS:
			break
	return got

## The root's full relief becomes the distance reference (shader `ref_h`).
func _set_ref(img: Image) -> void:
	_ref_img = img
	_ref_pyr = build_pyramid(img.get_data().to_float32_array(), img.get_width(), img.get_height())
	_mat.set_shader_parameter("ref_h", ImageTexture.create_from_image(img))
	_mat.set_shader_parameter("ref_px", Vector2(img.get_width(), img.get_height()))

## A free layer, or the least recently used tile's when none is free (the
## oldest first, so a tile parked `PARK_FRAMES` goes before one parked a
## moment). Tiles read this frame or the last (every drawn node and its
## ancestors: the drain runs before this frame's selection) are never taken,
## nor the root. -1 when nothing may be taken (the tile is dropped and asked
## again later).
func _alloc_layer() -> int:
	if not _free_layers.is_empty():
		return _free_layers.pop_back()
	var victim = null
	var oldest := _frame - 1
	for k in _tiles:
		if k == Vector3i.ZERO:
			continue
		var u: int = _tiles[k]["used"]
		if u < oldest:
			oldest = u
			victim = k
	if victim == null:
		return -1
	var layer: int = _tiles[victim]["layer"]
	_tiles.erase(victim)
	return layer

func _ask(key: Vector3i) -> bool:
	if _tiles.has(key) or _asked.has(key):
		return true
	if not _wg.view3d_request_tile(key.x, key.y, key.z, _n):
		return false
	_asked[key] = _frame
	return true

# -- selection -------------------------------------------------------------------

## The node's footprint in metres: [x0, z0, width, depth].
func _foot(key: Vector3i) -> Vector4:
	var n := float(1 << key.x)
	var sx := (_gw - 1) * _cell_m / n
	var sz := (_gh - 1) * _cell_m / n
	return Vector4(key.y * sx, key.z * sz, sx, sz)

## The relief range a node draws from: its own tile's and its two parents'.
func _range(key: Vector3i) -> Vector2:
	var r := Vector2(INF, -INF)
	var k := key
	for i in 3:
		if _tiles.has(k):
			r.x = minf(r.x, float(_tiles[k]["gmin"]))
			r.y = maxf(r.y, float(_tiles[k]["gmax"]))
		if k.x == 0:
			break
		k = Vector3i(k.x - 1, k.y >> 1, k.z >> 1)
	return r

## [min, max] of a pyramid over a node's footprint, with one texel of margin
## for the bilinear lookups that read it.
func _node_pyr(pyr: Array, key: Vector3i) -> Vector2:
	if pyr.is_empty():
		return Vector2.ZERO
	var w0: int = pyr[0]["w"]
	var h0: int = pyr[0]["h"]
	var n := float(1 << key.x)
	return pyramid_range(pyr, int(floor(key.y / n * (w0 - 1))) - 1, int(floor(key.z / n * (h0 - 1))) - 1,
		int(ceil((key.y + 1) / n * (w0 - 1))) + 1, int(ceil((key.z + 1) / n * (h0 - 1))) + 1)

## The largest `L_3d` anywhere in a node (an upper bound of the shader's): the
## level at the node's nearest point to the camera, with the distance's height
## taken over the reference relief's range, plus the boost's maximum there.
func node_level_bound(key: Vector3i, cam: Array, f_px: float, ortho_ppc: float) -> float:
	var ppc := ortho_ppc
	if ppc <= 0.0:
		var f := _foot(key)
		var rr := _node_pyr(_ref_pyr, key)
		var dx: float = maxf(maxf(f.x - float(cam[0]), float(cam[0]) - (f.x + f.z)), 0.0)
		var dz: float = maxf(maxf(f.y - float(cam[2]), float(cam[2]) - (f.y + f.w)), 0.0)
		var ylo := metres(rr.x) - float(cam[1])
		var yhi := metres(rr.y) - float(cam[1])
		var dy := maxf(maxf(ylo, -yhi), 0.0)
		ppc = _cell_m * f_px / maxf(_cell_m, sqrt(dx * dx + dy * dy + dz * dz))
	var lam := log(maxf(1.0, _gw * maxf(0.01, ppc) / 256.0)) / log(2.0)
	if _boost_live():
		var g := smoothstep(0.0, 1.0, log(ppc / _ppc_entry) / log(2.0))
		lam += g * clampf(_node_pyr(_boost_pyr, key).y, 0.0, 1.0)
	return lam

## Whether a node's box is outside the frustum (`planes` from
## `Camera3D.get_frustum()`, whose normals point out): the box corner furthest
## against a plane's normal is still in front of it.
static func outside(planes: Array, lo: Vector3, hi: Vector3) -> bool:
	for pl in planes:
		var p: Plane = pl
		var c := Vector3(lo.x if p.normal.x > 0.0 else hi.x, lo.y if p.normal.y > 0.0 else hi.y, lo.z if p.normal.z > 0.0 else hi.z)
		if p.distance_to(c) > 0.0:
			return true
	return false

## **The per-frame step.** `cam` is `View3DCamera.position()`, `planes` the
## camera's frustum in camera-relative space, `f_px` the focal length in
## physical px and `ortho_ppc` the top-down px per cell (0 in perspective).
## Uploads landed tiles, selects the nodes, asks for the missing ones and
## writes the instance buffer. True when anything drawn changed.
func update(cam: Array, planes: Array, f_px: float, ortho_ppc: float, bridge: Object) -> bool:
	_frame += 1
	if not _ensure_world():
		return false
	var changed := _drain()
	for k in _asked.keys():
		if _frame - int(_asked[k]) > ASK_EXPIRE_FRAMES:
			_asked.erase(k)
	if not _tiles.has(Vector3i.ZERO) or _ref_img == null:
		_ask(Vector3i.ZERO)
		reason = "waiting for the first tile"
		return changed
	_sync_boost(bridge)
	var t0 := Time.get_ticks_usec()
	var out: Array = []
	var want: Array = []
	var queue: Array = [Vector3i.ZERO]
	var head := 0
	while head < queue.size():
		var key: Vector3i = queue[head]
		head += 1
		_tiles[key]["used"] = _frame
		var f := _foot(key)
		var r := _range(key)
		var skirt := SKIRT_WIDTH_FRACTION * maxf(f.z, f.w) + metres(r.y) - metres(r.x)
		var lo := Vector3(f.x - float(cam[0]), metres(r.x) - skirt - float(cam[1]), f.y - float(cam[2]))
		var hi := Vector3(f.x + f.z - float(cam[0]), metres(r.y) - float(cam[1]), f.y + f.w - float(cam[2]))
		if outside(planes, lo, hi):
			continue
		var lmax := node_level_bound(key, cam, f_px, ortho_ppc)
		var split := key.x < _max_level and lmax >= key.x + 0.5
		var pre := key.x < _max_level and lmax >= key.x + 0.5 - PREFETCH_LEVELS
		var kids: Array = []
		if pre:
			for j in 2:
				for i in 2:
					kids.append(Vector3i(key.x + 1, key.y * 2 + i, key.z * 2 + j))
		var all_in := not kids.is_empty()
		for kk in kids:
			if not _tiles.has(kk):
				all_in = false
				want.append([key.x + 1, lmax, kk])
		## Two budgets bound a split: the drawn nodes (the tier's instance
		## row) and the resident tiles, because every node visited (each drawn
		## node and all its ancestors) must hold a layer. A first cut bounded
		## only the drawn nodes (512) against 256 layers; at x64 oblique the
		## wanted set outgrew the cache and tiles were synthesised, evicted and
		## asked for again without end (`_view3d_quadtree_probe.gd` leg F,
		## 2026-11-13: 3 222 CPU-seconds in five minutes).
		if split and all_in and queue.size() - head + out.size() + 4 <= _cap_nodes and queue.size() + 4 <= _cap_layers:
			queue.append_array(kids)
		else:
			out.append([key, skirt])
	## Coarse first, then the most wanted: missing tiles nearest a split first.
	## Only as many as there is room for: free layers plus the tiles not read
	## this frame (every visited node was marked used above), less those
	## already asked for. Asking beyond that only evicts what the next frame
	## asks for again.
	want.sort_custom(func(a, b): return a[0] < b[0] or (a[0] == b[0] and float(a[1]) > float(b[1])))
	var room := _free_layers.size() + (_tiles.size() - queue.size()) - _asked.size()
	for w in want:
		if room <= 0:
			break
		if not _ask(w[2]):
			break
		room -= 1
	var moved := _write(out, cam, f_px, ortho_ppc)
	last_select_ms = (Time.get_ticks_usec() - t0) / 1000.0
	reason = ""
	return changed or moved

## Writes the instance buffer: per node a camera-relative transform (footprint
## minus camera in 64-bit floats, §3.11; the basis's y scale is the skirt
## depth) and custom data (own, parent and grandparent layers, -1 where there
## is none, and the level with the node's child-position bits). True when the
## drawn set changed (count or levels), which is when the card re-reads it.
func _write(out: Array, cam: Array, f_px: float, ortho_ppc: float) -> bool:
	var sig := [drawn_count, level_min, level_max]
	drawn = {}
	level_min = -1
	level_max = -1
	var i := 0
	for e in out:
		var key: Vector3i = e[0]
		var f := _foot(key)
		var b := i * 16
		_buf[b + 0] = f.z
		_buf[b + 1] = 0.0
		_buf[b + 2] = 0.0
		_buf[b + 3] = f.x - float(cam[0])
		_buf[b + 4] = 0.0
		_buf[b + 5] = maxf(float(e[1]), 1.0)
		_buf[b + 6] = 0.0
		_buf[b + 7] = 0.0
		_buf[b + 8] = 0.0
		_buf[b + 9] = 0.0
		_buf[b + 10] = f.w
		_buf[b + 11] = f.y - float(cam[2])
		var l1 := -1.0
		var l2 := -1.0
		if key.x >= 1:
			l1 = float(_tiles[Vector3i(key.x - 1, key.y >> 1, key.z >> 1)]["layer"])
		if key.x >= 2:
			l2 = float(_tiles[Vector3i(key.x - 2, key.y >> 2, key.z >> 2)]["layer"])
		var bits := (key.y & 1) + 2 * (key.z & 1) + 4 * ((key.y >> 1) & 1) + 8 * ((key.z >> 1) & 1)
		_buf[b + 12] = float(_tiles[key]["layer"])
		_buf[b + 13] = l1
		_buf[b + 14] = l2
		_buf[b + 15] = float(key.x + 16 * bits)
		drawn[key] = true
		level_min = key.x if level_min < 0 else mini(level_min, key.x)
		level_max = maxi(level_max, key.x)
		i += 1
	drawn_count = i
	_mm.buffer = _buf
	_mm.visible_instance_count = i
	_mmi.visible = i > 0
	var n0 := float(_gw - 1) * _cell_m
	var m0 := float(_gh - 1) * _cell_m
	_mat.set_shader_parameter("cam_idx", Vector2(float(cam[0]) / n0, float(cam[2]) / m0))
	_mat.set_shader_parameter("cam_y", float(cam[1]))
	_mat.set_shader_parameter("f_px", f_px)
	_mat.set_shader_parameter("ortho_ppc", ortho_ppc)
	_mat.set_shader_parameter("debug_levels", debug_levels)
	return sig != [drawn_count, level_min, level_max]

## **An upper bound of the drawn surface's height near `(x_m, z_m)`**, metres
## (the camera's ground clamp, bar B4): the deepest drawn node over the point
## draws a mix of its own and its two parents' vertex grids, so the bound is
## the highest vertex of those three grids within two vertex steps of it (the
## morph moves a vertex at most one step). Where no drawn node covers the point
## (outside the view), the deepest resident tile chain answers instead.
func surface_bound(x_m: float, z_m: float) -> float:
	if not world_known() or not _tiles.has(Vector3i.ZERO):
		return 0.0
	var sx := x_m / ((_gw - 1) * _cell_m)
	var sz := z_m / ((_gh - 1) * _cell_m)
	var key := Vector3i.ZERO
	while key.x < _max_level and not drawn.has(key):
		var n := 1 << (key.x + 1)
		var kk := Vector3i(key.x + 1, clampi(int(floor(sx * n)), 0, n - 1), clampi(int(floor(sz * n)), 0, n - 1))
		if not _tiles.has(kk):
			break
		key = kk
	var best := 0.0
	var k := key
	for i in 3:
		if _tiles.has(k):
			best = maxf(best, _grid_max(k, sx, sz))
		if k.x == 0:
			break
		k = Vector3i(k.x - 1, k.y >> 1, k.z >> 1)
	return metres(best)

func _grid_max(key: Vector3i, sx: float, sz: float) -> float:
	var n := float(1 << key.x)
	var g: PackedFloat32Array = _tiles[key]["geo"]
	var u := (sx * n - key.y) * (_n - 1)
	var v := (sz * n - key.z) * (_n - 1)
	var best := -INF
	for j in range(clampi(int(floor(v)) - 2, 0, _n - 1), clampi(int(ceil(v)) + 2, 0, _n - 1) + 1):
		for i in range(clampi(int(floor(u)) - 2, 0, _n - 1), clampi(int(ceil(u)) + 2, 0, _n - 1) + 1):
			best = maxf(best, g[j * _n + i])
	return best
