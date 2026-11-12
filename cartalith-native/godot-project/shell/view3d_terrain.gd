extends Node3D
## No `class_name`, deliberately: `view3d.gd` preloads this file (3D-1's reason).

## **3D-2's terrain: the Basic tier's one displaced patch**
## (`3D_VIEWPORT_SCOPE.md` §5, milestone 3D-2; §3.1, §3.2, §3.6, §3.11, §3.12).
##
## What it draws: one 257 x 257 vertex patch (131 k triangles, §3.12's Basic
## row) over a **window** of pyramid tiles at the level the camera's ground
## scale asks for, plus the same patch once more over the whole world as a
## **backdrop** under it, so an oblique view never ends in empty background.
## Both are draped with the 2D picture: the window with the very tiles 2D draws
## at its level (`WorldGen.view3d_tile()`, colour and relief from one
## synthesis, §3.1), the backdrop with the 2D base map texture, which is what 2D
## shows at fit zoom.
##
## Why a window and not one world texture: the owner's hard rule that zoom gains
## detail at every level (Ruling CI item 1; the 2026-10-17 requirement). The
## window's level rises as the camera comes down, exactly as 2D's tiles do, so
## each step in brings finer tiles; one fixed world texture would stop gaining
## detail at its own resolution. It is one patch, so this is still Basic: the
## quadtree, the `L_3d` morph and texture arrays are 3D-3's (§3.2, §3.3).
##
## Camera-relative (§3.11): the camera stays at the origin, and each mesh's
## origin is `node_origin - camera_position`, subtracted in 64-bit GDScript
## floats before the cast to `Vector3` (`place()`).
##
## What it must never do:
## - change anything 2D draws: it calls only `view3d_tile()` and reads the
##   base map texture the 2D view already holds; it writes no world state;
## - call the engine while a generate holds `WorldGen` (`readable` is the
##   host's `_engine_readable()`), or synthesise more than the frame budget
##   allows, beyond the one tile each frame needs to make progress;
## - show a half-built window: tiles fill a pending mosaic, which replaces the
##   drawn one only when complete.

const TerrainShader := preload("res://shell/view3d_terrain.gdshader")

