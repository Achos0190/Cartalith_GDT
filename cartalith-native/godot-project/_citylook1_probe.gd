extends Node
## CITY_MAP_LOOK_SCOPE.md CL-1 verification: the map look of a town plan, in the real shell, against the ink plan.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . --resolution 1600x1000 _citylook1_probe.tscn -- --out <dir> [--vp 1600x1000]
##
## **Windowed, never `--headless`** (it reads rendered pixels; `ImageTexture.update()` is a no-op headless), and run
## from a project copy with its OWN `config/name` and a wiped user dir, so the `city_look` setting it flips is never
## the owner's. The harness refuses to run under `--headless`.
##
## What it protects (every check names it; a positive control backs each pixel claim):
##   1. TWO TERRAINS: two walled towns on measurably different ground (`layout["ground_color"]` distance above
##      `MIN_GROUND_DIST`), the second preferably with a citadel.
##   2. GROUND = TERRAIN: `ground_color` equals the mean of the DISPLAYED map texture over the same nine cells, read
##      independently (a different image, a different code path from the bridge's cache), to within 0.02; and the
##      rendered City Viewer corner pixels match `ground_color` in the map look but are the fixed parchment in the
##      ink look.
##   3. BOTH LOOKS DRAW: the frames differ heavily between looks and each has many distinct colours.
##   4. THE SWITCH PERSISTS: choosing a look in the City Viewer's dropdown writes `user://cartalith_settings.cfg`
##      (read back with a fresh `ConfigFile`), and the other choice writes the other value.
##   5. THE CITADEL IS DRAWN IN FULL: at the citadel, the screen pixel at a tower's centre, at the middle of the
##      outer curtain face and at the keep's centre are the look's tower cap, stone and keep roof colours, and in the
##      ink look at the same place they are not.
##   6. EXPORT USES THE CHOSEN LOOK: a tile of the export overlay (`ExportOverlay`, the code the 16K/32K export
##      runs) cut round a town holds the map look's dark roof colour many times over in the map look compared with the ink look (the ink plan's dark casing ink falls within tolerance of a few pixels, so the bar is a ratio of 8, not zero).
## Frames: `<out>/{map,ink}_<city>_z<zoom>.png` (city a, b), and the composite `<out>/composite.png` (rows: a map,
## a ink, b map, b ink; columns: zoom 1, 3, 6, 12).
## Result: `RESULT: PASS|FAIL (<n> failed) checks=<n>`; exit 0 pass, 1 fail, 2 bad argument or watchdog.

const DRAW := preload("res://shell/urban_layout_draw.gd")
const LOOK := preload("res://shell/urban_layout_look.gd")
const EXPORT_OVERLAY := preload("res://shell/export_overlay.gd")

## Two towns count as on "different terrain" when their ground colours are at least this far apart (RGB distance).
const MIN_GROUND_DIST := 0.10
## The City Viewer's zooms we frame (its own range is 0.25..12; this is its equivalent of the map's x1..x64).
const ZOOMS := [1.0, 3.0, 6.0, 12.0]
## Pixel tolerance for "this is the look's colour" (RGB, each channel).
const TOL := 0.06

var _out := ""
var _vp: SubViewport
var _app: Node
var _fails := 0
var _checks := 0

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  PASS  %s" if cond else "  FAIL  %s") % what)
	if not cond:
		_fails += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _near(a: Color, b: Color, tol: float = TOL) -> bool:
	return absf(a.r - b.r) <= tol and absf(a.g - b.g) <= tol and absf(a.b - b.b) <= tol

func _shot(name: String, rect: Rect2) -> Image:
	await RenderingServer.frame_post_draw
	var img: Image = _vp.get_texture().get_image()
	var crop := img.get_region(Rect2i(rect))
	crop.save_png(_out.path_join(name + ".png"))
	return crop

func _distinct(img: Image) -> int:
	var seen := {}
	for y in range(0, img.get_height(), 3):
		for x in range(0, img.get_width(), 3):
			var c := img.get_pixel(x, y)
			seen[Vector3i(int(c.r * 15.0), int(c.g * 15.0), int(c.b * 15.0))] = true
	return seen.size()

