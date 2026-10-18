extends Node
## `Preferences ▸ Graphics ▸ Lighting rig defaults` — §2.5's "azimuth,
## elevation, ambient, multidirectional on/off".
##
##   Godot_v4.7.1 --path . --resolution 1600x900 --rendering-driver opengl3 _lighting_probe.tscn
##
## The row's own reason said the only missing piece was "the project-level
## default store those per-layer values would seed from — a settings key, not
## new rendering". So what this asserts is that the key exists AND that it
## actually reaches the renderer: a settings value nothing reads would satisfy
## every structural check and change nothing on screen.
##
## **The contract it asserts (rewritten 2026-10-18).** Since 2026-09-01
## `DccSettings.lighting_defaults()` returns ONLY keys the user stored
## (`dcc_settings.gd`'s `appearance_defaults()` doc comment: an untouched install
## returns `{}` so the active quality tier's own rig stands). The old section 2
## read `d["sun_az_deg"]` off an untouched install, errored, and then hung with
## no verdict. So: an untouched or reset install stores nothing (`{}`,
## `has_lighting_default` false for all four); the ladders (`menus.gd`
## `_build_lighting_menu`) store the rung pressed; a Generate sends the stored
## keys to the engine; Reset erases them again (it does NOT un-send what a
## Generate already applied -- `reset_lighting_defaults()`'s doc -- so nothing
## here asserts the engine reverts). `LIGHTING_DEFAULTS` stays pinned to the
## reference-rig literals because the menu's label fallback reads it.
##
## Result: `  ok   <name>   got= want=` / `  FAIL ...` per check, `=== N: ... ===` section
##         headings, `[BOOT]`/`info` lines, a `[lighting] section restored` check, and a
##         final `_lighting_probe: PASS` or `_lighting_probe: N FAILURE(S)`.
##         Exit 0 only on PASS, 1 on any failure, 3 when the watchdog fired (a
##         `FAIL watchdog: stuck after section ...` line says where).

var _fail := 0

## Whole-run watchdog budget, seconds. A labelled judgement, not a measurement:
## the healthy run is a few seconds windowed; 120 s is far beyond that and still
## short enough that a hang is noticed.
const WATCHDOG_S := 120.0
## Frame cap for waiting on `generation_finished` (a 192x144 grid; judgement).
const GEN_WAIT_FRAMES := 1800

var _stashed := false
var _quitting := false
var _cfg_prior: ConfigFile = null   ## the whole settings file, parsed before boot
var _where := "boot"                ## last section entered, for the watchdog line

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _near(name: String, got: float, want: float, tol: float) -> void:
	var good := absf(got - want) <= tol
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name,
		"   got=%.3f want=%.3f (tol %.3f)" % [got, want, tol])

func _find_popup(app: Node, id: int) -> PopupMenu:
	var stack: Array = [app]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
			if c is PopupMenu and (c as PopupMenu).get_item_index(id) >= 0:
				return c as PopupMenu
	return null

## Hermetic settings (2026-10-18): this probe starts from a clean rig
## (`reset_lighting_defaults()` erases the whole `[lighting]` section) and the
## menu presses below write more keys; ending on step 5's Reset left the section
## ERASED, destroying any rig the user-data dir already carried. `_stash_lighting()`
## records which keys were stored (presence AND value: a stored copy of a default
## is a different statement from an absent key) plus a parse of the whole file,
## and `_quit()` -- the only exit after the stash -- erases the section and writes
## those keys back, then compares the `[lighting]` section against the pre-boot parse.
var _lit_prior: Dictionary = {}

func _stash_lighting() -> void:
	_stashed = true
	_cfg_prior = ConfigFile.new()
	_cfg_prior.load(DccSettings.CONFIG_PATH)    ## absent file -> empty parse, fine
	for k in DccSettings.LIGHTING_DEFAULTS:
		if DccSettings.has_lighting_default(String(k)):
			_lit_prior[String(k)] = float(DccSettings.lighting_defaults()[k])