## Vertices per patch side: §3.12's Basic row, "one 257² mesh", the figure the
## capability gate timed as Basic's workload (`View3DGate.WORKLOADS`).
const PATCH_VERTS := 257
## Tiles per window side. A labelled judgement, sized so the window spans the
## view at the pivot: a level-z tile is about 256 texels, so at one level below
## 2D (`LEVELS_BELOW_2D`) four tiles are about 2 000 screen pixels of ground
## at the pivot, more than a 1600-px desktop view or a phone held either way.
## 16 tiles at 3D-0's measured 7.1 ms each (§8.1) are about 114 ms of synthesis
## per level change, spread over frames by `FILL_BUDGET_MS`.
const WINDOW_TILES := 4
## The window's level relative to the level 2D draws at the same ground scale
## (`WorldGen.lod_level_for_zoom`). A labelled judgement: one level coarser
## halves the tiles a window needs per side for the same coverage, and the
## drape still gains a level at every zoom step that raises 2D's. Bar B6 (3D
## detail >= 2D at the same scale) is 3D-3's, with the quadtree.
const LEVELS_BELOW_2D := 1
## Synthesis per frame, in milliseconds: another tile is started only while
## the time spent plus the running mean tile cost stays inside this, and one
## tile is always built, so a slow device still progresses. A labelled
## judgement: about half of a 60 Hz frame, so one tile per frame on this
## machine's 7 ms tiles (§8.1) and one per frame wherever a tile costs more
## (unmeasured on the 6T). A single slow tile still lands in one frame: a tile
## cannot be split, and moving synthesis off the main thread is 3D-3's
## streaming (§3.13).
const FILL_BUDGET_MS := 8.0
## The window's rim, as a fraction of its side, over which its height eases to
## the backdrop's so the two surfaces meet (labelled judgement: one sixteenth,
## 16 of the patch's 256 quads).
const RIM_FRACTION := 1.0 / 16.0
## The backdrop is sunk below its own relief by this fraction of the camera's
## height above the ground, at least `SINK_MIN_M`: where it runs under the
## window's rim the window must win the depth test without z-fighting (bar B5).
## Labelled judgements: 24-bit depth at §3.11's near plane (5 % of that height)
## resolves far finer than 0.5 % of it at the rim's distance.
const SINK_FRACTION := 0.005
const SINK_MIN_M := 1.0
## The backdrop's hole is the window shrunk on each side by this fraction of
## the window's rim (`RIM_FRACTION`), so the backdrop runs under the outer half
## of the rim, where the window has eased to the backdrop's own relief and sits
## `sink` above it, and no gap can open between them. A labelled judgement; a
## first cut shrank it by one backdrop vertex step instead, which at deep zoom
## is a quarter of the window, and the sunk backdrop showed through the
## window's valleys as a flat plain (`_view3d_terrain_probe.gd` capture
## `02_closest`, 2026-11-12).
const HOLE_INSET_RIMS := 0.5
## The 2D pan step the window is re-centred after: it moves only when the pivot
## has drifted this many tiles off the window's centre (labelled judgement;
## hysteresis, so a pivot resting on a tile edge does not rebuild every frame).
const RECENTRE_TILES := 0.75
## The ground-bound sampler reads this many relief texels on each side of the
## point, per vertex step, because the drawn surface between vertices mixes the
## low-passed samples of the neighbouring vertices (one vertex step either way).
const BOUND_STEPS := 2.0
## At most this many texels per side are read in that neighbourhood
## (labelled judgement): up to 17 x 17 every texel is read, so the bound is
## exact over it; beyond, a stride keeps a call under 300 reads.
const BOUND_SAMPLES_MAX := 17

var _wg: Object = null
var _readable := Callable()
var _gw := 0
var _gh := 0
var _cell_m := 0.0
var _sea := 0.42
var _peak := 4000.0
var _exag := 1.0
## The base map's colour texture (the backdrop's drape).
var _base_tex: Texture2D = null

var _mesh: PlaneMesh
var _win_mi: MeshInstance3D
var _back_mi: MeshInstance3D
var _win_mat: ShaderMaterial
var _back_mat: ShaderMaterial

## The drawn window: {z, c0, r0, k, x0, y0, w, h (samples), mx, my (mosaic
## px), himg: Image}. Empty when none is drawn yet.
## `himg` keeps its mipmaps; `surface_bound()` reads level 0 only.
var _win := {}
## The backdrop: {x0, y0, w, h, himg, tw, th} for the level-0 relief tile.
var _back := {}
## The window being filled: the same keys as `_win` plus `todo` (tile list),
## `cimg` and `himg` under construction. Empty when nothing is pending.
var _pending := {}
## Tiles synthesised so far this world, and the last fill's statistics, for
## the acceptance probe and the card.
var tiles_built := 0
var last_fill_ms := 0.0
## The running mean of one tile's synthesis on this device, in ms (0 before the
## first), which `_fill()` budgets against: an exponential average with weight
## 0.2 per tile (labelled judgement: about the last five tiles).
var mean_tile_ms := 0.0
## Why no terrain is drawn, when none is ("" while it is drawn). Shown by the
## card; never a fake surface in its place.
var reason := "no world yet"
## **Probe seam only** (`_view3d_terrain_probe.gd`, as `View3DGate.debug_stub_vtf`
## is the gate's): added to the window's level, so the probe can draw the same
## camera a few levels coarser and show the drawn level carries more detail.
## Nothing in the shell sets it; 0 is the shipped behaviour.
var debug_level_offset := 0

