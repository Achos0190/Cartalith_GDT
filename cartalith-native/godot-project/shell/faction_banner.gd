extends Control
class_name FactionBanner

## `_civFactionBannerCanvas` (reference line 14849) -- `PARITY_AUDIT.md` §5
## item 10, an unregistered gap until now.
##
## A port of the reference's actual banner composition, not a new visual
## design: a shield outline (rectangle top, quadratic-curved point at the
## bottom) filled with the faction's own colour, dark-stroked, carrying one
## of **six** fixed geometric glyphs chosen by `fid % 6` in white at 85%
## alpha. Every proportion below is the reference's own literal -- `top` at
## `0.08s`, `bot` at `0.92s`, width `0.78s`, the shoulder at `0.55s`, the
## glyph centred at `0.48s` with radius `0.22s`, outline width
## `max(1, 0.045s)`, glyph stroke `max(1, 0.05s)`.
##
## The reference's own header says what this is and is not: "pure rendering,
## not simulation state … No image-upload system, no new persisted bytes."
## It caches per `fid:sizePx`; this does not, because a `Control._draw()` is
## already only called when something invalidates it, which is the same
## saving by a different mechanism.
##
## `Curve2D` stands in for canvas `quadraticCurveTo`: Godot's own `draw_*`
## has no quadratic primitive, and `Curve2D` with the control-point offsets
## a quadratic-to-cubic conversion gives is the exact same curve, not an
## approximation of it. The glyph shapes are polygons and one circle,
## drawn with `draw_colored_polygon`/`draw_circle` exactly as the
## reference's `fill()` does.

var _fid := 0
var _color := Color(0.6, 0.6, 0.6)
var _px := 48.0


func configure(fid: int, col: Color, px: int) -> void:
	_fid = maxi(0, fid)
	_color = col
	_px = float(px)
	custom_minimum_size = Vector2(_px, _px)
	size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	size_flags_vertical = Control.SIZE_SHRINK_CENTER
	queue_redraw()


func _draw() -> void:
	var s := _px
	var cx := s / 2.0
	var top := s * 0.08
	var bot := s * 0.92
	var w := s * 0.78
	var shoulder := s * 0.55

	var shield := _shield_points(cx, top, bot, w, shoulder)
	draw_colored_polygon(shield, _color)
	## `closed` so the outline runs all the way back to the top-left corner,
	## matching `closePath()` before `stroke()`.
	draw_polyline(shield + PackedVector2Array([shield[0]]), Color(0, 0, 0, 0.45), maxf(1.0, s * 0.045))

	var ink := Color(1, 1, 1, 0.85)
	var gx := cx
	var gy := s * 0.48
	var gr := s * 0.22
	match _fid % 6:
		0:
			draw_colored_polygon(PackedVector2Array([
				Vector2(gx, gy - gr), Vector2(gx + gr, gy + gr), Vector2(gx - gr, gy + gr)]), ink)
		1:
			draw_circle(Vector2(gx, gy), gr, ink)
		2:
			draw_rect(Rect2(gx - gr * 0.8, gy - gr * 0.8, gr * 1.6, gr * 1.6), ink)
		3:
			draw_colored_polygon(PackedVector2Array([
				Vector2(gx, gy - gr), Vector2(gx + gr * 0.6, gy),
				Vector2(gx, gy + gr), Vector2(gx - gr * 0.6, gy)]), ink)
		4:
			## The reference's five-pointed star: outer point, then an inner
			## vertex at `a + PI/5` with radius `gr*0.4`, five times round,
			## starting at `-PI/2` (straight up).
			var star := PackedVector2Array()
			for k in 5:
				var a := -PI / 2.0 + float(k) * TAU / 5.0
				star.append(Vector2(gx + cos(a) * gr, gy + sin(a) * gr))
				var a2 := a + PI / 5.0
				star.append(Vector2(gx + cos(a2) * gr * 0.4, gy + sin(a2) * gr * 0.4))
			draw_colored_polygon(star, ink)
		_:
			draw_colored_polygon(PackedVector2Array([
				Vector2(gx - gr, gy - gr * 0.3), Vector2(gx + gr, gy - gr * 0.3),
				Vector2(gx, gy + gr)]), ink)


## The shield outline as a filled polygon: two straight sides down to the
## shoulder, then the reference's two `quadraticCurveTo` sweeps meeting at
## the bottom point.
##
## `Curve2D.tessellate()` returns the sampled curve; a quadratic with
## control `Q` between `P0` and `P2` is the cubic with controls
## `P0 + 2/3(Q-P0)` and `P2 + 2/3(Q-P2)`, which is what the `in`/`out`
## offsets below encode -- an exact conversion, not a visual approximation.
func _shield_points(cx: float, top: float, bot: float, w: float, shoulder: float) -> PackedVector2Array:
	var left := cx - w / 2.0
	var right := cx + w / 2.0
	var pts := PackedVector2Array([Vector2(left, top), Vector2(right, top), Vector2(right, shoulder)])
	pts.append_array(_quad(Vector2(right, shoulder), Vector2(right, bot), Vector2(cx, bot)))
	pts.append_array(_quad(Vector2(cx, bot), Vector2(left, bot), Vector2(left, shoulder)))
	return pts


func _quad(p0: Vector2, q: Vector2, p2: Vector2) -> PackedVector2Array:
	var c := Curve2D.new()
	c.add_point(p0, Vector2.ZERO, (q - p0) * (2.0 / 3.0))
	c.add_point(p2, (q - p2) * (2.0 / 3.0), Vector2.ZERO)
	var out := c.tessellate(4)
	## Drop the first sample -- it repeats the previous segment's endpoint.
	out.remove_at(0)
	return out
