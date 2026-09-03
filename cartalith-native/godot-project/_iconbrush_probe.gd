extends Node
## Committed verification harness for the Icon tool's **density brush** --
## `UNIFIED_TOOL_PLAN.md` milestone E's last open half (`icon_bridge/brush.rs`,
## `cartography_workspace.gd::_build_icon_brush_controls`).
##
## Reaches what no Rust test can. `cargo test` proves `IconEditor::brush_stamp`
## and `set_brush` against their own contracts; what it cannot touch is the
## `#[func]` layer, because `WorldGen` is a cdylib `GodotClass` -- so
## `icon_brush_set`/`icon_brush`/`icon_brush_stamp` and the shell's own
## click/drag/release routing are only ever exercised here.
##
## It also re-establishes milestone E's OTHER two halves at their symbols
## rather than trusting a doc comment, which is why the persistence section
## exists: `LARGE_ITEM_RULINGS.md` scheduled this row as three gaps, and two of
## them turned out to have shipped already. That claim is worth an assertion.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _iconbrush_probe.tscn
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row.

var app: Node
var _fails := 0

## The sea level this probe generates with, below. `sample_cell()` reports raw
## `elevation` in the same units the brush's own gate compares against.
const SEA := 0.45


## `-1.0` for an out-of-grid cell, which reads as water and so can only make
## an assertion below stricter, never falsely satisfy one.
func _elev(x: int, y: int) -> float:
	return float(app.bridge.sample_cell(x, y).get("elevation", -1.0))


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _check(ok: bool, what: String) -> void:
	if ok:
		print("  ok: %s" % what)
	else:
		print("  FAIL: %s" % what)
		_fails += 1


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
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
	bridge.generate({
		"seed": 41207, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(8)
	var gw: int = bridge.world_gen.get_width()
	var gh: int = bridge.world_gen.get_height()
	print("--- world %dx%d ---" % [gw, gh])

	var pack := ProjectSettings.globalize_path(
		"res://../crates/cartalith-assets/tests/fixtures/reference_pack.zip")
	bridge.load_asset_pack(pack)
	if not bridge.has_asset_pack():
		print("  FAIL: no asset pack -- cannot arm the Icon tool")
		get_tree().quit(1)
		return

	# --- 1. the three bindings exist and round-trip -------------------------
	print("--- icon_brush_set / icon_brush ---")
	var b0: Dictionary = bridge.icon_brush()
	_check(not b0.is_empty(), "icon_brush() answers over a live world: %s" % b0)
	_check(not bool(b0.get("on", true)), "a fresh editor's brush is OFF")
	_check(is_equal_approx(float(b0.get("radius", 0.0)), 12.0)
		and is_equal_approx(float(b0.get("density", 0.0)), 0.6),
		"and carries IconBrush::default() -- the reference's own slider values")

	_check(bridge.icon_brush_set(true, 9.0, 0.9), "icon_brush_set accepted")
	var b1: Dictionary = bridge.icon_brush()
	_check(is_equal_approx(float(b1.get("radius", 0.0)), 9.0)
		and is_equal_approx(float(b1.get("density", 0.0)), 0.9),
		"and the engine read it back: %s" % b1)
	bridge.icon_brush_set(true, 9999.0, 9999.0)
	var b2: Dictionary = bridge.icon_brush()
	_check(is_equal_approx(float(b2.get("radius", 0.0)), 60.0)
		and is_equal_approx(float(b2.get("density", 0.0)), 2.0),
		"out-of-range values clamp to the reference slider's own ends: %s" % b2)
	bridge.icon_brush_set(true, 14.0, 0.9)

	# --- 2. a stamp with nothing armed places nothing -----------------------
	print("--- gates ---")
	bridge.icon_disarm()
	_check(bridge.icon_brush_stamp(gw * 0.5, gh * 0.5) == 0,
		"a stamp with nothing armed places nothing")
	_check(bridge.icon_list().is_empty(), "and left the map empty")

	app.arm_tool("icon")
	await _frames(4)
	_check(not bridge.icon_armed().is_empty(),
		"arming the Icon tool armed a family/slot: %s" % bridge.icon_armed())

	bridge.icon_brush_set(false, 14.0, 0.9)
	_check(bridge.icon_brush_stamp(gw * 0.5, gh * 0.5) == 0,
		"a stamp with the brush switched OFF places nothing")
	bridge.icon_brush_set(true, 14.0, 0.9)

	# --- 3. a real stamp, on land -------------------------------------------
	print("--- one stamp ---")
	## Find a land cell to paint on: the brush never paints into water, so a
	## stamp aimed at open sea legitimately places nothing and would make this
	## section pass for the wrong reason.
	var land := Vector2(-1, -1)
	for y in range(20, gh - 20, 7):
		for x in range(20, gw - 20, 7):
			if _elev(x, y) > SEA:
				land = Vector2(x, y)
				break
		if land.x >= 0:
			break
	_check(land.x >= 0, "found a land cell to paint on: %s" % land)
	var placed: int = bridge.icon_brush_stamp(land.x, land.y)
	print("  one stamp at %s placed %d icons" % [land, placed])
	_check(placed > 1, "one stamp paints a STAND, not a single icon (%d)" % placed)
	_check(bridge.icon_list().size() == placed, "and every one of them is on the map")
	var slot := String(bridge.icon_armed().get("slot", ""))
	var all_armed_slot := true
	var all_on_land := true
	for e in bridge.icon_list():
		var d: Dictionary = e
		if String(d.get("slot", "")) != slot:
			all_armed_slot = false
		if _elev(int(d.get("x", -1)), int(d.get("y", -1))) <= SEA:
			all_on_land = false
	_check(all_armed_slot, "every painted icon carries the armed slot (%s)" % slot)
	_check(all_on_land, "and every one is on land -- the brush's own gate, which the click path does not have")

	# --- 4. selection is untouched ------------------------------------------
	_check(bridge.icon_get_selected() < 0,
		"painting does not select, unlike click-placement (got %d)" % bridge.icon_get_selected())

	# --- 5. the shell's own click/drag/release routing ----------------------
	print("--- through the real tool handlers ---")
	var ws = null
	for w in app._workspaces:
		if w.get_script() != null and String(w.get_script().resource_path).ends_with("cartography_workspace.gd"):
			ws = w
	_check(ws != null, "found the CartographyWorkspace")
	if ws != null:
		ws._icon_brush_on = true
		ws._icon_brush_r = 14.0
		ws._icon_brush_density = 0.9
		ws._arm_icon_from_ui()
		var before: int = bridge.icon_list().size()
		app._on_map_clicked(land.x, land.y)
		await _frames(2)
		_check(ws._icon_brush_painting, "map_clicked started a stroke")
		var after_press: int = bridge.icon_list().size()
		_check(after_press > before, "and stamped on the press (%d -> %d)" % [before, after_press])
		app._on_map_dragged(land.x + 6, land.y + 6)
		app._on_map_dragged(land.x + 12, land.y + 2)
		await _frames(2)
		var after_drag: int = bridge.icon_list().size()
		_check(after_drag > after_press, "every drag sample stamps again (%d -> %d)" % [after_press, after_drag])
		app._on_map_released(land.x + 12, land.y + 2, true)
		await _frames(2)
		_check(not ws._icon_brush_painting, "map_released ended the stroke")
		var after_release: int = bridge.icon_list().size()
		app._on_map_dragged(land.x + 18, land.y + 4)
		await _frames(2)
		_check(bridge.icon_list().size() == after_release,
			"and a drag after the release paints nothing -- no press, no paint")

		## The reference's own `pointercancel` (line 9753). Disarming mid-stroke
		## must clear the flag, or a re-armed Icon tool resumes painting on the
		## next drag without a press.
		app._on_map_clicked(land.x, land.y)
		await _frames(2)
		_check(ws._icon_brush_painting, "a second stroke started")
		app.arm_tool("inspect")
		await _frames(2)
		_check(not ws._icon_brush_painting, "disarming the tool cancelled the stroke")

	# --- 6. persistence: the SLOT, not a doc comment ------------------------
	print("--- annotations/icons.json round trip ---")
	var n_before: int = bridge.icon_list().size()
	_check(n_before > 0, "there are %d icons to save" % n_before)
	var first: Dictionary = bridge.icon_list()[0]
	var path := ProjectSettings.globalize_path("user://_iconbrush_probe.zip")
	var documents: Dictionary = app._project_documents()
	_check(bool(bridge.save_project(path, documents)), "the project saved")
	bridge.icon_clear_all()
	_check(bridge.icon_list().is_empty(), "cleared in memory before the reload (teeth check)")
	app._load_project(path)
	await _frames(6)
	var n_back: int = bridge.icon_list().size()
	_check(n_back == n_before, "every painted icon came back: %d of %d" % [n_back, n_before])
	if n_back == n_before and n_back > 0:
		var back: Dictionary = bridge.icon_list()[0]
		_check(is_equal_approx(float(back.get("x", -1.0)), float(first.get("x", -2.0)))
			and is_equal_approx(float(back.get("y", -1.0)), float(first.get("y", -2.0))),
			"at the same cell (%s,%s)" % [back.get("x"), back.get("y")])
		_check(String(back.get("slot", "")) == String(first.get("slot", "?")),
			"in the same slot (%s)" % back.get("slot"))
		_check(is_equal_approx(float(back.get("scale", -1.0)), float(first.get("scale", -2.0))),
			"and at the per-instance scale the brush's own scatter rule gave it (%s)" % back.get("scale"))

	print("\n%s (%d failure%s)" % ["ALL PASS" if _fails == 0 else "FAILURES", _fails,
		"" if _fails == 1 else "s"])
	get_tree().quit(1 if _fails > 0 else 0)
