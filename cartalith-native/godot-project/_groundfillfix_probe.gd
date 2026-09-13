extends Control
## Lane URBAN Part A -- does `UrbanLayoutDraw._fill_ground_polygon`'s FALLBACK
## branch (the one `_settlepix_probe` cannot see paint, because the real
## failing block there is under 0.05 px) actually rasterise, at a scale a
## pixel check can register?
##
## `_settlepix_probe.tscn` stays the citation for the DEFECT (seed 24601
## "Sevjuniana", block index 324) and for the FIX's mutation test (removing
## `_fill_ground_polygon` there must bring the triangulation ERROR back). This
## probe isolates the fix's MECHANISM at a size a screen can show: the real
## block's four screen points, scaled x1000, still fail
## `Geometry2D.triangulate_polygon` (`_ready()` confirms this before drawing
## anything -- scaling offset and feature together preserves the ratio that
## causes the cancellation) while the absolute feature size grows to ~30-47
## screen units, comfortably visible.
##
## Those coordinates (~276300, 33450) are still what get fed to
## `_fill_ground_polygon`, unchanged -- the wide-shot points are placed on a
## CHILD `Node2D` whose own `position` is the negative of that offset, a real
## scene-graph transform (not `draw_set_transform`, which measured 0 painted
## pixels here: an immediate-mode transform on giant source coordinates,
## Godot's item-rect culling apparently does not account for). A child
## node's `position` is the ordinary, always-respected way anything in this
## engine draws "far away, panned into view", the same shape of transform a
## camera pan applies to a real settlement's local metres.
##
## Windowed only (MISTAKES.md: ImageTexture.update() no-ops headless).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _groundfillfix_probe.tscn

## The real block-324 screen points from `_settlepix_probe`, scaled x1000.
const BIG_QUAD := [
	Vector2(276344.3, 33488.39),
	Vector2(276318.1, 33474.25),
	Vector2(276343.8, 33454.79),
	Vector2(276350.2, 33462.65),
]
## The far node's position: puts BIG_QUAD's centroid at local (~39,70).
const FAR_POS := Vector2(-276300.0, -33400.0)
const FILL_COLOR := Color(0.831, 0.784, 0.671, 1.0) # BLOCK_GROUND, opaque
const BG_COLOR := Color(0.05, 0.05, 0.08, 1.0)

## A control patch, far from the fallback shape, drawn through the SAME
## helper's FAST path (an ordinary polygon that triangulates fine) -- must
## always paint, or the whole probe's pixel-reading method is broken.
const CONTROL_RECT := Rect2(150, 20, 40, 40)

var _far: Node2D
var _fails := 0

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		_fails += 1
		print("  FAIL  %s" % what)

## 8-bit round-trip through a PNG quantises each channel to 1/255 -- looser
## than `Color.is_equal_approx`'s default epsilon, which a value that is
## genuinely FILL_COLOR still fails by a few thousandths.
func _close(a: Color, b: Color, eps: float = 0.02) -> bool:
	return absf(a.r - b.r) < eps and absf(a.g - b.g) < eps \
		and absf(a.b - b.b) < eps and absf(a.a - b.a) < eps

func _settle() -> void:
	await RenderingServer.frame_post_draw
	await get_tree().process_frame
	await RenderingServer.frame_post_draw

func _draw() -> void:
	# Solid background, so "not FILL_COLOR" is unambiguous everywhere.
	draw_rect(Rect2(Vector2.ZERO, size), BG_COLOR)
	# The control patch: an ordinary, comfortably-large quad -- the fast path.
	var cr := CONTROL_RECT
	UrbanLayoutDraw._fill_ground_polygon(self, PackedVector2Array([
		cr.position, cr.position + Vector2(cr.size.x, 0),
		cr.position + cr.size, cr.position + Vector2(0, cr.size.y),
	]), FILL_COLOR)

func _draw_far() -> void:
	var big := PackedVector2Array()
	for p in BIG_QUAD:
		big.append(p)
	UrbanLayoutDraw._fill_ground_polygon(_far, big, FILL_COLOR)

