extends Node
## CITY_MAP_LOOK_SCOPE.md CL-8 (the title) and CL-9 (the harbour) verification, in the real shell.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . --resolution 1600x1000 _cl89_probe.tscn -- --out <dir>
##       [--seed 483920] [--km 800] [--sea 0.45] [--town <settlement index>] [--vp 1600x1000]
##
## **Windowed, never `--headless`** (it reads rendered pixels; `ImageTexture.update()` is a no-op headless), from a
## wiped `APPDATA` so the `city_look` setting it flips is never the owner's. Refuses to run headless.
##
## What it protects (a positive control backs each pixel claim):
##   1. THE KEY: every layout the engine returns carries `harbour` exactly when it carries `harbour_pt`, as a
##      dictionary of piers (an even, non-empty point list, each pier of a real length) and, where the engine
##      built one, a four-point mole; the settlement's own name and kind arrive as `place_name` / `place_kind`.
##   2. THE TITLE: in the map look the City Viewer's `_title_rect` is non-empty, inside the canvas, and holds
##      cream-fill and dark-stroke pixels at zoom 1, 3, 6 and 12; in the ink look it is empty and the same
##      rectangle holds none (control). The text is "THE <KIND> OF <NAME>" in capitals. The font is bundled
##      (Cinzel, with its OFL beside it), the stroke is 6 to 10 px and the fill/stroke contrast is above 4.5:1.
##   3. THE PIERS: at the first pier's midpoint the map look's pixel is the pier's timber, the ink look's is not,
##      and the same view with the `harbour` key erased is not (control); the mole's midpoint is stone; the
##      shallow shore band's colour is on screen in the map look and absent in the ink look.
##   4. EXPORT: an export copy (`ExportOverlay`, the code the 16K/32K export runs) at a scale that makes the town
##      wide enough, cut round the town, holds the title's cream pixels and the pier timber in the map look and
##      neither in the ink look.
## Frames: `<out>/{map,ink}_z<zoom>.png`, `map_nopier_z12.png`, `export_{map,ink}.png`, `composite.png`.
## Result: `RESULT: PASS|FAIL (<n> failed) checks=<n>`; exit 0 pass, 1 fail, 2 bad argument or watchdog.

const DRAW := preload("res://shell/urban_layout_draw.gd")
const LOOK := preload("res://shell/urban_layout_look.gd")
const EXPORT_OVERLAY := preload("res://shell/export_overlay.gd")

## The City Viewer's zooms we frame (its own range is 0.25..12).
const ZOOMS := [1.0, 3.0, 6.0, 12.0]
## Pixel tolerance for "this is the look's colour" (RGB, each channel).
const TOL := 0.05

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

## Pixels of `img` inside `rect` (clipped) whose colour is within `tol` of `c`.
func _count(img: Image, rect: Rect2, c: Color, tol: float = TOL) -> int:
	var r := Rect2i(rect).intersection(Rect2i(0, 0, img.get_width(), img.get_height()))
	var n := 0
	for y in range(r.position.y, r.end.y):
		for x in range(r.position.x, r.end.x):
			if _near(img.get_pixel(x, y), c, tol):
				n += 1
	return n

func _lum(c: Color) -> float:
	var f := func(v: float) -> float:
		return v / 12.92 if v <= 0.03928 else pow((v + 0.055) / 1.055, 2.4)
	return 0.2126 * f.call(c.r) + 0.7152 * f.call(c.g) + 0.0722 * f.call(c.b)

func _contrast(a: Color, b: Color) -> float:
	var la := _lum(a)
	var lb := _lum(b)
	return (maxf(la, lb) + 0.05) / (minf(la, lb) + 0.05)

