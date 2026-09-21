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

## Runs one more of the nine other stale-by-code paths through the same
## before/after, base-vs-screen shape section 2-4 above uses, but WITHOUT the
## hard `quit(2)` those sections use on a failed premise -- a later section's
## setup problem should not hide whether the earlier ones passed. `action`
## does the engine call AND the repaint (it IS the function under test, called
## exactly as its real caller calls it). `vh`/`screen_centre`/`half_w`/`half_h`
## are `_ready()`'s own locals, passed in because a nested `func` cannot close
## over another top-level function's locals in GDScript.
func _measure_path(name: String, action: Callable,
		vh: ViewportHost, screen_centre: Vector2, half_w: float, half_h: float) -> void:
	print("\n=== ", name, " ===")
	await _frames(3)
	var base_before := _crop_centre(vh.map_view.texture.get_image(), 0.3)
	var screen_before := _crop_at(_vp.get_texture().get_image(), screen_centre, half_w, half_h)
	await action.call()
	await _frames(20)
	var base_after := _crop_centre(vh.map_view.texture.get_image(), 0.3)
	var screen_after := _crop_at(_vp.get_texture().get_image(), screen_centre, half_w, half_h)
	var base_changed := base_before != base_after
	var screen_changed := screen_before != screen_after
	_ok(name + " -- PREMISE (base texture really changed)", base_changed)
	if not base_changed:
		print("  [SKIP] premise failed -- this path's setup did not change the field; ",
			"nothing to say about its LOD repaint")
		return
	_ok(name + " -- on-screen region shows the change", screen_changed,
		"base_changed=%s screen_changed=%s" % [base_changed, screen_changed])

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

	## === 5-14: the other nine "stale by code" paths (`OUTSTANDING_WORK.md`,
	## "the in-session tile cache is not invalidated by a sculpt") plus the
	## erode path found beside them. Each does its own real edit through the
	## bridge and then calls the EXACT shell function under test -- never
	## `refresh()`, which would hide a missing `invalidate_lod_tiles()` call
	## behind the reset it performs anyway.
	var rd = app.right_dock_ctrl
	var tb = app.tool_bar

	var stroke := func():
		bridge.sculpt_begin_stroke()
		bridge.sculpt_add_point(gx - 5.0, gy - 5.0)
		bridge.sculpt_add_point(gx - 1.0, gy - 1.0)
		bridge.sculpt_add_point(gx + 1.0, gy + 1.0)
		bridge.sculpt_add_point(gx + 5.0, gy + 5.0)
		bridge.sculpt_end_stroke()
		await _frames(3)

	print("\n=== 5: tool_bar.gd::_on_sculpt_commit (the 'Commit' chip) ===")
	await stroke.call()
	await _measure_path("tool_bar._on_sculpt_commit", func(): tb._on_sculpt_commit(),
		vh, screen_centre, half_w, half_h)

	print("\n=== 6: right_dock.gd::_on_sculpt_stack_commit ('Commit to map') ===")
	await stroke.call()
	await _measure_path("right_dock._on_sculpt_stack_commit", func(): rd._on_sculpt_stack_commit(),
		vh, screen_centre, half_w, half_h)

	## `land_only=false` so the dab lands whatever the cell under it is now,
	## after five height edits above -- the point under test is the repaint,
	## not the paint tool's own land/water gating. A distinct cell AND a
	## distinct paint value per call, but kept inside the SAME small offset
	## the earlier stroke used (+-6): the on-screen crop is a screen-space
	## window sized off `vh.size`, not the grid, so a +-16 grid-cell offset
	## painted well outside it at this zoom while still registering on the
	## wide texture-space BASE crop -- a probe bug, not a product one (first
	## run of this probe: sections 8/9 passed the premise and failed the
	## on-screen check because the dab was never in frame to begin with).
	## Radius widened to 14 (from the shell's own 6 default): a single small
	## dab can land entirely inside a patch that already carries the same
	## biome as the palette value being painted, which committed with zero
	## byte-level change and read as a false premise failure (measured: value
	## 1 at radius 6 did exactly that on one offset and not another, in two
	## otherwise-identical runs). A wide dab spans enough biome variety that
	## some of it differs from every palette value tried, almost always.
	var dab := func(ox: float, oy: float, value: int):
		bridge.paint_set_brush(value, 14.0, 1.0, 0.0, false, false)
		bridge.paint_stroke_at(gx + ox, gy + oy)
		await _frames(3)

	print("\n=== 7: world_workspace.gd::_on_paint_commit (paint commit 1/3) ===")
	await dab.call(-5.0, -5.0, 1)
	await _measure_path("world_workspace._on_paint_commit", func(): ws._on_paint_commit(),
		vh, screen_centre, half_w, half_h)

	print("\n=== 8: tool_bar.gd::_on_paint_commit (paint commit 2/3) ===")
	await dab.call(5.0, -5.0, 2)
	await _measure_path("tool_bar._on_paint_commit", func(): tb._on_paint_commit(),
		vh, screen_centre, half_w, half_h)

	print("\n=== 9: right_dock.gd::_on_paint_commit_from_dock (paint commit 3/3) ===")
	await dab.call(0.0, 5.0, 3)
	await _measure_path("right_dock._on_paint_commit_from_dock", func(): rd._on_paint_commit_from_dock(),
		vh, screen_centre, half_w, half_h)

	print("\n=== 10: world_workspace.gd::_run_erode ===")
	if bridge.has_world and bridge._has("erode_op"):
		await _measure_path("world_workspace._run_erode", func(): ws._run_erode(),
			vh, screen_centre, half_w, half_h)
	else:
		print("  [SKIP] this build's GDExtension has no WorldGen.erode_op()")

	print("\n=== 11: app.gd::undo_last ===")
	await stroke.call()
	await ws._on_sculpt_commit()   ## A fresh committed step for undo_last to pop --
	                                 ## already fixed, used only as setup here.
	await _frames(5)
	await _measure_path("app.undo_last", func(): app.undo_last(),
		vh, screen_centre, half_w, half_h)

	print("\n=== 12: app.gd::redo_last ===")
	await _measure_path("app.redo_last", func(): app.redo_last(),
		vh, screen_centre, half_w, half_h)

	print("\n=== 13: menus.gd::_redo_last ===")
	## A FRESH commit for setup, then `bridge.undo_last()` called RAW --
	## deliberately bypassing the shell's own repaint, since this section
	## means to measure `_redo_last()`'s repaint alone. But `map_view.texture`
	## is a plain field: skipping the repaint after the raw undo leaves it
	## showing the PRE-undo (committed) image, and `_measure_path`'s "before"
	## snapshot is read straight off it -- so without the explicit resync
	## below, "before" already equals what `_redo_last()` is about to
	## produce, and the premise fails on a stale baseline, not a real
	## no-op (measured: `_bridge.redo_last()` returned `true` -- the field
	## really did move -- while the crop still read unchanged).
	await stroke.call()
	await ws._on_sculpt_commit()   ## Setup only -- already fixed, proven above.
	await _frames(5)
	var undone_label13: String = bridge.undo_last()   ## Setup: NOT the path under test.
	if undone_label13 != "":
		vh.map_view.texture = bridge.color_texture()   ## Resync the baseline to the
		                                                 ## now-reverted field (see above).
		## And invalidate the LOD tiles for THIS raw setup undo too -- a real
		## undo path always repaints AND invalidates now, so skipping the
		## second half here would leave "before"'s ON-SCREEN region stuck
		## showing the pre-undo relief while "before"'s BASE crop already
		## shows the reverted one; "after" (post-redo, correctly invalidated
		## by the fix under test) would then match that same stale screen by
		## coincidence and the on-screen assertion would pass or fail for the
		## wrong reason. Measured: without this line, screen_changed read
		## `false` even though the fix's own `invalidate_lod_tiles()` call
		## demonstrably ran (PREMISE passed) -- the "before" screen was never
		## desynced from "after" in the first place.
		vh.invalidate_lod_tiles()
		await _frames(5)
		await _measure_path("menus._redo_last", func(): app.menus._redo_last(),
			vh, screen_centre, half_w, half_h)
	else:
		print("  [SKIP] nothing left on the undo stack to set up a redo with")

	## Picks a real `seq` from the live ledger rather than guessing one --
	## `undo_revert_to()` answers `0` (a silent no-op, not an error) for any
	## `seq` that is not a reversible height row.
	var height_rows := func() -> Array:
		var out: Array = []
		for row in bridge.undo_ledger():
			var d: Dictionary = row
			if String(d.get("kind", "")) == "height" and bool(d.get("reversible", false)):
				out.append(int(d["seq"]))
		out.sort()
		return out

	print("\n=== 14: right_dock.gd::_do_revert ===")
	var rows14: Array = height_rows.call()
	## The middle row, not the oldest: `_do_revert` drops every row AFTER its
	## target, and section 15 below needs an EARLIER row still on the ledger
	## to revert to in turn.
	if rows14.size() >= 2:
		var seq14: int = rows14[rows14.size() / 2]
		print("  info ledger has ", rows14.size(), " reversible height rows; reverting to seq=", seq14)
		await _measure_path("right_dock._do_revert(%d)" % seq14, func(): rd._do_revert(seq14),
			vh, screen_centre, half_w, half_h)
	else:
		print("  [SKIP] fewer than 2 reversible height rows on the ledger (", rows14.size(), ")")

	print("\n=== 15: dcc_shell.gd::_phone_revert_history ===")
	## Callable directly on `app` (`DccApp extends DccShell`) without booting
	## the phone shell: `_find_engine_bridge()`/`_find_viewport_host()` walk
	## the real tree, and `_show_phone_toast()` no-ops when `_phone` is false
	## (this probe never passes `--force-touch`), so nothing here needs the
	## phone UI to exist.
	var rows15: Array = height_rows.call()
	if rows15.size() >= 1:
		var seq15: int = rows15[0]
		print("  info ledger has ", rows15.size(), " reversible height rows; reverting to seq=", seq15)
		await _measure_path("app._phone_revert_history(%d)" % seq15,
			func(): app._phone_revert_history(seq15),
			vh, screen_centre, half_w, half_h)
	else:
		print("  [SKIP] no reversible height rows left on the ledger")

	print("\n_sculptlodcache_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
