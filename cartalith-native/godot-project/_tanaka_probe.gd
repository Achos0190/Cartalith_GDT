extends Node
## Tanaka illuminated contours (Ruling BI) -- the windowed, real-cdylib check.
##
##   Godot_v4.7.1-stable_win64.exe --path . _tanaka_probe.tscn
##
## `tests/tanaka_contours.rs` proves the law on literals and on a synthetic
## raster. This proves the four claims on a real generated world, through the
## real `build_color_texture()` the viewport calls:
##
##   1. OFF is byte-identical to the baseline -- including after the mode has
##      been turned on and back off, which is the form that catches a default
##      that is only usually the identity.
##   2. ON differs from OFF only along contour pixels (the plain contour's own
##      footprint, measured as the pixels contours-on moves versus contours-off).
##   3. A contour pixel whose ground FACES the sun is brighter than one that
##      faces away. "Faces" is classified from the world's own elevation
##      (`sample_cell`), not from any colour the renderer produced, and the
##      same classification is first checked against the plain hillshade so the
##      convention is the renderer's own light and not an assumption.
##   4. Turning the sun 180 degrees swaps which side is bright.
##
## Output is ASCII. A failing check prints `PROBE-FAIL`; the process still
## exits 0, so grep for it and for the final `RESULT` line.
##
## Optional environment: `TANAKA_OUT` names a directory that gets before/after
## PNGs at two zooms (the full map, and one deep-zoom tile).

const SEED := 20261006
const GW := 1024
const GH := 656
const CONTOURS := 0.8

var fails := 0

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("PROBE-FAIL  %s" % what)

## The bytes `build_color_texture()` actually hands the viewport, as RGB8.
func _screen(gen: WorldGen) -> Array:
	var tex: ImageTexture = gen.build_color_texture()
	if tex == null:
		return [PackedByteArray(), 0, 0]
	var im := tex.get_image()
	im.convert(Image.FORMAT_RGB8)
	return [im.get_data(), im.get_width(), im.get_height()]

func _lum(d: PackedByteArray, i: int) -> float:
	return 0.3 * d[i] + 0.59 * d[i + 1] + 0.11 * d[i + 2]

func _save_png(data: PackedByteArray, w: int, h: int, path: String, crop: Rect2i, scale: int) -> void:
	var im := Image.create_from_data(w, h, false, Image.FORMAT_RGB8, data)
	var c := im.get_region(crop)
	c.resize(crop.size.x * scale, crop.size.y * scale, Image.INTERPOLATE_NEAREST)
	c.save_png(path)

## Facing of the ground at pixel (px, py) toward a sun at `az_deg`, from the
## world's own elevation. The sun lies toward (sin az, -cos az) in screen axes
## (x right, y down; az 315 is the north-west); ground descends toward -grad,
## so it faces the sun when elevation RISES away from it. -1.0..1.0, or 2.0
## when the slope is too flat to have a trustworthy aspect.
func _facing(gen: WorldGen, px: int, py: int, sx: float, sy: float, az_deg: float) -> float:
	var gx := int(px * sx)
	var gy := int(py * sy)
	var e := func(x: int, y: int) -> float:
		return float(gen.sample_cell(x, y).get("elevation", NAN))
	var dx: float = (e.call(gx + 1, gy) - e.call(gx - 1, gy)) * 0.5
	var dy: float = (e.call(gx, gy + 1) - e.call(gx, gy - 1)) * 0.5
	var m := sqrt(dx * dx + dy * dy)
	if is_nan(m) or m < 0.004:
		return 2.0
	var a := deg_to_rad(az_deg)
	return (-dx * sin(a) + dy * cos(a)) / m

