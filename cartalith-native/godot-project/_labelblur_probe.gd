extends Node
## Does text under `_camera`'s scale rasterise at screen resolution, or is a
## bitmap magnified?  Owner report 2026-09-07: *"Settlement names are blurry,
## they don't seem to live on their own layer so that they stay visually sharp
## and don't get influenced by a zoom."*
##
## Runs `map_overlay.gd` ALONE under a scaled parent -- no shell, no engine --
## so the only ink in the frame is the overlay's own and every measured pixel
## is attributable.  FOUR text paths are measured, each in its own frame with
## nothing else drawn, at the same grid position (dead centre, which is the
## zoom pivot, so it does not move between frames):
##
##   pin  `_draw()`'s settlement-name loop, which wraps its `draw_string` in
##        `_crisp_begin()`/`_crisp_end()` -- the control, and already correct
##        before this probe existed;
##   gen  `_draw_labels()`' straight single-`draw_string` run;
##   trk  `_draw_labels()`' per-glyph tracked + oblique run;
##   arc  `_draw_labels()`' per-glyph `drawArcLabel` arc.
##
## The last three are one function with three transforms and the fix touched
## all three, so all three are measured.
##
## A path that rasterises at screen resolution holds its edge contrast when the
## camera zooms; a path that magnifies a bitmap loses it roughly in proportion.
## Each is also checked for the thing a sharpening fix must NOT do: change the
## drawn size.
##
## MUST run WINDOWED -- `--headless` never composites the canvas, so every
## comparison below would pass vacuously (`MISTAKES.md`, "Run a pixel probe").
## Run:
##   Godot_v4.7.1-stable_win64_console.exe --path . _labelblur_probe.tscn
## No command-line arguments are read.  The window size is set in code.
##
## Exit status (this project's convention):
##   0  every assertion held
##   1  an assertion failed
##   2  could not run (headless, no ink, or a capture that cannot vary)

const OVERLAY := preload("res://map_overlay.gd")

const VP := Vector2i(1280, 800)
const GW := 512
const GH := 384
const Z_LOW := 1.0
const Z_HIGH := 3.0

## Fixed mid-grey ground so neither the light fill nor the dark halo is
## measured against the theme.  `MISTAKES.md`: a pixel threshold is
## palette-bound, so the probe supplies the palette rather than naming one.
const GROUND := Color(0.42, 0.44, 0.40)

## Retained edge contrast at `Z_HIGH` relative to `Z_LOW`.  A path rasterising
## at screen resolution is at ~1.0; a bitmap magnified by 3 is at ~1/3.  The
## band between them is deliberately wide -- this separates two mechanisms,
## it does not pin a number.
const KEEP_MIN := 0.80

var _root: Control
var _cam: Control
var _ov: Control
var _fail: Array[String] = []

