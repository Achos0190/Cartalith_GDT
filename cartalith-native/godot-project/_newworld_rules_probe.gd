extends Node
## **A new world is made by the current generation rules** (owner Ruling EV, 2026-11-13, after *"Still see some heavy combing"*:
## the owner's world had rough routing, the lake balance and the shared road trunks OFF, in a build whose app defaults have all
## three ON, because an older save had been opened in that session).
##
## Opening a save writes its generation rules into the engine's live parameter table -- off, for every rule the save predates
## (`params::LOADS_WHEN_ABSENT`) -- and New World's Create generates from that table. This probe walks the real paths:
##
##   L0  boot: the table holds the app's rules (`params::defaults`).
##   L1  open the committed pre-Ruling-ER archive (`tests/fixtures/project_pre_substrate_2026-09-24.zip`, written by the app on
##       2026-09-24: it carries no `rough_routing`, `basin_routing`, `lake_balance`, `civ.merge_ways`, ... key, and an explicit
##       `passes.glacial: false`) through `DccApp._load_project`, the open path the pickers and the recent-worlds menu share: the
##       rules it predates load OFF, so the opened world regenerates as it was saved.
##   L2  regenerate that world (WORLD ▸ Generate's route, `NewWorldDialog.generate_checked()`, not a new world): still OFF.
##   L3  open New World (`DccApp.open_new_world`, what File ▸ New world, the picker's New world tile and the open-project tile
##       all reach): the dialog's Lake water balance box shows ON, the value Create will use, while the engine still holds OFF.
##   L4  Create (the form's own Create: the dialog's OK on desktop and tablet, the CREATE WORLD button on the phone): the new
##       world's table holds the app's rules again, and the bridge reports which rules it put back.
##   L5  open the archive again, open New World, untick Lake water balance (the box's own `toggled` handler), Create: the user's
##       choice is kept (OFF) and every other rule is the app's.
##   L6  a second New World straight after, nothing opened between: the session's choice still holds (OFF).
##   L7  open the archive a third time, New World: the choice made before the open still holds (the opened world's values are
##       that world's, not a choice), and every other rule is the app's.
##   L8  the choice changed by another writer (`EngineBridge.param_set`, World ▸ Hydrology's route), then New World: the box is
##       re-read on open (ON), and Create uses it.
##   L9  Edit ▸ Reset parameters (`EngineBridge.reset_params`): every rule is the app's.
##   L10 open the archive, then import a heightmap (`DccApp._import_heightmap_at`, a 64 x 40 PNG written to `user://`): the
##       import is a new world too, so its rules are the app's.
##
## **What it is not.** Create is pressed through the control's own signal, not a finger: `MISTAKES.md` records that a synthetic
## tap cannot reach a control inside an embedded `AcceptDialog` on a phone, so the touch form here proves the handlers and the
## engine, not the touch path (that is on glass).
##
## Expected values are LITERALS typed here from the fixture's own `params.json` and from `params::defaults`, never read back from
## the code under test. A positive control (L2, and L1 itself) must see the rules OFF, so a probe that read the wrong table could
## not pass.
##
## Run WINDOWED, one run per lock hold, with an isolated APPDATA (L1 and L5 record the archive in the recent-worlds list):
##   APPDATA=<scratch> godot --path . res://_newworld_rules_probe.tscn -- --vp 1280x900 --tag desk
##   APPDATA=<scratch> godot --path . res://_newworld_rules_probe.tscn -- --force-touch --vp 1080x2340 --tag phone
## Flags read below: `--vp WxH` (SubViewport size, default 1280x900), `--tag NAME` (line prefix, default `nwrules`),
## `--grid N` (New World's grid width, default 160: the rules do not depend on the size), `--force-touch` (read by
## `dcc_shell.gd`; without it the shell boots desktop). Any other `--flag` aborts. Every line is prefixed `[<tag>] `; the last is
## `RESULT <tag> fail=<n> checks=<n>`. Exit 0 all passed, 1 a check failed, 2 could not run (headless, no fixture, no dialog).

const FIXTURE := "res://../crates/cartalith-godot/tests/fixtures/project_pre_substrate_2026-09-24.zip"
## The rules the fixture's `params.json` does not carry (so they load OFF), plus `passes.glacial`, which it carries as an
## explicit `false`. Read off the archive with Python on 2026-11-13; typed here as literals.
const LOADED_OFF := ["rough_routing", "basin_routing", "lake_balance", "civ.merge_ways", "tect.orogen_relief",
	"tect.high_ceiling", "passes.glacial", "geology_model", "climate.rain_bspline", "volc.fill_edifice_pits",
	"volc.morphometry", "volc.kind_dome"]