func _ready() -> void:
	get_tree().create_timer(270.0).timeout.connect(func() -> void:
		print("PROBE-FAIL  watchdog: _ready never finished")
		get_tree().quit(0))

	var gen := WorldGen.new()
	gen.generate_sized(SEED, 1200.0, GW, GH)
	await get_tree().process_frame
	_ok(gen.has_method("set_npr"), "WorldGen.set_npr is published")
	var npr: Dictionary = gen.get_npr()
	_ok(npr.has("tanaka"), "get_npr publishes the tanaka key")
	_ok(float(npr.get("tanaka", -1.0)) == 0.0, "tanaka opens at 0 (default OFF)")

	print("\n== 1. OFF is the baseline, byte for byte ==")
	var none: Array = _screen(gen)                      # contours off
	gen.set_npr({"contours": CONTOURS})
	var base: Array = _screen(gen)                      # plain contours
	var w: int = base[1]
	var h: int = base[2]
	var base_d: PackedByteArray = base[0]
	print("  raster %d x %d (grid %d x %d)" % [w, h, GW, GH])
	_ok(base_d.size() == w * h * 3 and w > 0, "the baseline raster exists")
	_ok(base_d != (none[0] as PackedByteArray), "contours on moves pixels (the footprint is non-empty)")
	gen.set_npr({"tanaka": 1.0})
	var on: Array = _screen(gen)
	var on_d: PackedByteArray = on[0]
	_ok(on_d != base_d, "tanaka on moves pixels")
	gen.set_npr({"tanaka": 0.0})
	var off_again: Array = _screen(gen)
	_ok((off_again[0] as PackedByteArray) == base_d, "tanaka back to 0 is byte-identical to the baseline")

	print("\n== 2. ON differs from OFF only along contour pixels ==")
	var none_d: PackedByteArray = none[0]
	## The footprint is measured in BYTES: the pixels the plain contour moves by
	## at least one of 255 levels. The line's faint tails (coverage `t` squared,
	## so a few percent at the edge) round to the same byte without Tanaka, but
	## Tanaka's stronger, brighter ink can carry such a tail across a rounding
	## boundary -- one pixel of that is not a drawn line outside the contour. So
	## the claim is made against the footprint grown by ONE pixel, and the strict
	## count is printed beside it. `tests/tanaka_contours.rs` proves the strict
	## claim in exact floats.
	var fp := PackedByteArray()
	fp.resize(w * h)
	var footprint := 0
	for p in range(w * h):
		var i3 := p * 3
		if none_d[i3] != base_d[i3] or none_d[i3 + 1] != base_d[i3 + 1] or none_d[i3 + 2] != base_d[i3 + 2]:
			fp[p] = 1
			footprint += 1
	var moved := 0
	var outside_strict := 0
	var outside_grown := 0
	for py in range(h):
		for px in range(w):
			var p := py * w + px
			var i4 := p * 3
			if on_d[i4] == base_d[i4] and on_d[i4 + 1] == base_d[i4 + 1] and on_d[i4 + 2] == base_d[i4 + 2]:
				continue
			moved += 1
			if fp[p] == 1:
				continue
			outside_strict += 1
			var near := false
			for oy in range(-1, 2):
				for ox in range(-1, 2):
					var qx := px + ox
					var qy := py + oy
					if qx >= 0 and qy >= 0 and qx < w and qy < h and fp[qy * w + qx] == 1:
						near = true
			if not near:
				outside_grown += 1
	print("  footprint %d px, tanaka moved %d px, outside strict %d, outside grown by 1px %d" % [footprint, moved, outside_strict, outside_grown])
	_ok(footprint > 500 and moved > 200, "both counts are non-trivial (not a vacuous pass)")
	_ok(float(outside_grown) < 0.005 * float(moved), "under 0.5 percent of moved pixels lie more than one pixel from the plain contour footprint")
	_ok(float(outside_strict) < 0.25 * float(moved), "and the strictly-outside pixels are a minority of what moved")

	print("\n== 3. a contour pixel facing the sun is brighter than one facing away ==")
	var sx := float(GW) / float(w)
	var sy := float(GH) / float(h)
	# Convention check against the renderer's own hillshade: with contours off,
	# ground facing the sun (by the elevation classifier) must be lighter than
	# ground facing away. This ties the classifier to the real light.
	var hs_lit := 0.0
	var hs_dark := 0.0
	var hs_nl := 0
	var hs_nd := 0
	var step := 7
	for py in range(2, h - 2, step):
		for px in range(2, w - 2, step):
			var i := (py * w + px) * 3
			var f := _facing(gen, px, py, sx, sy, 315.0)
			if f > 1.5:
				continue
			if f > 0.8:
				hs_lit += _lum(none_d, i)
				hs_nl += 1
			elif f < -0.8:
				hs_dark += _lum(none_d, i)
				hs_nd += 1
	print("  hillshade check: %d lit / %d shadow samples" % [hs_nl, hs_nd])
	_ok(hs_nl >= 40 and hs_nd >= 40, "both facing classes are populated")
	_ok(hs_nl > 0 and hs_nd > 0 and hs_lit / hs_nl > hs_dark / hs_nd, "classified lit ground is lighter than shadow ground under the plain hillshade (lit %.1f vs shadow %.1f)" % [hs_lit / maxf(hs_nl, 1), hs_dark / maxf(hs_nd, 1)])

	gen.set_npr({"tanaka": 1.0})
	var on315_d: PackedByteArray = on_d
	## The measure is the contour's OWN contribution, `lum(on) - lum(base)`, per
	## pixel: comparing absolute luminances would mix in whether the ground
	## beneath happens to be lit (the hillshade check above shows it differs).
	## STRONG pixels are those the contour moves by more than 2 levels of
	## luminance. A contour's faint tails (coverage falls off as t squared) sit
	## inside rounding, and on a 1-px-per-cell map most contour pixels are tails;
	## counting them would measure noise, not the line. The 2-level floor is a
	## labelled judgement, and the full histogram is printed beside the verdict.
	var lit_sum := 0.0
	var dk_sum := 0.0
	var lit_n := 0
	var dk_n := 0
	var lit_pos := 0
	var dk_neg := 0
	var picked: Array = []
	var hist := {"lit": [0, 0, 0], "dark": [0, 0, 0]}
	for py in range(2, h - 2):
		for px in range(2, w - 2):
			var i := (py * w + px) * 3
			if base_d[i] == on315_d[i] and base_d[i + 1] == on315_d[i + 1] and base_d[i + 2] == on315_d[i + 2]:
				continue
			if (px * 31 + py * 17) % 5 != 0:
				continue                     # thin the sample: sample_cell is not free
			var f := _facing(gen, px, py, sx, sy, 315.0)
			if f > 1.5:
				continue
			var dl := _lum(on315_d, i) - _lum(base_d, i)
			var side := ""
			if f > 0.8:
				side = "lit"
			elif f < -0.8:
				side = "dark"
			if side == "":
				continue
			var bucket := 0 if dl < -2.0 else (2 if dl > 2.0 else 1)
			hist[side][bucket] += 1
			if absf(dl) <= 2.0:
				continue
			picked.append([i, f, dl])
			if side == "lit":
				lit_sum += dl
				lit_n += 1
				if dl > 0.0:
					lit_pos += 1
			else:
				dk_sum += dl
				dk_n += 1
				if dl < 0.0:
					dk_neg += 1
	print("  contribution histogram [< -2, within 2, > +2]: lit %s, shadow %s" % [str(hist["lit"]), str(hist["dark"])])
	print("  strong contour pixels: %d lit-facing / %d shadow-facing" % [lit_n, dk_n])
	_ok(lit_n >= 100 and dk_n >= 100, "both classes of strong contour pixel are populated")
	var lit_mean := lit_sum / maxf(lit_n, 1)
	var dk_mean := dk_sum / maxf(dk_n, 1)
	print("  mean contour contribution: lit side %+.1f, shadow side %+.1f levels" % [lit_mean, dk_mean])
	print("  lit-facing pixels that brightened: %d of %d; shadow-facing that darkened: %d of %d" % [lit_pos, lit_n, dk_neg, dk_n])
	_ok(lit_mean > 10.0 and dk_mean < -10.0, "the lit side is brightened and the shadow side darkened by the contour")
	_ok(lit_mean - dk_mean > 30.0, "lit-side contours are much brighter than shadow-side ones")
	_ok(lit_pos * 100 >= lit_n * 85 and dk_neg * 100 >= dk_n * 85, "at least 85 percent of each class goes the stated way")

	print("\n== 4. a 180-degree azimuth rotation swaps the sides ==")
	gen.set_appearance({"sun_az_deg": 135.0})
	var rot: Array = _screen(gen)
	var rot_d: PackedByteArray = rot[0]
	var flipped := 0
	for p in picked:
		var i2: int = p[0]
		var d315: float = p[2]
		var d135 := _lum(rot_d, i2) - _lum(base_d, i2)
		## Reversed: the contribution changes sign, and by more than rounding.
		if (d315 > 0.0 and d135 < -2.0) or (d315 < 0.0 and d135 > 2.0):
			flipped += 1
	print("  %d of %d strong contour pixels reversed" % [flipped, picked.size()])
	_ok(picked.size() >= 200 and flipped * 100 >= picked.size() * 85, "at least 85 percent of them reversed with the sun")
	gen.set_appearance({"sun_az_deg": 315.0})
	var back: Array = _screen(gen)
	_ok((back[0] as PackedByteArray) == on315_d, "sun back at 315 restores the same image")

	var out := OS.get_environment("TANAKA_OUT")
	if out != "":
		print("\n== screenshots to %s ==" % out)
		gen.set_npr({"tanaka": 0.0})
		var b0: Array = _screen(gen)
		gen.set_npr({"tanaka": 1.0})
		var b1: Array = _screen(gen)
		# The densest 160 x 100 window of Tanaka-moved pixels (summed over
		# 20-px blocks), so the picture shows the effect where it is, not a
		# lucky flat crop.
		var bx := 20
		var nbx := w / bx
		var nby := h / bx
		var cnt := PackedInt32Array()
		cnt.resize(nbx * nby)
		var b0d: PackedByteArray = b0[0]
		var b1d: PackedByteArray = b1[0]
		for py in range(nby * bx):
			for px in range(nbx * bx):
				var i5 := (py * w + px) * 3
				if b0d[i5] != b1d[i5] or b0d[i5 + 1] != b1d[i5 + 1]:
					cnt[(py / bx) * nbx + px / bx] += 1
		var best := -1
		var best_x := 0
		var best_y := 0
		for by in range(nby - 4):
			for bxi in range(nbx - 7):
				var sum := 0
				for dy in range(5):
					for dx in range(8):
						sum += cnt[(by + dy) * nbx + bxi + dx]
				if sum > best:
					best = sum
					best_x = bxi * bx
					best_y = by * bx
		var crop := Rect2i(best_x, best_y, 160, 100)
		print("  densest window %s (%d moved px)" % [str(crop), best])
		_save_png(b0[0], w, h, out.path_join("zoom1_before.png"), crop, 6)
		_save_png(b1[0], w, h, out.path_join("zoom1_after.png"), crop, 6)
		gen.set_appearance({"sun_az_deg": 135.0})
		var b2: Array = _screen(gen)
		_save_png(b2[0], w, h, out.path_join("zoom1_after_sun135.png"), crop, 6)
		gen.set_appearance({"sun_az_deg": 315.0})
		# Zoom 2: the deep-zoom tile (z4) holding the window's centre, through
		# the tile path, shown 3x.
		var z := 4
		var n: int = gen.lod_tiles_per_axis(z)
		var col := clampi(int((crop.position.x + 80) / float(w) * n), 0, n - 1)
		var row := clampi(int((crop.position.y + 50) / float(h) * n), 0, n - 1)
		gen.set_npr({"tanaka": 0.0})
		var t0: ImageTexture = gen.lod_synthesize_tile(z, col, row)
		gen.set_npr({"tanaka": 1.0})
		var t1: ImageTexture = gen.lod_synthesize_tile(z, col, row)
		print("  tile z%d (%d per axis), tile %d,%d: %s" % [z, n, col, row, str(t0 != null and t1 != null)])
		_ok(t0 != null and t1 != null, "the deep-zoom tile path renders both")
		if t0 != null and t1 != null:
			var i0 := t0.get_image()
			var i1 := t1.get_image()
			print("  tile size %d x %d" % [i0.get_width(), i0.get_height()])
			_ok(i0.get_data() != i1.get_data(), "the tile path draws Tanaka too (the tile differs)")
			i0.resize(i0.get_width() * 3, i0.get_height() * 3, Image.INTERPOLATE_NEAREST)
			i1.resize(i1.get_width() * 3, i1.get_height() * 3, Image.INTERPOLATE_NEAREST)
			i0.save_png(out.path_join("zoom2_before.png"))
			i1.save_png(out.path_join("zoom2_after.png"))

	print("\nRESULT %s (%d failed)" % ["GREEN" if fails == 0 else "RED", fails])
	get_tree().quit(0)
