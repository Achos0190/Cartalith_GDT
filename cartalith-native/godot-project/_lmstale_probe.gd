extends Node
## `OUTSTANDING_WORK.md`: *"Nothing tells the user a landmark result predates
## their icons."* End to end on the real shell: generate, run the landmark
## pass, commit one hand-placed icon, and assert both the engine's
## `stale_stages()["landmarks"]` flag and the dock's stale-note badge react --
## then re-run the pass and assert both clear.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x900 \
##       --rendering-driver opengl3 _lmstale_probe.tscn
##
## Windowed, never `--headless`: boots `shell/app.tscn`, same reason
## `_landmark_probe.gd`'s own header gives -- the dummy rasterizer returns
## null textures and the shell boot walks straight into them.
##
## `_lmicon_probe.gd`'s pattern for the icon half (direct `WorldGen`,
## `icon_arm`/`icon_place` against the fixture asset pack); `_landmark_probe.gd`
## section E's pattern for booting the real shell and reading `_lm_*` panel
## state off the live `CivilizationWorkspace`.

const PACK := "res://../crates/cartalith-assets/tests/fixtures/reference_pack.zip"

var _vp: SubViewport
var _civ: Node
var _fail := 0


## `_lm_stale_note` does not survive `generate()` -- a new world tears down
## and rebuilds the dock's category panels, so a `Label` reference captured
## before it is a dangling `previously freed` object afterward (caught by
## running this probe: it failed here first). Re-fetch every time instead of
## caching once.
func _note() -> Label:
	return _civ.get("_lm_stale_note") as Label


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)


func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(50)
	print("[BOOT] shell up")

	var bridge = app.get("bridge")
	if bridge == null or not bridge.has_method("landmark_run") \
			or not bridge.has_method("stale_stages"):
		print("_lmstale_probe: SKIPPED -- this build's EngineBridge has no landmark_run()")
		print("  or stale_stages(). Nothing here proves the real engine works.")
		get_tree().quit(0)
		return
	if not bridge.has_method("icon_place"):
		print("_lmstale_probe: SKIPPED -- no icon_place() in this build.")
		get_tree().quit(0)
		return

	_civ = app.find_child("CivilizationWorkspace", true, false)
	_ok("the CIVIL workspace is registered", _civ != null, true)
	if _civ == null:
		print("_lmstale_probe: FAILURE (no workspace)"); get_tree().quit(1); return

	print("\n=== generate a real world ===")
	bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(8)
	_ok("a world generated", bridge.has_world, true)
	_ok("the landmark dock's stale note exists (post-generate)", _note() != null, true)
	if _note() == null:
		print("_lmstale_probe: FAILURE (no _lm_stale_note)"); get_tree().quit(1); return

	print("\n=== run 1: a fresh landmark pass is never stale ===")
	await _civ.call("_lm_run")
	await _frames(8)
	var lm0: Dictionary = bridge.stale_stages().get("landmarks", {})
	_ok("engine: landmarks key absent right after a run", lm0.is_empty(), true)
	_civ.call("_lm_refresh_stale")
	await _frames(2)
	_ok("badge does not carry the icon-staleness sentence yet",
		String(_note().text).find("icon has been placed") >= 0, false)
	print("  info badge text: ", _note().text)

	print("\n=== commit one hand-placed icon (icon_place, not a drag sample) ===")
	_ok("asset pack loads", bridge.load_asset_pack(ProjectSettings.globalize_path(PACK)), true)
	_ok("icon arms", bridge.icon_arm("feature", 0, 1.0, 0.0, 0.0), true)
	var idx: int = bridge.icon_place(192.0, 144.0)
	_ok("icon_place committed (index >= 0)", idx >= 0, true)

	print("\n=== the engine flag and the UI badge both react ===")
	var lm1: Dictionary = bridge.stale_stages().get("landmarks", {})
	_ok("engine: landmarks key now present", lm1.is_empty(), false)
	_ok("...origin is icons", String(lm1.get("origin", "")), "icons")
	_ok("...reason is icon_placed", String(lm1.get("reason", "")), "icon_placed")
	## The 1 s poll timer (`_lm_stale_timer`) would pick this up on its own;
	## called directly here so the probe does not depend on wall-clock timing.
	_civ.call("_lm_refresh_stale")
	await _frames(2)
	_ok("badge now carries the icon-staleness sentence",
		String(_note().text).find("icon has been placed") >= 0, true)
	print("  info badge text: ", _note().text)

	print("\n=== a delete does not touch this flag (out of this row's scope) ===")
	## Documents the scoping decision rather than asserting a requirement:
	## the flag tracks placement, not every icon-collection edit. Left here
	## so a future change to that scope has something to update.
	print("  info (not asserted) icon_delete exists: ", bridge.has_method("icon_delete"))

	print("\n=== run 2: re-running the pass clears both ===")
	await _civ.call("_lm_run")
	await _frames(8)
	var lm2: Dictionary = bridge.stale_stages().get("landmarks", {})
	_ok("engine: landmarks key clears on re-run", lm2.is_empty(), true)
	_civ.call("_lm_refresh_stale")
	await _frames(2)
	_ok("badge no longer carries the icon-staleness sentence",
		String(_note().text).find("icon has been placed") >= 0, false)
	print("  info badge text: ", _note().text)

	print("\n_lmstale_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