func _ready() -> void:
	_mesh = PlaneMesh.new()
	_mesh.size = Vector2.ONE
	_mesh.center_offset = Vector3(0.5, 0.0, 0.5)
	_mesh.subdivide_width = PATCH_VERTS - 2
	_mesh.subdivide_depth = PATCH_VERTS - 2
	## Displacement moves vertices far outside the flat plane's box; a box
	## that tall keeps the patch from being frustum-culled while on screen.
	_mesh.custom_aabb = AABB(Vector3(0, -1e5, 0), Vector3(1, 2e5, 1))
	_win_mat = ShaderMaterial.new()
	_win_mat.shader = TerrainShader
	_back_mat = ShaderMaterial.new()
	_back_mat.shader = TerrainShader
	_back_mi = _make_instance(_back_mat)
	_win_mi = _make_instance(_win_mat)
	_back_mi.visible = false
	_win_mi.visible = false

func _make_instance(mat: ShaderMaterial) -> MeshInstance3D:
	var mi := MeshInstance3D.new()
	mi.mesh = _mesh
	mi.material_override = mat
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	add_child(mi)
	return mi

## Binds the engine. `wg` is the `WorldGen`, `readable` a `Callable() -> bool`
## that is false while a generate holds it, `base_tex` the 2D base map texture.
func bind(wg: Object, readable: Callable, base_tex: Texture2D) -> void:
	_wg = wg
	_readable = readable
	_base_tex = base_tex
	clear("waiting for the first tile")

## Forgets every tile (a new world, or a re-entry), so nothing of an old world
## is ever drawn over a new one.
func clear(why: String) -> void:
	_win = {}
	_back = {}
	_pending = {}
	_gw = 0
	_gh = 0
	_cell_m = 0.0
	reason = why
	if _win_mi != null:
		_win_mi.visible = false
		_back_mi.visible = false

## The vertical exaggeration, a shader uniform only (§3.1: it never writes the
## world). 1.0 is true scale (Ruling CT).
func set_exaggeration(x: float) -> void:
	_exag = x
	for m in [_win_mat, _back_mat]:
		m.set_shader_parameter("exag", _exag)

func exaggeration() -> float:
	return _exag

## Whether the engine can be asked right now.
func _can_read() -> bool:
	return _wg != null and _readable.is_valid() and bool(_readable.call()) and _wg.has_method("view3d_tile")

## The world's size and metres per cell, read once per world from the level-0
## tile (which the backdrop needs anyway). False when no tile can be made.
func _ensure_world() -> bool:
	if not _back.is_empty():
		return true
	if not _can_read():
		reason = "the engine is busy" if _wg != null else "no engine"
		if _wg != null and not _wg.has_method("view3d_tile"):
			reason = "this engine build has no WorldGen.view3d_tile() (rebuild the extension)"
		return false
	var d: Dictionary = _wg.view3d_tile(0, 0, 0)
	if d.is_empty():
		reason = "the engine made no tile for this world"
		return false
	var g: Vector2i = Vector2i(_wg.get_width(), _wg.get_height()) if _wg.has_method("get_width") else Vector2i.ZERO
	if g.x < 2 or g.y < 2:
		reason = "the world has no grid"
		return false
	_gw = g.x
	_gh = g.y
	_cell_m = float(d["cell_m"])
	_sea = float(d["sea_level"])
	_peak = float(d["peak_m"])
	var himg: Image = d["height"]
	himg.generate_mipmaps()
	_back = {"x0": float(d["tb_x"]), "y0": float(d["tb_y"]), "w": float(d["tb_w"]), "h": float(d["tb_h"]),
		"himg": himg, "tw": int(d["w"]), "th": int(d["h"])}
	var htex := ImageTexture.create_from_image(himg)
	for m in [_win_mat, _back_mat]:
		m.set_shader_parameter("sea_level", _sea)
		m.set_shader_parameter("peak_m", _peak)
		m.set_shader_parameter("exag", _exag)
	_back_mat.set_shader_parameter("hmap", htex)
	_back_mat.set_shader_parameter("h_xform", _tex_xform(int(d["w"]), int(d["h"])))
	_back_mat.set_shader_parameter("h_lod", _texel_lod(int(d["w"])))
	_back_mat.set_shader_parameter("rim", 0.0)
	_win_mat.set_shader_parameter("hfar", htex)
	_win_mat.set_shader_parameter("hf_lod", _texel_lod(int(d["w"])))
	## The backdrop's drape: the 2D base map, `gw x gh` pixels, pixel `i`
	## covering cell `i` (`map_overlay.gd`'s fit rect). Patch coordinate `v`
	## sits at sample `x0 + v * w`, whose pixel centre is at `(s + 0.5) / gw`.
	if _base_tex != null:
		_back_mat.set_shader_parameter("cmap", _base_tex)
		var bw := float(_back["w"])
		var bh := float(_back["h"])
		_back_mat.set_shader_parameter("c_xform", Vector4(bw / _gw, bh / _gh, (float(_back["x0"]) + 0.5) / _gw, (float(_back["y0"]) + 0.5) / _gh))
	else:
		var ctex := ImageTexture.create_from_image(_with_mips(d["rgba"]))
		_back_mat.set_shader_parameter("cmap", ctex)
		_back_mat.set_shader_parameter("c_xform", _tex_xform(int(d["w"]), int(d["h"])))
	_back_mi.visible = true
	reason = ""
	return true