## The ONLY exit after boot. Idempotent: the watchdog and a normal finish can
## race. A restore mismatch turns exit 0 into 1.
func _quit(code: int) -> void:
	if _quitting:
		return
	_quitting = true
	if _stashed:
		DccSettings.reset_lighting_defaults()
		for k in _lit_prior:
			DccSettings.set_lighting_default(k, _lit_prior[k])
		## Only `[lighting]` is compared: booting the real shell also writes other
		## sections of the file (e.g. `[layout]`), which this probe neither owns
		## nor changes, so they are not its to restore.
		var now := ConfigFile.new()
		now.load(DccSettings.CONFIG_PATH)
		var same := true
		var ka := _cfg_prior.get_section_keys("lighting") if _cfg_prior.has_section("lighting") else PackedStringArray()
		var kb := now.get_section_keys("lighting") if now.has_section("lighting") else PackedStringArray()
		if ka.size() != kb.size():
			same = false
		for k in ka:
			if not now.has_section_key("lighting", k) 					or str(now.get_value("lighting", k)) != str(_cfg_prior.get_value("lighting", k)):
				same = false
		print("  ", "ok  " if same else "FAIL", " [lighting] section restored   (",
			ProjectSettings.globalize_path(DccSettings.CONFIG_PATH), ")")
		if not same:
			_fail += 1
			if code == 0:
				code = 1
	## The verdict comes last so it includes the restore check above.
	print("_lighting_probe: ", "PASS" if code == 0 and _fail == 0 else str(maxi(_fail, 1)) + " FAILURE(S)")
	get_tree().quit(code)

## Waits for `generation_finished` with a frame cap instead of a bare `await`,
## which would hang forever if the signal never fired. Returns whether it fired.
func _await_generation(bridge) -> bool:
	var st := {"done": false}
	bridge.generation_finished.connect(func(_summary = null): st["done"] = true, CONNECT_ONE_SHOT)
	var n := 0
	while not st["done"] and n < GEN_WAIT_FRAMES:
		await get_tree().process_frame
		n += 1
	return st["done"]

