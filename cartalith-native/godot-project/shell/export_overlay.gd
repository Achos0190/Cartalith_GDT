class_name ExportOverlay
extends Node
## The map overlay drawn for a 16K/32K image export (`EXPORT_SCOPE.md` §7,
## milestone E4): a copy of the live `map_overlay.gd` laid out at the export's
## own pixel size inside a `SubViewport`, rendered one tile at a time and read
## back as the premultiplied RGBA8 that `WorldGen.export_session_submit_tile`
## composites over the terrain -- and (`run`) the driver that takes a whole
## export through the session, band by band, without freezing the UI.
##
## What it must never do: draw from the live overlay (an export is of the map as
## it was when the user pressed the button -- the copy owns its state, as
## `ExportSnapshot` owns the terrain's), take input (it lives in a `SubViewport`
## with no container, and its `mouse_filter` is IGNORE), or call `WorldGen`
## directly (every engine call goes through `EngineBridge`'s gated
## `export_session_*` / `urban_layouts`, which refuse while a generation holds
## the engine).
##
## **Registration.** The terrain band samples cell `c` at export pixel
## `c * (W-1)/(gw-1)` (`render.rs::bake_rect`, corner-aligned), while the live
## overlay draws cell `c`'s centre at texture coordinate `u = c + 0.5`
## (texel-centred, `map_overlay.gd::_cell_to_screen`). So the overlay copy is laid
## out with exactly `q.x = (W-1)/(gw-1)` local pixels per cell, filling its rect,
## and a parent `Node2D` maps local `L` to export pixel
## `L * (1, q.y/q.x) + (-q.x/2, -q.y/2)` -- per axis, because `(H-1)/(gh-1)` is
## not `q.x` and the overlay's own fit is uniform (`EXPORT_SCOPE.md` §5 point 3:
## a naive "size W, zoom 1" camera lands ≈8 px off at 32K's edges). The camera
## is a parent node rather than `SubViewport.canvas_transform`, which the
## urban-layout culler reads around (same point).
##
## **Uniform magnification** (owner, 2026-09-23, §5 point 4): the copy is the
## live overlay's fit view (camera zoom 1) at `symbol_gain()` times its size;
## `map_overlay.gd::prepare_export` carries the gain into every size the live
## overlay holds constant in screen pixels.
##
## **Borders** (Ruling AZ's overlay list; `content.territory`): the faction
## territory wash and the province lines are not this overlay -- they are two
## engine textures the live map shows in their own `TextureRect`s
## (`viewport_host.gd`'s `territory_view` / `province_view`). The export draws
## the same two textures, cell for cell where the live raster's nearest filter
## puts them, with the province line's style ink, behind the overlay as the
## live stack orders them -- through `BORDER_SHADER`, which says why not a
## magnified `TextureRect`. Labelled judgement: the owner's "borders" is read as
## both, since the province lines are the only other border the map draws.
##
## **Tile seams** (batch C). Every tile is the same copy, shifted by a whole
## number of pixels, so a symbol crossing a seam is drawn by both tiles from the
## same geometry. What did depend on the tile was changed: which towns stand
## their pin down and how far past the tile a town plan is drawn
## (`map_overlay.gd::_urban_plan_export`), the borders' texel choice
## (`BORDER_SHADER`), and the neatline clip, now a whole-pixel parent `Control`
## (`clip_pixels`). What remains is the renderer's own: antialiasing that moves
## a pixel by a level or two (seen with no seam at all when only the tile origin
## moves), and one lone pixel where a town plan's polygon edge falls on a pixel
## centre (a suspected rasterisation tie: the origin-only control did not
## reproduce that pixel). `_exportoverlay_probe.gd` §7-§8 measure it.

## Emitted by `run` as it goes: the fraction done (0..1, by band, the band's tiles
## pro rata), and a short sentence for a progress line.
signal progress(fraction: float, text: String)

const OVERLAY_SCRIPT := preload("res://map_overlay.gd")

