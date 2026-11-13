extends Node
## **River combs: frames and numbers from the app** (owner, 2026-11-13: *"Still see some heavy combing:"*, two phone
## screenshots of seed 51203; owner Ruling EV; `DELTA_GEOMETRY_RESEARCH.md` section 14).
##
## Generates ONE world in the app from a parameter table read from a file -- the app's own World info dump, one
## `key: value` per line, so the owner's world is rebuilt from exactly what the owner's phone printed -- plus `--set`
## overrides (the arms of a before/after), through `EngineBridge.generate` as a regenerate (`new_world` false), so the
## table is used exactly as written and Ruling EV's New World restore does not touch it. Then:
##   - reads every value back from `WorldGen.get_params()` and FAILS on a rejected, clamped or differing one (a world
##     that is not the one asked for must not report numbers);
##   - prints `WorldGen.river_comb` -- the comb and straight-run measure (`river_straight::comb_stats`,
##     `straight_stats`) on the traced network and on what Ruling EN's law draws at each zoom, over the whole map and
##     over `--window` -- and the settlement, road and delta counts (`get_settlements`, `get_roads`,
##     `river_delta_stats`: the consumers ran);
##   - saves the relief view and the River flow view (`ViewportHost.set_debug_layer("flow")`, the owner's first
##     screenshot) at each zoom of `--zooms`, centred on `--center`, as `<tag>_relief_z<zoom>.png` and
##     `<tag>_flow_z<zoom>.png`, and the numbers as `<tag>_comb.json`.
##
## A zoom is the camera zoom the readout shows as `zoom NN%` divided by 100 (`ViewportHost.zoom()`; the owner's
## screenshots read 788 %, so 7.88). Other overlays (territory, settlements, roads, ...) and the place names are hidden
## for the frames.
##
## MUST run windowed (`frame_post_draw` and `get_image` are dead under headless):
##   godot --path . res://_rivercomb_probe.tscn -- --out DIR --params FILE [--set k=v,k=v] [--seed 51203]
##       [--grid 2048x1024] [--km 40075] [--zooms 1,4,7.88,16] [--center 165,256] [--window 35,50,295,463]
##       [--vp 1600x1000] [--tag NAME]
## Defaults are the owner's 2026-11-13 world and view (seed 51203, 2048 x 1024, 40 075 km, centre cell (165,256), the
## window the two screenshots show, a 1600 x 1000 window, whose map view is about 950 x 834). Any other `--flag` aborts (exit 2). Exit 0 done, 1 a check
## failed, 2 could not run. The last line is `RESULT <tag> fail=<n> checks=<n>`.

const HIDE := ["territory", "provinces", "settlements", "roads", "sea_routes",
	"landmarks", "landmark_rejects", "urban_layouts", "conflict"]
const FLAGS := ["--out", "--params", "--set", "--seed", "--grid", "--km", "--zooms", "--center", "--window", "--vp", "--tag"]

var _out := ""
var _params_file := ""
var _sets := {}
var _seed := 51203
var _grid := Vector2i(2048, 1024)
var _km := 40075.0
var _zooms: Array = [1.0, 4.0, 7.88, 16.0]
var _center := Vector2(165, 256)
var _window := PackedFloat32Array([35, 50, 295, 463])
var _vp := Vector2i(1600, 1000)
var _tag := "comb"
var _app: Node
var _vh: Node
var _br: Node
var _fail := 0
var _checks := 0


func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])


func _check(ok: bool, what: String) -> void:
	_checks += 1
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])


func _quit(code: int) -> void:
	_log("RESULT %s fail=%d checks=%d" % [_tag, _fail, _checks])
	get_tree().quit(code)