const PIN := {
	"kind": "city", "x": GW / 2, "y": GH / 2, "faction": 1,
	"capital": false, "name": "Hammerfell", "population": 40000,
}
const LBL := {
	"text": "Hammerfell", "x": float(GW) * 0.5, "y": float(GH) * 0.5,
	"size": 13.0, "size_mode": "fixed", "color": "#f6ecd4",
	"angle": 0.0, "arc": 0.0, "generated": true,
}
## `_draw_labels()` has three draw paths, not one, and the fix touched all
## three: the single-`draw_string` straight run above, the per-glyph tracked/
## oblique run, and `drawArcLabel`'s per-glyph arc. A probe that only exercised
## the first would have said nothing about the two that carry a rotation.
const LBL_TRK := {
	"text": "Hammerfell", "x": float(GW) * 0.5, "y": float(GH) * 0.5,
	"size": 13.0, "size_mode": "fixed", "color": "#f6ecd4",
	"angle": 12.0, "arc": 0.0, "tracking_em": 0.06, "italic": true,
	"generated": true,
}
const LBL_ARC := {
	"text": "Hammerfell", "x": float(GW) * 0.5, "y": float(GH) * 0.5,
	"size": 13.0, "size_mode": "fixed", "color": "#f6ecd4",
	"angle": 0.0, "arc": 0.5, "generated": true,
}


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless server; the canvas is never composited.")
		get_tree().quit(2)
		return
	DisplayServer.window_set_size(VP)
	await get_tree().process_frame

	_root = Control.new()
	_root.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(_root)

	var bg := ColorRect.new()
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	bg.color = GROUND
	_root.add_child(bg)

	## Exactly `viewport_host.gd`'s camera: a FULL_RECT `Control` whose own
	## logical size equals the viewport's, with `scale` carrying the zoom.
	_cam = Control.new()
	_cam.set_anchors_preset(Control.PRESET_FULL_RECT)
	_root.add_child(_cam)

	_ov = Control.new()
	_ov.set_script(OVERLAY)
	_ov.set_anchors_preset(Control.PRESET_FULL_RECT)
	_cam.add_child(_ov)
	await get_tree().process_frame

	var pin_lo := await _shot("pin", Z_LOW)
	var pin_hi := await _shot("pin", Z_HIGH)
	var gen_lo := await _shot("gen", Z_LOW)
	var gen_hi := await _shot("gen", Z_HIGH)
	var trk_lo := await _shot("trk", Z_LOW)
	var trk_hi := await _shot("trk", Z_HIGH)
	var arc_lo := await _shot("arc", Z_LOW)
	var arc_hi := await _shot("arc", Z_HIGH)

	## Positive control.  Two captures of the same path at two zooms that come
	## back byte-identical mean the capture never saw the camera move, so no
	## comparison below could have failed.
	if pin_lo["img"].get_data() == pin_hi["img"].get_data():
		printerr("PROBE-CANNOT-RUN: pin captures byte-identical across zoom.")
		get_tree().quit(2)
		return
	if gen_lo["img"].get_data() == gen_hi["img"].get_data():
		printerr("PROBE-CANNOT-RUN: label captures byte-identical across zoom.")
		get_tree().quit(2)
		return
	for r in [pin_lo, pin_hi, gen_lo, gen_hi, trk_lo, trk_hi, arc_lo, arc_hi]:
		if int(r["ink"]) < 40:
			printerr("PROBE-CANNOT-RUN: %s drew %d ink px -- nothing to measure."
				% [r["tag"], r["ink"]])
			get_tree().quit(2)
			return

	for r in [pin_lo, pin_hi, gen_lo, gen_hi, trk_lo, trk_hi, arc_lo, arc_hi]:
		print("%-10s max|dL|=%.3f  top16=%.3f  ink=%5d  box=%s"
			% [r["tag"], r["max"], r["top"], r["ink"], r["box"]])

	var pin_keep: float = float(pin_hi["top"]) / maxf(float(pin_lo["top"]), 1e-6)
	var gen_keep: float = float(gen_hi["top"]) / maxf(float(gen_lo["top"]), 1e-6)
	print("edge contrast retained at %.0fx zoom -- `_crisp_begin` path %.2f, `_draw_labels` path %.2f"
		% [Z_HIGH / Z_LOW, pin_keep, gen_keep])
	## Height of the drawn glyph run, which says whether a path held its
	## on-screen size or grew with the camera.  Reported, not asserted: the two
	## paths size themselves by different rules and this is context for the
	## contrast numbers above, not a second claim.
	print("glyph-run height px -- pin %d -> %d, label %d -> %d"
		% [pin_lo["box"].size.y, pin_hi["box"].size.y, gen_lo["box"].size.y, gen_hi["box"].size.y])

	## Both paths must hold their edge contrast.  `_draw()`'s pin label already
	## did before this probe existed (it wraps its `draw_string` in
	## `_crisp_begin()`); `_draw_labels()` measured **0.37 at 3x** on the tree
	## as of 2026-09-08 -- a 13 px bitmap stretched over 38 screen pixels, and
	## the owner's report.  The second assertion is therefore written to FAIL
	## on the unfixed file, which is the only way it earns any authority.
	_expect(pin_keep > KEEP_MIN,
		"`_draw()`'s pin label kept >%.0f%% of its edge contrast at %.0fx: %.2f"
			% [KEEP_MIN * 100.0, Z_HIGH, pin_keep])
	_expect(gen_keep > KEEP_MIN,
		"`_draw_labels()` kept >%.0f%% of its edge contrast at %.0fx: %.2f"
			% [KEEP_MIN * 100.0, Z_HIGH, gen_keep])

	## Sharpening a label must not RESIZE it.  `_label_font_px()` is the size
	## model this file draws with; `label_box_at`/`label_handles` on the engine
	## side hit-test and place handles against the SAME model now
	## (`LARGE_ITEM_RULINGS.md` Ruling AG, 2026-09-23, verified by
	## `_labelboxmodel_probe.gd`) -- so a rasterisation fix that also moved the
	## drawn size would widen a gap that used to be silent and is now a real,
	## checked invariant.  The measured box is inked pixels including the halo,
	## so the tolerance covers font hinting at two sizes and the halo's own
	## rounding, not a size change.
	for pair in [[trk_lo, trk_hi, "tracked+oblique"], [arc_lo, arc_hi, "arched"]]:
		var lo: Dictionary = pair[0]
		var hi: Dictionary = pair[1]
		var keep: float = float(hi["top"]) / maxf(float(lo["top"]), 1e-6)
		var grew: float = float(hi["box"].size.x) / maxf(float(lo["box"].size.x), 1.0)
		print("%s: contrast retained %.2f, drawn width x%.2f (camera is x%.2f)"
			% [pair[2], keep, grew, Z_HIGH / Z_LOW])
		_expect(keep > KEEP_MIN, "the %s path kept its edge contrast: %.2f" % [pair[2], keep])
		_expect(absf(grew - Z_HIGH / Z_LOW) / (Z_HIGH / Z_LOW) < 0.05,
			"the %s path's drawn width still scales with the camera and nothing else: x%.2f"
				% [pair[2], grew])

	var want := float(gen_lo["box"].size.x) * Z_HIGH / Z_LOW
	var got := float(gen_hi["box"].size.x)
	_expect(absf(got - want) / want < 0.05,
		"the label's drawn width still scales with the camera and nothing else: %.0f px vs %.0f expected (%.1f%%)"
			% [got, want, 100.0 * (got - want) / want])

	for f in _fail:
		printerr("ASSERT-FAILED: ", f)
	print("PROBE-RESULT: ", "PASS" if _fail.is_empty() else "FAIL")
	print("images: ", ProjectSettings.globalize_path("user://"))
	get_tree().quit(0 if _fail.is_empty() else 1)