## The widest/tallest tile `render_tile` will make, and `run`'s default tile.
## A 32 768 px `SubViewport` read back a null image on this Godot version
## (`EXPORT_SCOPE.md` §5 point 1), so tiles are bounded on both axes.
## **Labelled judgement, partly measured**: `_exportoverlay_probe.gd` §9 read
## back 8192 and (once) 16384 px square tiles on the dev desktop
## (`EXPORT_SCOPE.md` §5, "E4 batches C and D"), so 4096 is well below the
## measured desktop ceiling; it stays the default because Android's limit is
## unmeasured
## and likely lower, and at two drawn frames a tile (`render_tile`) a 4096 px
## tile already keeps the tile count of a 32K export in the low hundreds.
const TILE_MAX := 4096

## Town layouts fetched per frame before the first tile (`_fetch_urban`), as a
## time budget. A town is a few milliseconds of generation on the main thread
## (`viewport_host.gd::_on_urban_layouts_needed`); 8 ms of a 16.7 ms frame keeps
## the UI moving while a large roster loads. Labelled judgement.
const URBAN_FETCH_BUDGET_MS := 8.0
## Towns per engine call inside that budget: small, so one call cannot overrun
## it by much. Labelled judgement.
const URBAN_FETCH_BATCH := 4

var _vp: SubViewport = null
## The neatline clip (`clip_pixels`): a whole-pixel `Control` with
## `clip_contents`, moved per tile; the camera and the copy hang under it.
var _clip: Control = null
## Its rect in export pixels.
var _clip_px := Rect2()
var _cam: Node2D = null
## The copy being drawn; null before `setup` succeeds.
var overlay: Control = null
## `layout()`'s answer for this export; empty before `setup`.
var _layout: Dictionary = {}
## Set by `cancel()`; `run` aborts the session at its next step.
var _cancelled := false
## The longest single main-thread step of each kind in the last `run`, in ms:
## each engine call by method name, `"setup"` (the overlay copy), and
## `"readback"` (a tile's `get_image`). What a long frame during an export is
## made of -- a measurement aid, read by the probes; reset by every `run`.
var step_ms: Dictionary = {}

## Records `ms` against `key` in `step_ms` if it is the longest yet.
func _note(key: String, ms: float) -> void:
	step_ms[key] = maxf(float(step_ms.get(key, 0.0)), ms)

## The export's overlay geometry for an `export_w x export_h` image of a
## `gw x gh` grid: `size` (the overlay copy's local size: the grid at `q.x` px
## per cell), `scale` and `origin` (the parent node's transform, before a tile's
## offset), and `q` (export pixels per cell, per axis). Empty for a degenerate
## image or grid (fewer than two pixels or cells on an axis), where the
## corner-aligned mapping divides by zero.
static func layout(export_w: int, export_h: int, gw: int, gh: int) -> Dictionary:
	if export_w < 2 or export_h < 2 or gw < 2 or gh < 2:
		return {}
	var qx := float(export_w - 1) / float(gw - 1)
	var qy := float(export_h - 1) / float(gh - 1)
	return {"size": Vector2(gw, gh) * qx, "scale": Vector2(1.0, qy / qx),
		"origin": Vector2(-0.5 * qx, -0.5 * qy), "q": Vector2(qx, qy)}

## Export pixel of texture coordinate `u` (the live overlay's: a cell's centre
## is `cell + 0.5`) under `lay` = `layout(...)`. The same affine `render_tile`
## puts on the camera, written out for a probe to check against `bake_rect`.
static func texture_to_export(u: Vector2, lay: Dictionary) -> Vector2:
	return (u - Vector2(0.5, 0.5)) * (lay["q"] as Vector2)

## Uniform magnification: the export rect's width over the live overlay's
## displayed rect width. `0.0` when the live overlay has no rect yet.
static func symbol_gain(lay: Dictionary, live_rect_w: float) -> float:
	if lay.is_empty() or live_rect_w <= 0.0:
		return 0.0
	return (lay["size"] as Vector2).x / live_rect_w

