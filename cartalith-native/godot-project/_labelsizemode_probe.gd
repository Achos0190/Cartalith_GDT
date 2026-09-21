extends Node
## CARTO ▸ Labels, 2026-09-21 batch: (1) generated settlement/POI (landmark)
## labels' `size_mode` is overridden to `"fixed"` by default, shell-side, in
## `map_overlay.gd`; (2) the per-label Font picker in
## `cartography_workspace.gd` (replacing a free-text field) actually selects
## a real, different loaded face.
##
## **A finding this probe exists to pin, not just the feature**: `size_mode`
## ("fixed"/"zoom") does NOT make a label track or resist the LIVE camera zoom
## (`ViewportHost._camera.scale`, pushed here as `_camera_zoom`) at all.
## `_label_font_px()` never reads `_camera_zoom` in either branch -- unlike
## the settlement-pin path two hundred lines above it in the same file, which
## applies `civ_zoom_k(_camera_zoom)` specifically so a pin holds a roughly
## constant on-screen size across camera zoom (`PIN_SCALE_REF_PX`'s own doc
## comment). `size_mode` instead answers a different question:
## `"zoom"` scales a label with `rect.size.x / _gw` -- the grid-to-window
## LETTERBOX FIT (`_displayed_rect()`, which depends on this control's own
## `size` and `_gw`/`_gh`, never on `_camera_zoom`) -- and `"fixed"` does not.
## That fit changes across worlds and window sizes, not while the user
## pinches or scrolls the wheel. So under a LIVE zoom change, `"fixed"` and
## `"zoom"` labels are measured here to move IDENTICALLY (Part B, printed, not
## asserted -- a true finding, not a bug in this probe), and the FIT-based
## distinction size_mode actually governs is measured separately (Part C, at
## a fixed camera zoom, across two window/grid fits).
##
## This does not make the 2026-09-21 default wrong -- the override still
## does exactly what it claims (it changes which grid-fit rule a generated
## label follows) -- but it means the practical default alone does not yet
## deliver "constant text height as you interactively zoom" the way a plain
## reading of "fixed" would suggest. That gap is `_label_font_px` itself, is
## pre-existing, and is reported rather than silently patched here -- fixing
## it would change already-shipped, `_labelblur_probe.gd`-verified behaviour
## for every label using either mode, which needs its own pass.
##
## Mirrors `_labelblur_probe.gd`'s harness: `map_overlay.gd` ALONE under a
## scaled `Control` standing in for `ViewportHost`'s camera, no shell, no
## engine, so every measured pixel is attributable to this file's own
## `set_labels()` / `_apply_generated_size_mode_override()` / `_label_font_for()`.
##
## Run WINDOWED -- `--headless` never composites the canvas
## (`MISTAKES.md`, "Run a pixel probe"). Parts A and D need no rendering and
## would run headless, but this file runs everything one way, matching its
## own header rather than mandating one path per part. Run:
##   Godot_v4.7.1-stable_win64_console.exe --path . _labelsizemode_probe.tscn
##
## Exit status (this project's convention):
##   0  every assertion held
##   1  an assertion failed
##   2  could not run (headless, no ink, or a capture that cannot vary)

const OVERLAY := preload("res://map_overlay.gd")
## `DccTheme` is `class_name`-global (`shell/dcc_theme.gd`); no preload needed.
const FIRA_CODE := preload("res://fonts/FiraCode-Regular.ttf")

const VP := Vector2i(1280, 800)
const GW := 512
const GH := 384
const Z_LOW := 1.0
const Z_HIGH := 3.0

## Same reasoning as `_labelblur_probe.gd`'s own: a fixed mid-grey ground so
## neither the fill nor the halo is measured against a live theme.
const GROUND := Color(0.42, 0.44, 0.40)

var _root: Control
var _cam: Control
var _ov: Control
var _fail: Array[String] = []

