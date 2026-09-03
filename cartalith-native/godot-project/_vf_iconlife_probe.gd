extends Node
## VERIFIER-OWNED, throwaway. Answers three things the lanes' own probes do not:
##   1. the CLICK path (not the brush) round-trips family/glyph, cell and scale
##      through annotations/icons.json;
##   2. what happens to placed icons when the world is REPLACED (absorb), and
##      whether that matches the vault store / measurement tools;
##   3. whether the shell's brush row and the engine's brush desync across a
##      regenerate, and whether a brush drag after one silently does nothing.
## Plus the live layer signal, driven through the REAL app rather than a
## synthetic ViewportHost.

var app: Node
var _fails := 0


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _check(ok: bool, what: String) -> void:
	if ok:
		print("  ok: %s" % what)
	else:
		print("  FAIL: %s" % what)
		_fails += 1


func _gen(seed: int) -> void:
	app.bridge.generate({
		"seed": seed, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while app.bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(8)


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 420.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	var bridge = app.bridge

	await _gen(41207)
	var pack := ProjectSettings.globalize_path(
		"res://../crates/cartalith-assets/tests/fixtures/reference_pack.zip")
	bridge.load_asset_pack(pack)
	if not bridge.has_asset_pack():
		print("  FAIL: no asset pack")
		get_tree().quit(1)
		return

	var ws = null
	for w in app._workspaces:
		if w.get_script() != null and String(w.get_script().resource_path).ends_with("cartography_workspace.gd"):
			ws = w

	# ---- 1. CLICK path round trip, glyph included -------------------------
	print("--- 1. click-place -> save -> reopen ---")
	app.arm_tool("icon")
	await _frames(4)
	## Arm a non-default variant so a reader that silently defaults the glyph
	## is caught: variant 2 of `settlement` is not slot index 0.
	ws._icon_variant_idx = 2
	ws._icon_scale = 1.7
	ws._arm_icon_from_ui()
	var armed: Dictionary = bridge.icon_armed()
	print("  armed: %s" % armed)
	var idx: int = bridge.icon_place(60.0, 44.0)
	_check(idx >= 0, "click-placed one icon (index %d)" % idx)
	var before: Dictionary = bridge.icon_get(idx)
	print("  placed: %s" % before)
	_check(bridge.icon_list().size() == 1, "exactly one icon on the map")

	var path := ProjectSettings.globalize_path("user://_vf_iconlife.zip")
	_check(bool(bridge.save_project(path, app._project_documents())), "saved")
	bridge.icon_clear_all()
	_check(bridge.icon_list().is_empty(), "cleared in memory (teeth)")
	app._load_project(path)
	await _frames(8)
	_check(bridge.icon_list().size() == 1, "one icon came back (%d)" % bridge.icon_list().size())
	if bridge.icon_list().size() == 1:
		var back: Dictionary = bridge.icon_list()[0]
		print("  reloaded: %s" % back)
		_check(String(back.get("family", "")) == String(before.get("family", "?")),
			"same FAMILY (%s)" % back.get("family"))
		_check(String(back.get("slot", "")) == String(before.get("slot", "?")),
			"same SLOT/glyph (%s)" % back.get("slot"))
		_check(String(back.get("set", "")) == String(before.get("set", "")),
			"same asset SET (%s)" % back.get("set"))
		_check(is_equal_approx(float(back.get("x", -9.0)), float(before.get("x", -8.0)))
			and is_equal_approx(float(back.get("y", -9.0)), float(before.get("y", -8.0))),
			"same cell (%s,%s)" % [back.get("x"), back.get("y")])
		_check(is_equal_approx(float(back.get("scale", -9.0)), float(before.get("scale", -8.0))),
			"same scale (%s vs armed %s)" % [back.get("scale"), armed.get("scale")])

	# ---- 2. the anchoring question: world replaced ------------------------
	print("--- 2. icons across a world REPLACE (absorb) ---")
	## Put something in each of the three stores the question compares.
	bridge.icon_place(70.0, 50.0)
	var n_icons: int = bridge.icon_list().size()
	_check(n_icons >= 1, "%d icon(s) on the map before the regenerate" % n_icons)
	var n_meas := -1
	if bridge.has_method("measure_list"):
		n_meas = bridge.measure_list().size()
	print("  vault links before: %s" % [bridge.vault_link_count() if bridge.has_method("vault_link_count") else "n/a"])

	await _gen(90210)
	_check(bridge.icon_list().is_empty(),
		"icons are DROPPED by the regenerate (%d left)" % bridge.icon_list().size())
	## The ENGINE drops the armed slot with the editor; the SHELL re-pushes it
	## from its own tool-options state in `_on_world_changed()` (line 758-759),
	## so tool state survives a regenerate while placed icons do not.
	_check(not bridge.icon_armed().is_empty(),
		"the shell re-armed the tool over the new world: %s" % bridge.icon_armed())
	var b_after: Dictionary = bridge.icon_brush()
	print("  engine brush after regenerate: %s" % b_after)

	# ---- 3. shell/engine brush desync across the regenerate ---------------
	print("--- 3. brush row vs engine after a regenerate ---")
	ws._icon_brush_on = true
	ws._icon_brush_r = 40.0
	ws._icon_brush_density = 1.5
	ws._arm_icon_from_ui()
	var b_set: Dictionary = bridge.icon_brush()
	print("  after _arm_icon_from_ui: %s" % b_set)
	await _gen(1337)
	var b_reset: Dictionary = bridge.icon_brush()
	print("  shell still shows on=%s r=%s d=%s ; engine now %s"
		% [ws._icon_brush_on, ws._icon_brush_r, ws._icon_brush_density, b_reset])
	## `absorb()` builds a fresh `IconEditor` whose brush is `IconBrush::
	## default()`; the row would then show 40/1.5 over an engine holding
	## 12/0.6. `_on_world_changed()` -> `_arm_icon_from_ui()` re-pushes all
	## three, which is exactly what `_arm_icon_from_ui`'s own doc claims. This
	## asserts that claim rather than believing it.
	_check(bool(b_reset.get("on", false))
		and is_equal_approx(float(b_reset.get("radius", 0.0)), ws._icon_brush_r)
		and is_equal_approx(float(b_reset.get("density", 0.0)), ws._icon_brush_density),
		"the shell re-pushed the brush over the new world, so row and engine agree")
	## And it really paints over the new world, rather than looking armed and
	## doing nothing.
	var n0: int = bridge.icon_list().size()
	app._on_map_clicked(60.0, 44.0)
	await _frames(2)
	var n1: int = bridge.icon_list().size()
	_check(n1 > n0, "a press after the regenerate paints a stand (%d -> %d)" % [n0, n1])
	_check(ws._icon_brush_painting, "and the stroke is live")
	app._on_map_released(60.0, 44.0, true)
	await _frames(2)
	_check(not ws._icon_brush_painting, "release ends it")

	# ---- 4. live layer sync through the REAL app --------------------------
	print("--- 4. cross-panel set_layer_visible, no world change ---")
	var cb = ws._layer_checks.get("landmark_rejects", null)
	_check(cb != null, "found the real landmark_rejects checkbox")
	if cb != null:
		var start: bool = cb.button_pressed
		app.viewport.set_layer_visible("landmark_rejects", not start)
		await _frames(2)
		_check(cb.button_pressed == (not start),
			"checkbox followed a third-party write with NO world change (%s -> %s)" % [start, cb.button_pressed])
		## and the read-back must not write back into the engine
		var eng: bool = app.viewport.layer_visible("landmark_rejects")
		_check(eng == (not start), "and the engine still holds what the third party wrote (%s)" % eng)

	print("\n%s (%d failure%s)" % ["ALL PASS" if _fails == 0 else "FAILURES", _fails,
		"" if _fails == 1 else "s"])
	get_tree().quit(1 if _fails > 0 else 0)
