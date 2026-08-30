extends Control
## Temporary, untracked verification harness for milestone 12's blocks/parcels
## and the City Viewer's new rendering.
##
## Run:
##   Godot_v4.7.1-stable_win64_console.exe --path . _roofshot.tscn
##
## Two things have to be true and neither is provable by a boot check:
##   1. the engine really produces blocks and parcels for real settlements on
##      a real world (not just for the golden fixtures' synthetic grids);
##   2. the drawing reads as a hand-illustrated town AND genuinely varies —
##      between roofs within a town, and between towns.
## It also times a redraw, because the City Viewer pans and zooms and the roof
## passes are per-parcel.
##
## Each town gets its own clipped child Control, because a town's approach
## roads run to its 1.7 km box edge and would otherwise scribble across every
## neighbouring panel.

const DRAW := preload("res://shell/urban_layout_draw.gd")
const SEED := 483920

var _shots: Array = []
var _cell := Vector2(560, 400)
var _cols := 3


func _ready() -> void:
	var gen := WorldGen.new()
	gen.generate_sized(SEED, 1200.0, 384, 288)
	var places := gen.get_settlements()
	print("ROOF world %dx%d, %d settlements" % [gen.get_width(), gen.get_height(), places.size()])

	## Six settlements spread across the population range, so the sample is not
	## six villages -- a town's parcel count is driven by `pop_target`.
	var ranked: Array = []
	for i in places.size():
		ranked.append([float(places[i].get("population", 0)), i])
	ranked.sort()
	ranked.reverse()
	var picks := PackedInt32Array()
	var n := ranked.size()
	for f in [0.0, 0.08, 0.2, 0.4, 0.65, 0.9]:
		var i: int = ranked[mini(n - 1, int(f * n))][1]
		if not picks.has(i):
			picks.append(i)

	var t0 := Time.get_ticks_usec()
	var layouts := gen.urban_layouts(picks)
	print("ROOF generated %d layouts in %.1f ms" % [layouts.size(),
		(Time.get_ticks_usec() - t0) / 1000.0])

	var firsts := {}
	for l in layouts:
		var d := l as Dictionary
		var s: Dictionary = places[int(d.get("index", -1))]
		var tones: PackedFloat32Array = d.get("parcel_tone", PackedFloat32Array())
		var lo := 2.0
		var hi := -1.0
		for t in tones:
			lo = minf(lo, t)
			hi = maxf(hi, t)
		print("ROOF   %-20s pop=%-6d blocks=%-4d parcels=%-5d tone=[%.3f..%.3f] site=%s" % [
			String(s.get("name", "?")), int(s.get("population", 0)),
			(d.get("blocks", []) as Array).size(),
			(d.get("parcels", []) as Array).size(), lo, hi,
			String(d.get("site_kind", "?"))])
		firsts[tones[0] if tones.size() > 0 else -1.0] = true
		_shots.append({"layout": d, "name": String(s.get("name", "?")),
			"pop": int(s.get("population", 0))})
	print("ROOF distinct first-tone across towns: %d of %d" % [firsts.size(), _shots.size()])

	size = Vector2(_cell.x * _cols, _cell.y * ceili(float(_shots.size()) / _cols))
	for i in _shots.size():
		var panel := Control.new()
		panel.position = Vector2((i % _cols) * _cell.x, (i / _cols) * _cell.y)
		panel.size = _cell
		panel.clip_contents = true
		panel.set_meta("shot", i)
		panel.draw.connect(_draw_panel.bind(panel))
		add_child(panel)

	await RenderingServer.frame_post_draw
	await get_tree().process_frame
	await RenderingServer.frame_post_draw

	## Redraw cost over the whole sheet (six towns at once, strictly worse than
	## the City Viewer's one).
	var t1 := Time.get_ticks_usec()
	for c in get_children():
		(c as Control).queue_redraw()
	await RenderingServer.frame_post_draw
	print("ROOF redraw (6 towns) %.1f ms" % ((Time.get_ticks_usec() - t1) / 1000.0))

	var img := get_viewport().get_texture().get_image()
	img.save_png("res://_roofshot.png")
	print("ROOF saved _roofshot.png %dx%d" % [img.get_width(), img.get_height()])
	get_tree().quit()


func _draw() -> void:
	draw_rect(Rect2(Vector2.ZERO, size), DRAW.GROUND)


func _draw_panel(panel: Control) -> void:
	var sh: Dictionary = _shots[int(panel.get_meta("shot"))]
	var layout: Dictionary = sh["layout"]
	panel.draw_rect(Rect2(Vector2.ZERO, _cell), DRAW.GROUND)

	## The City Viewer's own fit: blocks, else streets, else the box.
	var lo := Vector2(INF, INF)
	var hi := Vector2(-INF, -INF)
	for blk: PackedVector2Array in layout.get("blocks", []) as Array:
		for p in blk:
			lo = lo.min(p)
			hi = hi.max(p)
	if not (hi.x > lo.x and hi.y > lo.y):
		var st: Dictionary = layout.get("streets", {})
		for cls in st.keys():
			for p in (st[cls] as PackedVector2Array):
				lo = lo.min(p)
				hi = hi.max(p)
	if not (hi.x > lo.x and hi.y > lo.y):
		return
	var pad := (hi - lo) * 0.08
	var box := Rect2(lo - pad, (hi - lo) + pad * 2.0)
	var margin := 26.0
	var fit: float = minf((_cell.x - 2 * margin) / maxf(1.0, box.size.x),
		(_cell.y - 2 * margin) / maxf(1.0, box.size.y))
	var origin := (_cell - box.size * fit) * 0.5 - box.position * fit
	var to_screen := func(p: Vector2) -> Vector2: return origin + p * fit
	DRAW.draw_layout(panel, layout, to_screen, fit, 1.0, 1.0, true, 1.0)
	panel.draw_string(ThemeDB.fallback_font, Vector2(14, 24),
		"%s  (pop %d, %d lots)" % [sh["name"], sh["pop"],
		(layout.get("parcels", []) as Array).size()],
		HORIZONTAL_ALIGNMENT_LEFT, -1, 15, Color(0.169, 0.129, 0.094))
	panel.draw_rect(Rect2(Vector2.ZERO, _cell), Color(0, 0, 0, 0.22), false, 1.0)