## An image with its mipmaps, so a drape seen at a grazing angle is filtered
## rather than sparkling.
static func _with_mips(img: Image) -> Image:
	img.generate_mipmaps()
	return img

## Patch `[0, 1]` -> texture uv for an image whose pixel 0 sits at the patch's
## edge and pixel `n - 1` at its far edge (`tile_bounds`'s layout): pixel
## centres at `(0.5 + v * (n - 1)) / n`.
static func _tex_xform(nx: int, ny: int) -> Vector4:
	return Vector4(float(nx - 1) / nx, float(ny - 1) / ny, 0.5 / nx, 0.5 / ny)

## The relief mip level whose texel matches one vertex step of the patch over
## `n` texels: `log2(n / (PATCH_VERTS - 1))`, never below 0.
static func _texel_lod(n: int) -> float:
	return maxf(0.0, log(float(n) / float(PATCH_VERTS - 1)) / log(2.0))

func world_known() -> bool:
	return _gw > 0 and _cell_m > 0.0

func grid() -> Vector2i:
	return Vector2i(_gw, _gh)

func cell_m() -> float:
	return _cell_m

## The drawn window, for the card and the probe ({} when none).
func window() -> Dictionary:
	return _win

## True while a window is being filled.
func filling() -> bool:
	return not _pending.is_empty()

## **The per-frame step.** `level_2d` is the level 2D draws at the camera's
## ground scale and `max_level` the deepest allowed (the engine's and the
## quality tier's), `pivot_cell` the pivot in samples. Picks the window, and
## fills a pending one within `FILL_BUDGET_MS`. Returns true when the drawn
## window changed this frame.
func update(level_2d: int, max_level: int, pivot_cell: Vector2) -> bool:
	if not _ensure_world():
		return false
	var z := clampi(level_2d - LEVELS_BELOW_2D + debug_level_offset, 0, maxi(max_level, 0))
	var target := _window_for(z, pivot_cell)
	var key := [target["z"], target["c0"], target["r0"]]
	var drawn_key := [_win.get("z", -1), _win.get("c0", -1), _win.get("r0", -1)]
	var pend_key := [_pending.get("z", -1), _pending.get("c0", -1), _pending.get("r0", -1)]
	if key != drawn_key and key != pend_key:
		## Same level: keep the drawn window while the pivot is near enough to
		## its centre (hysteresis). A new level always rebuilds.
		if _win.is_empty() or int(_win["z"]) != z or _off_centre(_win, pivot_cell) > RECENTRE_TILES:
			_start(target)
	if _pending.is_empty():
		return false
	return _fill()

