extends Node
## Committed probe -- `GUI_GAP_REGISTER.md` SH-11.
##
## The contract `_zoom_at()`'s own doc comment states: *"Zooms so the world
## point under `screen_pt` stays under it."* Measure whether it does.
##
## A wheel notch at screen point P must leave the camera-LOCAL point that was
## under P still under P. Drift is reported in camera-local pixels and, at the
## current zoom, in grid cells.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _zoompivot_probe.tscn
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

var _app: Node
var _bridge


func _p(s: String) -> void:
	print("ZOOMPIVOT  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _local_under(vh: Control, screen_pt: Vector2) -> Vector2:
	var cam: Control = vh._camera
	return (screen_pt - vh.global_position - cam.position) / vh._zoom


func _wheel(vh: Control, at: Vector2, up: bool) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_WHEEL_UP if up else MOUSE_BUTTON_WHEEL_DOWN
	ev.pressed = true
	ev.position = at
	vh._input(ev)


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 200.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	_bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)

	var vh: Control = _app.viewport
	_p("ViewportHost global_position=%s size=%s  zoom=%.4f" % [
		str(vh.global_position), str(vh.size), vh._zoom])

	## Three probe points inside the map rect: centre, and two off-centre.
	var pts := [
		vh.global_position + vh.size * 0.5,
		vh.global_position + Vector2(vh.size.x * 0.25, vh.size.y * 0.30),
		vh.global_position + Vector2(vh.size.x * 0.80, vh.size.y * 0.70),
	]
	for p in pts:
		var before := _local_under(vh, p)
		var z0: float = vh._zoom
		_wheel(vh, p, true)
		await _frames(2)
		var after := _local_under(vh, p)
		var d: Vector2 = after - before
		_p("wheel-up at screen %s : zoom %.4f -> %.4f  local pivot drift = (%.2f, %.2f) px  |d|=%.2f" % [
			str(p), z0, vh._zoom, d.x, d.y, d.length()])
		## put the zoom back
		_wheel(vh, p, false)
		await _frames(2)

	## And the navpad's own button path, which passes a LOCAL point already.
	var before2 := _local_under(vh, vh.global_position + vh.size * 0.5)
	vh.zoom_step(1.35)
	await _frames(2)
	var after2 := _local_under(vh, vh.global_position + vh.size * 0.5)
	_p("zoom_step(1.35) centre drift = (%.2f, %.2f) px  |d|=%.2f" % [
		(after2 - before2).x, (after2 - before2).y, (after2 - before2).length()])

	_p("DONE")
	get_tree().quit(0)
