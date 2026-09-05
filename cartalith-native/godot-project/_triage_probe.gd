extends Node
## Throwaway triage: seed 77021 emits `Invalid polygon data, triangulation
## failed` six times per pass in `_setxvert_probe`, and the question is whether
## the 84 px plan thumbnail is what produces them or whether the same layout
## does it at every size. Draws one town's layout at the City Viewer's own
## parameters and at the thumbnail's, with a marker between, and lets the
## engine's own stderr line up against them.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _triage_probe.tscn

const DRAW := preload("res://shell/urban_layout_draw.gd")

var app: Node
var _layout: Dictionary = {}
var _mode := ""

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	app.bridge.generate({"seed": 77021, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var waited := 0
	while app.bridge.generating and waited < 4000:
		await get_tree().process_frame
		waited += 1
	await _frames(12)
	print("TRI generated, has_world=%s" % app.bridge.has_world)

	var got: Array = app.bridge.urban_layouts(PackedInt32Array([0]))
	if got.is_empty():
		print("TRI no layout")
		get_tree().quit(1)
		return
	_layout = got[0]
	print("TRI layout: blocks=%d parcels=%d buildings=%d markets=%d farmland=%d" % [
		(_layout.get("blocks", []) as Array).size(),
		(_layout.get("parcels", []) as Array).size(),
		(_layout.get("buildings", []) as Array).size(),
		(_layout.get("markets", []) as Array).size(),
		(_layout.get("farmland", []) as Array).size()])

	var host := Control.new()
	host.size = Vector2(900, 900)
	add_child(host)
	var c := Control.new()
	c.size = Vector2(400, 400)
	host.add_child(c)
	c.draw.connect(func(): _paint(c))

	for m in ["cityviewer-400px-detail1", "thumb-84px-ss4-detail0", "thumb-84px-ss1-detail0"]:
		_mode = m
		print("TRI ---- BEGIN %s ----" % m)
		c.queue_redraw()
		await RenderingServer.frame_post_draw
		await _frames(3)
		print("TRI ---- END   %s ----" % m)

	## Does the supersample buy anything a viewer can see? Both variants drawn
	## at once, side by side, and the framebuffer read back over each 84 px box.
	c.queue_free()
	var a := Control.new()
	a.position = Vector2(20, 20)
	a.size = Vector2(84, 84)
	host.add_child(a)
	a.draw.connect(func(): _paint_thumb(a, 4.0))
	var b := Control.new()
	b.position = Vector2(140, 20)
	b.size = Vector2(84, 84)
	host.add_child(b)
	b.draw.connect(func(): _paint_thumb(b, 1.0))
	await _frames(4)
	await RenderingServer.frame_post_draw
	var img: Image = get_viewport().get_texture().get_image()
	for pair in [["ss=4", a], ["ss=1", b]]:
		var ctl: Control = pair[1]
		var r := Rect2i(ctl.get_global_rect())
		var seen: Dictionary = {}
		var warm := 0
		for y in range(maxi(0, r.position.y), mini(img.get_height(), r.end.y)):
			for x in range(maxi(0, r.position.x), mini(img.get_width(), r.end.x)):
				var px := img.get_pixel(x, y)
				seen[px.to_rgba32()] = true
				if px.r - px.b > 0.08:
					warm += 1
		print("TRI %s: %d distinct colours, %d warm px, rect=%s"
			% [pair[0], seen.size(), warm, r])

	get_tree().quit(0)

func _paint_thumb(c: Control, ss: float) -> void:
	var box := _fit(_layout)
	var size := Vector2(84, 84) * ss
	if ss != 1.0:
		c.draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE / ss)
	c.draw_rect(Rect2(Vector2.ZERO, size), DRAW.GROUND)
	var fit: float = minf(size.x / maxf(1.0, box.size.x), size.y / maxf(1.0, box.size.y))
	var origin := (size - box.size * fit) * 0.5 - box.position * fit
	DRAW.draw_layout(c, _layout, func(p): return origin + p * fit, fit, ss, 1.0, false, 0.0)

func _paint(c: Control) -> void:
	var box := _fit(_layout)
	match _mode:
		"cityviewer-400px-detail1":
			var size := Vector2(400, 400)
			var margin := 16.0
			var fit: float = minf((size.x - 2 * margin) / maxf(1.0, box.size.x),
				(size.y - 2 * margin) / maxf(1.0, box.size.y))
			var origin := (size - box.size * fit) * 0.5 - box.position * fit
			DRAW.draw_layout(c, _layout, func(p): return origin + p * fit, fit, 1.0, 1.0, true, 1.0)
		"thumb-84px-ss4-detail0":
			var ss := 4.0
			var size := Vector2(84, 84) * ss
			c.draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE / ss)
			var fit: float = minf(size.x / maxf(1.0, box.size.x), size.y / maxf(1.0, box.size.y))
			var origin := (size - box.size * fit) * 0.5 - box.position * fit
			DRAW.draw_layout(c, _layout, func(p): return origin + p * fit, fit, ss, 1.0, false, 0.0)
		"thumb-84px-ss1-detail0":
			var size := Vector2(84, 84)
			var fit: float = minf(size.x / maxf(1.0, box.size.x), size.y / maxf(1.0, box.size.y))
			var origin := (size - box.size * fit) * 0.5 - box.position * fit
			DRAW.draw_layout(c, _layout, func(p): return origin + p * fit, fit, 1.0, 1.0, false, 0.0)

func _fit(layout: Dictionary) -> Rect2:
	var lo := Vector2(INF, INF)
	var hi := Vector2(-INF, -INF)
	for p in (layout.get("wall_ring", PackedVector2Array()) as PackedVector2Array):
		lo = lo.min(p)
		hi = hi.max(p)
	if not (hi.x > lo.x and hi.y > lo.y):
		for b: PackedVector2Array in layout.get("buildings", []) as Array:
			for p in b:
				lo = lo.min(p)
				hi = hi.max(p)
	if not (hi.x > lo.x and hi.y > lo.y):
		return Rect2(Vector2.ZERO, Vector2(float(layout.get("wm", 1700.0)),
			float(layout.get("hm", 1250.0))))
	var pad := (hi - lo) * 0.08
	return Rect2(lo - pad, (hi - lo) + pad * 2.0)
