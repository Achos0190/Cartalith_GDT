extends Node
## Ruling CW (VA-1..VA-4, `VOLCANO_APPEARANCE_RESEARCH.md`): the volcano colours
## reach the SCREEN, under Satellite relief only, and the Sample panel's cell
## dictionary carries the per-volcano attributes.
##
##   Godot_v4.7.1-stable_win64_console.exe --path <scratch copy> res://_volcanolook_probe.tscn -- --out DIR
##
## What it protects, on a real generated world (seed 24601, 800 km, 512x328,
## the app's own parameters, so the geology model and its volcano table are on):
## 1. `sample_cell` reports `volcano_kind` / `volcano_age_years` /
##    `volcano_activity` on at least one volcanic cell, with a known kind and
##    activity, and omits all three on a cell off every edifice.
## 2. Under the Satellite relief preset the engine's map texture AND the
##    viewport's own pixels move when `volcano_look` goes 1 -> 0, and only near
##    the volcanoes found in 1 (the screen leg: a texture-only check cannot
##    see the screen, `MISTAKES.md`).
## 3. Natural Vibrant carries `volcano_look` 0 and Satellite relief 1 (the
##    stage's only gate; identity at 0 is the Rust suite's).
## Writes `volcano_on.png` / `volcano_off.png` (viewport captures) to DIR.
## Result: `VOLC  ok|PROBE-FAIL ...` per check and `VOLC  done  N checks, M failed`;
## exit 0 = every check held, 1 = a check failed, 2 = could not run (headless,
## no `--out`, no world, no RenderWorkspace, watchdog). Windowed only.

const RW := preload("res://shell/workspaces/render_workspace.gd")