## `text` as the value it spells: `true`/`false`, else a number. Every value the dump prints is one or the other.
func _value(text: String):
	var t := text.strip_edges()
	if t == "true":
		return true
	if t == "false":
		return false
	return float(t)


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if not (a in FLAGS) or i + 1 >= args.size():
			_log("ABORT unknown or valueless flag %s" % a)
			get_tree().quit(2)
			return
		var v := String(args[i + 1])
		match a:
			"--out": _out = v
			"--params": _params_file = v
			"--set":
				for kv in v.split(","):
					var pr := kv.split("=")
					_sets[pr[0]] = _value(pr[1])
			"--seed": _seed = int(v)
			"--grid":
				var g := v.split("x")
				_grid = Vector2i(int(g[0]), int(g[1]))
			"--km": _km = float(v)
			"--zooms":
				_zooms.clear()
				for z in v.split(","):
					_zooms.append(float(z))
			"--center":
				var c := v.split(",")
				_center = Vector2(float(c[0]), float(c[1]))
			"--window":
				_window = PackedFloat32Array()
				for w in v.split(","):
					_window.append(float(w))
			"--vp":
				var p := v.split("x")
				_vp = Vector2i(int(p[0]), int(p[1]))
			"--tag": _tag = v
		i += 2
	if _out == "" or _params_file == "":
		_log("ABORT --out and --params are required")
		get_tree().quit(2)
		return
	var table := {}
	var f := FileAccess.open(_params_file, FileAccess.READ)
	if f == null:
		_log("ABORT cannot read %s" % _params_file)
		get_tree().quit(2)
		return
	while not f.eof_reached():
		var line := f.get_line().strip_edges()
		var colon := line.find(":")
		if colon > 0:
			table[line.substr(0, colon).strip_edges()] = _value(line.substr(colon + 1))
	for k in _sets:
		table[k] = _sets[k]
	_log("parameters: %d from %s, overrides %s" % [table.size(), _params_file, JSON.stringify(_sets)])
	DisplayServer.window_set_size(_vp)
	await get_tree().process_frame
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_vh = _app.viewport
	_br = _app.bridge
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	DirAccess.make_dir_recursive_absolute(_out)
	var wrote: Dictionary = _br.world_gen.set_params(table)
	_check((wrote.get("rejected", []) as Array).is_empty() and (wrote.get("clamped", []) as Array).is_empty(),
		"every parameter written as given (rejected %s, clamped %s)" % [str(wrote.get("rejected", [])), str(wrote.get("clamped", []))])
	## The request's own switches are the table's, so `generate()` writes back the values just set.
	var req := {"seed": _seed, "width_km": _km, "grid_w": _grid.x, "grid_h": _grid.y, "archetype": "",
		"villages": bool(table.get("civ.villages", true)), "metropolis": bool(table.get("civ.metropolis", false)),
		"recovery_phase": int(table.get("civ.recovery_phase", 0)), "biome_k": bool(table.get("civ.biome_k", false)),
		"dynamic_lithology": bool(table.get("tect.dynamic_lithology", false)),
		"volcanic_provinces": bool(table.get("volc.provinces", false)),
		"wind_deflection": bool(table.get("climate.terrain_wind_deflection", false)),
		"ocean_currents": bool(table.get("climate.currents", false)),
		"sea_level": float(table.get("sea_level", 0.42))}
	var t0 := Time.get_ticks_msec()
	_br.generate(req)
	while _br.generating:
		await get_tree().create_timer(0.25).timeout
	_log("generated seed %d %dx%d %.0f km in %.1f s" % [_seed, _grid.x, _grid.y, _km, (Time.get_ticks_msec() - t0) / 1000.0])
	await get_tree().create_timer(1.0).timeout
	var back: Dictionary = _br.world_gen.get_params()
	var differ := []
	for k in table:
		var want = table[k]
		var got = back.get(k)
		var same: bool = got != null and (bool(got) == want if typeof(want) == TYPE_BOOL else absf(float(got) - float(want)) <= 1e-6 * maxf(1.0, absf(float(want))))
		if not same:
			differ.append("%s=%s (asked %s)" % [k, str(got), str(want)])
	_check(differ.is_empty(), "every parameter reads back as asked after the run (%d checked; differing %s)" % [table.size(), str(differ)])
	for l in HIDE:
		_vh.set_layer_visible(l, false)
	_vh.set_layer_visible("rivers", true)
	## The place and region names too (`labels.rs`' list; nothing re-sets it until a world or annotation edit).
	_vh.overlay.set_labels([])
	_vh.set_debug_layer("off")
	_vh.map_view.texture = _br.color_texture()
	await _settle(8)
	var zs := PackedFloat32Array()
	for z in _zooms:
		zs.append(float(z))
	var whole: Dictionary = _br.world_gen.river_comb(zs, PackedFloat32Array())
	var win: Dictionary = _br.world_gen.river_comb(zs, _window)
	## The delta fans are built by the paint above (`river_delta_stats` is `{}` before one): mouths, the mouths
	## eligible for a fan, and the fans drawn -- a consumer of the river network, like the settlements and roads.
	var deltas: Dictionary = _br.world_gen.river_delta_stats() if _br.world_gen.has_method("river_delta_stats") else {}
	var report := {"tag": _tag, "seed": _seed, "grid": [_grid.x, _grid.y], "km": _km, "overrides": _sets,
		"settlements": (_br.world_gen.get_settlements() as Array).size(), "roads": (_br.world_gen.get_roads() as Array).size(),
		"deltas": deltas, "whole": whole, "window": win, "window_cells": Array(_window)}
	_log("settlements %d, roads %d, river mouths %s, fan-eligible %s, delta fans %s" % [report["settlements"], report["roads"],
		str(deltas.get("mouths", "-")), str(deltas.get("eligible", "-")), str(deltas.get("fans", "-"))])
	for name in ["whole", "window"]:
		var d: Dictionary = report[name]
		for stage in ["traced"] + _zooms.map(func(z): return _zkey(z)):
			var tot := float(d.get("%s_total" % stage, 0.0))
			if tot <= 0.0:
				_log("  %-6s %-6s nothing drawn" % [name, stage])
				continue
			_log("  %-6s %-6s total %8.1f  comb %5.1f %% (longest %5.1f)  straight %5.1f %% (longest %5.1f)" % [name, stage, tot,
				100.0 * float(d.get("%s_comb" % stage, 0.0)) / tot, float(d.get("%s_comb_longest" % stage, 0.0)),
				100.0 * float(d.get("%s_straight" % stage, 0.0)) / tot, float(d.get("%s_straight_longest" % stage, 0.0))])
	_check(float(whole.get("traced_total", 0.0)) > 0.0, "the world has traced rivers (positive control: an empty measure cannot pass)")
	for z in _zooms:
		_check(float(whole.get("%s_total" % _zkey(z), 0.0)) > 0.0, "rivers are drawn at zoom %s" % str(z))
	var jf := FileAccess.open(_out.path_join("%s_comb.json" % _tag), FileAccess.WRITE)
	jf.store_string(JSON.stringify(report, "  "))
	jf.close()
	for view in ["relief", "flow"]:
		_vh.set_debug_layer("off" if view == "relief" else "flow")
		_check(_vh.debug_view() == ("off" if view == "relief" else "flow"), "the %s view is drawn" % view)
		for z in _zooms:
			_vh.reset_view()
			await _settle(3)
			_vh.zoom_step(float(z) / _vh.zoom())
			_vh.move_view_to(_center.x, _center.y)
			await _settle_lod()
			var img := await _grab()
			_check(img != null, "frame %s x%s grabbed" % [view, str(z)])
			if img != null:
				img.save_png(_out.path_join("%s_%s_z%s.png" % [_tag, view, ("%05.2f" % float(z))]))
				_log("  frame %s zoom %.2f (readout %d %%) saved" % [view, _vh.zoom(), roundi(_vh.zoom() * 100.0)])
	_quit(1 if _fail > 0 else 0)


## The stage key `WorldGen.river_comb` writes for zoom `z`: Rust prints an `f32` with no trailing `.0` (`z1`, `z7.88`),
## where GDScript's `str(1.0)` is `1.0`.
func _zkey(z: float) -> String:
	return "z%d" % roundi(z) if is_equal_approx(z, roundf(z)) else "z%s" % str(z)


## The view host's region of the window, after the next frame is drawn.
func _grab() -> Image:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img == null:
		return null
	var gp: Vector2 = _vh.global_position
	var r := Rect2i(Vector2i(gp.round()), Vector2i(_vh.size.round())).intersection(Rect2i(0, 0, img.get_width(), img.get_height()))
	var sub := img.get_region(r)
	sub.convert(Image.FORMAT_RGBA8)
	return sub


## Until the deep-zoom pyramid has caught up (`lod_pending() == 0`, a condition, not a frame count); 20 s cap.
func _settle_lod() -> void:
	await _settle(6)
	var t0 := Time.get_ticks_msec()
	while _vh.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 20000:
		await get_tree().process_frame
	await get_tree().create_timer(0.6).timeout
	await _settle(4)


func _settle(n: int) -> void:
	for i in n:
		await RenderingServer.frame_post_draw
