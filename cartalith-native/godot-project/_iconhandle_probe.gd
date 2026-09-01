extends Node
## Committed verification harness for the Icon tool's on-canvas resize
## handle (`GUI_GAP_REGISTER.md` CA-05). Drives the *real shell* through
## `app.arm_tool`/`app._on_map_clicked`/`_on_map_dragged`/`_on_map_released`,
## the same registry the viewport's own pointer handler uses --
## `_authoring_shot.gd`'s own pattern for the sibling four authoring tools.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _iconhandle_probe.tscn -- --nowelcome
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

var app: Node


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _shot(name: String) -> void:
	await _frames(3)
	var img := get_viewport().get_texture().get_image()
	var out := "user://iconhandle_%s.png" % name
	img.save_png(out)
	print("ICONHANDLE saved ", ProjectSettings.globalize_path(out))


func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 900:
		await get_tree().process_frame
		waited += 1
	print("ICONHANDLE world generated: has_world=%s (%d frames)" % [app.bridge.has_world, waited])
	await _frames(8)
	var gw: int = app.bridge.world_gen.get_width()
	var gh: int = app.bridge.world_gen.get_height()
	print("ICONHANDLE grid = %dx%d" % [gw, gh])

	var pack := ProjectSettings.globalize_path("res://../crates/cartalith-assets/tests/fixtures/reference_pack.zip")
	var ok: bool = app.bridge.load_asset_pack(pack)
	print("ICONHANDLE load_asset_pack -> %s, has_asset_pack=%s" % [ok, app.bridge.has_asset_pack()])
	if not app.bridge.has_asset_pack():
		print("ICONHANDLE  !! no pack -- cannot arm the Icon tool")
		get_tree().quit(1)
		return

	app.arm_tool("icon")
	await _frames(4)
	print("ICONHANDLE armed_tool=%s icon_armed=%s" % [app.armed_tool, app.bridge.icon_armed()])

	# --- Place one icon -------------------------------------------------------
	var gx := gw * 0.5
	var gy := gh * 0.5
	app._on_map_clicked(gx, gy)
	await _frames(4)
	var idx: int = app.bridge.icon_get_selected()
	print("ICONHANDLE placed+selected idx=%d icon_list=%d" % [idx, app.bridge.icon_list().size()])
	if idx < 0:
		print("ICONHANDLE  !! placement failed")
		get_tree().quit(1)
		return
	var before: Dictionary = app.bridge.icon_get(idx)
	print("ICONHANDLE icon[%d] before = %s" % [idx, before])
	await _shot("00_placed")

	# --- The handle exists, at the selected icon's own box ---------------------
	var zoom: float = app.viewport.zoom()
	var h: Dictionary = app.bridge.icon_handles(idx, zoom).get("resize", {})
	print("ICONHANDLE handle (zoom=%s) = %s" % [zoom, h])
	if h.is_empty():
		print("ICONHANDLE  !! no resize handle returned")
		get_tree().quit(1)
		return

	# --- Drag the handle outward -- pointerdown on it, then move, then release -
	var hx: float = h["x"]
	var hy: float = h["y"]
	app._on_map_clicked(hx, hy)   ## CartographyWorkspace._on_icon_click: handle-hit precedence
	await _frames(2)
	var steps := 6
	var far := Vector2(hx, hy) + Vector2(hx - gx, hy - gy) * 2.0   ## push it further out along the same ray
	for i in range(1, steps + 1):
		var p: Vector2 = Vector2(hx, hy).lerp(far, float(i) / steps)
		app._on_map_dragged(p.x, p.y)
		await _frames(1)
	app._on_map_released(far.x, far.y, true)
	await _frames(4)

	var after: Dictionary = app.bridge.icon_get(idx)
	print("ICONHANDLE icon[%d] after drag = %s" % [idx, after])
	print("ICONHANDLE scale %s -> %s (grew=%s)" %
		[before.get("scale", 0.0), after.get("scale", 0.0), float(after.get("scale", 0.0)) > float(before.get("scale", 0.0))])
	await _shot("01_resized")

	# --- Persists through a zoom step + a redraw --------------------------------
	app.viewport.zoom_step(1.35)
	await _frames(3)
	app.viewport.refresh_annotations()
	await _frames(3)
	var after_zoom: Dictionary = app.bridge.icon_get(idx)
	print("ICONHANDLE scale after zoom+redraw = %s (unchanged=%s)" %
		[after_zoom.get("scale", 0.0), is_equal_approx(float(after_zoom.get("scale", 0.0)), float(after.get("scale", 0.0)))])
	var h2: Dictionary = app.bridge.icon_handles(idx, app.viewport.zoom()).get("resize", {})
	print("ICONHANDLE handle after zoom = %s (still present=%s)" % [h2, not h2.is_empty()])
	await _shot("02_after_zoom")

	print("ICONHANDLE done")
	get_tree().quit()
