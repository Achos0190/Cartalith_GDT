extends Node
## Reproduces, then proves the fix for, **"the in-session tile cache is not
## invalidated by a sculpt"** (`OUTSTANDING_WORK.md` §3.1, `GUI_GAP_REGISTER.md`).
##
## `viewport_host.gd` clears `_lod_tiles` in `refresh()` and in
## `_set_lod_active(false)`, but `world_workspace.gd::_on_sculpt_commit()`
## deliberately skips `refresh()` (its own comment: that would also reset the
## camera to fit) and writes `map_view.texture` directly instead. An
## already-built LOD tile `Sprite2D` holds its OWN synthesized shade-ratio
## texture plus a shader `base_tex` parameter captured from the OLD
## `map_view.texture` (`_build_lod_tile`) -- neither half has any reason to
## change just because the commit reassigned `map_view.texture` elsewhere, so
## with the pyramid active the screen keeps compositing pre-sculpt relief.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x900 _sculptlodcache_probe.tscn
##
## **Not `--headless`.** Sections 2 and 4 read the framebuffer back
## (`get_texture().get_image()`), which the dummy rasterizer under
## `--headless` cannot do (`MISTAKES.md`, "Run a pixel probe").
##
## Two crops of the SAME screen region, before/after ONE sculpt commit:
##   BASE    `map_view.texture` itself, cropped around the grid centre -- a
##           positive control that must ALWAYS change, independent of the
##           LOD layer, proving the stroke really altered the baked field.
##   SCREEN  the actual composited framebuffer over that same ground, cropped
##           around the viewport control's own centre -- the thing under
##           test. Unchanged there while BASE changed is the bug; moving
##           together is the fix.
##
## Exit 0 pass, 1 the real assertion failed, 2 could not even set up the
## premise (no `.gdextension`, no world, never reached deep zoom, the stroke
## did not land, or the base texture did not change).

var _vp: SubViewport
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, cond: bool, detail: String = "") -> void:
	print("  ", "ok  " if cond else "FAIL", " ", name, ("  -- " + detail) if detail != "" else "")
	if not cond:
		_fail += 1

## Centred square crop of `img`, `frac` of its SHORTER side per side.
func _crop_centre(img: Image, frac: float) -> PackedByteArray:
	var w := img.get_width()
	var h := img.get_height()
	var half := maxi(int(minf(w, h) * frac * 0.5), 1)
	var cx := w / 2
	var cy := h / 2
	var x0 := clampi(cx - half, 0, w - 1)
	var y0 := clampi(cy - half, 0, h - 1)
	var x1 := clampi(cx + half, x0 + 1, w)
	var y1 := clampi(cy + half, y0 + 1, h)
	return img.get_region(Rect2i(x0, y0, x1 - x0, y1 - y0)).get_data()

## Same idea, centred on an arbitrary pixel -- `img` here is the WHOLE
## window's framebuffer, not just the map, so the crop has to be placed
## explicitly rather than taking the image's own middle.
func _crop_at(img: Image, center: Vector2, half_w: float, half_h: float) -> PackedByteArray:
	var w := img.get_width()
	var h := img.get_height()
	var x0 := clampi(int(center.x - half_w), 0, w - 1)
	var y0 := clampi(int(center.y - half_h), 0, h - 1)
	var x1 := clampi(int(center.x + half_w), x0 + 1, w)
	var y1 := clampi(int(center.y + half_h), y0 + 1, h)
	return img.get_region(Rect2i(x0, y0, x1 - x0, y1 - y0)).get_data()