## The two rules the fixture carries as `true`.
const LOADED_ON := ["integrate_drainage", "tect.narrow_plate_base_blur"]
## `params::defaults`' value of every rule checked, as literals: ON ...
const APP_ON := ["rough_routing", "basin_routing", "lake_balance", "civ.merge_ways", "tect.orogen_relief",
	"tect.high_ceiling", "passes.glacial", "geology_model", "climate.rain_bspline", "volc.fill_edifice_pits",
	"volc.morphometry", "volc.kind_dome", "integrate_drainage", "tect.narrow_plate_base_blur"]
## ... and OFF (the geology processes and their three dependants stay off in the app).
const APP_OFF := ["geology_processes", "relief_model", "geology_karst", "geology_carve"]
const SEED := 51203

var app: Node
var bridge: Node
var dlg: Node
var _vp: SubViewport
var _tag := "nwrules"
var _grid := 160
var _fail := 0
var _checks := 0


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])


func _check(ok: bool, what: String) -> void:
	_checks += 1
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])


func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	return String(args[i + 1]) if i >= 0 and i + 1 < args.size() else dflt


## `key`'s value in the engine's live table as text, or `absent` (a key the engine does not publish is never read as `false`).
func _val(p: Dictionary, key: String) -> String:
	return str(p[key]) if p.has(key) else "absent"


## Asserts every key of `keys` is `want` in the live table.
func _expect(keys: Array, want: bool, what: String) -> void:
	var p: Dictionary = bridge.world_gen.get_params()
	for k in keys:
		_check(p.has(k) and bool(p[k]) == want, "%s: %s = %s (expected %s)" % [what, k, _val(p, k), str(want)])


## Opens New World through the shell's own entry and waits for the dialog.
func _open_new_world() -> bool:
	app.open_new_world()
	await _frames(30)
	return dlg.visible


## Sets the dialog's seed and grid (no auto-populate: the rules are the subject), presses the form's own Create, waits.
func _create() -> void:
	dlg._ap_master = false
	dlg.seed_input.value = SEED
	dlg.seed_input.apply()
	dlg.grid_w_input.value = _grid
	dlg.grid_w_input.apply()
	await _frames(2)
	if dlg._phone_create != null:
		dlg._phone_create.pressed.emit()
	else:
		dlg.get_ok_button().pressed.emit()
	await bridge.generation_finished
	await _frames(8)


func _open_fixture(path: String) -> bool:
	var ok: bool = app._load_project(path)
	await _frames(8)
	return ok