func _ready() -> void:
	get_tree().create_timer(WATCHDOG_S).timeout.connect(func() -> void:
		print("  FAIL watchdog: stuck after section '", _where, "'")
		_quit(3))
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	_stash_lighting()
	DccSettings.reset_lighting_defaults()
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await _frames(40)
	var bridge = app.get("bridge")
	if bridge == null:
		_ok("shell exposes its bridge", false, true)
		_quit(1); return
	print("[BOOT] shell up · defaults reset to the reference rig")

	_where = "1"
	print("")
	print("=== 1: the four ladders exist and none is disabled ===")
	var az := _find_popup(app, 170)
	var alt := _find_popup(app, 190)
	var amb := _find_popup(app, 210)
	var num := _find_popup(app, 230)
	_ok("Azimuth ladder", az != null, true)
	_ok("Elevation ladder", alt != null, true)
	_ok("Ambient ladder", amb != null, true)
	_ok("Multidirectional ladder", num != null, true)
	if az == null or alt == null or amb == null or num == null:
		_quit(1); return
	var any_disabled := false
	for pair in [[az, 170, 8], [alt, 190, 7], [amb, 210, 6], [num, 230, 6]]:
		var pm: PopupMenu = pair[0]
		for k in int(pair[2]):
			var i := pm.get_item_index(int(pair[1]) + k)
			if i >= 0 and pm.is_item_disabled(i):
				any_disabled = true
	_ok("no rung anywhere is disabled", any_disabled, false)
	print("  info azimuth rungs: ", az.item_count, "  elevation: ", alt.item_count,
		"  ambient: ", amb.item_count, "  lights: ", num.item_count)

	_where = "2"
	print("")
	print("=== 2: an untouched install stores nothing; the reference rig is the label fallback ===")
	## `reset_lighting_defaults()` ran at boot, so this is the untouched state.
	var d: Dictionary = DccSettings.lighting_defaults()
	print("  info ", d)
	_ok("untouched install sends the engine nothing (the tier's rig stands)", d.is_empty(), true)
	for k in ["sun_az_deg", "sun_alt_deg", "relief_ambient", "relief_lights"]:
		_ok("not stored: " + k, DccSettings.has_lighting_default(k), false)
	## Literal pins of the reference rig `LIGHTING_DEFAULTS` carries (315 deg and
	## 45 deg from the reference's single NW sun; ambient 0.35 and one light as
	## the dict's own doc comment states), not self-comparisons.
	_near("azimuth label fallback 315", float(DccSettings.LIGHTING_DEFAULTS["sun_az_deg"]), 315.0, 0.01)
	_near("elevation label fallback 45", float(DccSettings.LIGHTING_DEFAULTS["sun_alt_deg"]), 45.0, 0.01)
	_near("ambient label fallback 0.35", float(DccSettings.LIGHTING_DEFAULTS["relief_ambient"]), 0.35, 0.001)
	_near("one light (the reference's single sun)", float(DccSettings.LIGHTING_DEFAULTS["relief_lights"]), 1.0, 0.01)

	_where = "3"
	print("")
	print("=== 3: the MENU writes the store ===")
	az.id_pressed.emit(170 + 4)       ## 180 deg
	num.id_pressed.emit(230 + 4)      ## 8 lights
	await _frames(4)
	var d2: Dictionary = DccSettings.lighting_defaults()
	_ok("azimuth and lights are now stored", d2.has("sun_az_deg") and d2.has("relief_lights"), true)
	_ok("...and nothing else is (elevation, ambient untouched)",
		d2.has("sun_alt_deg") or d2.has("relief_ambient"), false)
	_near("azimuth is now 180", float(d2.get("sun_az_deg", -1.0)), 180.0, 0.01)
	_near("lights is now 8", float(d2.get("relief_lights", -1.0)), 8.0, 0.01)
	az.about_to_popup.emit()
	await _frames(2)
	_ok("...and the ladder checks that rung", az.is_item_checked(az.get_item_index(174)), true)
	_ok("...and only that rung", az.is_item_checked(az.get_item_index(177)), false)

	_where = "4"
	print("")
	print("=== 4: a Generate actually applies it — the half a settings key alone would fake ===")
	bridge.generate({"seed": 771, "width_km": 600.0, "grid_w": 192, "grid_h": 144,
		"sea_level": 0.5, "villages": true})
	if not await _await_generation(bridge):
		_ok("generation finished within the frame cap", false, true)
		_quit(1); return
	await _frames(16)
	var live: Dictionary = bridge.appearance()
	print("  info engine sun_az_deg=", live.get("sun_az_deg", "?"),
		"  relief_lights=", live.get("relief_lights", "?"))
	_near("the ENGINE now reports azimuth 180", float(live.get("sun_az_deg", -1.0)), 180.0, 0.01)
	_near("...and 8 light directions", float(live.get("relief_lights", -1.0)), 8.0, 0.01)

	_where = "5"
	print("")
	print("=== 5: reset erases rather than storing a copy of the default ===")
	## A stored copy of a default is a default that cannot move: change
	## LIGHTING_DEFAULTS later and anyone who once pressed Reset keeps the old
	## number forever. Erasing the section is what avoids that.
	var reset_pm := _find_popup(app, 250)
	_ok("Reset row exists", reset_pm != null, true)
	if reset_pm == null:
		_quit(1); return
	reset_pm.id_pressed.emit(250)
	await _frames(4)
	var d3: Dictionary = DccSettings.lighting_defaults()
	_ok("reset leaves nothing stored (not a stored copy of 315 / 1)", d3.is_empty(), true)
	_ok("azimuth no longer stored", DccSettings.has_lighting_default("sun_az_deg"), false)
	_ok("lights no longer stored", DccSettings.has_lighting_default("relief_lights"), false)

	print("")
	_quit(1 if _fail > 0 else 0)