## `export_options.rs`'s `content` dictionary as `map_overlay.gd::prepare_export`
## reads it. The one translation: `settlements: true` with no tier means towns,
## `"town"` -- the Rust parser's own default (`export_options.rs::
## SettlementTier::DEFAULT`, Ruling AZ); it was `"all"` before 2026-10-13, and
## the two must move together.
static func overlay_content(content: Dictionary) -> Dictionary:
	var out := content.duplicate()
	var tier := String(content.get("settlement_tier", ""))
	if tier == "" and bool(content.get("settlements", false)):
		tier = "town"
	if content.has("settlements") and not bool(content["settlements"]):
		tier = ""
	out["settlement_tier"] = tier
	return out

## The tiles that cover rows `[y0, y0+rows)` of a `w`-wide image, at most
## `tile_max` on a side, row-major: `[x, y, w, h]` each. Every pixel exactly
## once -- the session refuses a gap or an overlap (`export_session.rs`).
static func band_tiles(w: int, y0: int, rows: int, tile_max: int) -> Array:
	var out := []
	if w <= 0 or rows <= 0 or tile_max <= 0:
		return out
	var y := y0
	while y < y0 + rows:
		var th := mini(tile_max, y0 + rows - y)
		var x := 0
		while x < w:
			var tw := mini(tile_max, w - x)
			out.append([x, y, tw, th])
			x += tw
		y += th
	return out

## Builds the copy of `live` (a `map_overlay.gd` instance with a world) for an
## `export_w x export_h` export of `content`, inside this node's own
## `SubViewport`. `borders` are the live map's territory and province
## `TextureRect`s, drawn behind the copy when `content.territory` is on (see
## the file header); pass none to draw no borders. Returns an empty string, or
## why it could not -- no world, no rect, or a degenerate size -- in which case
## nothing was built.
func setup(live: Control, export_w: int, export_h: int, content: Dictionary, borders: Array = []) -> String:
	var lay := layout(export_w, export_h, int(live.get("_gw")), int(live.get("_gh")))
	if lay.is_empty():
		return "degenerate export %dx%d of a %dx%d grid" % [export_w, export_h, live.get("_gw"), live.get("_gh")]
	var gain := symbol_gain(lay, live.displayed_rect().size.x)
	if gain <= 0.0:
		return "the live overlay has no displayed rect to size symbols against"
	_layout = lay
	_vp = SubViewport.new()
	_vp.transparent_bg = true
	_vp.disable_3d = true
	_vp.render_target_update_mode = SubViewport.UPDATE_DISABLED
	add_child(_vp)
	_borders.clear()
	if bool(content.get("territory", false)):
		for src in borders:
			var t := _border_layer(src as TextureRect, lay["q"])
			if t != null:
				_vp.add_child(t)
				_borders.append(t)
	overlay = OVERLAY_SCRIPT.new()
	overlay.copy_state_from(live)
	overlay.prepare_export(gain, overlay_content(content))
	overlay.position = Vector2.ZERO
	overlay.size = lay["size"]
	_clip_px = clip_pixels(overlay.export_clip_rect(), lay, export_w, export_h)
	_clip = Control.new()
	_clip.clip_contents = true
	_clip.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_clip.size = _clip_px.size
	_vp.add_child(_clip)
	_cam = Node2D.new()
	_cam.scale = lay["scale"]
	_cam.position = (lay["origin"] as Vector2) - _clip_px.position
	_clip.add_child(_cam)
	_cam.add_child(overlay)
	return ""

## The overlay's neatline clip in export pixels: `local` (the copy's
## `export_clip_rect()`, in its local space) through `lay`'s affine, each edge
## rounded to the nearest whole pixel; the whole `w x h` image when `local` is
## empty (no frame, no clip). Whole pixels because a clip edge at a fraction is
## rounded by the renderer per tile, and a tile-dependent rounding is a seam
## (`map_overlay.gd::_draw`). Within half a pixel of the live overlay's edge.
static func clip_pixels(local: Rect2, lay: Dictionary, w: int, h: int) -> Rect2:
	if local.size.x <= 0.0 or local.size.y <= 0.0:
		return Rect2(0, 0, w, h)
	var s: Vector2 = lay["scale"]
	var o: Vector2 = lay["origin"]
	var p0 := (local.position * s + o).round()
	var p1 := (local.end * s + o).round()
	return Rect2(p0, p1 - p0)