func _ready() -> void:
	_tag = _arg("--tag", "nwrules")
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--force-touch", "--vp", "--tag", "--grid"]):
			_log("ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	_grid = int(_arg("--grid", "160"))
	var fixture := ProjectSettings.globalize_path(FIXTURE)
	if DisplayServer.get_name() == "headless" or not FileAccess.file_exists(fixture):
		_log("ABORT headless, or no fixture at %s; nothing was measured" % fixture)
		get_tree().quit(2)
		return
	var parts: PackedStringArray = _arg("--vp", "1280x900").split("x")
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)
	bridge = app.bridge
	dlg = app.new_world_dialog
	var has_api: bool = bridge.world_gen.has_method("restore_new_world_rules") and bridge.world_gen.has_method("get_new_world_params")
	_log("viewport %dx%d phone=%s touch=%s, extension has the restore: %s" % [_vp.size.x, _vp.size.y, app.is_phone(), DccTheme.is_touch(), has_api])

	_log("-- L0 boot")
	_expect(APP_ON, true, "boot")
	_expect(APP_OFF, false, "boot")

	_log("-- L1 open the pre-Ruling-ER archive")
	_check(await _open_fixture(fixture), "the archive opens")
	_expect(LOADED_OFF, false, "opened")
	_expect(LOADED_ON, true, "opened")

	_log("-- L2 regenerate the opened world (not a new world)")
	dlg.generate_checked()
	await bridge.generation_finished
	await _frames(8)
	_expect(LOADED_OFF, false, "regenerated")
	_check(bridge.last_new_world_restored.is_empty(), "a regenerate restores nothing (%s)" % str(bridge.last_new_world_restored))

	_log("-- L3 open New World over it")
	if not await _open_new_world():
		_check(false, "the New World dialog is up")
		_finish()
		return
	_check(dlg.lake_balance_check != null, "the dialog has its Lake water balance box")
	if dlg.lake_balance_check != null:
		_check(dlg.lake_balance_check.button_pressed, "the box shows ON (what Create will use)")
	_check(bridge.param_get("lake_balance") == false, "while the engine still holds the opened world's OFF")

	_log("-- L4 Create")
	await _create()
	_expect(APP_ON, true, "new world")
	_expect(APP_OFF, false, "new world")
	for k in ["rough_routing", "basin_routing", "lake_balance", "civ.merge_ways"]:
		_check(k in bridge.last_new_world_restored, "the bridge reports %s put back (%s)" % [k, str(bridge.last_new_world_restored)])

	_log("-- L5 open it again, untick Lake water balance, Create")
	_check(await _open_fixture(fixture), "the archive opens again")
	_expect(["lake_balance", "rough_routing"], false, "reopened")
	if not await _open_new_world():
		_check(false, "the New World dialog is up again")
		_finish()
		return
	if dlg.lake_balance_check != null:
		_check(dlg.lake_balance_check.button_pressed, "the box shows ON again after the reopen")
		dlg.lake_balance_check.button_pressed = false
		await _frames(2)
	_check(bridge.param_get("lake_balance") == false, "the untick reached the engine")
	await _create()
	_expect(["lake_balance"], false, "the user's choice")
	_expect(["rough_routing", "basin_routing", "civ.merge_ways", "volc.kind_dome", "passes.glacial"], true, "the other rules")

	_log("-- L6 another New World, nothing opened between")
	if not await _open_new_world():
		_check(false, "the New World dialog is up a third time")
		_finish()
		return
	if dlg.lake_balance_check != null:
		_check(not dlg.lake_balance_check.button_pressed, "the box shows the session's choice (OFF)")
	await _create()
	_expect(["lake_balance"], false, "the session's choice carries")
	_expect(["rough_routing", "basin_routing", "civ.merge_ways"], true, "and the rules stay the app's")

	_log("-- L7 open the archive a third time, New World: the choice survives an open")
	_check(await _open_fixture(fixture), "the archive opens a third time")
	_expect(["rough_routing", "basin_routing"], false, "opened again")
	if not await _open_new_world():
		_check(false, "the New World dialog is up a fourth time")
		_finish()
		return
	if dlg.lake_balance_check != null:
		_check(not dlg.lake_balance_check.button_pressed, "the box shows the session's choice (OFF), not the opened world's")
	await _create()
	_expect(["lake_balance"], false, "the choice made before the open")
	_expect(["rough_routing", "basin_routing", "civ.merge_ways", "passes.glacial"], true, "and the app's rules")

	_log("-- L8 the choice changed elsewhere (World > Hydrology's own write), then New World")
	bridge.param_set("lake_balance", true)
	if not await _open_new_world():
		_check(false, "the New World dialog is up a fifth time")
		_finish()
		return
	if dlg.lake_balance_check != null:
		_check(dlg.lake_balance_check.button_pressed, "the box is re-read on open and shows ON (not the stale OFF)")
	await _create()
	_expect(["lake_balance"], true, "the latest choice")

	_log("-- L9 Edit > Reset parameters")
	bridge.reset_params()
	await _frames(2)
	_expect(APP_ON, true, "reset")

	_log("-- L10 open the archive, then import a heightmap (the other run that makes a new world)")
	_check(await _open_fixture(fixture), "the archive opens a fourth time")
	_expect(["rough_routing", "lake_balance", "civ.merge_ways"], false, "opened before the import")
	if not bridge.import_api:
		_check(false, "this build has the heightmap import")
		_finish()
		return
	var img := Image.create(64, 40, false, Image.FORMAT_L8)
	for y in 40:
		for x in 64:
			var v := 0.5 + 0.4 * sin(float(x) * 0.2) * cos(float(y) * 0.25)
			img.set_pixel(x, y, Color(v, v, v))
	var hm := OS.get_user_data_dir().path_join("nwrules_hm.png")
	img.save_png(hm)
	app._import_heightmap_at(hm)
	await bridge.generation_finished
	await _frames(8)
	_expect(["rough_routing", "basin_routing", "lake_balance", "civ.merge_ways", "volc.kind_dome"], true, "imported world")
	_finish()


func _finish() -> void:
	_log("RESULT %s fail=%d checks=%d" % [_tag, _fail, _checks])
	get_tree().quit(1 if _fail > 0 else 0)
