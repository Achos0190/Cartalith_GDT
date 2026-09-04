extends SceneTree

## `OUTSTANDING_WORK.md` §2.6's bounded paint-preview upload, over the real
## gdext boundary.
##
## What no Rust test in this repository can reach: `build_paint_preview_patch`
## and `build_paint_preview_texture` are `#[func]`s on a cdylib `GodotClass`,
## so `PackedByteArray::from` / `Image::create_from_data` /
## `ImageTexture::create_from_image` and the returned Dictionary have never
## been marshalled until this runs. `tests/paint_preview_cost.rs` proves the
## two rasters agree in `PaintEditor`; this proves the two *textures* agree
## after crossing the binding, pixel for pixel, and that the empty-Dictionary
## and full-grid answers survive marshalling as distinguishable things.

const GW := 256
const GH := 192


func _init() -> void:
	var fails := 0
	var wg: WorldGen = WorldGen.new()
	if not wg.has_method("build_paint_preview_patch"):
		print("FAIL: build_paint_preview_patch absent from the extension")
		quit(1)
		return

	wg.generate_sized(24601, 640.0, GW, GH)

	# 1. Nothing committed, nothing pending. The patch call must say "nothing
	#    to draw" as an EMPTY Dictionary -- not a zero-sized rectangle, and
	#    not a full-grid one.
	var empty: Dictionary = wg.build_paint_preview_patch()
	print("  before any dab: patch = ", empty)
	if empty.has("texture") or not empty.is_empty():
		print("  FAIL: an unpainted world returned a patch")
		fails += 1
	if wg.build_paint_preview_texture() != null:
		print("  FAIL: an unpainted world returned a full texture")
		fails += 1

	# 2. A short drag, then the two rasters must be identical inside the
	#    window. Radius 12 at (128, 96) so the window is a genuine
	#    sub-rectangle of a 256x192 grid.
	wg.paint_set_layer("biome")
	wg.paint_set_brush(3, 12.0, 1.0, 0.0, false, false)
	wg.paint_stroke_at(128.0, 96.0)
	wg.paint_stroke_at(134.0, 100.0)
	wg.paint_stroke_at(140.0, 104.0)
	print("  draft dabs: ", wg.paint_draft_count())

	var patch: Dictionary = wg.build_paint_preview_patch()
	if not patch.has("texture"):
		print("  FAIL: a live draft returned no texture")
		quit(1)
		return
	var px: int = int(patch["x"])
	var py: int = int(patch["y"])
	var pw: int = int(patch["w"])
	var ph: int = int(patch["h"])
	print("  patch: %dx%d at (%d, %d)  vs grid %dx%d" % [pw, ph, px, py, GW, GH])
	if pw >= GW or ph >= GH:
		print("  FAIL: the patch is not a sub-rectangle, so it saves nothing")
		fails += 1

	var full_tex: ImageTexture = wg.build_paint_preview_texture()
	var patch_tex: ImageTexture = patch["texture"]
	if full_tex == null or patch_tex == null:
		print("  FAIL: a texture failed to marshal")
		quit(1)
		return
	if patch_tex.get_width() != pw or patch_tex.get_height() != ph:
		print("  FAIL: texture size %dx%d disagrees with the reported window %dx%d"
			% [patch_tex.get_width(), patch_tex.get_height(), pw, ph])
		fails += 1
	if full_tex.get_width() != GW or full_tex.get_height() != GH:
		print("  FAIL: the full texture is not grid-sized")
		fails += 1

	var full_img: Image = full_tex.get_image()
	var patch_img: Image = patch_tex.get_image()
	if full_img == null or patch_img == null:
		print("  FAIL: get_image() returned null headless -- this check cannot run")
		quit(1)
		return

	# 3. Identical pixels, not "looks right".
	var differing := 0
	var opaque := 0
	for y in range(ph):
		for x in range(pw):
			var a: Color = patch_img.get_pixel(x, y)
			var b: Color = full_img.get_pixel(px + x, py + y)
			if a != b:
				differing += 1
				if differing == 1:
					print("  first difference at patch (%d, %d) / grid (%d, %d): %s vs %s"
						% [x, y, px + x, py + y, str(a), str(b)])
			if a.a > 0.0:
				opaque += 1
	print("  window pixels: %d, differing from the full re-upload: %d, opaque: %d"
		% [pw * ph, differing, opaque])
	if differing != 0:
		print("  FAIL: the patch does not match a full re-upload")
		fails += 1
	if opaque == 0:
		print("  FAIL: the window is entirely transparent, so the comparison proved nothing")
		fails += 1

	# 4. Every pixel the dabs changed is inside the window -- checked against
	#    the full texture from BEFORE the dabs, so an under-reported window
	#    would show up as a change outside it. (The `before` here is the
	#    committed layer, which at this point is empty, so "changed" means
	#    "became opaque".)
	var outside_opaque := 0
	for y in range(GH):
		for x in range(GW):
			var inside: bool = x >= px and x < px + pw and y >= py and y < py + ph
			if not inside and full_img.get_pixel(x, y).a > 0.0:
				outside_opaque += 1
	print("  opaque pixels outside the window: ", outside_opaque)
	if outside_opaque != 0:
		print("  FAIL: the window missed pixels the draft painted")
		fails += 1

	# 5. Commit. `touched_bounds()` is now `None`, which means "the draft
	#    touched nothing" -- the OPPOSITE of "nothing is dirty". The patch
	#    must come back as the WHOLE GRID, and it must not be blank.
	var summary: Dictionary = wg.paint_commit()
	print("  commit -> biome ", summary.get("biome", {}))
	var after: Dictionary = wg.build_paint_preview_patch()
	if not after.has("texture"):
		print("  FAIL: a committed layer with an empty draft returned NO patch")
		fails += 1
	else:
		print("  after commit: %dx%d at (%d, %d)"
			% [int(after["w"]), int(after["h"]), int(after["x"]), int(after["y"])])
		if int(after["w"]) != GW or int(after["h"]) != GH or int(after["x"]) != 0 or int(after["y"]) != 0:
			print("  FAIL: a committed layer must return the whole grid, not a window")
			fails += 1
		var after_img: Image = (after["texture"] as ImageTexture).get_image()
		var painted := 0
		for y in range(GH):
			for x in range(GW):
				if after_img.get_pixel(x, y).a > 0.0:
					painted += 1
		print("  committed opaque pixels: ", painted)
		if painted == 0:
			print("  FAIL: the committed layer rendered blank")
			fails += 1

	# 6. Discard a fresh draft: back to the whole grid, still not empty, and
	#    still not confusable with case 1.
	wg.paint_stroke_at(60.0, 60.0)
	wg.paint_discard()
	var discarded: Dictionary = wg.build_paint_preview_patch()
	if not discarded.has("texture") or int(discarded["w"]) != GW:
		print("  FAIL: after a discard the committed layer stopped being drawable: ", discarded)
		fails += 1
	else:
		print("  after discard: %dx%d (whole grid, committed layer intact)"
			% [int(discarded["w"]), int(discarded["h"])])

	print("PROBE %s (%d failures)" % ["PASS" if fails == 0 else "FAIL", fails])
	quit(0 if fails == 0 else 1)
