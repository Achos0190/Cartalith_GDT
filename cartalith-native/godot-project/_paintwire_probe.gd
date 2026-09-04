extends Node

## `ViewportHost.set_preview_patch()` against the full-upload path it replaces,
## over a real `EngineBridge` and a real `ViewportHost`.
##
## What no Rust test and no `--check-only` can reach: the shell now shows a
## raster it *composited itself* out of windows, and a wrong window shows stale
## pixels while nothing fails. So every assertion here is a byte comparison of
## what is actually on `_preview_layer` against `build_paint_preview_texture()`
## taken at the same instant -- the path this one is allowed to replace only if
## it is indistinguishable from it.
##
## `_paintpatch_probe.gd` proves the engine's two rasters agree. This proves
## the SHELL's accumulated composite agrees, which is a different claim: the
## patch is correct per call there, and correct after twenty of them here.

const GW := 256
const GH := 192

var bridge: EngineBridge
var host: ViewportHost
var fails := 0
var patch_hits := 0
var full_hits := 0
var clear_hits := 0


## **Run this WINDOWED. It refuses to run otherwise, and the refusal is the
## point.** `ImageTexture.update()` is a no-op under the dummy renderer
## `--headless` selects: measured 2026-09-04, a 4x4 texture updated from a
## mutated source image still reads back its original pixel headless and reads
## back the new one windowed. Every assertion below goes through that call, so
## headless they all fail identically whether or not the code is correct --
## a check that cannot pass is exactly as useless as one that cannot fail.
func _guard() -> bool:
	var img := Image.create_empty(2, 2, false, Image.FORMAT_RGBA8)
	img.fill(Color(0, 0, 0, 0))
	var t := ImageTexture.create_from_image(img)
	img.set_pixel(1, 1, Color(1, 0, 0, 1))
	t.update(img)
	return t.get_image().get_pixel(1, 1).a > 0.0