func _diff_frac(a: Image, b: Image) -> float:
	var n := 0
	var d := 0
	for y in range(0, mini(a.get_height(), b.get_height()), 4):
		for x in range(0, mini(a.get_width(), b.get_width()), 4):
			n += 1
			if not _near(a.get_pixel(x, y), b.get_pixel(x, y), 0.02):
				d += 1
	return float(d) / float(maxi(1, n))

func _find_script(n: Node, file: String) -> Node:
	for c in n.get_children():
		if c.get_script() != null and String(c.get_script().resource_path).ends_with(file):
			return c
		var r := _find_script(c, file)
		if r != null:
			return r
	return null

func _stored_look() -> String:
	var cf := ConfigFile.new()
	cf.load(DccSettings.CONFIG_PATH)
	return String(cf.get_value("city_look", "style", "<unset>"))

func _ready() -> void:
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		push_error("citylook1 probe watchdog")
		get_tree().quit(2))
	if DisplayServer.get_name() == "headless":
		push_error("_citylook1_probe must be windowed")
		get_tree().quit(2)
		return
	var vp_size := Vector2i(1600, 1000)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--vp":
				var wh := args[i + 1].split("x")
				vp_size = Vector2i(int(wh[0]), int(wh[1]))
			"--out":
				_out = args[i + 1]
			_:
				push_error("unknown argument " + args[i])
				get_tree().quit(2)
				return
		i += 2
	if _out == "":
		push_error("--out <dir> is required")
		get_tree().quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.0).timeout
	var bridge = _app.bridge
	bridge.generate({"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)

	print("\n== 1. two towns on different ground ==")
	var places: Array = bridge.settlements()
	var cands: Array = []
	for k in places.size():
		var got: Array = bridge.urban_layouts(PackedInt32Array([k]))
		if got.is_empty():
			continue
		var l: Dictionary = got[0]
		var ring: PackedVector2Array = l.get("wall_ring", PackedVector2Array())
		if ring.size() < 6 or String(l.get("wall_style", "")) != "curtain" or not l.has("ground_color"):
			continue
		cands.append({"k": k, "g": l["ground_color"] as Color, "cit": (l.get("citadel_wall", PackedVector2Array()) as PackedVector2Array).size() == 4,
			"n": (l.get("buildings", []) as Array).size()})
	print("  %d walled curtain towns carry a ground colour (of %d settlements)" % [cands.size(), places.size()])
	_ok(cands.size() >= 2, "at least two walled towns with a ground colour")
	if cands.size() < 2:
		_finish()
		return
	## Town B must have a citadel (the pixel checks need one); town A is the candidate whose ground is farthest from it.
	var b: Dictionary = {}
	for c in cands:
		if bool(c["cit"]) and (b.is_empty() or int(c["n"]) > int(b["n"])):
			b = c
	_ok(not b.is_empty(), "a walled town with a citadel exists in this world")
	if b.is_empty():
		_finish()
		return
	var a: Dictionary = {}
	var best := -1.0
	for c in cands:
		var d := Vector3((c["g"] as Color).r, (c["g"] as Color).g, (c["g"] as Color).b).distance_to(
			Vector3((b["g"] as Color).r, (b["g"] as Color).g, (b["g"] as Color).b))
		if int(c["k"]) != int(b["k"]) and d > best and int(c["n"]) > 60:
			best = d
			a = c
	print("  town A #%d ground %s, town B #%d ground %s, distance %.3f" % [int(a["k"]), a["g"], int(b["k"]), b["g"], best])
	_ok(best >= MIN_GROUND_DIST, "the two towns' ground colours differ by at least %.2f (%.3f)" % [MIN_GROUND_DIST, best])

	print("\n== 2. ground colour is the terrain's ==")
	var tex: Texture2D = _app.viewport.map_view.texture
	var img: Image = tex.get_image()
	var gs: Vector2i = bridge.grid_size()
	for t in [a, b]:
		var s: Dictionary = places[int(t["k"])]
		var acc := Vector3.ZERO
		var n := 0
		for o in [Vector2i(0, 0), Vector2i(2, 0), Vector2i(-2, 0), Vector2i(0, 2), Vector2i(0, -2),
				Vector2i(2, 2), Vector2i(-2, 2), Vector2i(2, -2), Vector2i(-2, -2)]:
			var gx: int = int(s["x"]) + o.x
			var gy: int = int(s["y"]) + o.y
			if String(bridge.sample_cell(gx, gy).get("water", "land")) != "land":
				continue
			var px := clampi(int((gx + 0.5) * float(img.get_width()) / gs.x), 0, img.get_width() - 1)
			var py := clampi(int((gy + 0.5) * float(img.get_height()) / gs.y), 0, img.get_height() - 1)
			var c := img.get_pixel(px, py)
			acc += Vector3(c.r, c.g, c.b)
			n += 1
		acc /= float(maxi(n, 1))
		var g: Color = t["g"]
		var dist := Vector3(g.r, g.g, g.b).distance_to(acc)
		print("  town #%d: layout ground %s vs displayed-map mean (%.3f, %.3f, %.3f) over %d land cells: %.4f" % [int(t["k"]), g, acc.x, acc.y, acc.z, n, dist])
		_ok(n >= 1 and dist <= 0.02, "town #%d ground_color equals the displayed map's colour at its cells" % int(t["k"]))

	print("\n== 3-5. the City Viewer, both looks, four zooms ==")
	var frames := {}   # "map_a_1" -> Image
	var cits := {}
	for ci_i in 2:
		var tw: Dictionary = a if ci_i == 0 else b
		var tag := "a" if ci_i == 0 else "b"
		var k: int = int(tw["k"])
		_app.open_city_viewer(k)
		await _frames(12)
		var cv = _app.city_viewer_window
		cv.size = Vector2i(1560, 960)
		cv.position = Vector2i(20, 20)
		await _frames(8)
		var layout: Dictionary = cv._layout
		var cw: PackedVector2Array = layout.get("citadel_wall", PackedVector2Array())
		var focus := Vector2.ZERO
		if cw.size() == 4:
			focus = (cw[0] + cw[1] + cw[2] + cw[3]) * 0.25
		else:
			focus = layout.get("market", Vector2.ZERO)
		for look_idx in [0, 1]:
			cv._look_picker.select(look_idx)
			cv._look_picker.item_selected.emit(look_idx)
			await _frames(3)
			var want := "map" if look_idx == 0 else "ink"
			_ok(_stored_look() == want, "town %s: choosing '%s' in the dropdown stores city_look=%s (read back from the settings file: %s)" % [tag, want, want, _stored_look()])
			for z in ZOOMS:
				cv._reset_view()
				await _frames(2)
				cv._zoom = z
				var box: Rect2 = cv._fit_box()
				var sz: Vector2 = cv._canvas.size
				var fit: float = minf((sz.x - 32.0) / maxf(1.0, box.size.x), (sz.y - 32.0) / maxf(1.0, box.size.y))
				var vs: float = fit * z
				if z > 1.0:
					cv._pan = sz * 0.5 - focus * vs - ((sz - box.size * vs) * 0.5 - box.position * vs)
				else:
					cv._pan = Vector2.ZERO
				cv._canvas.queue_redraw()
				await _frames(4)
				var rect := Rect2(cv._canvas.get_global_transform_with_canvas().origin + Vector2(cv.position), cv._canvas.size)
				var im := await _shot("%s_%s_z%d" % [want, tag, int(z)], rect)
				frames["%s_%s_%d" % [want, tag, int(z)]] = im
				cits["%s_%s_%d" % [want, tag, int(z)]] = {"vs": vs, "origin": (sz - box.size * vs) * 0.5 - box.position * vs + cv._pan, "layout": layout}
		## Positive controls and the claims, from the captured frames.
		var m1: Image = frames["map_%s_1" % tag]
		var i1: Image = frames["ink_%s_1" % tag]
		_ok(_distinct(m1) > 40 and _distinct(i1) > 40, "town %s: both looks draw (%d and %d distinct colours)" % [tag, _distinct(m1), _distinct(i1)])
		var df := _diff_frac(m1, i1)
		_ok(df > 0.5, "town %s: the two looks differ on %.0f%% of sampled pixels at zoom 1 (control)" % [tag, df * 100.0])
		var g: Color = tw["g"]
		var hit := 0
		for p in [Vector2i(3, 3), Vector2i(m1.get_width() - 4, 3), Vector2i(3, m1.get_height() - 4), Vector2i(m1.get_width() - 4, m1.get_height() - 4)]:
			if _near(m1.get_pixelv(p), g, 0.09):
				hit += 1
		var ink_hit := 0
		for p in [Vector2i(3, 3), Vector2i(m1.get_width() - 4, 3), Vector2i(3, m1.get_height() - 4), Vector2i(m1.get_width() - 4, m1.get_height() - 4)]:
			if _near(i1.get_pixelv(p), DRAW.GROUND, 0.03):
				ink_hit += 1
		_ok(hit >= 2, "town %s: map-look corner pixels carry the terrain ground %s (%d of 4 match)" % [tag, g, hit])
		_ok(ink_hit >= 2, "town %s: ink-look corner pixels are the fixed parchment (%d of 4 match; control)" % [tag, ink_hit])
		if cw.size() == 4:
			var cd: Dictionary = cits["map_%s_6" % tag]
			var ci2: Dictionary = cits["ink_%s_6" % tag]
			var mm: Image = frames["map_%s_6" % tag]
			var ii: Image = frames["ink_%s_6" % tag]
			var tow: PackedVector2Array = layout.get("citadel_towers", PackedVector2Array())
			var sp := func(d: Dictionary, p: Vector2) -> Vector2i:
				var q: Vector2 = (d["origin"] as Vector2) + p * float(d["vs"])
				return Vector2i(int(q.x), int(q.y))
			if tow.size() > 0:
				var tp: Vector2i = sp.call(cd, tow[0])
				var inside := tp.x >= 0 and tp.y >= 0 and tp.x < mm.get_width() and tp.y < mm.get_height()
				_ok(inside, "town %s: the first citadel tower is on screen at zoom 6 (%s)" % [tag, tp])
				if inside:
					_ok(_near(mm.get_pixelv(tp), LOOK.TOWER_TOP, 0.05), "town %s: map look, the citadel tower cap at its centre (%s vs %s)" % [tag, mm.get_pixelv(tp), LOOK.TOWER_TOP])
					_ok(not _near(ii.get_pixelv(sp.call(ci2, tow[0])), LOOK.TOWER_TOP, 0.05), "town %s: ink look has no tower cap there (control)" % tag)
			var edge_mid: Vector2 = (cw[1] + cw[2]) * 0.5
			var ep: Vector2i = sp.call(cd, edge_mid)
			var ec := mm.get_pixelv(ep)
			_ok(_near(ec, LOOK.STONE, 0.05) or _near(ec, LOOK.TOWER_TOP, 0.05), "town %s: the curtain wall band at the middle of the outer face is stone (%s)" % [tag, ec])
			var keep: PackedVector2Array = layout.get("citadel_keep", PackedVector2Array())
			if keep.size() >= 3:
				var kc := Vector2.ZERO
				for p in keep:
					kc += p
				kc /= float(keep.size())
				var kp: Vector2i = sp.call(cd, kc)
				_ok(_near(mm.get_pixelv(kp), LOOK.KEEP_ROOF, 0.05), "town %s: the keep's roof at its centre (%s vs %s)" % [tag, mm.get_pixelv(kp), LOOK.KEEP_ROOF])
				_ok(not _near(ii.get_pixelv(sp.call(ci2, kc)), LOOK.KEEP_ROOF, 0.05), "town %s: ink look keep is not the map look roof (control)" % tag)
		cv.hide()
		await _frames(2)

	print("\n== 6. export uses the chosen look ==")
	await _export_check(bridge)

	_compose(frames)
	DccSettings.set_city_look("ink")   # leave the (scratch) config as a default install has it
	_finish()

## Cuts an export tile round the first plan-drawn town in each look and counts the map look's dark roof colour.
func _export_check(bridge) -> void:
	var ov: Control = _find_script(_app, "map_overlay.gd")
	_ok(ov != null and int(ov.get("_gw")) > 1, "the live overlay holds a world")
	if ov == null:
		return
	var gw := int(ov.get("_gw"))
	var gh := int(ov.get("_gh"))
	var W := 4096
	var H := int(round(float(W) * gh / gw))
	## 60 km at 4096 px puts a 1.7 km town box at ~116 px, above `EXPORT_URBAN_MIN_BOX_PX` (75); 120 km gave 58 and no plan.
	var km: float = ov.get("_map_width_km")
	ov.set("_map_width_km", 60.0)
	var eu := EXPORT_OVERLAY.new()
	add_child(eu)
	var why: String = eu.setup(ov, W, H, {"settlements": true, "settlement_tier": "town"}, [])
	ov.set("_map_width_km", km)
	_ok(why == "", "an export copy with town plans at %d px (%s)" % [W, why])
	if why != "":
		return
	print("  export copy needs %d plans; %d settlements; live overlay %d settlements" % [eu.overlay.export_urban_needs().size(), (eu.overlay.get("_settlements") as Array).size(), (ov.get("_settlements") as Array).size()])
	var fetched: String = await eu._fetch_urban(bridge)
	var layouts: Dictionary = eu.overlay.get("_urban_layouts")
	var pick := -1
	var most := 0
	for i in layouts.keys():
		var nb := 0 if layouts[i] == null else (layouts[i].get("buildings", []) as Array).size()
		if nb > most:
			most = nb
			pick = i
	print("  export copy: %d layouts, richest has %d buildings (fetch: '%s')" % [layouts.size(), most, fetched])
	_ok(pick >= 0 and most >= 30, "the export copy laid out a town plan (%d buildings)" % most)
	if pick < 0:
		return
	var s0: Dictionary = (eu.overlay.get("_settlements") as Array)[pick]
	var lay: Dictionary = EXPORT_OVERLAY.layout(W, H, gw, gh)
	var tp: Vector2 = EXPORT_OVERLAY.texture_to_export(Vector2(int(s0["x"]), int(s0["y"])) + Vector2(0.5, 0.5), lay)
	var R := 512
	var ux := clampi(int(tp.x) - R / 2, 0, W - R)
	var uy := clampi(int(tp.y) - R / 2, 0, H - R)
	var counts := {}
	for want in ["map", "ink"]:
		DRAW.look_override = want
		eu.overlay.queue_redraw()
		await _frames(2)
		var px: PackedByteArray = await eu.render_tile(ux, uy, R, R)
		var dark := 0
		for q in range(0, px.size(), 4):
			var c := Color8(px[q], px[q + 1], px[q + 2], px[q + 3])
			if c.a > 0.9 and _near(c, LOOK.ROOF, 0.04):
				dark += 1
		counts[want] = dark
	DRAW.look_override = ""
	print("  export tile %dx%d round %s: map-look roof-brown px = %d, ink-look = %d" % [R, R, s0.get("name", "?"), counts["map"], counts["ink"]])
	_ok(int(counts["map"]) > 60 and int(counts["ink"]) * 8 < int(counts["map"]), "the export tile draws the map look's roofs only when that look is chosen (%d vs %d)" % [counts["map"], counts["ink"]])
	eu.queue_free()

## The composite: four rows (a map, a ink, b map, b ink) by four columns (zoom 1, 3, 6, 12), each cell scaled to 640 wide.
func _compose(frames: Dictionary) -> void:
	var cw := 640
	var rows := [["map", "a"], ["ink", "a"], ["map", "b"], ["ink", "b"]]
	var first: Image = frames["map_a_1"]
	var ch := int(round(float(cw) * first.get_height() / first.get_width()))
	var sheet := Image.create(cw * ZOOMS.size(), ch * rows.size(), false, Image.FORMAT_RGBA8)
	sheet.fill(Color(0.1, 0.1, 0.1))
	for r in rows.size():
		for c in ZOOMS.size():
			var key := "%s_%s_%d" % [rows[r][0], rows[r][1], int(ZOOMS[c])]
			if not frames.has(key):
				continue
			var im: Image = (frames[key] as Image).duplicate()
			im.resize(cw, ch, Image.INTERPOLATE_LANCZOS)
			sheet.blit_rect(im, Rect2i(0, 0, cw, ch), Vector2i(c * cw, r * ch))
	sheet.save_png(_out.path_join("composite.png"))
	print("  composite %dx%d -> %s" % [sheet.get_width(), sheet.get_height(), _out.path_join("composite.png")])

func _finish() -> void:
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
