extends SceneTree

# Adversarial verification of Lane A2. Unlike _layersync_probe.gd, this drives
# the REAL CartographyWorkspace._sync_layers() over REAL CheckBoxes built by
# the REAL DccWidgets.toggle() wiring, so the claim under test -- "the checkbox
# follows the engine" -- is actually asserted, not asserted-by-proxy.

var fails := 0
var writebacks: Array = []

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
	# viewport_host.gd::_build() lines 366-367 set both false at construction;
	# a raw TextureRect defaults visible=true, so match the real node here.
	vp.territory_view.visible = false
	vp.province_view.visible = false
	app.viewport = vp
	ws.app = app

	# Build the eight checkboxes EXACTLY as cartography_workspace.gd's two
	# loops do -- same DccWidgets.toggle call, same seed, same callback -- and
	# record every write-back the callback would make.
	var host := VBoxContainer.new()
	var ids: Array = []
	for layer in ws.LIVE_LAYERS:
		var id := String(layer.id)
		ids.append(id)
		ws._layer_checks[id] = DccWidgets.toggle(host, String(layer.label),
			vp.layer_visible(id),
			func(on: bool): writebacks.append([id, on]); vp.set_layer_visible(id, on))

	print("== 0. build-time seed matches the engine, and does not write back ==")
	_ok(writebacks.is_empty(), "constructing 8 toggles fired 0 callbacks")
	for layer in ws.LIVE_LAYERS:
		var id := String(layer.id)
		var cb: CheckBox = ws._layer_checks[id]
		_ok(cb.button_pressed == vp.layer_visible(id),
			"%s: checkbox %s == engine %s at build" % [id, cb.button_pressed, vp.layer_visible(id)])
		# and the const default it replaced still agrees, so no silent flip
		_ok(cb.button_pressed == bool(layer.on),
			"%s: build seed still equals LIVE_LAYERS.on (%s)" % [id, layer.on])

	print("\n== 1. engine flipped by a THIRD PARTY, then _sync_layers() ==")
	# invert every layer through set_layer_visible(), the way
	# civilization_workspace.gd's _lm_show_rejects() does.
	var want := {}
	for id in ids:
		var v: bool = not vp.layer_visible(id)
		want[id] = v
		vp.set_layer_visible(id, v)
	# before sync: every checkbox must be STALE (proves the test has teeth)
	var stale := 0
	for id in ids:
		if (ws._layer_checks[id] as CheckBox).button_pressed != want[id]:
			stale += 1
	_ok(stale == 8, "before sync all 8 checkboxes are stale (teeth check): %d/8" % stale)

	writebacks.clear()
	ws._sync_layers()
	for id in ids:
		var cb: CheckBox = ws._layer_checks[id]
		_ok(cb.button_pressed == want[id],
			"%s: checkbox follows engine after _sync_layers() (%s)" % [id, want[id]])
	_ok(writebacks.is_empty(),
		"_sync_layers() fired 0 handler write-backs (set_pressed_no_signal holds); got %s" % [writebacks])

	print("\n== 2. invert back, sync again (both directions) ==")
	for id in ids:
		vp.set_layer_visible(id, not want[id])
	writebacks.clear()
	ws._sync_layers()
	for id in ids:
		_ok((ws._layer_checks[id] as CheckBox).button_pressed == (not want[id]),
			"%s: follows engine back" % id)
	_ok(writebacks.is_empty(), "second _sync_layers() fired 0 write-backs")

	print("\n== 3. _sync_layers() is reached from _on_world_changed ==")
	var src := FileAccess.get_file_as_string("res://shell/workspaces/cartography_workspace.gd")
	_ok(src.contains("\t_sync_layers()"), "_on_world_changed body calls _sync_layers()")

	print("\n%s (%d failure%s)" % ["ALL PASS" if fails == 0 else "FAILURES", fails, "" if fails == 1 else "s"])
	quit(1 if fails > 0 else 0)