## The border layers of this export (`_border_layer`), resized and re-aimed per
## tile by `render_tile`.
var _borders: Array[ColorRect] = []

## The border raster's shader: each export pixel fetches the grid cell whose
## footprint holds it, from the pixel's GLOBAL coordinate.
##
## **Why a shader and not a magnified `TextureRect`** (measured, batch C): with
## a `TextureRect` under the camera, nearest sampling decides a pixel whose
## centre lies within float error of a texel edge by interpolated UVs, whose
## rounding depends on the tile's size and position -- so one tile and twelve
## tiles of the same 4096 px export disagreed on two whole columns of province
## line and wash (35 pixels by up to 103 levels, `_exportoverlay_probe.gd` §8's
## first run). Here the cell index is computed from `tile_origin +
## floor(FRAGCOORD)` -- an exact integer pixel, the same for that pixel in any
## tile -- so the same pixel takes the same cell whatever tile draws it.
## `FRAGCOORD` counts from the viewport's top-left (measured: scratch
## `_fragdiag`, gl_compatibility); `floor` makes the pixel index exact whether
## the renderer puts the centre at `x` or `x + 0.5`.
##
## Cell `c` covers export pixels `[c*q - q/2, c*q + q/2)` per axis: the live
## raster's texel-centred cell, carried through `layout()`'s affine, i.e. where
## the overlay's own cell centre lands (`texture_to_export`). `restyle` is the
## live province line's `BORDER_STYLE_SHADER` (`viewport_host.gd`): the
## preset's ink over the texture's own alpha.
const BORDER_SHADER := "shader_type canvas_item;\n" \
	+ "uniform sampler2D tex : filter_nearest;\n" \
	+ "uniform vec2 tile_origin;\n" \
	+ "uniform vec2 q;\n" \
	+ "uniform bool restyle = false;\n" \
	+ "uniform vec4 border_ink = vec4(0.0);\n" \
	+ "uniform float border_alpha = 1.0;\n" \
	+ "void fragment() {\n" \
	+ "\tvec2 px = tile_origin + floor(FRAGCOORD.xy) + 0.5;\n" \
	+ "\tivec2 c = ivec2(floor(px / q + 0.5));\n" \
	+ "\tivec2 g = textureSize(tex, 0);\n" \
	+ "\tif (c.x < 0 || c.y < 0 || c.x >= g.x || c.y >= g.y) { COLOR = vec4(0.0); }\n" \
	+ "\telse {\n" \
	+ "\t\tvec4 t = texelFetch(tex, c, 0);\n" \
	+ "\t\tCOLOR = restyle ? vec4(border_ink.rgb, t.a * border_alpha) : t;\n" \
	+ "\t}\n" \
	+ "}\n"

## One border layer: a viewport-sized `ColorRect` (at the viewport's origin, not
## under the camera) drawing `src`'s grid texture through `BORDER_SHADER`, with
## `src`'s modulation and, for the province lines, its style material's ink and
## alpha. Null for a missing source or texture (no territory: a reopened
## archive without a claim grid, `lib.rs::build_territory_texture`).
func _border_layer(src: TextureRect, q: Vector2) -> ColorRect:
	if src == null or src.texture == null:
		return null
	var sh := Shader.new()
	sh.code = BORDER_SHADER
	var m := ShaderMaterial.new()
	m.shader = sh
	m.set_shader_parameter("tex", src.texture)
	m.set_shader_parameter("q", q)
	var styled := src.material as ShaderMaterial
	if styled != null and styled.get_shader_parameter("border_ink") is Color:
		m.set_shader_parameter("restyle", true)
		m.set_shader_parameter("border_ink", styled.get_shader_parameter("border_ink"))
		m.set_shader_parameter("border_alpha", float(styled.get_shader_parameter("border_alpha")))
	var r := ColorRect.new()
	r.material = m
	r.color = Color.WHITE
	r.modulate = src.modulate * src.self_modulate
	r.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return r

