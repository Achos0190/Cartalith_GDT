extends Node
## **New World -> engine -> WORLD side panel: one source of truth, measured.** (Owner bug report 2026-11-12: "the plate count from
## the new world screen isnt being transmitted to the generators side screen ... Same when using a preset for world structure this is
## completely overridden when we use the more regular terrain generator settings. Resulting in vastly different worlds.")
##
## Protects three properties, for several plate counts and every World-structure preset:
##   1. after Create, the engine's parameters are what the dialog set (the plate count; an archetype's five dials and steering on);
##   2. after Create, the WORLD dock's controls SHOW those values (`world_workspace.gd::_param_ctrls`, read back from the
##      controls themselves, never from the engine);
##   3. a Generate from the dock with no edits reproduces the same world (colour-texture, drawn-water and settlement digests) and
##      leaves the parameters alone; and a dial the user moves in the dock SURVIVES that Generate (an archetype is not re-applied
##      over it).
## Plus (--labels) the MS-8 agent's open finding: two Creates of the same seed with no pass run gave different generated label lists.
##
## Expected values are LITERALS written here (the archetype table is `ARCHETYPES` in `cartalith-godot/src/lib.rs`), never read back
## from the code under test. A positive control forces the panel read to be able to fail: a control holding 14 must be judged
## unequal to an expected 23.
##
## Run WINDOWED, under the lock (the colour texture is a no-op headless):
##   godot --path . res://_newworld_sync_probe.tscn -- --vp 1280x900 --tag desk
##   godot --path . res://_newworld_sync_probe.tscn -- --force-touch --vp 1600x2560 --tag tab
## Flags: `--vp WxH`, `--tag NAME`, `--force-touch`, `--grid N` (default 192), `--labels` (also run the label leg first), `--only-labels` (that leg alone), `--seed N` (the label leg's seed). Anything else
## aborts. Exit 0 all passed, 1 a check failed, 2 could not run.
## Result lines are prefixed `[<tag>] `; the last is `RESULT <tag> fail=<n> checks=<n>`.

## Literal archetype table as [continentality, fragmentation, tectonic_energy, ocean_depth, hotspot_density] (`ARCHETYPES`,
## `cartalith-godot/src/lib.rs`). Typed here on purpose.
const ARCH := {
	"earth": [0.30, 0.50, 0.60, 0.60, 0.20],
	"supercontinent": [0.60, 0.10, 0.50, 0.70, 0.10],
	"archipelago": [0.15, 0.90, 0.80, 0.30, 0.50],
	"volcanic": [0.05, 1.00, 0.90, 0.80, 1.00],
	"rift": [0.40, 0.35, 0.75, 0.55, 0.30],
}
const WS_KEYS := ["world_structure.continentality", "world_structure.fragmentation", "world_structure.tectonic_energy",
	"world_structure.ocean_depth", "world_structure.hotspot_density"]
## Classic plate counts to try: both ends of `tect.plates`' 4..40 range and one between, none equal to the engine default 14, so a
## stale panel (showing 14) cannot pass by coincidence.
const PLATE_COUNTS := [6, 23, 38]
const SEED := 777
## `--seed N` overrides it for the label leg only (the MS-8 probe used 4242).
var _label_seed := SEED

var app: Node
var bridge: Node
var dlg: Node
var ws: Node
var _vp: SubViewport
var _tag := "nwsync"
var _fail := 0
var _checks := 0
var _grid := 192


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
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt


