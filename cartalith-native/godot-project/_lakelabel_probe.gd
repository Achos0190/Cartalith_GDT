extends Node
## Owner report, 2026-09-27: *"Sometimes lakes get labelled when there isn't a
## lake visible on the map."* Two causes were found and fixed:
##
##  A. `labels_generate` named lakes off `CivData::water_bodies`, a copy taken
##     at generate time and never refreshed by a sculpt, while the map redraws
##     its lakes from the live heightfield (`build_color_texture` ->
##     `drawn_water_classification`). So after a sculpt filled a lake, pressing
##     Generate labels -- which the Labels panel's "terrain has changed" cue
##     asks for (Ruling BB) -- still named the vanished lake.
##  B. A crescent lake's label sat at its centroid, on land
##     (`cartalith_civ::labels::lake_label_anchors`; unit-tested in Rust).
##
## This probe drives A end to end: generate, label, dig a pit in dry high
## ground with freehand Lower strokes and commit, label again -- the new lake
## must be named, because the map draws it (see `_ready` for why it digs rather
## than fills). Before the fix the label pass read the pre-sculpt copy and
## named nothing there. The premises (dry ground, the ground really lowered)
## are read from the bridge, not assumed.
##
##   Godot --headless --path . _lakelabel_probe.tscn

var app: Node
var _fails := 0
var _checks := 0


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _check(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("LAKELABEL %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _p(s: String) -> void:
	print("LAKELABEL %s" % s)


func _water_labels(bridge) -> Array:
	var out: Array = []
	for lb: Dictionary in bridge.labels_render_list():
		if String(lb.get("class", "")) == "water" and bool(lb.get("generated", false)):
			out.append(lb)
	return out


func _ready() -> void:
	var wd := get_tree().create_timer(900.0)
	wd.timeout.connect(func():
		_p("WATCHDOG")
		get_tree().quit(3))
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	var bridge = app.bridge
	var lakes: Array = []
	for seed in [483920, 24601, 71077345, 552017]:
		bridge.generate({"seed": seed, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
			"archetype": "", "villages": true, "sea_level": 0.42})
		while bridge.generating:
			await get_tree().create_timer(0.25).timeout
		await _frames(4)
		var r: Dictionary = bridge.labels_generate({"enabled": {"water": true}, "cull": {"on": false}})
		lakes = _water_labels(bridge)
		_p("seed %d: labels ok=%s total=%s water=%d" % [seed, r.get("ok"), r.get("total"), lakes.size()])
		if not lakes.is_empty():
			break
	_check("P0 a world with named lakes", not lakes.is_empty())
	if lakes.is_empty():
		_finish()
		return

	## Every named lake's anchor is a lake cell (the civ copy is fresh right
	## after generate, so sample_cell's `water` is the drawn classification here).
	var off: Array = []
	for lb: Dictionary in lakes:
		var c: Dictionary = bridge.sample_cell(roundi(float(lb["x"])), roundi(float(lb["y"])))
		if String(c.get("water", "")) != "lake":
			off.append("%s@%s,%s=%s" % [lb["text"], lb["x"], lb["y"], c.get("water")])
	_check("P1 every lake label is anchored on a lake cell (%d labels)" % lakes.size(), off.is_empty(), "%s" % [off])

	## Why the probe DIGS a lake rather than filling one: measured 2026-09-27, a
	## lattice of freehand Raise strokes over a named lake raised every land
	## cell around it by ~2 800 m and left each lake cell at +0 m, so a sculpt
	## cannot remove a lake here. It can make one, and that is the same defect
	## from the other side: the labels read a classification the sculpt did
	## not refresh. Find dry, high ground well away from any water.
	var g: Vector2i = bridge.grid_size()
	var spot := Vector2i(-1, -1)
	## Land for 40 cells all round: a dig that reaches the sea is an inlet,
	## rightly ocean, not a lake.
	for gy in range(60, g.y - 60, 16):
		for gx in range(60, g.x - 60, 16):
			var ok := true
			for oy in range(-40, 41, 8):
				for ox in range(-40, 41, 8):
					var c: Dictionary = bridge.sample_cell(gx + ox, gy + oy)
					if String(c.get("water", "")) != "land" or float(c.get("elevation_m", 0.0)) < 600.0:
						ok = false
			if ok:
				spot = Vector2i(gx, gy)
				break
		if spot.x >= 0:
			break
	_check("P2 found dry high ground to dig in", spot.x >= 0)
	if spot.x < 0:
		_finish()
		return
	var near_before := _near(lakes, spot)
	_p("dig at %s; lake labels within 12 cells before: %s" % [spot, near_before])
	_check("P3 premise: no lake is named there yet", near_before.is_empty())
	_check("P4 sculpt feature set to freehand lower", bridge.sculpt_set_feature("freehand")
		and bridge.sculpt_set_freehand_mode("lower"))
	for oy in range(-6, 7, 3):
		for ox in range(-6, 7, 3):
			bridge.sculpt_begin_stroke()
			bridge.sculpt_add_point(spot.x + ox - 0.5, spot.y + oy)
			bridge.sculpt_add_point(spot.x + ox, spot.y + oy)
			bridge.sculpt_end_stroke()
	var e0 := float(bridge.sample_cell(spot.x, spot.y).get("elevation_m", 0.0))
	var summary: Dictionary = bridge.sculpt_commit("lake label probe")
	await _frames(4)
	var e1 := float(bridge.sample_cell(spot.x, spot.y).get("elevation_m", 0.0))
	_p("commit: stamps %s, still_stale %s; elevation at the dig %.0f -> %.0f m" % [summary.get("stamps_applied"),
		summary.get("still_stale"), e0, e1])
	_check("P5 premise: the dig lowered the ground", e1 < e0 - 200.0)
	## Collision culling off for both runs (set on the first): a new lake's
	## label must not be hidden by a neighbour's box, which is a different
	## question from whether it is named at all.
	## The engine refreshes nothing for labels on a commit (Ruling BB); the user
	## presses Generate labels, which is this call.
	var r2: Dictionary = bridge.labels_generate({})
	for row in r2.get("classes", []):
		if String(row.get("key", "")) == "water":
			_p("water class after the dig: %s" % [row])
	var dug: Dictionary = bridge.sample_cell(spot.x, spot.y)
	_p("at the dig now: water=%s (civ copy, stale by design until Recompute)" % dug.get("water"))
	var lakes2 := _water_labels(bridge)
	var near_after := _near(lakes2, spot)
	_p("after the dig: water labels %d (was %d); within 12 cells of the dig: %s" % [lakes2.size(), lakes.size(), near_after])
	_check("P6 Generate labels names the lake the dig made (the map draws it from the live field)",
		not near_after.is_empty())
	_finish()


func _near(ls: Array, at: Vector2i) -> Array:
	var out: Array = []
	for lb: Dictionary in ls:
		if Vector2(float(lb["x"]), float(lb["y"])).distance_to(Vector2(at)) <= 12.0:
			out.append("%s@%.1f,%.1f" % [lb["text"], float(lb["x"]), float(lb["y"])])
	return out


func _finish() -> void:
	_p("---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)