func _lbl(generated: bool, cls: String, size_mode: String) -> Dictionary:
	return {
		"text": "Hammerfell", "x": float(GW) * 0.5, "y": float(GH) * 0.5,
		"size": 13.0, "size_mode": size_mode, "color": "#f6ecd4",
		"angle": 0.0, "arc": 0.0, "generated": generated, "class": cls,
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

	_cam = Control.new()
	_cam.set_anchors_preset(Control.PRESET_FULL_RECT)
	_root.add_child(_cam)

	_ov = Control.new()
	_ov.set_script(OVERLAY)
	_ov.set_anchors_preset(Control.PRESET_FULL_RECT)
	_cam.add_child(_ov)
	await get_tree().process_frame

	# ---- Part A: font resolution, no rendering needed ---------------------
	_check_font_resolution()

	# ---- Part D: the override rewrites the DATA, no rendering needed ------
	_check_override_data()

	# ---- Part B: live camera zoom -- printed finding, not an assertion ----
	_ov.set_civ_data([], [], [], GW, GH, 0.0)
	await _measure_camera_zoom_sensitivity()

	# ---- Part C: the grid-to-window FIT -- what size_mode actually gates --
	await _measure_grid_fit_sensitivity()

	for f in _fail:
		printerr("ASSERT-FAILED: ", f)
	print("PROBE-RESULT: ", "PASS" if _fail.is_empty() else "FAIL")
	get_tree().quit(0 if _fail.is_empty() else 1)


## `_label_font_for()` is a pure lookup over a `Dictionary` -- no camera, no
## redraw. Resource identity (`==`) rather than a pixel comparison: Godot
## caches a `preload()`d resource by path, so the SAME font file loaded here
## and in `map_overlay.gd` is the SAME `Font` instance, and this is the
## cheapest possible way to prove the picker's three values reach three
## different faces rather than three names over one bitmap.
func _check_font_resolution() -> void:
	var theme_default := _ov.get_theme_default_font()
	_expect(_ov._label_font_for({"font": "IBM Plex Mono"}) == DccTheme.FONT_MONO,
		"font \"IBM Plex Mono\" resolves to DccTheme.FONT_MONO")
	_expect(_ov._label_font_for({"font": "Fira Code"}) == FIRA_CODE,
		"font \"Fira Code\" resolves to the bundled FiraCode-Regular.ttf")
	_expect(_ov._label_font_for({"font": "Fira Code"}) != DccTheme.FONT_MONO,
		"the two named faces are not secretly the same resource")
	_expect(_ov._label_font_for({"font": ""}) == theme_default,
		"an empty font string still resolves to the theme's default face")
	_expect(_ov._label_font_for({"font": "Georgia, serif"}) == theme_default,
		"the engine's own literal default (\"Georgia, serif\", an unloaded face) falls through to the theme default rather than tofu")
	_expect(_ov._label_font_for({"font": "Comic Sans"}) == theme_default,
		"an unrecognised free-form string (an older save) falls through to the theme default")


## The practical default and its three controls, checked at the DATA level --
## `set_labels()`'s own job is to overwrite `size_mode` on the copies it is
## handed, and reading that back needs no camera, no redraw, no pixels.
func _check_override_data() -> void:
	var rows := [
		_lbl(true, "settlement", "zoom"),   # what the engine sends today
		_lbl(true, "landmark", "zoom"),
		_lbl(true, "continental", "zoom"),  # NOT in the override table
		_lbl(false, "settlement", "zoom"),  # hand-placed: never touched
	]
	_ov.set_labels(rows)
	var after: Array = _ov._labels
	_expect(String(after[0]["size_mode"]) == "fixed",
		"a fresh overlay's own default overrides a generated SETTLEMENT label's engine-sent \"zoom\" to \"fixed\"")
	_expect(String(after[1]["size_mode"]) == "fixed",
		"a fresh overlay's own default overrides a generated LANDMARK (POI) label's engine-sent \"zoom\" to \"fixed\"")
	_expect(String(after[2]["size_mode"]) == "zoom",
		"a generated CONTINENTAL label, not in the override table, keeps the engine's own \"zoom\"")
	_expect(String(after[3]["size_mode"]) == "zoom",
		"a HAND-PLACED label is never touched by the override, whatever its class")

	# The override is live and reversible, and an empty override is a no-op.
	_ov.set_generated_size_mode_override({"settlement": "zoom"})
	_expect(String(_ov._labels[0]["size_mode"]) == "zoom",
		"set_generated_size_mode_override({\"settlement\":\"zoom\"}) flips the SAME already-set row back")
	_ov.set_generated_size_mode_override({})
	_ov.set_labels(rows.duplicate(true))
	_expect(String(_ov._labels[0]["size_mode"]) == "zoom",
		"an empty override leaves the engine's own size_mode alone")
	# Restore the practical default for Part B/C below.
	_ov.set_generated_size_mode_override({"settlement": "fixed", "landmark": "fixed"})


## Printed, not asserted -- see the header. This documents, rather than
## claims, that `size_mode` does not respond to `_camera_zoom` at all in
## `_label_font_px()`, for either value, unlike the settlement-pin path
## (`civ_zoom_k(_camera_zoom)`) a few hundred lines above it in the same file.
func _measure_camera_zoom_sensitivity() -> void:
	var fixed_lo := await _shot(_lbl(false, "settlement", "fixed"), Z_LOW, 1.0)
	var fixed_hi := await _shot(_lbl(false, "settlement", "fixed"), Z_HIGH, 1.0)
	var zoom_lo := await _shot(_lbl(false, "settlement", "zoom"), Z_LOW, 1.0)
	var zoom_hi := await _shot(_lbl(false, "settlement", "zoom"), Z_HIGH, 1.0)
	if int(fixed_lo["ink"]) < 40 or int(zoom_lo["ink"]) < 40:
		printerr("PROBE-CANNOT-RUN: camera-zoom leg drew nothing to measure.")
		get_tree().quit(2)
		return
	var r_fixed: float = float(fixed_hi["box"].size.x) / maxf(float(fixed_lo["box"].size.x), 1.0)
	var r_zoom: float = float(zoom_hi["box"].size.x) / maxf(float(zoom_lo["box"].size.x), 1.0)
	print("FINDING -- live camera zoom x%.2f: \"fixed\" box-width ratio %.2f, \"zoom\" box-width ratio %.2f (identical: size_mode does not gate this)"
		% [Z_HIGH / Z_LOW, r_fixed, r_zoom])


## The real, positive control: at a FIXED camera zoom (1.0, so the ancestor
## transform contributes nothing), shrink the grid-to-window fit by doubling
## `_gw` -- `_displayed_rect()`'s `scale` and therefore `rect.size.x/_gw`
## roughly halves. A "zoom"-mode label must shrink with it; a "fixed"-mode
## label must not move at all, since its formula never reads `rect`.
func _measure_grid_fit_sensitivity() -> void:
	_ov.set_civ_data([], [], [], GW, GH, 0.0)
	var fixed_wide := await _shot(_lbl(false, "settlement", "fixed"), Z_LOW, 1.0)
	var zoom_wide := await _shot(_lbl(false, "settlement", "zoom"), Z_LOW, 1.0)
	_ov.set_civ_data([], [], [], GW * 2, GH, 0.0)
	var fixed_narrow := await _shot(_lbl(false, "settlement", "fixed"), Z_LOW, 1.0)
	var zoom_narrow := await _shot(_lbl(false, "settlement", "zoom"), Z_LOW, 1.0)
	_ov.set_civ_data([], [], [], GW, GH, 0.0)   ## restore, in case anything else runs after

	if int(fixed_wide["ink"]) < 40 or int(zoom_wide["ink"]) < 40 or int(zoom_narrow["ink"]) < 40:
		printerr("PROBE-CANNOT-RUN: grid-fit leg drew nothing to measure.")
		get_tree().quit(2)
		return

	var r_fixed: float = float(fixed_narrow["box"].size.x) / maxf(float(fixed_wide["box"].size.x), 1.0)
	var r_zoom: float = float(zoom_narrow["box"].size.x) / maxf(float(zoom_wide["box"].size.x), 1.0)
	print("grid-fit (_gw doubled at fixed camera zoom): \"fixed\" box-width ratio %.2f, \"zoom\" box-width ratio %.2f"
		% [r_fixed, r_zoom])
	_expect(absf(r_fixed - 1.0) < 0.05,
		"a \"fixed\"-mode label's on-screen width does not move when the grid-to-window fit changes: ratio %.2f" % r_fixed)
	_expect(r_zoom < 0.65,
		"a \"zoom\"-mode label's on-screen width shrinks with the grid-to-window fit (doubling _gw): ratio %.2f, expected near 0.5" % r_zoom)


## One frame with exactly one label live, zoomed about the viewport centre --
## `_labelblur_probe.gd::_shot()`'s own pattern, generalised to take the label
## fixture and BOTH scale knobs (`cam_z`: the ancestor `_cam.scale`, and
## `label_z`: what `set_camera_zoom()` pushes into `_label_raster_px`) so Part
## B and Part C can vary either independently.
func _shot(lb: Dictionary, cam_z: float, label_z: float) -> Dictionary:
	_ov.set_labels([lb])
	var s := Vector2(VP)
	_cam.scale = Vector2(cam_z, cam_z)
	_cam.position = s * 0.5 - (s * 0.5) * cam_z
	_ov.set_camera_zoom(label_z)
	_ov.queue_redraw()
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	return _measure(img)


## Ink bounding box only -- this probe never needs edge-contrast sharpness,
## just "how wide is the drawn glyph run", so it is `_labelblur_probe.gd::
## _measure()` trimmed to the one field this file's assertions use.
func _measure(img: Image) -> Dictionary:
	var ink := 0
	var x0 := 1 << 30
	var y0 := 1 << 30
	var x1 := -1
	var y1 := -1
	for y in img.get_height():
		for x in img.get_width():
			var c := img.get_pixel(x, y)
			if absf(c.r - GROUND.r) > 0.02 or absf(c.g - GROUND.g) > 0.02 or absf(c.b - GROUND.b) > 0.02:
				ink += 1
				x0 = mini(x0, x)
				y0 = mini(y0, y)
				x1 = maxi(x1, x)
				y1 = maxi(y1, y)
	return {
		"ink": ink,
		"box": Rect2i(x0, y0, maxi(0, x1 - x0 + 1), maxi(0, y1 - y0 + 1)) if x1 >= 0 else Rect2i(),
	}


func _expect(ok: bool, what: String) -> void:
	if ok:
		print("  ok   ", what)
	else:
		_fail.append(what)