## The window of `WINDOW_TILES` tiles per side at level `z` centred as nearly
## as the level's grid allows on `pivot_cell`.
func _window_for(z: int, pivot_cell: Vector2) -> Dictionary:
	var n := 1 << z
	var k := mini(WINDOW_TILES, n)
	var span_x := float(_gw - 1) / n
	var span_y := float(_gh - 1) / n
	var c0 := clampi(int(floor(pivot_cell.x / span_x - k * 0.5 + 0.5)), 0, n - k)
	var r0 := clampi(int(floor(pivot_cell.y / span_y - k * 0.5 + 0.5)), 0, n - k)
	return {"z": z, "c0": c0, "r0": r0, "k": k}

## How far, in tiles, the pivot is from a window's centre (the larger axis).
func _off_centre(w: Dictionary, pivot_cell: Vector2) -> float:
	var n := 1 << int(w["z"])
	var span_x := float(_gw - 1) / n
	var span_y := float(_gh - 1) / n
	var k := float(w["k"])
	var cx := pivot_cell.x / span_x - (int(w["c0"]) + k * 0.5)
	var cy := pivot_cell.y / span_y - (int(w["r0"]) + k * 0.5)
	return maxf(absf(cx), absf(cy))

func _start(t: Dictionary) -> void:
	var todo: Array = []
	for j in int(t["k"]):
		for i in int(t["k"]):
			todo.append(Vector2i(int(t["c0"]) + i, int(t["r0"]) + j))
	_pending = t.duplicate()
	_pending["todo"] = todo
	_pending["ms"] = 0.0

## Builds pending tiles until the frame budget is spent (at least one), then
## finishes the mosaic over three more frames (relief mipmaps, colour mipmaps,
## then the upload and swap), so no single frame pays all of it: measured
## before staging, a level change's swap frame reached 50 ms on this desktop
## (`_view3d_terrain_probe.gd` leg F, 2026-11-12). True when the drawn window
## changed.
func _fill() -> bool:
	if _pending.has("stage"):
		var t1 := Time.get_ticks_usec()
		var st := int(_pending["stage"])
		if st == 0:
			(_pending["himg"] as Image).generate_mipmaps()
		elif st == 1:
			(_pending["cimg"] as Image).generate_mipmaps()
		_pending["ms"] = float(_pending["ms"]) + (Time.get_ticks_usec() - t1) / 1000.0
		if st < 2:
			_pending["stage"] = st + 1
			return false
		_swap()
		return true
	if not _can_read():
		return false
	var t0 := Time.get_ticks_usec()
	var todo: Array = _pending["todo"]
	while not todo.is_empty():
		var c: Vector2i = todo.pop_front()
		var tt := Time.get_ticks_usec()
		var d: Dictionary = _wg.view3d_tile(int(_pending["z"]), c.x, c.y)
		var tile_ms := (Time.get_ticks_usec() - tt) / 1000.0
		mean_tile_ms = tile_ms if tiles_built == 0 else lerpf(mean_tile_ms, tile_ms, 0.2)
		if d.is_empty():
			## A tile the engine refuses leaves no window rather than a hole
			## drawn as flat ground; the drawn one stays.
			reason = "the engine refused tile %d/%d/%d" % [int(_pending["z"]), c.x, c.y]
			_pending = {}
			return false
		tiles_built += 1
		var tw := int(d["w"])
		var th := int(d["h"])
		if not _pending.has("cimg"):
			var k := int(_pending["k"])
			var mx := k * (tw - 1) + 1
			var my := k * (th - 1) + 1
			_pending["mx"] = mx
			_pending["my"] = my
			_pending["cimg"] = Image.create_empty(mx, my, false, Image.FORMAT_RGBA8)
			_pending["himg"] = Image.create_empty(mx, my, false, Image.FORMAT_RF)
			_pending["x0"] = float(d["tb_x"]) - (c.x - int(_pending["c0"])) * float(d["tb_w"])
			_pending["y0"] = float(d["tb_y"]) - (c.y - int(_pending["r0"])) * float(d["tb_h"])
			_pending["w"] = float(d["tb_w"]) * k
			_pending["h"] = float(d["tb_h"]) * k
		## Adjacent tiles share their edge pixel (`tile_bounds`), so tile `i`
		## starts at `i * (w - 1)` and the shared column is written twice with
		## the same sample.
		var at := Vector2i((c.x - int(_pending["c0"])) * (tw - 1), (c.y - int(_pending["r0"])) * (th - 1))
		(_pending["cimg"] as Image).blit_rect(d["rgba"], Rect2i(0, 0, tw, th), at)
		(_pending["himg"] as Image).blit_rect(d["height"], Rect2i(0, 0, tw, th), at)
		if (Time.get_ticks_usec() - t0) / 1000.0 + mean_tile_ms > FILL_BUDGET_MS:
			break
	_pending["ms"] = float(_pending["ms"]) + (Time.get_ticks_usec() - t0) / 1000.0
	if todo.is_empty():
		_pending["stage"] = 0
	return false

