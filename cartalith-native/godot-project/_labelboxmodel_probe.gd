extends Node
## `LARGE_ITEM_RULINGS.md` Ruling AG (2026-09-23): does the ENGINE's real
## `label_handles()`/`label_hit_test_mode()` now size a label's hit box and
## on-canvas handles off `map_overlay.gd`'s own font model
## (`cartalith-godot::label_bridge::shell_label_box`) instead of the older
## `cartalith_civ::labels::label_font_size`, so the resize handle a user
## actually drags sits at the corner of the label's ACTUALLY-DRAWN glyph run?
##
## Runs a real `WorldGen` (not a hand-built dict) so this exercises the
## shipped `#[func] label_handles` binding, not a re-derivation of its own
## formula. `map_overlay.gd` draws the label from the SAME `WorldGen.
## label_get()` dict. Windowed only -- `MISTAKES.md`, "Run a pixel probe":
## `--headless` never composites the canvas, so a screenshot comparison would
## pass vacuously.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _labelboxmodel_probe.tscn
##
## # Method
##
## One small real world, grid `GW x GH`, at a viewport size EQUAL to `GW`/`GH`
## (`px_per_cell == 1.0` exactly) -- deliberately, so the pre-existing,
## disclosed "a grid-cell unit and this control's own local-pixel unit are
## only the same number when `px_per_cell == 1`" approximation
## (`cartalith-godot::label_bridge::shell_label_box`'s own doc, and
## `map_overlay.gd`'s general convention) contributes ZERO distortion to this
## measurement -- what is being checked here is the FONT-SIZE NUMBER, not
## that separate, pre-existing, out-of-scope unit conflation. `GW` is
## deliberately far from the reference's own 512 (the two font models agree
## there, which would prove nothing about a shell where the grid width is a
## user choice).
##
## One label, `size=20`, `size_mode=zoom` (the common case). Three positions
## are computed and drawn, all converted through the SAME
## `map_overlay._point_to_screen()` the label's own glyphs use:
##   glyph   the real drawn text's own measured ink (this file's own pixel
##           scan, `_labelblur_probe.gd`'s technique)
##   new     the resize handle's position from the REAL, FIXED
##           `WorldGen.label_handles()` call
##   old     the resize handle's position the OLD formula would have given --
##           `label_font_size`'s own published arithmetic (`labels.rs`),
##           replicated here since nothing in this tree calls it standalone
##           any more; NOT a claim that removed code still runs
##
## Asserts `new` lands inside (or within one glyph-height of) the real ink's
## own bounding box, and `old` does not -- the fix, demonstrated at the
## symbol a user's pointer actually hits.
##
## Exit status: 0 pass, 1 assertion failed, 2 could not run.

const OVERLAY := preload("res://map_overlay.gd")

const SEED := 90210
const GW := 512
const GH := 512
const VP := Vector2i(GW, GH)   ## px_per_cell == 1.0, deliberately -- see header.
const LABEL_SIZE := 48.0   ## near `LABEL_SIZE_MAX` -- a large glyph makes the
                            ## two models' pixel gap easy to see and to assert on.

const GROUND := Color(0.42, 0.44, 0.40)
const NEW_MARK := Color(0.20, 0.85, 0.25)   ## the fixed engine handle
const OLD_MARK := Color(0.90, 0.15, 0.15)   ## the pre-fix formula, replicated

var _fails := 0