func _ready() -> void:
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] the .gdextension did not load.")
		get_tree().quit(2)
		return

	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(30)
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(15)
	print("[BOOT] shell up")

	var bridge = app.bridge
	var vh = app.viewport
	## **LOD-D6.** This probe reads `_lod_tiles` a frame or four after
	## moving the camera and asserts on what it finds, so it needs a tile
	## to exist the moment `_update_lod()` returns. Since LOD-D6 synthesis
	## runs on a worker and a tile lands some frames later, which is a
	## different premise from the one this probe was written against.
	## `_lod_sync` puts synthesis back on this thread; it moves no pixel
	## (both paths end in `lod_worker::LodSnapshot::render_tile`) and only
	## changes WHEN the tile appears. What this probe measures -- the
	## compositor -- is unaffected. `_d6async_probe.gd` is what exercises
	## the threading.
	vh._lod_sync = true
	var ws = app._world_workspace()

	print("\n=== 1: a world, zoomed past the LOD threshold ===")
	bridge.generate({
		"seed": 24601, "width_km": 640.0, "grid_w": 96, "grid_h": 64,
		"sea_level": 0.5, "villages": true,
	})
	var gen_ok = await bridge.generation_finished
	await _frames(10)
	_ok("generation reported ok", gen_ok)
	_ok("world present", bridge.has_world)
	if not bridge.has_world:
		print("[ABORT] no world -- nothing else here can run")
		get_tree().quit(2)
		return

	var guard := 0
	while vh.zoom() <= 2.3 and guard < 20:
		vh.zoom_step(1.35)
		guard += 1
		await _frames(2)
	await _frames(20)   ## Lets the tile backlog (`_process()`) finish before
	                     ## anything gets sampled. It also waited out
	                     ## `_set_lod_active`'s 0.15 s fade tween until LOD-D3
	                     ## removed that; the backlog alone still needs it.
	print("  info zoom=", vh.zoom(), " in ", guard, " steps")
	_ok("LOD is active", vh.lod_active())
	var tiles: Dictionary = vh.get("_lod_tiles")
	_ok("live LOD tiles exist", tiles.size() > 0, "count=%d" % tiles.size())
	if not vh.lod_active() or tiles.is_empty():
		print("[ABORT] never reached deep zoom with real tiles -- nothing else here can run")
		get_tree().quit(2)
		return

	## The stroke lands dead centre: `reset_view()` fits the whole grid
	## centred in the control, and `zoom_step()` always pivots on the
	## control's own `size * 0.5` (`_zoom_at`'s own doc comment on the
	## navpad's +/-), so the grid's centre cell is the one screen point that
	## stays under the control's own centre at any zoom this loop reaches.
	var g = bridge.grid_size()
	var gx: float = g.x * 0.5
	var gy: float = g.y * 0.5
	var screen_centre: Vector2 = vh.global_position + vh.size * 0.5
	var half_w: float = vh.size.x * 0.2
	var half_h: float = vh.size.y * 0.2
	print("  info grid=", g, " stroke centred at (", gx, ",", gy,
		")  screen centre=", screen_centre, " crop half=", half_w, "x", half_h)

	print("\n=== 2: baseline, before the commit ===")
	await _frames(3)
	var base_before := _crop_centre(vh.map_view.texture.get_image(), 0.3)
	var screen_before := _crop_at(_vp.get_texture().get_image(), screen_centre, half_w, half_h)

	print("\n=== 3: a real stroke, committed through the exact path under test ===")
	bridge.sculpt_begin_stroke()
	bridge.sculpt_add_point(gx - 6.0, gy - 6.0)
	bridge.sculpt_add_point(gx - 2.0, gy - 2.0)
	bridge.sculpt_add_point(gx + 2.0, gy + 2.0)
	bridge.sculpt_add_point(gx + 6.0, gy + 6.0)
	bridge.sculpt_end_stroke()
	await _frames(3)
	var stamps: int = bridge.sculpt_stamp_count()
	_ok("the stroke really landed", stamps > 0, "stamps=%d" % stamps)
	if stamps == 0:
		print("[ABORT] no draft to commit -- nothing else here can run")
		get_tree().quit(2)
		return

	ws._on_sculpt_commit()   ## `world_workspace.gd`'s sculpt-commit path --
	                          ## the exact function this row is about.
	await _frames(20)

	print("\n=== 4: after the commit, the same two regions ===")
	var base_after := _crop_centre(vh.map_view.texture.get_image(), 0.3)
	var screen_after := _crop_at(_vp.get_texture().get_image(), screen_centre, half_w, half_h)

	var base_changed := base_before != base_after
	var screen_changed := screen_before != screen_after
	_ok("PREMISE -- the commit really rebaked the base texture (positive control)",
		base_changed, "%d bytes sampled" % base_before.size())
	if not base_changed:
		print("[ABORT] premise failed -- the stroke did not change the baked field; ",
			"this run cannot say anything about the LOD layer")
		get_tree().quit(2)
		return
	_ok("the ON-SCREEN (LOD-composited) region shows the post-sculpt relief",
		screen_changed, "base_changed=%s screen_changed=%s (%d bytes sampled)"
			% [base_changed, screen_changed, screen_before.size()])

	print("\n_sculptlodcache_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