## The last stage: uploads the finished mosaic and draws it.
func _swap() -> void:
	var p := _pending
	_pending = {}
	var himg: Image = p["himg"]
	var cimg: Image = p["cimg"]
	var t0 := Time.get_ticks_usec()
	_win_mat.set_shader_parameter("hmap", ImageTexture.create_from_image(himg))
	_win_mat.set_shader_parameter("cmap", ImageTexture.create_from_image(cimg))
	var mx := int(p["mx"])
	var my := int(p["my"])
	_win_mat.set_shader_parameter("h_xform", _tex_xform(mx, my))
	_win_mat.set_shader_parameter("c_xform", _tex_xform(mx, my))
	_win_mat.set_shader_parameter("h_lod", _texel_lod(maxi(mx, my)))
	last_fill_ms = float(p["ms"]) + (Time.get_ticks_usec() - t0) / 1000.0
	_win = {"z": p["z"], "c0": p["c0"], "r0": p["r0"], "k": p["k"], "x0": p["x0"], "y0": p["y0"],
		"w": p["w"], "h": p["h"], "mx": mx, "my": my, "himg": himg}
	## The window's rim blends to the backdrop: patch -> backdrop uv through
	## the shared sample space.
	var b := _back
	var tw := int(b["tw"])
	var th := int(b["th"])
	var sx := float(_win["w"]) / float(b["w"])
	var sy := float(_win["h"]) / float(b["h"])
	var ox := (float(_win["x0"]) - float(b["x0"])) / float(b["w"])
	var oy := (float(_win["y0"]) - float(b["y0"])) / float(b["h"])
	_win_mat.set_shader_parameter("hf_xform", Vector4(sx * (tw - 1) / tw, sy * (th - 1) / th,
		(0.5 + ox * (tw - 1)) / tw, (0.5 + oy * (th - 1)) / th))
	_win_mat.set_shader_parameter("rim", RIM_FRACTION)
	## The backdrop's hole: the window in backdrop patch units, shrunk by
	## `HOLE_INSET_RIMS` of the window's rim.
	var inset_x := sx * RIM_FRACTION * HOLE_INSET_RIMS
	var inset_y := sy * RIM_FRACTION * HOLE_INSET_RIMS
	if int(_win["k"]) >= (1 << int(_win["z"])):
		## The window is the whole world: nothing of the backdrop shows.
		_back_mat.set_shader_parameter("hole", Vector4(-1.0, -1.0, 2.0, 2.0))
	else:
		_back_mat.set_shader_parameter("hole", Vector4(ox + inset_x, oy + inset_y, ox + sx - inset_x, oy + sy - inset_y))
	_win_mi.visible = true
	reason = ""