var GRID := Vector2i(512, 328)
var _out := ""
var _fail := 0
var _checks := 0
var _app: Node
var _vh: Control
var _br: Node
var _rw: Node
var _tiles: Array = []


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if cond:
		print("VOLC  ok    " + what)
	else:
		_fail += 1
		print("PROBE-FAIL VOLC  " + what)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _find(n: Node, script_file: String) -> Node:
	if n.get_script() != null and String(n.get_script().resource_path).ends_with(script_file):
		return n
	for c in n.get_children(true):
		var r := _find(c, script_file)
		if r != null:
			return r
	return null


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless -- run windowed.")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--out" and i + 1 < args.size():
			_out = args[i + 1]
			i += 2
			continue
		printerr("PROBE-CANNOT-RUN: unknown argument %s (this probe reads --out only)" % args[i])
		get_tree().quit(2)
		return
	if _out == "":
		printerr("PROBE-CANNOT-RUN: --out DIR is required")
		get_tree().quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		printerr("PROBE-CANNOT-RUN: watchdog"); get_tree().quit(2))
	DisplayServer.window_set_size(Vector2i(1600, 1000))
	await _frames(2)
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_vh = _app.viewport
	_br = _app.bridge
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	_br.generate({"seed": 24601, "width_km": 800.0, "grid_w": GRID.x, "grid_h": GRID.y,
		"archetype": "", "villages": false, "sea_level": 0.42})
	var spins := 0
	while _br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	await get_tree().create_timer(1.0).timeout
	if _br.generating:
		printerr("PROBE-CANNOT-RUN: no generated world")
		get_tree().quit(2)
		return
	_app.select_domain_category("cartography", "Style")
	await _frames(8)
	_rw = _find(_app, "render_workspace.gd")
	if _rw == null:
		printerr("PROBE-CANNOT-RUN: no RenderWorkspace")
		get_tree().quit(2)
		return
	_tiles = _rw.get("_preset_tiles")
	_vh.reset_view()
	await _settle()
	await _leg_sample()
	await _leg_screen()
	print("VOLC  done  %d checks, %d failed" % [_checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)


func _settle() -> void:
	await _frames(8)
	var t0 := Time.get_ticks_msec()
	while _vh.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 20000:
		await get_tree().process_frame
	await get_tree().create_timer(0.3).timeout
	await _frames(4)


func _idx(n: String) -> int:
	for k in RW.STYLE_PRESETS.size():
		if String(RW.STYLE_PRESETS[k][0]) == n:
			return k
	return -1


func _press_base(k: int) -> void:
	(_tiles[k]["button"] as Button).emit_signal("pressed")
	await _settle()


func _tex_bytes() -> PackedByteArray:
	var t: Texture2D = _vh.map_view.texture
	if t == null:
		return PackedByteArray()
	var im := t.get_image()
	im.convert(Image.FORMAT_RGB8)
	return im.get_data()


func _screen() -> Image:
	await RenderingServer.frame_post_draw
	var im := get_viewport().get_texture().get_image()
	im.convert(Image.FORMAT_RGB8)
	return im


func _diff(a: PackedByteArray, b: PackedByteArray) -> int:
	if a.size() != b.size():
		return -1
	var n := 0
	for k in range(0, a.size(), 3):
		if absi(a[k] - b[k]) + absi(a[k + 1] - b[k + 1]) + absi(a[k + 2] - b[k + 2]) > 6:
			n += 1
	return n


var _volc_cells: Array = []


## Leg 1: the cell dictionary.
func _leg_sample() -> void:
	# Ruling DJ added "supervolcano" and "flood_basalt" (`EdificeKind::key`).
	var kinds := ["cinder", "strato", "shield", "dome", "maar", "supervolcano", "flood_basalt"]
	var acts := ["active", "dormant", "extinct"]
	var bare_seen := false
	for gy in range(0, GRID.y, 6):
		for gx in range(0, GRID.x, 6):
			var c: Dictionary = _br.sample_cell(gx, gy)
			if c.has("volcano_activity"):
				_volc_cells.append(Vector2i(gx, gy))
				if _volc_cells.size() == 1:
					_ok(kinds.has(String(c.get("volcano_kind", ""))), "a volcanic cell names its kind (%s)" % String(c.get("volcano_kind", "-")))
					_ok(acts.has(String(c["volcano_activity"])), "and its activity (%s)" % String(c["volcano_activity"]))
					var yrs := float(c.get("volcano_age_years", -1.0))
					_ok(yrs >= 10.0 and yrs <= 3.2e6, "and an age in the approved 10 yr .. 3 Myr span (%.0f)" % yrs)
			elif not bare_seen and not c.is_empty():
				bare_seen = true
				_ok(not c.has("volcano_kind") and not c.has("volcano_age_years"), "a cell off every edifice omits the volcano keys")
	_ok(_volc_cells.size() > 0, "the world has volcanic cells (%d sampled)" % _volc_cells.size())


## Legs 2 and 3: the screen.
func _leg_screen() -> void:
	var sat := _idx("Satellite relief")
	var vib := _idx("Natural Vibrant")
	if sat < 0 or vib < 0:
		_ok(false, "the Satellite relief and Natural Vibrant presets exist")
		return
	await _press_base(sat)
	print("VOLC  info  after the Satellite relief preset: volcano_look = %s, rock_slope_warm_deg = %s" % [str(_br.appearance().get("volcano_look", "missing")), str(_br.appearance().get("rock_slope_warm_deg", "missing"))])
	var on_tex := _tex_bytes()
	var on_img: Image = await _screen()
	# `set_appearance` only stores the override; the workspace's own
	# `_refresh_map` is what re-renders (as `_apply_preset` does).
	_br.set_appearance({"volcano_look": 0.0})
	_rw.call("_refresh_map")
	await _settle()
	var off_tex := _tex_bytes()
	var off_img: Image = await _screen()
	on_img.save_png(_out.path_join("volcano_on.png"))
	off_img.save_png(_out.path_join("volcano_off.png"))
	var dt := _diff(on_tex, off_tex)
	var ds := _diff(on_img.get_data(), off_img.get_data())
	_ok(dt > 50, "Satellite relief: the map texture moves with volcano_look (%d texels)" % dt)
	_ok(ds > 50, "Satellite relief: the SCREEN moves with volcano_look (%d pixels)" % ds)
	# Only on or beside volcanoes. The per-cell colour moves only on an
	# edifice (the Rust suite's `ground_off_the_edifices_is_byte_identical`);
	# the finished texture then shows changes a few cells out too (measured
	# 2026-10-24: 431 moved texels with no edifice cell within one cell, none
	# beyond 12; the cause is presumably the finishing stages' neighbourhood
	# reads, local contrast and grade influence -- not separately measured),
	# so each moved texel must
	# lie within `HALO` cells of a cell `sample_cell` places on an edifice.
	# HALO 12 is a judgement: the Satellite relief look's local-contrast
	# radius at 512 cells. A stage leaking over the map fails it.
	const HALO := 12
	var far := 0
	var max_mag := 0
	var cache := {}
	var w: int = _vh.map_view.texture.get_width() if _vh.map_view.texture != null else 0
	if w == GRID.x and on_tex.size() == off_tex.size():
		for k in range(0, on_tex.size(), 3):
			var mag := absi(on_tex[k] - off_tex[k]) + absi(on_tex[k + 1] - off_tex[k + 1]) + absi(on_tex[k + 2] - off_tex[k + 2])
			if mag > 6:
				var p := Vector2i((k / 3) % w, (k / 3) / w)
				var near := false
				for dy in range(-HALO, HALO + 1, 2):
					for dx in range(-HALO, HALO + 1, 2):
						var q: Vector2i = p + Vector2i(dx, dy)
						if q.x < 0 or q.y < 0 or q.x >= GRID.x or q.y >= GRID.y:
							continue
						if not cache.has(q):
							cache[q] = Dictionary(_br.sample_cell(q.x, q.y)).has("volcano_activity")
						if cache[q]:
							near = true
							break
					if near:
						break
				if not near:
					far += 1
					max_mag = maxi(max_mag, mag)
		print("VOLC  info  moved texels with no edifice cell within %d cells: %d (largest change %d)" % [HALO, far, max_mag])
		_ok(far == 0, "no texel moved more than HALO cells from an edifice (%d did)" % far)
	# Natural Vibrant: the look carries the stage at 0 (it is reachable only
	# through the look that sets it; identity at 0 is the Rust suite's
	# `every_other_look_is_byte_identical_with_the_table_attached`).
	_br.drop_appearance_overrides(PackedStringArray(["volcano_look"]))
	await _press_base(vib)
	var ap: Dictionary = _br.appearance()
	_ok(float(ap.get("volcano_look", -1.0)) == 0.0, "Natural Vibrant carries volcano_look 0 (%s)" % str(ap.get("volcano_look", "missing")))
	await _press_base(sat)
	ap = _br.appearance()
	_ok(float(ap.get("volcano_look", -1.0)) == 1.0, "Satellite relief carries volcano_look 1 (%s)" % str(ap.get("volcano_look", "missing")))
