class_name ExportOverlay
extends Node
## The map overlay drawn for a 16K/32K image export (`EXPORT_SCOPE.md` §7,
## milestone E4, batch B): a copy of the live `map_overlay.gd` laid out at the
## export's own pixel size inside a `SubViewport`, rendered one tile at a time
## and read back as the premultiplied RGBA8 that
## `WorldGen.export_session_submit_tile` composites over the terrain.
##
## What it must never do: draw from the live overlay (an export is of the map as
## it was when the user pressed the button -- the copy owns its state, as
## `ExportSnapshot` owns the terrain's), or take input (it lives in a
## `SubViewport` with no container, and its `mouse_filter` is IGNORE).
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
## Batch B is the one-tile renderer and its registration. Many tiles per band,
## many bands, labels crossing a tile seam, the off-main-thread terrain render
## with progress (Ruling AZ: no UI freeze), and the dialog are batches C, D and
## E5 -- `OUTSTANDING_WORK.md`'s export row.

const OVERLAY_SCRIPT := preload("res://map_overlay.gd")

## The widest/tallest tile `render_tile` will make. A 32 768 px `SubViewport`
## read back a null image on this Godot version (`EXPORT_SCOPE.md` §5 point 1),
## so tiles are bounded on both axes. **Labelled judgement, not a measurement**:
## 4096 is a quarter of that failing size and the size the batch-B probe ran;
## batch C owns measuring the real ceiling on desktop and Android.
const TILE_MAX := 4096

var _vp: SubViewport = null
var _cam: Node2D = null
## The copy being drawn; null before `setup` succeeds.
var overlay: Control = null
## `layout()`'s answer for this export; empty before `setup`.
var _layout: Dictionary = {}

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
## reads it. The one translation: `settlements: true` with no tier means every
## settlement, `"all"` (the Rust parser's own default, `export_raster.rs`).
static func overlay_content(content: Dictionary) -> Dictionary:
	var out := content.duplicate()
	var tier := String(content.get("settlement_tier", ""))
	if tier == "" and bool(content.get("settlements", false)):
		tier = "all"
	if content.has("settlements") and not bool(content["settlements"]):
		tier = ""
	out["settlement_tier"] = tier
	return out

## Builds the copy of `live` (a `map_overlay.gd` instance with a world) for an
## `export_w x export_h` export of `content`, inside this node's own
## `SubViewport`. Returns an empty string, or why it could not -- no world, no
## rect, or a degenerate size -- in which case nothing was built.
func setup(live: Control, export_w: int, export_h: int, content: Dictionary) -> String:
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
	_cam = Node2D.new()
	_cam.scale = lay["scale"]
	_vp.add_child(_cam)
	overlay = OVERLAY_SCRIPT.new()
	overlay.copy_state_from(live)
	overlay.prepare_export(gain, overlay_content(content))
	_cam.add_child(overlay)
	overlay.position = Vector2.ZERO
	overlay.size = lay["size"]
	return ""

## The overlay over export pixels `[x, x+w) x [y, y+h)`, as `w*h*4` bytes of
## premultiplied RGBA8 (a `transparent_bg` readback, which is what
## `export_session_submit_tile` composites). Empty when `setup` did not run, the
## tile is empty or larger than `TILE_MAX`, or the readback failed. Awaits two
## drawn frames: the copy's content redraw queues its layer items, which Godot
## draws later in the same frame (`map_overlay.gd::_build_now`), and the second
## is margin for the viewport's own update.
func render_tile(x: int, y: int, w: int, h: int) -> PackedByteArray:
	if overlay == null or w <= 0 or h <= 0 or w > TILE_MAX or h > TILE_MAX:
		return PackedByteArray()
	_vp.size = Vector2i(w, h)
	_cam.position = (_layout["origin"] as Vector2) - Vector2(x, y)
	overlay.queue_redraw()
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	_vp.render_target_update_mode = SubViewport.UPDATE_DISABLED
	var img := _vp.get_texture().get_image()
	if img == null or img.get_width() != w or img.get_height() != h:
		return PackedByteArray()
	if img.get_format() != Image.FORMAT_RGBA8:
		img.convert(Image.FORMAT_RGBA8)
	return img.get_data()