## **Camera-relative placement** (§3.11): each mesh's origin is its footprint
## corner minus the camera position, in metres, subtracted here in 64-bit
## floats; `cam` is `View3DCamera.position()`'s `[x, y, z]`. `hag` is the
## camera's height above the ground, which sizes the backdrop's sink.
func place(cam: Array, hag: float) -> void:
	if _back.is_empty():
		return
	var sink := maxf(SINK_MIN_M, SINK_FRACTION * maxf(hag, 0.0))
	## Only the backdrop sinks. The window's rim eases to the backdrop's
	## unsunk relief, so across the underlap band the window stays `sink`
	## above the backdrop and wins the depth test there (bar B5).
	_back_mat.set_shader_parameter("sink_m", sink)
	_set_xf(_back_mi, _back, cam)
	if not _win.is_empty():
		_set_xf(_win_mi, _win, cam)

func _set_xf(mi: MeshInstance3D, f: Dictionary, cam: Array) -> void:
	var ox: float = float(f["x0"]) * _cell_m - float(cam[0])
	var oz: float = float(f["y0"]) * _cell_m - float(cam[2])
	var oy: float = -float(cam[1])
	mi.transform = Transform3D(Basis.from_scale(Vector3(float(f["w"]) * _cell_m, 1.0, float(f["h"]) * _cell_m)), Vector3(ox, oy, oz))

## **An upper bound of the drawn surface's height near `(x_m, z_m)`**, in
## metres: the camera's ground clamp reads it (bar B4). The drawn surface is
## the low-passed relief interpolated between vertices, so it never rises above
## the highest relief texel within `BOUND_STEPS` vertex steps; this reads the
## full-resolution relief over that neighbourhood. Inside the window it reads
## the window's relief, elsewhere the backdrop's (minus its sink, which only
## lowers it, so the bound stays an upper bound without it).
func surface_bound(x_m: float, z_m: float) -> float:
	if _cell_m <= 0.0:
		return 0.0
	var sx := x_m / _cell_m
	var sy := z_m / _cell_m
	var f := _back
	if not _win.is_empty() and sx >= float(_win["x0"]) and sx <= float(_win["x0"]) + float(_win["w"]) \
			and sy >= float(_win["y0"]) and sy <= float(_win["y0"]) + float(_win["h"]):
		f = _win
	if f.is_empty():
		return 0.0
	var img: Image = f["himg"]
	var iw := img.get_width()
	var ih := img.get_height()
	## Texels per vertex step, and so the neighbourhood's half-width in texels.
	var step := float(maxi(iw, ih) - 1) / float(PATCH_VERTS - 1)
	var r := int(ceil(maxf(1.0, BOUND_STEPS * step)))
	var u := int(round((sx - float(f["x0"])) / float(f["w"]) * (iw - 1)))
	var v := int(round((sy - float(f["y0"])) / float(f["h"]) * (ih - 1)))
	## Every texel when the neighbourhood is small enough, else a stride: the
	## window's 1 021 texels over 256 vertex steps give r = 8, read whole.
	var stride := maxi(1, int(ceil(2.0 * r / float(BOUND_SAMPLES_MAX - 1))))
	var best := 0.0
	for py in range(clampi(v - r, 0, ih - 1), clampi(v + r, 0, ih - 1) + 1, stride):
		for px in range(clampi(u - r, 0, iw - 1), clampi(u + r, 0, iw - 1) + 1, stride):
			best = maxf(best, img.get_pixel(px, py).r)
	return metres(best)

## Normalised height -> drawn metres: the shader's `metres()`, the law
## `sample_bridge::FieldRefs::elevation_m` reports with, flattened at sea level
## (§3.6), times the exaggeration.
func metres(h: float) -> float:
	return maxf((h - _sea) / maxf(1.0 - _sea, 1e-6) * _peak, 0.0) * _exag