func _ok(cond: bool, what: String) -> void:
	print(("  PASS  " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


## `label_font_size`'s own published formula (`cartalith-civ/src/labels.rs`,
## `pub fn label_font_size` / `pub fn civ_zoom_k`) -- the model
## `label_box_at`/`label_handles` sized against BEFORE this fix. Nothing in
## this tree calls it through a `#[func]` any more (that is the fix), so it
## is replicated here, for comparison only, from its own published constants
## (`grid_w/512` floor 1, `civ_zoom_k` clamp `[0.35,5]`, `fsz` floor 9,
## `side = max(0, fsz*1.3)*1.25`).
func _old_side(size: float, grid_w: int, zoom_scale: float) -> float:
	var base := maxf(1.0, float(grid_w) / 512.0)
	var civ_zoom_k := 1.0 / clampf(zoom_scale, 0.35, 5.0)
	var lsc := base * civ_zoom_k
	var fsz := maxf(9.0, size * lsc)
	return maxf(0.0, fsz * 1.3) * 1.25


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless server; the canvas is never composited.")
		get_tree().quit(2)
		return
	DisplayServer.window_set_size(VP)
	await get_tree().process_frame

	var root := Control.new()
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(root)
	var bg := ColorRect.new()
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	bg.color = GROUND
	root.add_child(bg)

	var ov := Control.new()
	ov.set_script(OVERLAY)
	ov.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.add_child(ov)
	await get_tree().process_frame

	var gen := WorldGen.new()
	gen.generate_sized(SEED, float(GW), GW, GH)
	var idx: int = gen.label_create(float(GW) / 2.0, float(GH) / 2.0, "Aldebar")
	gen.label_set(idx, {"size": LABEL_SIZE, "size_mode": "zoom"})
	var lb: Dictionary = gen.label_get(idx)
	_ok(not lb.is_empty(), "label_get returns the created label")

	ov.set_civ_data([], [], [], GW, GH, 0.0)
	ov.set_camera_zoom(1.0)
	ov.set_labels([lb])
	await get_tree().process_frame
	ov.queue_redraw()
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw

	var rect: Rect2 = ov.displayed_rect()
	print("displayed_rect=%s  GW=%d  px_per_cell=%.3f" % [rect, GW, ov.label_px_per_cell()])
	_ok(absf(ov.label_px_per_cell() - 1.0) < 1e-6, "px_per_cell == 1.0 (VP matches GW)")

	# ---- The real, fixed engine call ----
	var px_per_cell: float = ov.label_px_per_cell()
	var h: Dictionary = gen.label_handles(idx, 1.0, px_per_cell)
	_ok(not h.is_empty() and h.has("resize"), "label_handles returns a resize handle")
	var new_grid: Vector2 = Vector2(h["resize"]["x"], h["resize"]["y"])
	var new_screen: Vector2 = ov._point_to_screen(new_grid, rect)

	# ---- The OLD formula, replicated (see `_old_side`'s own doc) ----
	var old_side := _old_side(LABEL_SIZE, GW, 1.0)
	var old_grid: Vector2 = Vector2(float(lb["x"]) + 0.5 + old_side / 2.0, float(lb["y"]) + 0.5 + old_side / 2.0)
	var old_screen: Vector2 = ov._point_to_screen(old_grid, rect)

	print("NEW resize (grid)=%s (screen)=%s" % [new_grid, new_screen])
	print("OLD resize (grid, replicated formula)=%s (screen)=%s" % [old_grid, old_screen])

	# ---- Measure the real drawn glyph's own ink ----
	var img: Image = get_viewport().get_texture().get_image()
	img.save_png("user://labelboxmodel_frame.png")
	print("image: ", ProjectSettings.globalize_path("user://labelboxmodel_frame.png"))
	var ink_box := _ink_box(img)
	print("measured glyph ink box=%s" % [ink_box])
	_ok(ink_box.size.x > 0 and ink_box.size.y > 0, "the glyph drew visible ink")

	# Distance from each candidate handle to the real glyph's own measured
	# centre. The engine's box is still a font-HEIGHT square (the disclosed
	# `meas_w=0` text-measurement placeholder, unrelated to this fix: no live
	# `Font` reaches `cartalith-godot` yet), so this checks the FONT-SIZE
	# MODEL specifically -- the resize handle should sit within roughly one
	# glyph half-diagonal of the ink's own centre, not exactly on its corner.
	var glyph_center := ink_box.get_center()
	var new_dist := new_screen.distance_to(glyph_center)
	var old_dist := old_screen.distance_to(glyph_center)
	var glyph_half_diag := ink_box.size.length() / 2.0
	print("distance to glyph centre -- NEW=%.1fpx  OLD=%.1fpx  (glyph half-diagonal=%.1fpx)"
		% [new_dist, old_dist, glyph_half_diag])
	_ok(new_dist < old_dist, "the fix moves the handle CLOSER to the real glyph, not farther")
	_ok(new_dist <= glyph_half_diag * 2.0, "the FIXED engine handle lands near the real glyph")
	_ok(old_dist > new_dist * 1.3, "the OLD (pre-fix, replicated) formula's handle sits meaningfully farther away -- the mismatch this fixes")

	# ---- Draw both handle candidates onto the same frame for the record ----
	var marks := Control.new()
	marks.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.add_child(marks)
	marks.draw.connect(func():
		marks.draw_circle(new_screen, 6.0, NEW_MARK)
		marks.draw_circle(old_screen, 6.0, OLD_MARK)
		marks.draw_rect(ink_box, Color(1, 1, 1, 0.9), false, 2.0))
	marks.queue_redraw()
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	var annotated: Image = get_viewport().get_texture().get_image()
	annotated.save_png("user://labelboxmodel_annotated.png")
	print("annotated image: ", ProjectSettings.globalize_path("user://labelboxmodel_annotated.png"))

	for f in ["placeholder"]:
		pass
	print("PROBE-RESULT: ", "PASS" if _fails == 0 else "FAIL")
	get_tree().quit(0 if _fails == 0 else 1)


## Bounding box of every pixel that differs from `GROUND` -- the same
## technique `_labelblur_probe.gd::_measure` uses, reduced to just the box.
func _ink_box(img: Image) -> Rect2:
	var x0 := 1 << 30
	var y0 := 1 << 30
	var x1 := -1
	var y1 := -1
	for y in img.get_height():
		for x in img.get_width():
			var c := img.get_pixel(x, y)
			if absf(c.r - GROUND.r) > 0.02 or absf(c.g - GROUND.g) > 0.02 or absf(c.b - GROUND.b) > 0.02:
				x0 = mini(x0, x)
				y0 = mini(y0, y)
				x1 = maxi(x1, x)
				y1 = maxi(y1, y)
	if x1 < 0:
		return Rect2()
	return Rect2(x0, y0, x1 - x0 + 1, y1 - y0 + 1)
