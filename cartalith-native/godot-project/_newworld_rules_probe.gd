extends Node
## **A new world is made by the current generation rules** (owner Ruling EV, 2026-11-13, after *"Still see some heavy combing"*:
## the owner's world had rough routing, the lake balance and the shared road trunks OFF, in a build whose app defaults have all
## three ON, because an older save had been opened in that session).
##
## Opening a save writes its generation rules into the engine's live parameter table -- off, for every rule the save predates
## (`params::LOADS_WHEN_ABSENT`) -- and New World's Create generates from that table. This probe walks the real paths:
##
##   L0  boot: the table holds the app's rules (`params::defaults`), and New World's rules notice is hidden (Create changes no rule).
##   L1  open the committed pre-Ruling-ER archive (`tests/fixtures/project_pre_substrate_2026-09-24.zip`, written by the app on
##       2026-09-24: it carries no `rough_routing`, `basin_routing`, `lake_balance`, `civ.merge_ways`, ... key, and an explicit
##       `passes.glacial: false`) through `DccApp._load_project`, the open path the pickers and the recent-worlds menu share: the
##       rules it predates load OFF, so the opened world regenerates as it was saved.
##   L2  regenerate that world AT ITS OWN SEED (WORLD ▸ Generate's route, `NewWorldDialog.generate_checked()`, not a new world):
##       still OFF, nothing restored.
##   L2b a generate at ANOTHER seed by the same route (what New seed then Generate world does; verifier finding, 2026-11-13: it
##       made the new seed's world by the opened save's rules): the app's rules, the bridge reports them, and the status hint names
##       them. Then the archive is opened again for L2c.
##   L2c a heightmap import that FAILS (a file that is not a picture) over the opened world: the restore before it is put back, so
##       the open world keeps its own rules (verifier finding, 2026-11-13), and what Create would use is unchanged (the put-back
##       is not a choice).
##   L3  open New World (`DccApp.open_new_world`, what File ▸ New world, the picker's New world tile and the open-project tile
##       all reach): the dialog's Lake water balance box shows ON, the value Create will use, while the engine still holds OFF;
##       the rules notice names the four rules Create will turn on.
##   L4  Create (the form's own Create: the dialog's OK on desktop and tablet, the CREATE WORLD button on the phone): the new
##       world's table holds the app's rules again, the bridge reports which rules it put back, and the status hint names them.
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
##   L11 the preview draws the world Create makes (verifier finding, 2026-11-13): an archive written here from the fixture
##       WITHOUT `tect.narrow_plate_base_blur` (the one rule the preview's substrate reads; the fixture itself carries it ON, so
##       L1's archive cannot show this) loads it OFF; New World at a 320-column grid draws the mask of the table Create uses
##       (`EngineBridge.new_world_params_snapshot`), not the live table's. Positive control: the two masks differ.
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
##   APPDATA=<scratch> godot --path . res://_newworld_rules_probe.tscn -- --force-touch --vp 2560x1600 --tag tablet
## Flags read below: `--vp WxH` (SubViewport size, default 1280x900), `--tag NAME` (line prefix, default `nwrules`),
## `--grid N` (New World's grid width, default 160: the rules do not depend on the size), `--shots DIR` (save L3's frame,
## the New World screen with its rules notice), `--force-touch` (read by `dcc_shell.gd`; without it the shell boots
## desktop). Any other `--flag` aborts. Every line is prefixed `[<tag>] `; the last is
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
## `--shots DIR`: save the probe viewport with the New World screen and its rules notice up (L3) as
## `<tag>_newworld_notice.png`, to be looked at. Empty (the default) saves nothing.
var _shots := ""
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
	## `.value` alone: an `apply()` in the same frame re-parses the field's not-yet-resynced display text over the value just
	## written (measured 2026-11-13: Create then made the boot seed's world at 2 048 columns, not SEED at `_grid`).
	dlg.seed_input.value = SEED
	dlg.grid_w_input.value = _grid
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


## The status bar's hint line as drawn (`DccShell._status_labels`), `""` when the slot is absent.
func _hint() -> String:
	var labels = app.get("_status_labels")
	return String((labels["hint"] as Label).text) if labels is Dictionary and (labels as Dictionary).has("hint") else ""


## The rules notice's text when it is shown, `""` when hidden or absent (an older dialog).
func _notice() -> String:
	var l = dlg.get("rules_notice_label")
	return String((l as Label).text) if l != null and (l as Label).visible else ""


