extends Node
## TEMPORARY, untracked probe -- exact per-way-type colour recovery.
##
##   Godot_v4.7.1-stable_win64.exe --path . _waycolor_probe.tscn
##
## Drives the REAL `map_overlay.gd` draw code (`_draw_way_segment`,
## `_draw_sea_route_segment`, `_draw_manual_route_segment`) on two known flat
## backgrounds -- pure black and pure white -- so the composited colour AND the
## effective alpha of every stroke can be solved for exactly:
##
##   over black:  b = C*a
##   over white:  w = C*a + (1-a)      =>  a = 1 - (w - b),  C = b / a
##
## Full pixel coverage is the precondition, so the probe sets
## `set_camera_zoom(0.2)`: `_crisp_begin()` draws inside a 1/k transform, so
## every width constant lands 5x thicker on screen and the stroke centre row is
## fully covered by both the underlayer and the overlay. Geometry is unchanged
## (`_stroke_points` multiplies by the same k), so this is the same code path a
## real frame takes, only wider.

const W := 1200
const H := 800
const ZOOM := 0.2

## y row / sample x / label, one straight horizontal way each.
var _rows := [
	{"y": 80, "x": 200, "kind": "road", "type": "highway"},
	{"y": 160, "x": 200, "kind": "road", "type": "regional"},
	{"y": 240, "x": 104, "kind": "road", "type": "road"},
	{"y": 320, "x": 102, "kind": "road", "type": "track"},
	{"y": 400, "x": 105, "kind": "road", "type": "ancient"},
	{"y": 480, "x": 105, "kind": "sea", "type": "sea_lane"},
	{"y": 560, "x": 110, "kind": "route", "type": "route"},
	{"y": 640, "x": 110, "kind": "route_sel", "type": "route_selected"},
]

## The reference's literal `drawCivLayer` §2a/§2b constants (lines 15511-15560),
## as 0-255 rgb + alpha, so the composite the probe recovers can be checked
## against a number that came from the HTML rather than from this port.
var _ref := {
	"highway":        {"u": [20, 10, 5], "ua": 0.55, "o": [210, 145, 55], "oa": 0.98, "dash": 0.0, "gap": 0.0},
	"regional":       {"u": [25, 14, 5], "ua": 0.45, "o": [178, 118, 52], "oa": 0.88, "dash": 0.0, "gap": 0.0},
	"road":           {"u": [30, 20, 10], "ua": 0.40, "o": [160, 100, 60], "oa": 0.75, "dash": 1.8, "gap": 1.3},
	"track":          {"u": [30, 20, 10], "ua": 0.35, "o": [100, 120, 60], "oa": 0.75, "dash": 1.3, "gap": 2.0},
	"ancient":        {"u": [20, 10, 5], "ua": 0.35, "o": [120, 110, 100], "oa": 0.65, "dash": 2.5, "gap": 1.3},
	"sea_lane":       {"u": [10, 30, 60], "ua": 0.40, "o": [30, 130, 200], "oa": 0.70, "dash": 2.6, "gap": 2.0},
	"route":          {"u": [40, 25, 5], "ua": 0.50, "o": [200, 160, 60], "oa": 0.85, "dash": 5.0, "gap": 3.0},
	"route_selected": {"u": [40, 25, 5], "ua": 0.50, "o": [255, 210, 80], "oa": 0.98, "dash": 5.0, "gap": 3.0},
}


## Full-coverage composite of one reference row over pure black, plus its
## effective alpha -- underlayer first, overlay on top, ordinary source-over.
func _predict(t: String) -> Array:
	var d: Dictionary = _ref[t]
	var ua: float = d["ua"]
	var oa: float = d["oa"]
	var out := []
	for i in 3:
		var u: float = float(d["u"][i]) * ua
		out.append(float(d["o"][i]) * oa + u * (1.0 - oa))
	return [out, oa + ua * (1.0 - oa)]


var _vp: SubViewport
var _bg: ColorRect
var _ov: Control


func _p(s: String) -> void:
	print("WAYCOLOR  %s" % s)


func _lum(c: Color) -> float:
	return c.r + c.g + c.b


