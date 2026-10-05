extends Node
## Rivers as disconnected segments -- `OUTSTANDING_WORK.md` §2.3, owner-reported
## 2026-09-23, cause unverified. Live half of `tests/river_gap_harness.rs`:
## that harness counts gaps on the plan the renderer is handed; this probe
## counts them on what `get_rivers()` actually returns (the RDP + spline render
## points and the `pieces` the viewport strokes) and takes screenshots of the
## worst ones.
##
## It MEASURES; it asserts only that rivers exist (a probe that cannot fail on
## an empty world proves nothing). A drawn run's LOOSE END is its last drawn
## render point when that point is on land, has no water in its 3x3, and no
## other drawn run's traced cell lies within 1.5 cells of it. Each is sized by
## the distance to the nearest other drawn cell. The shipped bridge reach is one
## D8 step (`river_draw_plan`), so a gap above 1.5 is one the plan left open.
##
## MUST run WINDOWED (`MISTAKES.md`, "Run a pixel probe"):
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 \
##       _rivergap_probe.tscn -- --out DIR [--seeds 483920,24601] [--shots 3] [--km 800]
##                              [--sites X:Y,X:Y]
##
## `--sites` pins the screenshot sites (grid cells of the first seed) so a
## BEFORE and an AFTER run photograph the same places: without it the sites are
## the worst remaining loose ends, which after the continuation fix are
## different (and far fewer) ones.
##
## Prints PROBE-FAIL (and still exits 0) on a failed assertion; grep for it.

var _out := "user://rivergap/"
var _fails := 0
var _seeds: Array[int] = [483920, 24601, 71077345]
var _shots := 3
var _km := 800.0
var _sites_arg: Array[Vector2] = []
## Gap-size histogram upper edges, in cells: the plan's bridge reach (1.5), then
## a measurement ladder. Last bin is open-ended.
const EDGES := [1.5, 3.0, 6.0, 12.0, 24.0]


func _ok(cond: bool, what: String) -> void:
	print(("  PASS  " if cond else "  PROBE-FAIL  ") + what)
	if not cond:
		_fails += 1


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless -- run windowed.")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := args[i]
		if a == "--out" and i + 1 < args.size():
			_out = args[i + 1]
			i += 1
		elif a == "--seeds" and i + 1 < args.size():
			_seeds.clear()
			for s in args[i + 1].split(","):
				_seeds.append(int(s))
			i += 1
		elif a == "--shots" and i + 1 < args.size():
			_shots = int(args[i + 1])
			i += 1
		elif a == "--sites" and i + 1 < args.size():
			_sites_arg.clear()
			for s in args[i + 1].split(","):
				var xy := s.split(":")
				_sites_arg.append(Vector2(float(xy[0]), float(xy[1])))
			i += 1
		elif a == "--km" and i + 1 < args.size():
			_km = float(args[i + 1])
			i += 1
		else:
			printerr("PROBE-CANNOT-RUN: unknown argument %s" % a)
			get_tree().quit(2)
			return
		i += 1
	if not _out.ends_with("/"):
		_out += "/"
	DirAccess.make_dir_recursive_absolute(_out)

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var vh: Control = app.viewport
	var br: Node = app.bridge
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await get_tree().process_frame
	print("integrate_drainage (app param table): %s" % str(br.param_get("integrate_drainage")))
	_ok(br.param_get("integrate_drainage") == true, "the app boots with integrate_drainage ON")

	var first := true
	for seed_value in _seeds:
		print("generating seed %d, 2048 x 1311, %.0f km..." % [seed_value, _km])
		br.generate({"seed": seed_value, "width_km": _km, "grid_w": 2048, "grid_h": 1311,
			"archetype": "", "villages": true, "sea_level": 0.42})
		var spins := 0
		while br.generating and spins < 2400:
			await get_tree().create_timer(0.25).timeout
			spins += 1
		if br.generating:
			printerr("PROBE-CANNOT-RUN: generation never finished.")
			get_tree().quit(2)
			return
		await get_tree().create_timer(1.0).timeout
		vh.reset_view()
		vh.set_debug_layer("off")
		await _settle(5)
		var sites := _measure(br, seed_value)
		## The delta fans of the drawn network (`river_delta_stats`; built
		## lazily on the first paint that wants them), so a BEFORE and an AFTER
		## DLL show whether the downhill continuation moved them.
		var stats: Dictionary = br.world_gen.river_delta_stats()
		if stats.is_empty():
			vh.map_view.texture = br.color_texture()
			await _settle(8)
			stats = br.world_gen.river_delta_stats()
		print("seed %d delta fans: mouths %s eligible %s fans %s branches %s dropped_dry %s skipped %s"
			% [seed_value, str(stats.get("mouths")), str(stats.get("eligible")), str(stats.get("fans")),
			str(stats.get("branches")), str(stats.get("dropped_dry")), str(stats.get("skipped"))])
		if first:
			first = false
			if not _sites_arg.is_empty():
				sites = _sites_arg
			for s in range(mini(_shots, sites.size())):
				await _shoot(vh, sites[s], "s%d_site%d" % [seed_value, s])
			## And the whole map, rivers on, for the overall look.
			vh.reset_view()
			await _settle(10)
			get_viewport().get_texture().get_image().save_png("%ss%d_fit.png" % [_out, seed_value])
	print("\n_rivergap_probe: %s (%d failures); images in %s"
		% ["ALL PASS" if _fails == 0 else "PROBE-FAIL FAILURES", _fails, ProjectSettings.globalize_path(_out)])
	get_tree().quit(0)