## The overlay over export pixels `[x, x+w) x [y, y+h)`, as `w*h*4` bytes of
## premultiplied RGBA8 (a `transparent_bg` readback, which is what
## `export_session_submit_tile` composites). Empty when `setup` did not run, the
## tile is empty or larger than `TILE_MAX` on a side (or `max_side`, for a
## measurement), or the readback failed. Awaits two drawn frames: the copy's
## content redraw queues its layer items, which Godot draws later in the same
## frame (`map_overlay.gd::_build_now`), and the second is margin for the
## viewport's own update.
func render_tile(x: int, y: int, w: int, h: int, max_side: int = TILE_MAX) -> PackedByteArray:
	if overlay == null or w <= 0 or h <= 0 or w > max_side or h > max_side:
		return PackedByteArray()
	_vp.size = Vector2i(w, h)
	## Only the clip moves per tile, by whole pixels; the camera under it is
	## fixed (`setup`), so the overlay's transform is the same in every tile.
	_clip.position = _clip_px.position - Vector2(x, y)
	for b in _borders:
		b.position = Vector2.ZERO
		b.size = Vector2(w, h)
		(b.material as ShaderMaterial).set_shader_parameter("tile_origin", Vector2(x, y))
	overlay.queue_redraw()
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	_vp.render_target_update_mode = SubViewport.UPDATE_DISABLED
	var t0 := Time.get_ticks_usec()
	var img := _vp.get_texture().get_image()
	if img == null or img.get_width() != w or img.get_height() != h:
		return PackedByteArray()
	if img.get_format() != Image.FORMAT_RGBA8:
		img.convert(Image.FORMAT_RGBA8)
	var data := img.get_data()
	_note("readback", (Time.get_ticks_usec() - t0) / 1000.0)
	return data

## Ask `run` to stop: it aborts the session (removing the partial file) at its
## next step and returns `{ok: false, cancelled: true}`.
func cancel() -> void:
	_cancelled = true

## **One whole export with overlays**, end to end (E4 batches C and D): opens
## the session (`export_session_begin`, which snapshots the terrain), builds the
## overlay copy from `live` (and `borders`, see `setup`), fetches the town plans
## the export will draw, then for every band waits -- frame by frame, never
## blocking -- until its terrain has rendered on the engine's own thread
## (`export_session_poll`), draws its tiles (`band_tiles`, `tile_max` on a side)
## and submits them, and closes the file (`export_session_close`, polled -- the
## writer finishes on its own thread). Emits `progress` throughout.
##
## Ruling AZ's "no UI freeze": nothing here waits inside a frame. The terrain
## renders off the main thread (`export_session.rs`, `BandFeed`), and so does
## the file's last write (`export_session_close`); a tile is two frames of
## drawing; a town layout fetch is budgeted per frame. A generation
## started mid-export is waited out (the bridge answers `busy`), since the
## session renders from its own snapshot and the copy owns its own state.
##
## Returns the session's `{ok, path, width, height, format, color_space, bytes,
## bands, ms}`, or `{ok: false, error}` with the partial file removed --
## `cancelled: true` beside it when `cancel()` stopped it. Must be called on a
## node inside the tree (it awaits frames); one `run` at a time per node.
func run(bridge: Node, path: String, opts: Dictionary, live: Control, borders: Array = [], tile_max: int = TILE_MAX) -> Dictionary:
	_cancelled = false
	step_ms = {}
	if tile_max <= 0 or tile_max > TILE_MAX:
		return {"ok": false, "error": "tile size %d is outside 1..%d" % [tile_max, TILE_MAX]}
	progress.emit(0.0, "Opening the export")
	var begun: Dictionary = await _engine(bridge, "export_session_begin", [path, opts])
	if not bool(begun.get("ok", false)):
		return begun
	var W := int(begun["width"])
	var H := int(begun["height"])
	var bands := int(begun["bands"])
	var band_rows := int(begun["band_rows"])
	var ts := Time.get_ticks_usec()
	var why := setup(live, W, H, opts.get("content", {}), borders)
	_note("setup", (Time.get_ticks_usec() - ts) / 1000.0)
	if why != "":
		return await _fail(bridge, why)
	var fetched: String = await _fetch_urban(bridge)
	if fetched != "":
		return await _fail(bridge, fetched)
	for b in bands:
		var y0 := b * band_rows
		var rows := mini(band_rows, H - y0)
		while true:
			if _cancelled:
				return await _fail(bridge, "cancelled")
			var p: Dictionary = await _engine(bridge, "export_session_poll", [])
			if not bool(p.get("ok", false)):
				return _done(p)
			if int(p.get("next_band", -1)) != b:
				return await _fail(bridge, "the session is at band %s where band %d was due" % [p.get("next_band"), b])
			if bool(p.get("band_ready", false)):
				break
			progress.emit(float(b) / bands, "Rendering terrain band %d of %d" % [b + 1, bands])
			await get_tree().process_frame
		var tiles := band_tiles(W, y0, rows, tile_max)
		for ti in tiles.size():
			if _cancelled:
				return await _fail(bridge, "cancelled")
			var t: Array = tiles[ti]
			var rgba := await render_tile(t[0], t[1], t[2], t[3], tile_max)
			if rgba.is_empty():
				return await _fail(bridge, "the overlay tile at (%d, %d) %dx%d could not be read back" % t)
			var sub: Dictionary = await _engine(bridge, "export_session_submit_tile", [b, t[0], t[1], t[2], t[3], rgba])
			if not bool(sub.get("ok", false)):
				return _done(sub)
			progress.emit((float(b) + float(ti + 1) / tiles.size()) / bands,
				"Drawing band %d of %d" % [b + 1, bands])
	## Closed off the main thread: the writer's last band is ~270 ms of
	## encoding at 4096 px (`export_session_finish` measured), so the file is
	## handed to the engine's closer and polled once a frame.
	while true:
		var fin: Dictionary = await _engine(bridge, "export_session_close", [])
		if not bool(fin.get("pending", false)):
			if bool(fin.get("ok", false)):
				progress.emit(1.0, "Export written")
			return _done(fin)
		progress.emit(1.0, "Writing the file")
		await get_tree().process_frame
	return {}