## Writes a copy of the fixture archive to `user://` with `key` removed from `params.json`'s `cartalith` block: a save written
## before that key existed. Returns the OS path, or `""` when the fixture could not be read or the copy not written.
func _fixture_without(fixture: String, key: String, name: String) -> String:
	var r := ZIPReader.new()
	if r.open(fixture) != OK:
		return ""
	var out := OS.get_user_data_dir().path_join(name)
	var w := ZIPPacker.new()
	if w.open(out) != OK:
		r.close()
		return ""
	var removed := false
	for f in r.get_files():
		var bytes: PackedByteArray = r.read_file(f)
		if f == "params.json":
			var doc = JSON.parse_string(bytes.get_string_from_utf8())
			if doc is Dictionary and (doc as Dictionary).has("cartalith") and (doc["cartalith"] as Dictionary).has(key):
				(doc["cartalith"] as Dictionary).erase(key)
				removed = true
			bytes = JSON.stringify(doc).to_utf8_buffer()
		w.start_file(f)
		w.write_file(bytes)
		w.close_file()
	w.close()
	r.close()
	return out if removed else ""


## The preview request `NewWorldDialog._redraw_preview` builds over `base`: the dialog's own seed, width, grid height, archetype,
## extent and plate count laid over a whole parameter table.
func _preview_request(base: Dictionary) -> Dictionary:
	var p := base.duplicate()
	p["seed"] = int(dlg.seed_input.value)
	p["width_km"] = float(dlg.width_input.value)
	p["grid_h"] = int(dlg.grid_h_input.value)
	p["archetype"] = dlg._archetype
	p["world"] = dlg.extent_input.selected == 1
	p["tect.plates"] = dlg._classic_plates()
	return p


