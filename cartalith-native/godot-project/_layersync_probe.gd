extends SceneTree
## Committed verification harness for A2 (`GUI_GAP_REGISTER.md`-shaped row,
## "the CARTO Layers checkboxes hold no runtime state").
##
##   Godot_v4.7.1-stable_win64.exe --headless --path . --script _layersync_probe.gd
##
## Pure logic, no rendering and no generated world needed -- `layer_visible()`
## on both `MapOverlay` and `ViewportHost` is a synchronous dictionary/field
## read, so this probes it directly rather than booting `app.tscn`.
##
## Section 1 proves the six `MapOverlay` cases round-trip through the new
## getter. Section 2 proves `ViewportHost.layer_visible()`'s own split (two
## arms answer from a node's `.visible`, six delegate to `overlay`) for all
## eight ids `set_layer_visible()` accepts, including the two political ones
## `cartography_workspace.gd`'s second loop drives. Section 3 is the actual
## regression this row exists to close: flip a layer through
## `set_layer_visible()` -- exactly what `civilization_workspace.gd`'s
## landmark-funnel "Show rejected" chip does -- and confirm the read-back
## sees it, with no call into the checkbox at all (the checkbox side is
## `_layer_checks`/`_sync_layers()` in `cartography_workspace.gd`, which
## needs `DccApp`/`EngineBridge` to build and is out of this probe's reach --
## see that file's own doc comment on `_sync_layers()` for the mechanism this
## proves the engine half of).

var fails := 0

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

func _initialize() -> void:
	print("== 1. MapOverlay.layer_visible() mirrors each set_* setter ==")
	var overlay: Control = load("res://map_overlay.gd").new()
	# Defaults must match LIVE_LAYERS' own `on` field
	# (`cartography_workspace.gd`) -- a fresh overlay and an untouched
	# checkbox must already agree before either is ever touched.
	_ok(overlay.layer_visible("settlements") == true, "settlements defaults on")
	_ok(overlay.layer_visible("roads") == true, "roads defaults on")
	_ok(overlay.layer_visible("sea_routes") == true, "sea_routes defaults on")
	_ok(overlay.layer_visible("landmarks") == true, "landmarks defaults on")
	_ok(overlay.layer_visible("landmark_rejects") == false, "landmark_rejects defaults OFF")
	_ok(overlay.layer_visible("urban_layouts") == true, "urban_layouts defaults on")

	overlay.set_show_settlements(false)
	_ok(overlay.layer_visible("settlements") == false, "settlements off after set_show_settlements(false)")
	overlay.set_show_roads(false)
	_ok(overlay.layer_visible("roads") == false, "roads off after set_show_roads(false)")
	overlay.set_show_sea_routes(false)
	_ok(overlay.layer_visible("sea_routes") == false, "sea_routes off after set_show_sea_routes(false)")
	overlay.set_landmarks_visible(false)
	_ok(overlay.layer_visible("landmarks") == false, "landmarks off after set_landmarks_visible(false)")
	overlay.set_landmark_rejects_visible(true)
	_ok(overlay.layer_visible("landmark_rejects") == true, "landmark_rejects ON after set_landmark_rejects_visible(true)")
	overlay.set_show_urban_layouts(false)
	_ok(overlay.layer_visible("urban_layouts") == false, "urban_layouts off after set_show_urban_layouts(false)")

	print("\n== 2. ViewportHost.layer_visible() -- all eight set_layer_visible() ids ==")
	var vp: Control = load("res://shell/viewport_host.gd").new()
	# `territory_view`/`province_view`/`overlay` are plain (non-@onready)
	# fields on ViewportHost -- app.gd wires them from its own scene tree;
	# this probe wires the same shape by hand so no .tscn is needed.
	vp.overlay = load("res://map_overlay.gd").new()
	vp.territory_view = TextureRect.new()
	vp.province_view = TextureRect.new()
	for id in ["settlements", "roads", "sea_routes", "landmarks", "landmark_rejects", "urban_layouts", "territory", "provinces"]:
		vp.set_layer_visible(id, true)
		_ok(vp.layer_visible(id) == true, "%s: true round-trips" % id)
		vp.set_layer_visible(id, false)
		_ok(vp.layer_visible(id) == false, "%s: false round-trips" % id)

	print("\n== 3. THE regression: a second writer, then read-back sees it ==")
	## This is `civilization_workspace.gd::_lm_show_rejects()`'s own call,
	## verbatim -- the landmark funnel's "Show rejected" chip. Before this
	## row, nothing in CARTO ever asked `layer_visible()` at all, so this
	## write was invisible to the Layers checkbox forever, not just until
	## the next world change.
	vp.set_layer_visible("landmark_rejects", true)
	_ok(vp.layer_visible("landmark_rejects") == true,
		"landmark_rejects reads back true after the funnel's own set_layer_visible() call")

	print("\n%s (%d failure%s)" % ["ALL PASS" if fails == 0 else "FAILURES", fails, "" if fails == 1 else "s"])
	quit(1 if fails > 0 else 0)