## One frame with exactly one text path live, zoomed about the viewport centre
## the way `_zoom_at()` does, so the grid-centre subject lands on the same
## pixel in every frame.
func _shot(which: String, z: float) -> Dictionary:
	_ov.set_civ_data([PIN] if which == "pin" else [], [], [], GW, GH, 0.0)
	var lbls: Array = []
	match which:
		"gen": lbls = [LBL]
		"trk": lbls = [LBL_TRK]
		"arc": lbls = [LBL_ARC]
	_ov.set_labels(lbls)
	var s := Vector2(VP)
	_cam.scale = Vector2(z, z)
	_cam.position = s * 0.5 - (s * 0.5) * z
	_ov.set_camera_zoom(z)
	_ov.queue_redraw()
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	var tag := "%s z=%.0f" % [which, z]
	img.save_png("user://labelblur_%s_z%d.png" % [which, int(z)])
	var m := _measure(img)
	m["tag"] = tag
	m["img"] = img
	return m


## Sharpness as the strongest horizontal luminance steps over the whole frame
## -- safe because the frame holds exactly one text path and nothing else.
##
## `max` is the single steepest adjacent-pixel step; `top` is the mean of the
## 16 steepest, which is what the assertions use: one pixel pair can be a
## stray, sixteen cannot.  `ink` counts pixels differing from `GROUND`, and
## `box` is their bounding rect, so a frame that drew nothing (or drew it
## off-screen) cannot report a pass.
func _measure(img: Image) -> Dictionary:
	var deltas := PackedFloat32Array()
	var ink := 0
	var mx := 0.0
	var x0 := 1 << 30
	var y0 := 1 << 30
	var x1 := -1
	var y1 := -1
	for y in img.get_height():
		var prev := _lum(img.get_pixel(0, y))
		for x in range(1, img.get_width()):
			var c := img.get_pixel(x, y)
			if absf(c.r - GROUND.r) > 0.02 or absf(c.g - GROUND.g) > 0.02 or absf(c.b - GROUND.b) > 0.02:
				ink += 1
				x0 = mini(x0, x)
				y0 = mini(y0, y)
				x1 = maxi(x1, x)
				y1 = maxi(y1, y)
			var l := _lum(c)
			var d := absf(l - prev)
			if d > 0.02:
				deltas.append(d)
			if d > mx:
				mx = d
			prev = l
	var arr := Array(deltas)
	arr.sort()
	arr.reverse()
	var n: int = mini(16, arr.size())
	var top := 0.0
	for i in n:
		top += float(arr[i])
	return {
		"max": mx,
		"top": (top / float(n)) if n > 0 else 0.0,
		"ink": ink,
		"box": Rect2i(x0, y0, maxi(0, x1 - x0 + 1), maxi(0, y1 - y0 + 1)) if x1 >= 0 else Rect2i(),
	}


func _lum(c: Color) -> float:
	return 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b


func _expect(ok: bool, what: String) -> void:
	if ok:
		print("  ok   ", what)
	else:
		_fail.append(what)