func _ready() -> void:
	get_tree().create_timer(1400.0).timeout.connect(func() -> void:
		push_error("cl89 probe watchdog")
		get_tree().quit(2))
	if DisplayServer.get_name() == "headless":
		push_error("_cl89_probe must be windowed")
		get_tree().quit(2)
		return
	var vp_size := Vector2i(1600, 1000)
	var seed_v := 483920
	var km := 800.0
	var sea := 0.45
	var want_town := -1
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--vp":
				var wh := args[i + 1].split("x")
				vp_size = Vector2i(int(wh[0]), int(wh[1]))
			"--out":
				_out = args[i + 1]
			"--seed":
				seed_v = int(args[i + 1])
			"--km":
				km = float(args[i + 1])
			"--sea":
				sea = float(args[i + 1])
			"--town":
				want_town = int(args[i + 1])
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
	bridge.generate({"seed": seed_v, "width_km": km, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": sea})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)

	print("\n== 1. the engine's key, and the settlement's name ==")
	var places: Array = bridge.settlements()
	var with_h := 0
	var without_h := 0
	var bad_shape := 0
	var mole_n := 0
	var best := -1
	var best_score := -1
	var no_harbour_town := -1
	for k in places.size():
		var got: Array = bridge.urban_layouts(PackedInt32Array([k]))
		if got.is_empty():
			continue
		var l: Dictionary = got[0]
		var has_pt := l.has("harbour_pt")
		var has_h := l.has("harbour")
		if has_pt != has_h:
			bad_shape += 1
			print("    malformed: town %d harbour_pt=%s harbour=%s" % [k, has_pt, has_h])
		if has_h:
			with_h += 1
			var h: Dictionary = l["harbour"]
			var piers: PackedVector2Array = h.get("piers", PackedVector2Array())
			var ok_piers := piers.size() >= 4 and piers.size() % 2 == 0
			for q in range(0, piers.size() - 1, 2):
				var len := piers[q].distance_to(piers[q + 1])
				if len < 1.0 or len > 90.0:
					ok_piers = false
			if h.has("mole"):
				mole_n += 1
				if (h["mole"] as PackedVector2Array).size() != 4:
					ok_piers = false
			if not ok_piers:
				bad_shape += 1
				print("    malformed piers: town %d %s" % [k, piers])
			## Pick the town for the frames: a coast or bay site beats a river one, a curtain wall and a big
			## plan beat a small one, a water body beats none.
			var ring: PackedVector2Array = l.get("wall_ring", PackedVector2Array())
			var score := 0
			if String(l.get("wall_style", "")) == "curtain" and ring.size() >= 6:
				score += 1000
			if String(l.get("site_kind", "")) in ["coast", "bay"]:
				score += 4000
			if h.has("mole"):
				score += 500
			if (l.get("water_poly", PackedVector2Array()) as PackedVector2Array).size() >= 3:
				score += 600
			score += mini(int((l.get("buildings", []) as Array).size() / 10), 300)
			if l.has("place_name") and String(places[k].get("kind", "")) in ["town", "city", "capital", "metropolis"]:
				score += 100
			if score > best_score:
				best_score = score
				best = k
		else:
			without_h += 1
			if no_harbour_town < 0:
				no_harbour_town = k
			if l.has("harbour"):
				bad_shape += 1
		if l.has("place_name") != (String(places[k].get("name", "")).strip_edges() != ""):
			bad_shape += 1
			print("    name mismatch: town %d name='%s' has=%s" % [k, places[k].get("name", ""), l.has("place_name")])
	print("  %d layouts with a harbour (%d with a mole), %d without; %d malformed" % [with_h, mole_n, without_h, bad_shape])
	_ok(with_h >= 5 and without_h >= 5, "both a harbour and a no-harbour population exist in this world (%d / %d)" % [with_h, without_h])
	_ok(bad_shape == 0, "`harbour` is present exactly when `harbour_pt` is, shaped as piers (even, 1 to 90 m: a river harbour's piers are short, `water.rs` `min(river_w * 0.55, 13)`) and a 4-point mole, and `place_name` follows the name")
	if want_town >= 0:
		best = want_town
	if best < 0:
		_finish()
		return
	var tw: Dictionary = bridge.urban_layouts(PackedInt32Array([best]))[0]
	var tplace: Dictionary = places[best]
	print("  town #%d %s kind=%s site=%s wall=%s/%d water_poly=%d piers=%d mole=%s buildings=%d (seed %d, %.0f km, sea %.2f)" % [best, tplace.get("name", "?"), tplace.get("kind", "?"),
		tw.get("site_kind", "?"), tw.get("wall_style", "-"), (tw.get("wall_ring", PackedVector2Array()) as PackedVector2Array).size(),
		(tw.get("water_poly", PackedVector2Array()) as PackedVector2Array).size(), ((tw["harbour"] as Dictionary)["piers"] as PackedVector2Array).size() / 2,
		(tw["harbour"] as Dictionary).has("mole"), (tw.get("buildings", []) as Array).size(), seed_v, km, sea])
	_ok(String(tw.get("place_name", "")) == String(tplace.get("name", "")), "the framed town's layout carries the settlement's name (%s)" % tw.get("place_name", "<none>"))
	_ok(LOOK.title_text(tw).begins_with("THE ") and LOOK.title_text(tw).ends_with(String(tplace["name"]).to_upper()), "the title text is capitals, '%s'" % LOOK.title_text(tw))
	if no_harbour_town >= 0:
		_ok(LOOK.title_text({"place_name": "x"}) == "X", "a layout with a name and no kind gives the name alone, never an invented word")
	_ok(LOOK.title_text({}) == "", "a layout with no name has no title")
	var words := {"metropolis": "CITY", "capital": "CITY", "city": "CITY", "town": "TOWN", "village": "VILLAGE", "hamlet": "HAMLET"}
	var word_ok := true
	for kd in words.keys():
		if LOOK.title_text({"place_name": "Ash", "place_kind": kd}) != "THE %s OF ASH" % words[kd]:
			word_ok = false
	_ok(word_ok, "the word follows the kind, for each of the six kinds the engine has")

	print("\n== 2. the bundled font and the stroke ==")
	_ok(LOOK.TITLE_FONT_FILE.resource_path == "res://fonts/Cinzel-Variable.ttf" and (LOOK.TITLE_FONT_FILE as Font).get_font_name() == "Cinzel", "the title face is the bundled Cinzel (%s)" % (LOOK.TITLE_FONT_FILE as Font).get_font_name())
	var ofl := FileAccess.get_file_as_string("res://fonts/Cinzel-OFL.txt")
	_ok(ofl.contains("SIL OPEN FONT LICENSE Version 1.1") and ofl.contains("Cinzel Project Authors"), "the licence text is bundled beside the font")
	_ok(is_equal_approx(LOOK.title_stroke_px(24.0), 6.0) and is_equal_approx(LOOK.title_stroke_px(40.0), 10.0) and is_equal_approx(LOOK.title_stroke_px(100.0), 10.0) and is_equal_approx(LOOK.title_stroke_px(10.0), 6.0), "the stroke is 6 px at 24, 10 px at 40 and beyond, never under 6")
	var ctr := _contrast(LOOK.TITLE_FILL, LOOK.TITLE_STROKE)
	_ok(ctr >= 4.5, "title fill against stroke contrast %.1f:1 (at least 4.5)" % ctr)
	var fw: float = LOOK.title_font().get_string_size("THE CITY OF", HORIZONTAL_ALIGNMENT_LEFT, -1, 40).x
	var sw: float = (load("res://fonts/FiraSans-Bold.ttf") as Font).get_string_size("THE CITY OF", HORIZONTAL_ALIGNMENT_LEFT, -1, 40).x
	_ok(absf(fw - sw) > 8.0, "the title face measures differently from the UI sans (%.0f vs %.0f px), so it is not the fallback" % [fw, sw])

	print("\n== 3. the City Viewer, both looks, four zooms ==")
	var frames := {}
	_app.open_city_viewer(best)
	await _frames(12)
	var cv = _app.city_viewer_window
	cv.size = Vector2i(1560, 960)
	cv.position = Vector2i(20, 20)
	await _frames(8)
	var layout: Dictionary = cv._layout
	_ok(layout.has("harbour") and layout.has("place_name"), "the viewer's own layout carries the harbour and the name")
	var harb: Dictionary = layout.get("harbour", {})
	var piers: PackedVector2Array = harb.get("piers", PackedVector2Array())
	var focus := Vector2.ZERO
	if piers.size() >= 2:
		focus = (piers[0] + piers[1]) * 0.5
	var mole: PackedVector2Array = harb.get("mole", PackedVector2Array())
	var views := {}
	for look_idx in [0, 1]:
		cv._look_picker.select(look_idx)
		cv._look_picker.item_selected.emit(look_idx)
		await _frames(3)
		var want := "map" if look_idx == 0 else "ink"
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
			var im := await _shot("%s_z%d" % [want, int(z)], rect)
			frames["%s_%d" % [want, int(z)]] = im
			views["%s_%d" % [want, int(z)]] = {"vs": vs, "origin": (sz - box.size * vs) * 0.5 - box.position * vs + cv._pan, "title": cv._title_rect, "size": sz}
	## The title, per zoom.
	var cream := LOOK.TITLE_FILL
	var ink := LOOK.TITLE_STROKE
	for z in ZOOMS:
		var mv: Dictionary = views["map_%d" % int(z)]
		var tr: Rect2 = mv["title"]
		var canvas := Rect2(Vector2.ZERO, mv["size"])
		_ok(tr.size.x > 100.0 and tr.size.y > 18.0 and canvas.encloses(tr), "zoom %d, map look: the title is drawn and lies inside the canvas (%s)" % [int(z), tr])
		var im: Image = frames["map_%d" % int(z)]
		var nf := _count(im, tr, cream, 0.04)
		var ns := _count(im, tr, ink, 0.04)
		_ok(nf >= 150 and ns >= 150, "zoom %d, map look: the rectangle holds the cream fill (%d px) and the dark stroke (%d px)" % [int(z), nf, ns])
		var iv: Dictionary = views["ink_%d" % int(z)]
		var ii: Image = frames["ink_%d" % int(z)]
		var nfi := _count(ii, tr, cream, 0.04)
		_ok((iv["title"] as Rect2).size == Vector2.ZERO and nfi * 10 < maxi(nf, 1), "zoom %d, ink look: no title drawn, and the same rectangle holds %d cream px against %d (control)" % [int(z), nfi, nf])
	## The piers.
	if piers.size() >= 2:
		var sp := func(d: Dictionary, p: Vector2) -> Vector2i:
			var q: Vector2 = (d["origin"] as Vector2) + p * float(d["vs"])
			return Vector2i(int(q.x), int(q.y))
		var wood_ok := func(c: Color) -> bool:
			return _near(c, LOOK.PIER_WOOD, 0.07) or _near(c, LOOK.PIER_PLANK, 0.07)
		var mv12: Dictionary = views["map_12"]
		var iv12: Dictionary = views["ink_12"]
		var m12: Image = frames["map_12"]
		var i12: Image = frames["ink_12"]
		var mid: Vector2i = sp.call(mv12, focus)
		_ok(mid.x > 0 and mid.y > 0 and mid.x < m12.get_width() and mid.y < m12.get_height(), "the first pier's midpoint is on screen at zoom 12 (%s)" % mid)
		## The deck is a few pixels wide at zoom 12; sample a small neighbourhood so a plank joint cannot hide it.
		var wood_hits := 0
		for dy in range(-2, 3):
			for dx in range(-2, 3):
				if wood_ok.call(m12.get_pixelv(mid + Vector2i(dx, dy))):
					wood_hits += 1
		var wood_hits_ink := 0
		for dy in range(-2, 3):
			for dx in range(-2, 3):
				if wood_ok.call(i12.get_pixelv(mid + Vector2i(dx, dy))):
					wood_hits_ink += 1
		_ok(wood_hits >= 12, "map look, zoom 12: the first pier's midpoint is timber (%d of 25 pixels)" % wood_hits)
		_ok(wood_hits_ink <= 2, "ink look, zoom 12: no timber there (%d of 25; control)" % wood_hits_ink)
		## The deck's WIDTH, measured across the pier at its midpoint in the frame, against the literal the design
		## names (a 3.4 m timber jetty; `PIER_WIDTH_M` is a labelled judgement) times this view's pixels per metre: a
		## constant that drifted, or a floor that hid it, would fail here and not in the colour counts above.
		var pdir := (piers[1] - piers[0]).normalized()
		var pn := Vector2(-pdir.y, pdir.x)
		var deck_px := 0.0
		var t := -90.0
		while t <= 90.0:
			var q := Vector2(mid) + pn * t
			if wood_ok.call(m12.get_pixelv(Vector2i(int(q.x), int(q.y)))):
				deck_px += 0.5
			t += 0.5
		var want_px: float = 3.4 * float(mv12["vs"])
		_ok(absf(deck_px - want_px) <= 0.15 * want_px, "zoom 12: the pier deck measures %.1f px across, a 3.4 m jetty at %.2f px per metre is %.1f" % [deck_px, float(mv12["vs"]), want_px])
		## Every pier, at zoom 12 of its own: the same layout with the key erased must show no timber there.
		var wood_all := _count(m12, Rect2(0, 0, m12.get_width(), m12.get_height()), LOOK.PIER_WOOD, 0.04)
		var wood_all_ink := _count(i12, Rect2(0, 0, i12.get_width(), i12.get_height()), LOOK.PIER_WOOD, 0.04)
		print("  timber-coloured pixels in the zoom 12 frame: map %d, ink %d" % [wood_all, wood_all_ink])
		_ok(wood_all > 300 and wood_all_ink * 10 < wood_all, "the zoom 12 map frame holds the pier timber (%d px against %d in the ink look)" % [wood_all, wood_all_ink])
		## Control: the same view without the harbour key.
		cv._look_picker.select(0)
		cv._look_picker.item_selected.emit(0)
		await _frames(3)
		cv._reset_view()
		await _frames(2)
		cv._zoom = 12.0
		var box12: Rect2 = cv._fit_box()
		var sz12: Vector2 = cv._canvas.size
		var fit12: float = minf((sz12.x - 32.0) / maxf(1.0, box12.size.x), (sz12.y - 32.0) / maxf(1.0, box12.size.y))
		cv._pan = sz12 * 0.5 - focus * fit12 * 12.0 - ((sz12 - box12.size * fit12 * 12.0) * 0.5 - box12.position * fit12 * 12.0)
		var saved: Dictionary = cv._layout["harbour"]
		cv._layout.erase("harbour")
		cv._canvas.queue_redraw()
		await _frames(4)
		var rect12 := Rect2(cv._canvas.get_global_transform_with_canvas().origin + Vector2(cv.position), cv._canvas.size)
		var np := await _shot("map_nopier_z12", rect12)
		cv._layout["harbour"] = saved
		var wood_none := _count(np, Rect2(0, 0, np.get_width(), np.get_height()), LOOK.PIER_WOOD, 0.04)
		_ok(wood_none * 10 < wood_all, "with the harbour key erased the same view holds %d timber px against %d (control)" % [wood_none, wood_all])
		## The mole: the engine builds one on an open `coast` site only, so a bay or river town has none. When the real
		## layout has none, a SYNTHETIC mole (labelled as such) is injected beside the first pier to prove the draw.
		var synthetic := false
		if mole.size() < 2:
			synthetic = true
			var dir := (piers[1] - piers[0]).normalized()
			var nrm := Vector2(-dir.y, dir.x)
			mole = PackedVector2Array([piers[0] + nrm * 40.0, piers[0] + nrm * 40.0 + dir * 30.0,
				piers[0] + nrm * 70.0 + dir * 50.0, piers[0] + nrm * 100.0 + dir * 50.0])
			(cv._layout["harbour"] as Dictionary)["mole"] = mole
		var mc := (mole[1] + mole[2]) * 0.5
		cv._reset_view()
		await _frames(2)
		cv._zoom = 6.0
		var fit6 := fit12
		cv._pan = sz12 * 0.5 - mc * fit6 * 6.0 - ((sz12 - box12.size * fit6 * 6.0) * 0.5 - box12.position * fit6 * 6.0)
		cv._canvas.queue_redraw()
		await _frames(4)
		var rect6 := Rect2(cv._canvas.get_global_transform_with_canvas().origin + Vector2(cv.position), cv._canvas.size)
		var mi := await _shot("map_mole_z6", rect6)
		var mctr := Vector2i(int(sz12.x * 0.5), int(sz12.y * 0.5))
		var stone := 0
		for dy in range(-3, 4):
			for dx in range(-3, 4):
				if _near(mi.get_pixelv(mctr + Vector2i(dx, dy)), LOOK.STONE, 0.04):
					stone += 1
		_ok(stone >= 20, "the breakwater mole's middle is stone at zoom 6 (%d of 49 pixels; %s mole)" % [stone, "a SYNTHETIC control" if synthetic else "the engine's"])
		if synthetic:
			(cv._layout["harbour"] as Dictionary).erase("mole")
	## The shore band.
	var band_m := _count(frames["map_3"], Rect2(0, 0, 1560, 960), LOOK.SEA_SHORE, 0.03)
	var band_i := _count(frames["ink_3"], Rect2(0, 0, 1560, 960), LOOK.SEA_SHORE, 0.03)
	var deep_m := _count(frames["map_1"], Rect2(0, 0, 1560, 960), LOOK.SEA, 0.03)
	print("  zoom 3 shore-band pixels: map %d, ink %d; zoom 1 deep-water pixels in the map look: %d" % [band_m, band_i, deep_m])
	_ok(band_m > 400 and band_i * 10 < band_m, "the shallow shore band is on screen in the map look (%d px) and not in the ink look (%d)" % [band_m, band_i])
	_ok(deep_m > 1500, "deep water is on screen in the map look at zoom 1 (%d px)" % deep_m)
	cv.hide()
	await _frames(2)

	print("\n== 4. export uses both ==")
	var exp_imgs := await _export_check(bridge, best, tw)

	_compose(frames, exp_imgs)
	DccSettings.set_city_look("ink")   # leave the (scratch) config as a default install has it
	_finish()

## Cuts export tiles round the chosen town in each look: one over the title (its cream must be present in the map
## look and absent in the ink look) and one over the first pier (its timber must come from the `harbour` key: the
## same tile with the key erased holds fewer timber-coloured pixels). The export copy's own geometry is used to find
## both: a model-metre point goes through the town's market anchor, orientation and the copy's grid-per-metre, exactly
## as `map_overlay.gd::_draw_urban_layouts` does, then `EXPORT_OVERLAY.texture_to_export`. Returns `{map, ink}` images
## of both tiles (`title_*`, `pier_*`, for the composite), or `{}` on failure.
func _export_check(bridge, town: int, tw: Dictionary) -> Dictionary:
	var ov: Control = _find_script(_app, "map_overlay.gd")
	_ok(ov != null and int(ov.get("_gw")) > 1, "the live overlay holds a world")
	if ov == null:
		return {}
	var gw := int(ov.get("_gw"))
	var gh := int(ov.get("_gh"))
	var W := 4096
	var H := int(round(float(W) * gh / gw))
	## The town box must be at least `TITLE_MIN_BOX_PX` (480) wide in export pixels; here ~1500 so a pier deck is a few
	## pixels wide.
	var box: Rect2 = LOOK._town_box(tw)
	var export_km := float(W) * maxf(300.0, box.size.x) / (1000.0 * 1000.0)
	var km: float = ov.get("_map_width_km")
	var src: Dictionary = bridge.settlements()[town]
	## Only the chosen town's rank and above is drawn as a plan: neighbouring plans would overlap at this scale.
	var tier := String(src.get("kind", "town"))
	ov.set("_map_width_km", export_km)
	var eu := EXPORT_OVERLAY.new()
	add_child(eu)
	var why: String = eu.setup(ov, W, H, {"settlements": true, "settlement_tier": tier}, [])
	ov.set("_map_width_km", km)
	_ok(why == "", "an export copy with %s-tier town plans at %d px, %.2f km across (%s)" % [tier, W, export_km, why])
	if why != "":
		return {}
	var fetched: String = await eu._fetch_urban(bridge)
	var layouts: Dictionary = eu.overlay.get("_urban_layouts")
	## The copy's own settlement list is filtered by tier, so its indices are NOT the engine's: find the chosen town
	## by name and position.
	var ov_set: Array = eu.overlay.get("_settlements")
	var town_j := -1
	for q in ov_set.size():
		if String((ov_set[q] as Dictionary).get("name", "")) == String(src.get("name", "")) and int((ov_set[q] as Dictionary).get("x", -1)) == int(src["x"]):
			town_j = q
	_ok(town_j >= 0, "the export copy lists the chosen town (copy index %d of %d)" % [town_j, ov_set.size()])
	if town_j < 0:
		return {}
	_ok(layouts.has(town_j) and layouts[town_j] != null and (layouts[town_j] as Dictionary).has("harbour") and (layouts[town_j] as Dictionary).has("place_name"),
		"the export copy laid out the chosen town and its layout carries the harbour and the name (fetch: '%s'; %d plans)" % [fetched, layouts.size()])
	if not layouts.has(town_j) or layouts[town_j] == null:
		return {}
	var lay_t: Dictionary = layouts[town_j]
	var lay: Dictionary = EXPORT_OVERLAY.layout(W, H, gw, gh)
	var gpm := float(gw) / (float(eu.overlay.get("_map_width_km")) * 1000.0)
	var anchor: Vector2 = lay_t.get("market", Vector2.ZERO)
	var rot := float(lay_t.get("orient", 0.0))
	var to_exp := func(mp: Vector2) -> Vector2:
		var l := mp - anchor
		return EXPORT_OVERLAY.texture_to_export(Vector2(
			float(src["x"]) + 0.5 + (l.x * cos(rot) - l.y * sin(rot)) * gpm,
			float(src["y"]) + 0.5 + (l.x * sin(rot) + l.y * cos(rot)) * gpm), lay)
	## Title tile: from the town box's top-left in export pixels (the corner bounding box, as the draw does).
	var lo := Vector2(INF, INF)
	for corner in [box.position, Vector2(box.end.x, box.position.y), Vector2(box.position.x, box.end.y), box.end]:
		lo = lo.min(to_exp.call(corner))
	var TW := 1100
	var TH := 260
	var tx := clampi(int(lo.x) - 20, 0, W - TW)
	var ty := clampi(int(lo.y) - 20, 0, H - TH)
	## Pier tile: round the first pier's midpoint.
	var piers: PackedVector2Array = (lay_t["harbour"] as Dictionary)["piers"]
	var pm: Vector2 = to_exp.call((piers[0] + piers[1]) * 0.5)
	var PR := 420
	var px0 := clampi(int(pm.x) - PR / 2, 0, W - PR)
	var py0 := clampi(int(pm.y) - PR / 2, 0, H - PR)
	print("  export px: town box top-left (%.0f, %.0f), first pier midpoint (%.0f, %.0f); title tile at (%d, %d), pier tile at (%d, %d)" % [lo.x, lo.y, pm.x, pm.y, tx, ty, px0, py0])
	## At this scale (about 1 px per metre) the capitals' plans overlap and a later town's plan covers ours, so every
	## OTHER plan is dropped from the copy (a null entry is a refused town: drawn as its pin), leaving the chosen
	## town's plan alone.
	for q in layouts.keys():
		if int(q) != town_j:
			layouts[q] = null
	var timber := func(img: Image) -> int:
		var r := Rect2(0, 0, img.get_width(), img.get_height())
		return _count(img, r, LOOK.PIER_WOOD, 0.05) + _count(img, r, LOOK.PIER_PLANK, 0.05) + _count(img, r, LOOK.PIER_EDGE, 0.03)
	var out := {}
	var cream := {}
	var wood := {}
	for want in ["map", "ink"]:
		DRAW.look_override = want
		eu.overlay.queue_redraw()
		await _frames(2)
		var tpx: PackedByteArray = await eu.render_tile(tx, ty, TW, TH)
		var timg := Image.create_from_data(TW, TH, false, Image.FORMAT_RGBA8, tpx)
		timg.save_png(_out.path_join("export_title_%s.png" % want))
		out["title_" + want] = timg
		cream[want] = _count(timg, Rect2(0, 0, TW, TH), LOOK.TITLE_FILL, 0.03)
		var ppx: PackedByteArray = await eu.render_tile(px0, py0, PR, PR)
		var pimg := Image.create_from_data(PR, PR, false, Image.FORMAT_RGBA8, ppx)
		pimg.save_png(_out.path_join("export_pier_%s.png" % want))
		out["pier_" + want] = pimg
		wood[want] = timber.call(pimg)
	## Control: the SAME map-look pier tile with the town's `harbour` key erased.
	DRAW.look_override = "map"
	var saved_h: Dictionary = lay_t["harbour"]
	lay_t.erase("harbour")
	eu.overlay.queue_redraw()
	await _frames(2)
	var ppx2: PackedByteArray = await eu.render_tile(px0, py0, PR, PR)
	var pimg2 := Image.create_from_data(PR, PR, false, Image.FORMAT_RGBA8, ppx2)
	pimg2.save_png(_out.path_join("export_pier_map_noharbour.png"))
	lay_t["harbour"] = saved_h
	var wood_without: int = timber.call(pimg2)
	DRAW.look_override = ""
	print("  export: title tile cream px map %d / ink %d; pier tile timber-coloured px map %d / ink %d / map without the key %d" % [cream["map"], cream["ink"], wood["map"], wood["ink"], wood_without])
	_ok(int(cream["map"]) > 400 and int(cream["ink"]) * 10 < int(cream["map"]), "the export's title tile carries the title's cream only in the map look (%d vs %d)" % [cream["map"], cream["ink"]])
	_ok(int(wood["map"]) - wood_without >= 40, "the export's pier timber comes from the harbour key: %d timber-coloured px with it, %d without" % [wood["map"], wood_without])
	eu.queue_free()
	return out

func _find_script(n: Node, file: String) -> Node:
	for c in n.get_children():
		if c.get_script() != null and String(c.get_script().resource_path).ends_with(file):
			return c
		var r := _find_script(c, file)
		if r != null:
			return r
	return null

## The composite: row 1 the map look at zoom 1, 3, 6, 12; row 2 the ink look at zoom 1, the map look without
## piers (zoom 12), and the export's title tile and pier tile in the map look.
func _compose(frames: Dictionary, exp: Dictionary) -> void:
	var cw := 640
	var first: Image = frames["map_1"]
	var ch := int(round(float(cw) * first.get_height() / first.get_width()))
	var sheet := Image.create(cw * 4, ch * 2, false, Image.FORMAT_RGBA8)
	sheet.fill(Color(0.1, 0.1, 0.1))
	var put := func(im: Image, c: int, r: int) -> void:
		var d: Image = im.duplicate()
		var hh := int(round(float(cw) * d.get_height() / d.get_width()))
		d.resize(cw, hh, Image.INTERPOLATE_LANCZOS)
		sheet.blit_rect(d, Rect2i(0, 0, cw, hh), Vector2i(c * cw, r * ch))
	for c in ZOOMS.size():
		put.call(frames["map_%d" % int(ZOOMS[c])], c, 0)
	put.call(frames["ink_1"], 0, 1)
	var np := Image.load_from_file(_out.path_join("map_nopier_z12.png"))
	if np != null:
		put.call(np, 1, 1)
	if exp.has("title_map"):
		put.call(exp["title_map"], 2, 1)
		put.call(exp["pier_map"], 3, 1)
	sheet.save_png(_out.path_join("composite.png"))
	print("  composite %dx%d -> %s" % [sheet.get_width(), sheet.get_height(), _out.path_join("composite.png")])

func _finish() -> void:
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