## A bridge call that waits out a generation: while the bridge answers `busy`
## (a run holds the engine), it retries a frame later. Any other answer is
## returned as it came.
func _engine(bridge: Node, method: String, args: Array) -> Dictionary:
	while true:
		var t0 := Time.get_ticks_usec()
		var r: Dictionary = bridge.callv(method, args)
		_note(method, (Time.get_ticks_usec() - t0) / 1000.0)
		if not bool(r.get("busy", false)):
			return r
		await get_tree().process_frame
	return {}

## Every town plan the copy will draw and lacks (`map_overlay.gd::
## export_urban_needs`), fetched through the bridge a few at a time within
## `URBAN_FETCH_BUDGET_MS` a frame, and handed to the copy. `""`, or why not.
## A refused town is recorded by `set_urban_layouts` and drawn as its pin.
func _fetch_urban(bridge: Node) -> String:
	var need: PackedInt32Array = overlay.export_urban_needs()
	var i := 0
	while i < need.size():
		if _cancelled:
			return "cancelled"
		if bridge.get("generating"):
			await get_tree().process_frame
			continue
		var t0 := Time.get_ticks_usec()
		while i < need.size() and float(Time.get_ticks_usec() - t0) < URBAN_FETCH_BUDGET_MS * 1000.0:
			var batch := need.slice(i, mini(i + URBAN_FETCH_BATCH, need.size()))
			overlay.set_urban_layouts(batch, bridge.urban_layouts(batch))
			i += batch.size()
		progress.emit(0.0, "Laying out towns (%d of %d)" % [i, need.size()])
		await get_tree().process_frame
	return ""

## Abort the session (waiting out a generation first), tear the copy down, and
## return `{ok: false, error: why}` -- with `cancelled: true` when `why` is
## `"cancelled"`.
func _fail(bridge: Node, why: String) -> Dictionary:
	await _engine(bridge, "export_session_abort", [])
	var out := {"ok": false, "error": why}
	if why == "cancelled":
		out["cancelled"] = true
	return _done(out)

## Free the `SubViewport` and the copy, and pass `r` through.
func _done(r: Dictionary) -> Dictionary:
	if _vp != null:
		_vp.queue_free()
	_vp = null
	_clip = null
	_cam = null
	_borders.clear()
	overlay = null
	_layout = {}
	return r