## Counts loose ends on `br.rivers(1)` and returns the worst ones' grid
## positions (largest gap first, among those whose gap is 3..12 cells so a
## screenshot at deep zoom can show both sides).
## The probe's grid width (`grid_w` in the `generate` call); x wraps at it.
const GRID_W := 2048
## And its height (`grid_h`); y does not wrap.
const GRID_H := 1311


func _measure(br: Node, seed_value: int) -> Array[Vector2]:
	var rivers: Array = br.rivers(1)
	_ok(rivers.size() > 10, "seed %d: the world has rivers (%d runs)" % [seed_value, rivers.size()])
	var drawn_cells := {}   ## Vector2i -> run index, drawn runs' traced cells
	for k in rivers.size():
		var r: Dictionary = rivers[k]
		if r.has("parallel_of"):
			continue
		for p in r["points"]:
			var c := Vector2i(p)
			if not drawn_cells.has(c):
				drawn_cells[c] = k
	## The runs' DRAWN render points too, after every traced cell (first writer
	## stays the traced one): a run that ends on another run's downhill
	## continuation ends on a cell that is not a traced one.
	for k in rivers.size():
		var r: Dictionary = rivers[k]
		if r.has("parallel_of"):
			continue
		for p in r["render_points"]:
			var c := Vector2i(p)
			if not drawn_cells.has(c):
				drawn_cells[c] = k
	var drawn := 0
	var joined := 0
	var at_water := 0
	var loose: Array = []   ## [gap, Vector2 end, run]
	for k in rivers.size():
		var r: Dictionary = rivers[k]
		if r.has("parallel_of"):
			continue
		drawn += 1
		var rp: PackedVector2Array = r["render_points"]
		var pcs: PackedInt32Array = r["pieces"]
		if rp.size() == 0 or pcs.size() < 2:
			continue
		## The last DRAWN render point: the end of the last piece.
		var last_ix: int = clampi(pcs[pcs.size() - 1] - 1, 0, rp.size() - 1)
		var e := rp[last_ix]
		var ec := Vector2i(e)
		if drawn_cells.get(ec, k) != k:
			joined += 1
			continue
		var wet := false
		for dy in range(-1, 2):
			for dx in range(-1, 2):
				var w := String(br.sample_cell(posmod(ec.x + dx, GRID_W), ec.y + dy).get("water", "land"))
				if w != "land":
					wet = true
		if wet:
			at_water += 1
			continue
		var best := 1.0e9
		for dy in range(-24, 25):
			for dx in range(-24, 25):
				## The world wraps in x (not in y): an end on the seam looks across it.
				var o: int = drawn_cells.get(Vector2i(posmod(ec.x + dx, GRID_W), ec.y + dy), -1)
				if o == -1 or o == k:
					continue
				var d := Vector2(dx, dy).length()
				if d < best:
					## Not one of this run's own tributaries.
					var op: PackedVector2Array = rivers[o]["points"]
					if drawn_cells.get(Vector2i(op[op.size() - 1]), -1) == k:
						continue
					best = d
		loose.append([best, e, k])
	var bins := []
	bins.resize(EDGES.size() + 1)
	bins.fill(0)
	var beyond := 0
	## Ends ON the map's border: a river that leaves the map (the seam in x, the
	## edge in y) is not a loose end on dry land, so they are counted apart.
	var on_border := 0
	for l in loose:
		var le: Vector2 = l[1]
		if le.x < 1.0 or le.x > GRID_W - 1.0 or le.y < 1.0 or le.y > GRID_H - 2.0:
			on_border += 1
		var b := EDGES.size()
		for j in EDGES.size():
			if l[0] <= EDGES[j]:
				b = j
				break
		bins[b] += 1
		if l[0] > 1.5:
			beyond += 1
	print("seed %d LIVE get_rivers(1): drawn runs %d; ends joined %d / at water %d / loose %d (gap > 1.5 cells: %d)"
		% [seed_value, drawn, joined, at_water, loose.size(), beyond])
	print("  loose gap histogram (cells) edges %s: %s (last bin open); %d of the loose ends lie on the map border"
		% [str(EDGES), str(bins), on_border])
	loose.sort_custom(func(a, b): return a[0] > b[0])
	var sites: Array[Vector2] = []
	for l in loose:
		if l[0] >= 3.0 and l[0] <= 12.0:
			sites.append(l[1])
		if sites.size() >= 6:
			break
	for s in sites:
		print("  site %s" % str(s))
	return sites


## Screenshots one site at three zooms (a wide look, a mid one, the deep one the
## owner would see when following a river), rivers on.
func _shoot(vh: Control, site: Vector2, tag: String) -> void:
	for z in [3.0, 8.0, 20.0]:
		vh.reset_view()
		await _settle(3)
		vh.zoom_step(z / vh.zoom())
		vh.move_view_to(site.x, site.y)
		await _settle(60)
		vh.set_layer_visible("rivers", true)
		await _settle(4)
		get_viewport().get_texture().get_image().save_png("%s%s_z%d.png" % [_out, tag, int(z)])


func _settle(n: int) -> void:
	for f in n:
		await RenderingServer.frame_post_draw