func _ready() -> void:
	size = Vector2(220, 140)

	_far = Node2D.new()
	_far.position = FAR_POS
	add_child(_far)
	_far.draw.connect(_draw_far)

	# -- confirm the premise before drawing anything: this exact shape, at
	# this exact scale, really does still fail the real triangulator. If it
	# did not, this probe would be exercising the fast path and would prove
	# nothing about the fallback. --
	var big_check := PackedVector2Array()
	for p in BIG_QUAD:
		big_check.append(p)
	var still_fails: bool = Geometry2D.triangulate_polygon(big_check).is_empty()
	_ok(still_fails, "premise: the x1000 shape STILL fails Geometry2D.triangulate_polygon")
	if not still_fails:
		print("  ABORT premise false -- this probe cannot test the fallback branch this way")
		get_tree().quit(2)
		return

	await _settle()
	queue_redraw()
	_far.queue_redraw()
	await _settle()
	var img := get_viewport().get_texture().get_image()
	img.save_png("res://_groundfillfix_drawn.png")

	# Control patch: must have painted (positive control -- if this fails the
	# whole method, not just the fallback, is broken).
	#
	# **Not `img.width / self.size.x`.** This control's `size` was set to
	# 220x140 by hand and the viewport does not shrink to match it (the
	# viewport is the WINDOW's, always) -- confirmed above (`img=1152x648`
	# against `viewport_rect=(1152,648)`, a bare 1:1, while `self.size` stayed
	# the unrelated 220x140 the anchor system had not yet overridden). The
	# viewport IS the thing the image is a capture of, so it is the only
	# correct denominator regardless of what this control's own size reads.
	var scale_factor: float = float(img.get_width()) / get_viewport_rect().size.x
	var ctl_x := int((CONTROL_RECT.position.x + CONTROL_RECT.size.x * 0.5) * scale_factor)
	var ctl_y := int((CONTROL_RECT.position.y + CONTROL_RECT.size.y * 0.5) * scale_factor)
	var ctl_px := img.get_pixel(ctl_x, ctl_y)
	_ok(_close(ctl_px, FILL_COLOR), "control patch (fast path) painted at (%d,%d): got %s" % [ctl_x, ctl_y, ctl_px])

	# The fallback shape's own centroid, in the far node's LOCAL space, then
	# into window pixels via the node's real position (not a draw-time
	# transform) and the measured display scale.
	var centroid := Vector2.ZERO
	for p in BIG_QUAD:
		centroid += p
	centroid /= BIG_QUAD.size()
	var world := _far.position + centroid
	var fx := int(world.x * scale_factor)
	var fy := int(world.y * scale_factor)
	print("GROUNDFILLFIX fallback centroid window px ~ (%d,%d) scale=%.2f" % [fx, fy, scale_factor])
	var fb_px := img.get_pixel(fx, fy)
	_ok(_close(fb_px, FILL_COLOR), "fallback shape painted at its centroid: got %s" % fb_px)

	# A region scan around the fallback shape's centroid: count pixels that
	# are FILL_COLOR (not just background), so one lucky sample cannot carry
	# the result.
	var count := 0
	var total := 0
	var r := int(40 * scale_factor)
	for dy in range(-r, r + 1):
		for dx in range(-r, r + 1):
			var x := fx + dx
			var y := fy + dy
			if x < 0 or y < 0 or x >= img.get_width() or y >= img.get_height():
				continue
			total += 1
			if _close(img.get_pixel(x, y), FILL_COLOR):
				count += 1
	print("GROUNDFILLFIX region scan around fallback shape: %d/%d px match FILL_COLOR" % [count, total])
	_ok(count > 0, "fallback shape painted SOME FILL_COLOR pixels in its region")

	print("GROUNDFILLFIX %s (%d failed)" % ["ALL PASS" if _fails == 0 else "SOME FAILED", _fails])
	get_tree().quit(1 if _fails > 0 else 0)
