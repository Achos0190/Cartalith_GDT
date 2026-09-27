extends Node
## Windowed regression probe for "Droplet erosion after an undo reads the
## undone surface's rainfall" (`OUTSTANDING_WORK.md`), fixed 2026-09-29:
## `cartalith_godot::erode_bridge::run_erode_with_recompute` (called from
## `WorldGen::erode_op`, which `world_workspace.gd::_run_erode()`'s Erode
## button drives) now flushes any stale hydrology/climate BEFORE the droplets
## read `ws.rainfall`, not only after -- so an undo's own
## `mark_changed_tiles(Height, ..., "undo")` (`WorldGen::undo_last`) no longer
## leaks a stale rainfall into the next erode's droplet spawn.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x900 _dropletundo_probe.tscn
##
## **Windowed**, not `--headless`: saves the framebuffer before and after each
## erode (`MISTAKES.md`, "Run a pixel probe").
##
## Real shell path: `WorldGen.erode_op({})` -- the reference's own defaults
## (`ErodeOpts::default`, droplets 60000, climate-coupled whenever the world
## carries rainfall, which a fresh `generate()` always does) -- through
## `_app.undo_last()`, the same call the Undo chip/menu item makes (repaints
## included). `use_gpu` forced off throughout: this probe is about the
## droplet/rainfall ordering, not GPU determinism (`_thermalgpu_probe.gd`
## already covers that).
##
## Three erodes off the SAME starting field, each preceded by an undo back to
## it: A vs B is the fix under test (erode -> undo -> erode); B vs C is the
## CPU-vs-CPU control -- a second undo/erode cycle, so a match between A and B
## cannot be a two-run coincidence.
##
## Exit 0 pass, 1 an assertion failed, 2 the premise could not be set up.

const SEED := 771001

var _app: Node
var _bridge: Node
var _fail := 0


func _p(s: String) -> void:
	print(s)


func _ok(name: String, cond: bool, detail: String = "") -> void:
	print("  ", "ok  " if cond else "FAIL", " ", name, ("  -- " + detail) if detail != "" else "")
	if not cond:
		_fail += 1


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


## Every cell's elevation, drainage (`flow_discharge`) and precipitation
## (`rainfall`), as `sample_cell` reports them -- the three fields
## `refresh_climate` writes and the droplets read.
func _fields() -> Dictionary:
	var gw: int = _bridge.world_gen.get_width()
	var gh: int = _bridge.world_gen.get_height()
	var e := PackedFloat64Array()
	var dr := PackedFloat64Array()
	var pr := PackedFloat64Array()
	e.resize(gw * gh)
	dr.resize(gw * gh)
	pr.resize(gw * gh)
	for y in gh:
		for x in gw:
			var s: Dictionary = _bridge.world_gen.sample_cell(x, y)
			e[y * gw + x] = float(s.get("elevation", -1.0))
			dr[y * gw + x] = float(s.get("drainage", -1.0))
			pr[y * gw + x] = float(s.get("precipitation", -1.0))
	return {"e": e, "d": dr, "p": pr}


func _count_diff(a: PackedFloat64Array, b: PackedFloat64Array) -> int:
	var n := 0
	for i in a.size():
		if a[i] != b[i]:
			n += 1
	return n


func _shot(path: String) -> void:
	await _frames(6)
	var img := get_viewport().get_texture().get_image()
	img.save_png(path)


## The real Erode button's own engine call (`world_workspace.gd::_run_erode`
## calls exactly this with its own `_erode_op` dictionary; the reference
## defaults used here, `{}`, are the same dictionary's own starting values,
## `ERODE_DEFAULTS`), then the same two repaint lines `_run_erode` makes.
func _erode() -> Dictionary:
	var r: Dictionary = _bridge.world_gen.erode_op({})
	_app.viewport.map_view.texture = _bridge.color_texture()
	_app.viewport.invalidate_lod_tiles()
	return r


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	_bridge = _app.bridge
	_bridge.generate({
		"seed": SEED, "width_km": 900.0, "grid_w": 128, "grid_h": 96,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if not _bridge.has_world or not _bridge._has("erode_op"):
		_p("!! no world, or no erode_op in this build")
		get_tree().quit(2)
		return

	_bridge.param_set("use_gpu", false)
	_ok("use_gpu forced off (CPU-only comparison)", not bool(_bridge.world_gen.get_params().get("use_gpu", true)))

	var f0 := _fields()
	await _shot("user://dropletundo_f0.png")

	var rA := _erode()
	var fA := _fields()
	await _shot("user://dropletundo_fA.png")
	_ok("erode A reports ok", bool(rA.get("ok", false)), str(rA))
	_ok("erode A actually moved the surface (positive control)", _count_diff(f0["e"], fA["e"]) > 0)
	_ok("erode A's droplets were climate-coupled -- the read this defect is about", bool(rA.get("climate_coupled", false)), str(rA))

	_app.undo_last()
	await _frames(6)
	_ok("undo restored the starting field exactly", _fields()["e"] == f0["e"])

	var rB := _erode()
	var fB := _fields()
	await _shot("user://dropletundo_fB.png")
	_ok("erode B reports ok", bool(rB.get("ok", false)), str(rB))

	_app.undo_last()
	await _frames(6)
	_ok("second undo also restores the starting field exactly", _fields()["e"] == f0["e"])

	var rC := _erode()
	var fC := _fields()
	await _shot("user://dropletundo_fC.png")
	_ok("erode C reports ok", bool(rC.get("ok", false)), str(rC))

	var de_ab := _count_diff(fA["e"], fB["e"])
	var dd_ab := _count_diff(fA["d"], fB["d"])
	var dp_ab := _count_diff(fA["p"], fB["p"])
	_p("  A vs B (the fix under test): elevation differ %d, drainage differ %d, precipitation differ %d (of %d cells)"
		% [de_ab, dd_ab, dp_ab, fA["e"].size()])
	_ok("THE FIX: elevation is bit-identical across erode -> undo -> erode, with droplets on", de_ab == 0, "%d cells differ" % de_ab)
	_ok("drainage (flow_discharge) is bit-identical too", dd_ab == 0, "%d cells differ" % dd_ab)
	_ok("precipitation (rainfall) is bit-identical too", dp_ab == 0, "%d cells differ" % dp_ab)

	var de_bc := _count_diff(fB["e"], fC["e"])
	var dd_bc := _count_diff(fB["d"], fC["d"])
	var dp_bc := _count_diff(fB["p"], fC["p"])
	_p("  B vs C (CPU-vs-CPU control): elevation differ %d, drainage differ %d, precipitation differ %d"
		% [de_bc, dd_bc, dp_bc])
	_ok("CPU-vs-CPU CONTROL: a second undo/erode cycle is bit-identical too, not a two-run coincidence",
		de_bc == 0 and dd_bc == 0 and dp_bc == 0, "e=%d d=%d p=%d" % [de_bc, dd_bc, dp_bc])

	_p("### DROPLETUNDO %s  ###" % ("PASS" if _fail == 0 else "FAIL (%d)" % _fail))
	get_tree().quit(0 if _fail == 0 else 1)