func _line(y: int) -> PackedVector2Array:
	return PackedVector2Array([Vector2(100, y), Vector2(1100, y)])


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 120.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		_p("WATCHDOG -- probe did not finish")
		get_tree().quit(2))
	wd.start()

	_vp = SubViewport.new()
	_vp.size = Vector2i(W, H)
	_vp.transparent_bg = false
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)

	_bg = ColorRect.new()
	_bg.size = Vector2(W, H)
	_bg.color = Color.BLACK
	_vp.add_child(_bg)

	_ov = Control.new()
	_ov.set_script(load("res://map_overlay.gd"))
	_ov.size = Vector2(W, H)
	_vp.add_child(_ov)

	var roads: Array = []
	var seas: Array = []
	var routes: Array = []
	for r in _rows:
		var pts := _line(int(r["y"]))
		match String(r["kind"]):
			"road":
				roads.append({"points": pts, "brks": PackedInt32Array(),
					"way_type": String(r["type"]), "name": "", "km": 1.0, "manual": false})
			"sea":
				seas.append({"points": pts, "brks": PackedInt32Array(), "name": "", "km": 1.0})
			"route", "route_sel":
				routes.append({"render_points": pts, "render_brks": PackedInt32Array(),
					"points": pts, "brks": PackedInt32Array(), "name": "", "km": 1.0})

	_ov.set_camera_zoom(ZOOM)
	_ov.set_civ_data([], roads, seas, W, H, 0.0)
	_ov.set_manual_routes(routes)
	_ov.set_selected_manual_route(1)   ## second route row is the selected one
	_ov.queue_redraw()

	var img_black := await _capture(Color.BLACK)
	var img_white := await _capture(Color.WHITE)

	_p("--- recovered drawn colour (C) and effective alpha (a) per row ---")
	for r in _rows:
		var y := int(r["y"])
		var x := int(r["x"])
		var b := img_black.get_pixel(x, y)
		var w := img_white.get_pixel(x, y)
		var ar := 1.0 - (w.r - b.r)
		var ag := 1.0 - (w.g - b.g)
		var ab := 1.0 - (w.b - b.b)
		var a := (ar + ag + ab) / 3.0
		var c := Color(b.r / maxf(a, 1e-6), b.g / maxf(a, 1e-6), b.b / maxf(a, 1e-6))
		var pred: Array = _predict(String(r["type"]))
		var pc: Array = pred[0]
		var pa: float = pred[1]
		var dmax := maxf(maxf(absf(b.r * 255 - pc[0]), absf(b.g * 255 - pc[1])), absf(b.b * 255 - pc[2]))
		_p("%-16s @(%d,%d)  measured black=(%3d,%3d,%3d) a=%.3f | reference predicts (%5.1f,%5.1f,%5.1f) a=%.3f | max delta %.1f/255  %s" % [
			String(r["type"]), x, y,
			int(round(b.r * 255)), int(round(b.g * 255)), int(round(b.b * 255)), a,
			pc[0], pc[1], pc[2], pa, dmax, "OK" if dmax <= 1.5 and absf(a - pa) <= 0.01 else "MISMATCH"])
		_p("%-16s      recovered C=(%3d,%3d,%3d) from white=(%3d,%3d,%3d)" % [
			"", int(round(c.r * 255)), int(round(c.g * 255)), int(round(c.b * 255)),
			int(round(w.r * 255)), int(round(w.g * 255)), int(round(w.b * 255))])

	## Underlayer-only sample: a row offset from the centre far enough to clear
	## the (narrower) overlay but stay inside the (wider) underlayer. Only
	## meaningful for the two-stroke types; a single-stroke way reports its one
	## colour twice, which is itself the finding.
	_p("--- underlayer-only sample (centre row +/- offset, inside underlay, outside overlay) ---")
	var offs := {"highway": 4, "regional": 3, "road": 2, "track": 2, "ancient": 2,
		"sea_lane": 3, "route": 6, "route_selected": 9}
	for r in _rows:
		var y := int(r["y"]) + int(offs.get(String(r["type"]), 3))
		var x := int(r["x"])
		var b := img_black.get_pixel(x, y)
		var w := img_white.get_pixel(x, y)
		var a := 1.0 - ((w.r - b.r) + (w.g - b.g) + (w.b - b.b)) / 3.0
		var c := Color(b.r / maxf(a, 1e-6), b.g / maxf(a, 1e-6), b.b / maxf(a, 1e-6))
		_p("%-16s @(%d,%d)  =>  C=(%3d,%3d,%3d) a=%.3f" % [
			String(r["type"]), x, y,
			int(round(c.r * 255)), int(round(c.g * 255)), int(round(c.b * 255)), a])

	## Measured stroke extent: scan the column at the sample x for any pixel that
	## differs from the black background, so total drawn width is a measurement
	## rather than a reading of the constant.
	_p("--- measured total stroke extent (px, at %.1fx width scale) ---" % (1.0 / ZOOM))
	for r in _rows:
		var y0 := int(r["y"])
		var x := int(r["x"])
		var top := y0
		var bot := y0
		for dy in range(1, 40):
			if img_black.get_pixel(x, y0 - dy).r + img_black.get_pixel(x, y0 - dy).g + img_black.get_pixel(x, y0 - dy).b > 0.002:
				top = y0 - dy
			if img_black.get_pixel(x, y0 + dy).r + img_black.get_pixel(x, y0 + dy).g + img_black.get_pixel(x, y0 + dy).b > 0.002:
				bot = y0 + dy
		_p("%-16s extent %d px  (=> %.2f px at 1x)" % [String(r["type"]), bot - top + 1,
			(bot - top + 1) * ZOOM])

	## Dash duty cycle, measured along the row: run-lengths of "brighter than the
	## underlayer alone" tell on/off in screen px.
	_p("--- measured dash period along the row (on px / off px at %.1fx) ---" % (1.0 / ZOOM))
	for r in _rows:
		var y := int(r["y"])
		var runs: Array = []
		var cur_on := false
		var run := 0
		## Threshold midway between the row's own darkest and brightest pixel --
		## the underlayer is solid the whole length, so the darkest IS a gap and
		## the brightest a dash. Sampling one fixed "base" x instead silently
		## lands inside a dash whenever the period divides that way.
		var lo := 9.0
		var hi := 0.0
		for x in range(150, 700):
			var s := _lum(img_black.get_pixel(x, y))
			lo = minf(lo, s)
			hi = maxf(hi, s)
		var mid := (lo + hi) * 0.5
		if hi - lo < 0.02:
			_p("%-16s solid (no dash: row range %.3f)" % [String(r["type"]), hi - lo])
			continue
		for x in range(150, 700):
			var on: bool = _lum(img_black.get_pixel(x, y)) > mid
			if x == 150:
				cur_on = on
				run = 1
				continue
			if on == cur_on:
				run += 1
			else:
				runs.append([cur_on, run])
				cur_on = on
				run = 1
		runs.append([cur_on, run])
		var summary := ""
		for i in range(mini(6, runs.size())):
			summary += ("on%d " % runs[i][1]) if runs[i][0] else ("off%d " % runs[i][1])
		## Period is measurable exactly (consecutive on-run starts); the on/off
		## split is not, because an antialiased dash cap bleeds ~width/2 past
		## each end, moving px from off into on. Period is the honest check.
		var period := 0
		var first := -1
		for i in range(runs.size()):
			if not runs[i][0]:
				continue
			if first < 0:
				first = i
			elif i - first >= 2:
				break
		var acc := 0
		var started := false
		var cycles := 0
		for i in range(runs.size()):
			if runs[i][0] and not started:
				started = true
				acc = 0
				continue
			if started:
				acc += runs[i][1]
				if runs[i][0]:
					cycles += 1
					period = acc
					break
		var ref_period: float = (float(_ref[String(r["type"])]["dash"]) + float(_ref[String(r["type"])]["gap"])) / ZOOM
		_p("%-16s %s| period %d px, reference %.1f px  %s" % [String(r["type"]), summary,
			period, ref_period, "OK" if absf(period - ref_period) <= 1.0 else "MISMATCH"])

	img_black.save_png("user://waycolor_black.png")
	img_white.save_png("user://waycolor_white.png")
	_p("shots -> %s" % ProjectSettings.globalize_path("user://waycolor_black.png"))
	get_tree().quit(0)


func _capture(bg: Color) -> Image:
	_bg.color = bg
	_ov.queue_redraw()
	for i in 4:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	return _vp.get_texture().get_image()