func _ready() -> void:
	_tag = _arg("--tag", "nwsync")
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--force-touch", "--vp", "--tag", "--grid", "--labels", "--only-labels", "--seed"]):
			_log("ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	_grid = int(_arg("--grid", "192"))
	_label_seed = int(_arg("--seed", str(SEED)))
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
	for w in app.get("_workspaces"):
		if (w.get_script() as Script).get_global_name() == "WorldWorkspace":
			ws = w
	if ws == null or DisplayServer.get_name() == "headless":
		_log("ABORT no WorldWorkspace, or headless; nothing was measured")
		get_tree().quit(2)
		return
	_log("viewport %dx%d phone=%s tablet=%s touch=%s" % [_vp.size.x, _vp.size.y, app.is_phone(), DccTheme.is_tablet(), DccTheme.is_touch()])
	app.open_new_world()
	await _frames(30)
	if not dlg.visible:
		_check(false, "the New World dialog is up")
		_finish()
		return
	dlg._ap_master = false  # terrain only: the passes are MS-8's probe, not this one's
	_positive_control()
	if "--labels" in OS.get_cmdline_user_args() or "--only-labels" in OS.get_cmdline_user_args():
		await _labels_leg()
	if "--only-labels" in OS.get_cmdline_user_args():
		_finish()
		return
	for n in PLATE_COUNTS:
		await _case("classic plates=%d" % n, "", n)
	var names: Array = dlg._archetype_names
	_check(names.size() == ARCH.size(), "the dialog lists %d archetypes (the literal table has %d)" % [names.size(), ARCH.size()])
	for nm in names:
		await _case("archetype %s" % nm, String(nm), 0)
	# Back to Classic after an archetype: the steering the archetype switched on must go off, or the plate count is ignored.
	await _case("classic plates=23 after an archetype", "", 23)
	_finish()


func _finish() -> void:
	_log("RESULT %s fail=%d checks=%d" % [_tag, _fail, _checks])
	get_tree().quit(1 if _fail > 0 else 0)


## The panel read has to be able to fail: a slider holding 14 must be judged unequal to an expected 23.
func _positive_control() -> void:
	var s := (ws.get("_param_ctrls")["tect.plates"]["slider"]) as HSlider
	var before := s.value
	ws.set("_resyncing", true)
	s.value = 14.0
	var judged_equal := _near(s, 23.0)
	s.value = before
	ws.set("_resyncing", false)
	_check(not judged_equal, "positive control: a panel showing 14 is NOT judged equal to 23")


func _near(s: HSlider, want: float) -> bool:
	return absf(s.value - want) <= s.step * 0.5 + 1e-6


## What the dock's control for `key` shows. Read from the control, never the engine.
func _panel(key: String):
	var e: Dictionary = ws.get("_param_ctrls").get(key, {})
	if e.has("slider"):
		return (e["slider"] as HSlider).value
	if e.has("bool"):
		return (e["bool"] as CheckBox).button_pressed
	return null


func _digest() -> Dictionary:
	var img: Image = bridge.color_texture().get_image()
	var hc := HashingContext.new()
	hc.start(HashingContext.HASH_SHA256)
	hc.update(img.get_data())
	return {"map": hc.finish().hex_encode().substr(0, 16),
		"water": String(bridge.world_gen.drawn_water_digest().get("fnv1a", "none")),
		## No settlement digest: this probe Creates with the passes OFF (`_ap_master = false`), and since the owner's 2026-11-13
		## report a Create with them off leaves NO settlements, while the dock's Generate (a regenerate, no passes) keeps what
		## every generate computes, so the two legitimately differ and a digest of them said nothing about the terrain.
		}


## Presses the dialog's Generate and waits for the world.
func _create() -> void:
	dlg.get_ok_button().pressed.emit()
	await bridge.generation_finished
	await _frames(8)


## The dock's own Generate path (`world_workspace.gd::_regenerate_now`, what a slider release and the Generate button reach).
func _dock_generate() -> void:
	ws._regenerate_now()
	await bridge.generation_finished
	await _frames(8)


func _case(label: String, archetype: String, plates: int) -> void:
	_log("case %s" % label)
	dlg.seed_input.value = SEED
	dlg.seed_input.apply()
	dlg.grid_w_input.value = _grid
	dlg.grid_w_input.apply()
	await _frames(2)
	var idx := 0
	if archetype != "":
		idx = dlg._archetype_names.find(archetype) + 1
	dlg.archetype_input.select(idx)
	dlg._on_archetype_selected(idx)
	if archetype == "" and plates > 0:
		dlg._plates_manual = true
		dlg._plates_value = plates
		dlg._refresh_plates()
	await _frames(2)
	await _create()
	# 1. the engine took what the dialog set
	var snap: Dictionary = bridge.param_snapshot()
	if archetype == "":
		_check(int(snap["tect.plates"]) == plates, "engine tect.plates == %d (got %s)" % [plates, str(snap["tect.plates"])])
		_check(snap["world_structure.enabled"] == false, "Classic leaves steering off, so the plate count is honoured")
	else:
		var want: Array = ARCH[archetype]
		_check(snap["world_structure.enabled"] == true, "%s: steering on" % archetype)
		for i in 5:
			_check(absf(float(snap[WS_KEYS[i]]) - float(want[i])) < 1e-9, "%s: engine %s == %s (got %s)" % [archetype, WS_KEYS[i], str(want[i]), str(snap[WS_KEYS[i]])])
	# 2. the dock shows it
	if archetype == "":
		_check(_near(ws.get("_param_ctrls")["tect.plates"]["slider"], float(plates)), "dock Plates control shows %d (shows %s)" % [plates, str(_panel("tect.plates"))])
		_check(_panel("world_structure.enabled") == false, "dock shows steering off")
	else:
		var want2: Array = ARCH[archetype]
		_check(_panel("world_structure.enabled") == true, "%s: dock shows steering on" % archetype)
		for i in 5:
			_check(_near(ws.get("_param_ctrls")[WS_KEYS[i]]["slider"], float(want2[i])), "%s: dock %s shows %s (shows %s)" % [archetype, WS_KEYS[i], str(want2[i]), str(_panel(WS_KEYS[i]))])
	# 3. a Generate from the dock with no edits reproduces the world and leaves the parameters alone
	var d1 := _digest()
	var snap1: Dictionary = bridge.param_snapshot()
	await _dock_generate()
	var d2 := _digest()
	var snap2: Dictionary = bridge.param_snapshot()
	_check(str(d1) == str(d2), "dock Generate with no edits reproduces the world (%s vs %s)" % [str(d1), str(d2)])
	_check(str(snap1) == str(snap2), "dock Generate with no edits leaves every engine parameter as it was")
	# 4. a dial the user moves in the dock survives the next Generate (a preset is not re-applied over it): the engine keeps it,
	# the dock shows it, and the WORLD itself changed (its digest moved), which is what the owner saw missing -- the old one-shot
	# archetype path left the stored value alone and generated the preset anyway, so only the digest can tell.
	if archetype != "":
		var key: String = WS_KEYS[0]
		var sl := ws.get("_param_ctrls")[key]["slider"] as HSlider
		var literal: float = 0.77 if absf(float(ARCH[archetype][0]) - 0.77) > 0.05 else 0.21
		_check(absf(sl.value - literal) > 0.05, "%s: the dock starts away from the user's %s (shows %s), so setting it is a real change" % [archetype, str(literal), str(sl.value)])
		sl.value = literal
		await _frames(2)
		await _dock_generate()
		var after: Dictionary = bridge.param_snapshot()
		var d3 := _digest()
		_check(absf(float(after[key]) - literal) < 1e-9, "%s: the user's continentality %s survives a dock Generate (engine %s)" % [archetype, str(literal), str(after[key])])
		_check(_near(sl, literal), "%s: the dock still shows the user's %s (shows %s)" % [archetype, str(literal), str(sl.value)])
		_check(str(d3) != str(d2), "%s: the world generated with the user's continentality differs from the preset's world" % archetype)


# -- labels (the MS-8 agent's open finding) ----------------------------------------------------------------------------------

func _labels_leg() -> void:
	_log("leg labels: Creates of one seed, no pass run, on a fresh app (the first Create has never had a world before it)")
	dlg.archetype_input.select(0)
	dlg._on_archetype_selected(0)
	dlg.seed_input.value = _label_seed
	dlg.seed_input.apply()
	dlg.grid_w_input.value = _grid
	dlg.grid_w_input.apply()
	await _frames(2)
	var carto: Node
	for w in app.get("_workspaces"):
		if (w.get_script() as Script).get_global_name() == "CartographyWorkspace":
			carto = w
	if carto == null:
		_check(false, "the CARTO workspace is mounted")
		return
	for i in 3:
		await _create()
		var at_create := var_to_str(bridge.labels_render_list()).sha256_text().substr(0, 10)
		var n_create: int = bridge.labels_render_list().size()
		var ppc: float = app.viewport.label_px_per_cell()
		# The same world, the same labels pass, rerun NOW (view settled): equal means the Create-time pass saw the same inputs.
		carto._regenerate_labels()
		await _frames(2)
		var again := var_to_str(bridge.labels_render_list()).sha256_text().substr(0, 10)
		var n_again: int = bridge.labels_render_list().size()
		_log("  create %d: %d labels at Create, %d on a rerun now; label_px_per_cell now=%s" % [i, n_create, n_again, str(ppc)])
		_check(at_create == again, "create %d: the labels the Create produced equal a rerun over the same world and view (%s vs %s)" % [i, at_create, again])