func _ready() -> void:
	_tag = _arg("--tag", "nwrules")
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--force-touch", "--vp", "--tag", "--grid", "--shots"]):
			_log("ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	_grid = int(_arg("--grid", "160"))
	_shots = _arg("--shots", "")
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
	if await _open_new_world():
		_check(_notice() == "", "at boot the rules notice is hidden (Create changes no rule): '%s'" % _notice())
		dlg.hide()
		await _frames(4)
	else:
		_check(false, "the New World dialog opens at boot")

	_log("-- L1 open the pre-Ruling-ER archive")
	_check(await _open_fixture(fixture), "the archive opens")
	_expect(LOADED_OFF, false, "opened")
	_expect(LOADED_ON, true, "opened")
	var fixture_seed := int(bridge.world_gen.get_seed())
	## A literal from the fixture's own `project.json` (read with Python on 2026-11-13), so a seed read wrongly cannot pass.
	_check(fixture_seed == 24601, "the opened world's seed is the archive's 24601 (got %d)" % fixture_seed)

	_log("-- L2 regenerate the opened world at its own seed (not a new world)")
	## `.value` alone, no `apply()`: the dialog is hidden, and a hidden `SpinBox` re-parses its stale display text over a value
	## written by code (`NewWorldDialog.request()`'s own note), which is what the New seed action's write avoids too.
	dlg.seed_input.value = fixture_seed
	dlg.grid_w_input.value = _grid
	await _frames(2)
	_check(int(dlg.request()["seed"]) == fixture_seed, "premise: the request carries the opened world's seed (%d)" % int(dlg.request()["seed"]))
	dlg.generate_checked()
	await bridge.generation_finished
	await _frames(8)
	_expect(LOADED_OFF, false, "regenerated")
	_check(bridge.last_new_world_restored.is_empty(), "a regenerate at the world's own seed restores nothing (%s)" % str(bridge.last_new_world_restored))

	_log("-- L2b the same route at another seed (New seed, then Generate world)")
	dlg.seed_input.value = SEED
	await _frames(2)
	_check(int(dlg.request()["seed"]) == SEED, "premise: the request carries the other seed (%d)" % int(dlg.request()["seed"]))
	dlg.generate_checked()
	await bridge.generation_finished
	await _frames(8)
	_check(int(bridge.world_gen.get_seed()) == SEED, "the world is the new seed's (%d)" % int(bridge.world_gen.get_seed()))
	_expect(APP_ON, true, "another seed")
	_expect(APP_OFF, false, "another seed")
	for k in ["rough_routing", "basin_routing", "lake_balance", "civ.merge_ways"]:
		_check(k in bridge.last_new_world_restored, "the bridge reports %s put back (%s)" % [k, str(bridge.last_new_world_restored)])
	var hint := _hint()
	_check(hint.contains("made by this build's generation rules") and hint.contains("Rough routing") and hint.contains("Lake water balance"),
		"the status hint names the rules it put back: '%s'" % hint)

	_log("-- L2c a heightmap import that fails, over the opened world")
	_check(await _open_fixture(fixture), "the archive opens for the failing import")
	_expect(["rough_routing", "lake_balance", "civ.merge_ways"], false, "opened before the failing import")
	var bad := OS.get_user_data_dir().path_join("nwrules_not_a_picture.png")
	var fh := FileAccess.open(bad, FileAccess.WRITE)
	fh.store_string("this is not a PNG")
	fh.close()
	if bridge.import_api:
		app._import_heightmap_at(bad)
		var ok_import: bool = await bridge.generation_finished
		await _frames(8)
		_check(not ok_import, "premise: the import fails")
		_expect(LOADED_OFF, false, "after the failed import")
		_check(bridge.last_new_world_restored.is_empty(), "a failed run reports nothing restored (%s)" % str(bridge.last_new_world_restored))
		## A shell without the call (before 2026-11-13) answers a failed check, never a script error: the probe must
		## finish and say what it could not see.
		var nw: Dictionary = bridge.new_world_params_snapshot() if bridge.has_method("new_world_params_snapshot") else {}
		_check(nw.get("rough_routing") == true and nw.get("lake_balance") == true,
			"and Create would still use the app's rules: the put-back was not a choice (rough_routing %s, lake_balance %s)" % [str(nw.get("rough_routing")), str(nw.get("lake_balance"))])
	else:
		_check(false, "this build has the heightmap import")

	_log("-- L3 open New World over it")
	if not await _open_new_world():
		_check(false, "the New World dialog is up")
		_finish()
		return
	_check(dlg.lake_balance_check != null, "the dialog has its Lake water balance box")
	if dlg.lake_balance_check != null:
		_check(dlg.lake_balance_check.button_pressed, "the box shows ON (what Create will use)")
	_check(bridge.param_get("lake_balance") == false, "while the engine still holds the opened world's OFF")
	var note := _notice()
	for want in ["Rough routing on", "Basin routing on", "Lake water balance on", "Merge roads into shared trunks on"]:
		_check(note.contains(want), "the rules notice names '%s': '%s'" % [want, note])
	if _shots != "":
		await RenderingServer.frame_post_draw
		var shot := _vp.get_texture().get_image()
		if shot != null:
			DirAccess.make_dir_recursive_absolute(_shots)
			shot.save_png(_shots.path_join("%s_newworld_notice.png" % _tag))
			_log("     saved %s_newworld_notice.png" % _tag)

	_log("-- L4 Create")
	await _create()
	_expect(APP_ON, true, "new world")
	_expect(APP_OFF, false, "new world")
	for k in ["rough_routing", "basin_routing", "lake_balance", "civ.merge_ways"]:
		_check(k in bridge.last_new_world_restored, "the bridge reports %s put back (%s)" % [k, str(bridge.last_new_world_restored)])
	hint = _hint()
	_check(hint.contains("made by this build's generation rules") and hint.contains("Basin routing"), "the status hint names them: '%s'" % hint)

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
	_check(_notice() == "", "over a world made by this session's rules the notice is hidden: '%s'" % _notice())
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

	_log("-- L11 the preview draws the table Create uses")
	var noblur := _fixture_without(fixture, "tect.narrow_plate_base_blur", "nwrules_no_blur_key.zip")
	_check(noblur != "", "an archive without tect.narrow_plate_base_blur was written from the fixture")
	if noblur == "":
		_finish()
		return
	_check(await _open_fixture(noblur), "that archive opens")
	_expect(["tect.narrow_plate_base_blur"], false, "a save without the key")
	if not await _open_new_world():
		_check(false, "the New World dialog is up for the preview")
		_finish()
		return
	## 320 columns: the preview's own width, so the plate-base blur radius is the target's (at 2 048 both blurs floor at
	## the engine's 2-cell minimum on the 320-cell preview, and no mask could tell them apart).
	dlg.grid_w_input.value = 320
	var t := 0
	while t < 120 and not (dlg.preview_state == "drawn" and String(dlg._preview_key).ends_with("|320|320")):
		await _frames(2)
		t += 1
	_check(dlg.preview_state == "drawn", "the preview is drawn (state %s)" % dlg.preview_state)
	if not bridge.has_method("new_world_params_snapshot"):
		_check(false, "this shell has EngineBridge.new_world_params_snapshot (the table Create uses)")
		_finish()
		return
	var nw_req := _preview_request(bridge.new_world_params_snapshot())
	var key_now := "%s|%d|%d" % [str(nw_req), 320, 320]
	_log("     preview: grid %dx%d, drawn %d cells (h %d), the dialog's key is this request's: %s" % [int(dlg.grid_w_input.value), int(dlg.grid_h_input.value), dlg.preview_mask.size(), dlg.preview_h, str(String(dlg._preview_key) == key_now)])
	var nw_mask: PackedByteArray = bridge.preview_land_mask(nw_req, 320, 320)
	var live_mask: PackedByteArray = bridge.preview_land_mask(_preview_request(bridge.params_snapshot()), 320, 320)
	_check(not nw_mask.is_empty() and nw_mask != live_mask, "positive control: the live table's mask differs from Create's (%d vs %d land cells)" % [nw_mask.count(1), live_mask.count(1)])
	_check(dlg.preview_mask == nw_mask, "the dialog draws Create's mask (%d land cells drawn, %d Create's, %d the live table's)" % [dlg.preview_mask.count(1), nw_mask.count(1), live_mask.count(1)])
	_finish()


func _finish() -> void:
	_log("RESULT %s fail=%d checks=%d" % [_tag, _fail, _checks])
	get_tree().quit(1 if _fail > 0 else 0)
