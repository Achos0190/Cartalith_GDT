## Ruling FG measurement capture (scratch, never committed): the emboss findings' harness
## (`WORLD_EMBOSS_FINDINGS_2026-11-15.md` section 3) rebuilt, since its `_emboss_probe.gd` went with
## its worktree. Windowed only. One `--cfg tag,world,width_km,gw,gh` per world; every world is
## generated once per law in `--laws` (zoom_detail_km values, named on/off), and captured at each
## `--shots` entry: `fit`, `xN` (N times the fit zoom), `zN` (absolute camera zoom N), `sM`
## (M metres of ground per screen pixel). The view is centred on grid fraction `--pivot fx,fy`
## at every shot. Writes `<tag>_<on|off>_<shot>.png` (2560x1440) into `--out`, and one
## `META` line per shot (zoom, km/px, LOD level, static memory, host rect).
extends Node
var app: Node
var _vp: SubViewport
var _out := "C:/cw/fg_shots"
var _cfgs: Array = []
var _shots: PackedStringArray = ["fit", "x8", "x32", "s50"]
var _laws: PackedFloat64Array = [2.0, 0.0]
var _pivot := Vector2(0.5, 0.5)
var _seed := 1200

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _settle(host) -> void:
	await _frames(8)
	var t0 := Time.get_ticks_msec()
	while host.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 60000:
		await _frames(4)
	await _frames(12)

func _shot(name: String) -> void:
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png(_out.path_join(name))
	print("shot ", name)

func _gen(world: bool, width: float, gw: int, gh: int) -> void:
	var bridge = app.bridge
	bridge.param_set("world", world)
	var req := {"seed": _seed, "width_km": width, "grid_w": gw, "grid_h": gh, "archetype": "", "villages": true, "sea_level": 0.42}
	bridge.generate(req)
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	var t0 := Time.get_ticks_msec()
	while app.busy_overlay != null and app.busy_overlay.visible and Time.get_ticks_msec() - t0 < 60000:
		await _frames(10)
	await _frames(30)

func _km_per_px(host) -> float:
	var rect: Rect2 = host.overlay.displayed_rect()
	return host._width_km / maxf(1e-9, rect.size.x * host._zoom)

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--out": _out = args[i + 1]; i += 1
			"--cfg": _cfgs.append(args[i + 1].split(",")); i += 1
			"--shots": _shots = args[i + 1].split(","); i += 1
			"--laws":
				_laws = PackedFloat64Array()
				for s in args[i + 1].split(","): _laws.append(float(s))
				i += 1
			"--pivot":
				var p := args[i + 1].split(","); _pivot = Vector2(float(p[0]), float(p[1])); i += 1
			_:
				printerr("unknown argument ", args[i]); get_tree().quit(2); return
		i += 1
	DirAccess.make_dir_recursive_absolute(_out)
	var wd := get_tree().create_timer(5400.0)
	wd.timeout.connect(func(): print("WATCHDOG"); get_tree().quit(3))
	_vp = SubViewport.new()
	_vp.size = Vector2i(2560, 1440)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(4)
	DccSettings.set_marker_style("circles")
	for c in _cfgs:
		var tag: String = c[0]
		var world: bool = c[1] == "1"
		var width := float(c[2])
		var gw := int(c[3])
		var gh := int(c[4])
		var shots: PackedStringArray = c[5].split(";") if c.size() > 5 else _shots
		for law in _laws:
			var lname := "on" if law > 0.0 else "off"
			print("set_appearance -> ", app.bridge.set_appearance({"zoom_detail_km": law}))
			var t0 := Time.get_ticks_msec()
			await _gen(world, width, gw, gh)
			print("CFG ", tag, " world=", world, " width=", width, " grid=", gw, "x", gh, " law=", law, " gen_ms=", Time.get_ticks_msec() - t0,
				" max_level=", app.bridge.lod_max_level(), " budget=", app.bridge.lod_budget())
			var host = app.viewport
			var pg := Vector2(_pivot.x * gw, _pivot.y * gh)
			for s in shots:
				host.reset_view()
				await _frames(6)
				var fit_zoom: float = host._zoom
				var fit_kmpx := _km_per_px(host)
				var want := fit_zoom
				if s == "fit":
					want = fit_zoom
				elif s.begins_with("x"):
					want = fit_zoom * float(s.substr(1))
				elif s.begins_with("z"):
					want = float(s.substr(1)) / 100.0
				elif s.begins_with("s"):
					want = fit_zoom * fit_kmpx / (float(s.substr(1)) / 1000.0)
				var c0: Vector2 = host.size * 0.5
				## Zoom in steps of at most 2x, settling at each, as a user's wheel does:
				## a single jump past several levels leaves the incoming level blending
				## against the base map until its parent level lands (LOD-D3's rule in
				## `_refresh_lod_blend`), which is not what a zooming viewer sees.
				while host._zoom < want * 0.999:
					host._zoom_at(c0, minf(2.0, want / host._zoom))
					host.move_view_to(pg.x, pg.y)
					await _settle(host)
				host.move_view_to(pg.x, pg.y)
				var ts := Time.get_ticks_msec()
				await _settle(host)
				var r: Rect2 = host.get_global_rect()
				print("META ", tag, "_", lname, "_", s, " zoom=", host._zoom, " km_px=", _km_per_px(host), " fit_km_px=", fit_kmpx,
					" level=", app.bridge.lod_level_for_zoom(host._zoom * host.overlay.displayed_rect().size.x / float(gw)),
					" settle_ms=", Time.get_ticks_msec() - ts, " static_mem=", OS.get_static_memory_usage(), " host=", r)
				await _shot("%s_%s_%s.png" % [tag, lname, s])
	print("PROBE-RESULT: DONE")
	get_tree().quit(0)
