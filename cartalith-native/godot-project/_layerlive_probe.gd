extends SceneTree
## Verifies the "Lane B" sweep's row 3 (2026-09-03): a layer flipped from
## OUTSIDE CartographyWorkspace -- civilization_workspace.gd's landmark-funnel
## "Show rejected" chip is the one real caller today -- now reaches the CARTO
## checkbox LIVE, with no explicit `_sync_layers()` call and no world change
## in between. Before this row, `ViewportHost.set_layer_visible()` emitted no
## signal at all, so the only thing that ever called `_sync_layers()` was
## `_on_world_changed()` -- `_verify_layers_probe.gd` (Lane A2) already proves
## `_sync_layers()` itself is correct when called; what was missing is
## something calling it on a live third-party write, which is what this adds.
##
##   Godot_v4.7.1-stable_win64.exe --headless --path . --script _layerlive_probe.gd

var fails := 0

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

func _initialize() -> void:
	var ws: Node = load("res://shell/workspaces/cartography_workspace.gd").new()
	var app: Node = load("res://shell/app.gd").new()
	var vp: Control = load("res://shell/viewport_host.gd").new()
	vp.overlay = load("res://map_overlay.gd").new()
	vp.territory_view = TextureRect.new()
	vp.province_view = TextureRect.new()
	# viewport_host.gd::_build() sets both false at construction; match it.
	vp.territory_view.visible = false
	vp.province_view.visible = false
	app.viewport = vp
	ws.app = app

	# Reproduce _build()'s own checkbox construction AND its new signal
	# connection -- the two lines this row actually touches -- without running
	# the full _build() (which needs `bridge`/`categories`/the tool group this
	# probe does not construct; `_verify_layers_probe.gd` established this same
	# reduced-setup pattern for the A2 regression).
	var host := VBoxContainer.new()
	for layer in ws.LIVE_LAYERS:
		var id := String(layer.id)
		ws._layer_checks[id] = DccWidgets.toggle(host, String(layer.label),
			vp.layer_visible(id),
			func(on: bool): vp.set_layer_visible(id, on))
	vp.layer_visibility_changed.connect(func(_layer: String, _shown: bool): ws._sync_layers())

	print("== the regression: a THIRD PARTY write, no _sync_layers() call, no world change ==")
	# Exactly civilization_workspace.gd::_lm_show_rejects()'s own call.
	var before: bool = (ws._layer_checks["landmark_rejects"] as CheckBox).button_pressed
	_ok(before == false, "landmark_rejects checkbox starts unpressed (teeth check)")
	vp.set_layer_visible("landmark_rejects", true)
	var after: bool = (ws._layer_checks["landmark_rejects"] as CheckBox).button_pressed
	_ok(after == true,
		"landmark_rejects checkbox follows the funnel's set_layer_visible() with NO explicit sync call")

	print("\n== an unknown layer id still does not emit (no false positive) ==")
	var sea_before: bool = (ws._layer_checks["sea_routes"] as CheckBox).button_pressed
	vp.set_layer_visible("not_a_real_layer", true)
	var sea_after: bool = (ws._layer_checks["sea_routes"] as CheckBox).button_pressed
	_ok(sea_before == sea_after, "an unknown-layer call touches no checkbox")

	print("\n%s (%d failure%s)" % ["ALL PASS" if fails == 0 else "FAILURES", fails, "" if fails == 1 else "s"])
	quit(1 if fails > 0 else 0)