func _ready() -> void:
	if not _guard():
		print("PROBE REFUSED: ImageTexture.update() does not reach get_image() in this "
			+ "renderer (%s). Re-run without --headless." % DisplayServer.get_name())
		get_tree().quit(2)
		return
	bridge = EngineBridge.new()
	add_child(bridge)
	host = ViewportHost.new()
	add_child(host)
	host.setup(bridge)
	await get_tree().process_frame

	bridge.world_gen.generate_sized(24601, 640.0, GW, GH)
	bridge.world_gen.paint_set_layer("biome")

	# 1. Nothing committed, nothing pending. The engine says "nothing to draw
	#    at all" with an empty Dictionary; the shell must CLEAR, and must not
	#    read it as "upload everything" or fall through to a full raster.
	var handled: bool = _show()
	_expect(handled, "the empty answer was reported as a failure")
	_expect(host._preview_layer.texture == null, "an unpainted world left a texture on screen")
	_expect(clear_hits == 1 and full_hits == 0, "the empty answer did not take the clearing path")
	_same("empty world")

	# 2. Twenty dabs. Dab 1 has no raster to composite onto and must fall back
	#    once; every dab after it must take the bounded path, and the screen
	#    must equal a full re-upload after each one.
	bridge.world_gen.paint_set_brush(3, 12.0, 1.0, 0.0, false, false)
	for i in range(20):
		bridge.world_gen.paint_stroke_at(100.0 + float(i) * 3.0, 90.0 + float(i) * 2.0)
		_show()
		_same("dab %d" % (i + 1))
	print("  after 20 dabs: %d patched, %d full, %d cleared" % [patch_hits, full_hits, clear_hits])
	_expect(full_hits == 1, "expected exactly one fallback (the first dab), got %d" % full_hits)
	_expect(patch_hits >= 19, "the bounded path ran only %d times -- this proves nothing" % patch_hits)

	# 3. Erase INSIDE the window already painted. This is the state a window
	#    drawn *over* the previous raster gets wrong and a window blitted INTO
	#    it gets right: the pixel has to come back transparent, not keep the
	#    colour underneath it.
	var before := _opaque(host._preview_layer.texture.get_image())
	bridge.world_gen.paint_set_brush(3, 12.0, 1.0, 0.0, true, false)
	bridge.world_gen.paint_stroke_at(110.0, 96.0)
	_show()
	_same("erase inside the window")
	var after := _opaque(host._preview_layer.texture.get_image())
	print("  erase: %d opaque -> %d opaque" % [before, after])
	_expect(after < before, "the erase removed no pixels, so it tested nothing")

	# 3b. Sculpt shares `_preview_layer` and sets it with the ONE-argument call.
	#     A window must refuse to composite onto a raster nobody declared as
	#     its base. Handed the PAINT raster deliberately: a real sculpt preview
	#     is RGB8 today and the format test would refuse it whatever the flag
	#     said, so only an otherwise-perfect base can test the declaration
	#     itself. The assertion is on the decision, not on what is drawn.
	host.set_preview_texture(bridge.build_paint_preview_texture())
	_expect(not host.set_preview_patch(bridge.build_paint_preview_patch()),
		"a window was composited onto a raster that was never declared a base")
	_show()
	_same("after refusing an undeclared base")

	# 4. Commit clears (the caller's own behaviour), then the first dab after it
	#    falls back once more and the screen still matches.
	bridge.world_gen.paint_commit()
	host.set_preview_texture(null)
	var was_full := full_hits
	bridge.world_gen.paint_set_brush(5, 12.0, 1.0, 0.0, false, false)
	bridge.world_gen.paint_stroke_at(60.0, 60.0)
	_show()
	_same("first dab after a commit")
	_expect(full_hits == was_full + 1, "the first dab after a commit did not re-seed the base")
	bridge.world_gen.paint_stroke_at(64.0, 64.0)
	_show()
	_same("second dab after a commit")
	_expect(full_hits == was_full + 1, "the second dab after a commit fell back too")

	# 5. Discard over a COMMITTED layer. The draft now touches nothing, which
	#    the engine answers with a FULL-GRID window -- the opposite of "nothing
	#    is dirty". The whole committed raster must be on screen, not a blank.
	bridge.world_gen.paint_discard()
	host.set_preview_texture(bridge.build_paint_preview_texture(), true)
	var d: Dictionary = bridge.build_paint_preview_patch()
	_expect(int(d.get("w", 0)) == GW and int(d.get("h", 0)) == GH,
		"an empty draft over a committed layer did not answer with the whole grid: %s" % str(d))
	_expect(host.set_preview_patch(d), "the full-grid window was refused")
	_same("full-grid window after a discard")
	_expect(_opaque(host._preview_layer.texture.get_image()) > 0,
		"the committed layer rendered blank after a discard")

	# 6. The world replaced under a live stroke, at a DIFFERENT resolution and
	#    with no preview refresh in between -- so the mirror is 256x192 while
	#    the next window is addressed against 128x96 and would still fit inside
	#    it. The bounded path must refuse rather than blit at the right index of
	#    the wrong raster.
	bridge.world_gen.paint_stroke_at(120.0, 90.0)
	_show()
	bridge.world_gen.generate_sized(24601, 640.0, 128, 96)
	bridge.world_gen.paint_set_layer("biome")
	bridge.world_gen.paint_set_brush(3, 12.0, 1.0, 0.0, false, false)
	bridge.world_gen.paint_stroke_at(64.0, 48.0)
	var refused: bool = not host.set_preview_patch(bridge.build_paint_preview_patch())
	_expect(refused, "a window from a 128x96 world was blitted into a 256x192 mirror")
	host.set_preview_texture(bridge.build_paint_preview_texture(), true)
	_same("after a mid-stroke world replacement")
	_expect(host._preview_layer.texture.get_width() == 128,
		"the preview is still the old grid's size")

	# 7. The five malformed windows, called directly because nothing the engine
	#    emits can produce them. Measured 2026-09-04, `blit_rect` does not
	#    raise on either: an out-of-range destination is **clipped with no
	#    error at all** (a 4x4 source at (2,2) of a 4x4 image writes the 2x2
	#    overlap and nothing else), and a format mismatch logs an engine error
	#    and writes nothing. Neither reaches the caller, so without these
	#    guards `set_preview_patch` would report a successful update over a
	#    raster it had partly or wholly not changed. Each of the four bounds
	#    tests is covered separately -- three survived mutation when only `x`
	#    past the right edge was checked.
	bridge.world_gen.paint_stroke_at(66.0, 50.0)
	var odd := Image.create_empty(4, 4, false, Image.FORMAT_RGB8)
	odd.fill(Color.RED)
	for case in ["x past the right edge", "y past the bottom edge", "negative x",
			"negative y", "another pixel format"]:
		## Two, because the first re-seeds the base a refusal just dropped and
		## the second is what actually fills the mirror from it.
		_show()
		_show()
		_expect(host._preview_image != null, "%s needs a live mirror and has none" % case)
		var bad: Dictionary = bridge.build_paint_preview_patch()
		match case:
			"x past the right edge": bad["x"] = int(bad["x"]) + 10000
			"y past the bottom edge": bad["y"] = int(bad["y"]) + 10000
			"negative x": bad["x"] = -1
			"negative y": bad["y"] = -1
			_: bad = {"texture": ImageTexture.create_from_image(odd), "x": 0, "y": 0, "w": 4, "h": 4}
		_expect(not host.set_preview_patch(bad), "a window with %s was accepted" % case)

	print("PROBE %s (%d failures)" % ["PASS" if fails == 0 else "FAIL", fails])
	get_tree().quit(0 if fails == 0 else 1)


## `world_workspace.gd::_paint_show_preview()`, transcribed rather than called,
## because reaching the real one needs the whole shell armed. Kept literally in
## step with it -- if that function grows a branch this must grow the same one.
func _show() -> bool:
	var patch: Dictionary = bridge.build_paint_preview_patch()
	var had := patch.has("texture")
	if host.set_preview_patch(patch):
		if had:
			patch_hits += 1
		else:
			clear_hits += 1
		return true
	host.set_preview_texture(bridge.build_paint_preview_texture(), true)
	full_hits += 1
	return false


## What is on screen, against what a full re-upload would have put there.
func _same(tag: String) -> void:
	var shown: Texture2D = host._preview_layer.texture
	var full: Texture2D = bridge.build_paint_preview_texture()
	if shown == null or full == null:
		_expect(shown == null and full == null,
			"%s: one path has a texture and the other does not (%s / %s)" % [tag, str(shown), str(full)])
		return
	var a: Image = shown.get_image()
	var b: Image = full.get_image()
	if a.get_size() != b.get_size() or a.get_format() != b.get_format():
		_expect(false, "%s: %s %s vs %s %s" % [tag, str(a.get_size()), a.get_format(), str(b.get_size()), b.get_format()])
		return
	_expect(a.get_data() == b.get_data(), "%s: the composited raster differs from a full re-upload" % tag)


func _opaque(img: Image) -> int:
	var n := 0
	for y in range(img.get_height()):
		for x in range(img.get_width()):
			if img.get_pixel(x, y).a > 0.0:
				n += 1
	return n


func _expect(ok: bool, msg: String) -> void:
	if not ok:
		fails += 1
		print("  FAIL: ", msg)
