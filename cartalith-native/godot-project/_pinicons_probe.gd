extends Node
## Owner icon sets as teardrop options (2026-11-15) and Ruling FC (a town's pins show only when the town is visible).
## Windowed (refuses --headless: `ImageTexture` and the framebuffer are no-ops there), under a wiped user directory
## (`APPDATA=<empty dir>`):
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _pinicons_probe.tscn -- --vp 1600x900 --out <dir>
##   ... -- --vp 1080x2340 --density 2.625 --force-touch --out <dir>      (the phone)
##
## Legs. T: the one kind-to-icon table (a mapped kind has a file, the borrowed-picture kinds the owner refused -- fort, a
## temple in the painted set -- do not). R: a scratch canvas draws `PinMarker.draw` and the framebuffer says the texture is
## drawn (its silhouette is not the drawn pin's), an unmapped kind draws the drawn pin EXACTLY (same pixels as `pins`), and a
## tilt keeps the tip. H: the hit footprint follows the textured, rotated head. F: Ruling FC on the map overlay. V: the City
## Viewer draws and picks the textured pin, whatever the map's zoom, plus screenshots when `--out` is given.
## Exit 0 = held, 1 = a check failed, 2 = could not run.

const Pin := preload("res://shell/pin_marker.gd")

var app: Node
var _vp: SubViewport
var _bridge
var _ov: Control
var _touch := false
var _checks := 0
var _fails := 0
var _out := ""
var _form := "desktop"


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("PINI %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _p(s: String) -> void:
	print("PINI %s" % s)


func _finish() -> void:
	DccSettings.set_marker_style("pins")
	_p("---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)


func _ready() -> void:
	var vp_size := Vector2i(1600, 900)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--vp":
			var wh := String(args[i + 1]).split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
		elif a == "--density":
			i += 2
		elif a == "--out":
			_out = String(args[i + 1])
			i += 2
		elif a == "--force-touch":
			_touch = true
			i += 1
		else:
			_p("ABORT unknown argument %s" % a)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		_p("ABORT run windowed")
		get_tree().quit(2)
		return
	var wd := get_tree().create_timer(900.0)
	wd.timeout.connect(func():
		_p("WATCHDOG")
		get_tree().quit(3))
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(4)
	_vp.notification(Node.NOTIFICATION_VP_MOUSE_ENTER)
	_bridge = app.bridge
	_bridge.generate({"seed": 1200, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if not _bridge.has_world:
		_ok("a world generated", false)
		_finish()
		return
	_ov = app.viewport.overlay
	await _bridge.landmark_run()
	await _frames(20)
	if DccTheme.is_phone():
		app.element_chips.hide_chips()
		app._close_all_phone_overlays()
		await _frames(10)
	_form = "phone" if DccTheme.is_phone() else "desktop"
	_leg_t()
	await _leg_r()
	_leg_h()
	await _leg_f()
	await _leg_v()
	_finish()


# ---- T: the table ------------------------------------------------------------------------------------------------

## True when the set has a pin file named `name` on disk: the ground truth the table checks are measured against, so the
## probe never asserts the table against itself.
func _file(iset: String, name: String) -> bool:
	return FileAccess.file_exists(ProjectSettings.globalize_path("res://icons/pins/%s/%s.png" % [iset, name]))


func _leg_t() -> void:
	print("== T. the kind-to-icon table ==")
	_ok("T1: glyph temple has a file", Pin.icon_name_for("glyph", "temple") == "temple")
	# Re-pointed 2026-11-15 (commits c516ddb3, 0216fe10, d3560db1 drew ~40 glyph pins and 3209bc13/85b9e2ff the painted
	# extras): a set's OWN file named like the kind wins over a `KIND_ICON` row, so quarry no longer shares the mine pick.
	# The expected facts are the files on disk (`_file`), not the table.
	_ok("T0: the files this leg reasons about exist on disk (quarry, temple, monument in both sets, fort, tavern)",
		_file("glyph", "quarry") and _file("painted", "quarry") and _file("glyph", "temple") and _file("painted", "temple")
		and _file("glyph", "monument") and _file("painted", "monument") and _file("glyph", "fort") and _file("glyph", "tavern")
		and not _file("glyph", "peak") and not _file("glyph", "citadel") and not _file("painted", "citadel") and not _file("glyph", "hazard"))
	_ok("T1: glyph and painted quarry have their own file (it wins over the old shared mine pick)",
		Pin.icon_name_for("glyph", "quarry") == "quarry" and Pin.icon_name_for("painted", "quarry") == "quarry")
	_ok("T1: an alias row still serves a kind with no file of its own (peak -> mountain, resource_extraction_site -> mine)",
		Pin.icon_name_for("glyph", "peak") == "mountain" and Pin.icon_name_for("glyph", "resource_extraction_site") == "mine"
		and Pin.icon_name_for("painted", "resource_extraction_site") == "mine")
	_ok("T1: painted monument has a file", Pin.icon_name_for("painted", "monument") == "monument")
	# Re-pointed 2026-11-15: painted/temple.png (the coded painting, owner "Do A") and glyph/monument.png now exist, so each
	# resolves to its own file and neither borrows the other's picture (the borrowing this pair guarded is still refused).
	_ok("T2: painted temple is its own file, NOT lent the obelisk", Pin.icon_name_for("painted", "temple") == "temple")
	_ok("T2: glyph monument is its own file, NOT lent the temple", Pin.icon_name_for("glyph", "monument") == "monument")
	_ok("T2: painted has its own shrine, fort, tomb and tavern files (the extra sheet)", Pin.icon_name_for("painted", "shrine") == "shrine"
		and Pin.icon_name_for("painted", "fort") == "fort" and Pin.icon_name_for("painted", "tomb") == "tomb" and Pin.icon_name_for("painted", "tavern") == "tavern")
	_ok("T2: the owner's three named mappings", Pin.icon_name_for("painted", "battlefield_historic") == "battlefield"
		and Pin.icon_name_for("painted", "destroyed_fortress") == "ruin" and Pin.icon_name_for("painted", "historic_crossing") == "historic_crossing"
		and Pin.icon_name_for("painted", "other") == "other")
	# Re-pointed 2026-11-15: the glyph set now draws these kinds' own pins (fort is a timber stockade), so the owner's rule
	# "Fort can't borrow" reads as: each resolves to ITS OWN file in both sets, never `castle`. `citadel` has no file in either
	# set and no row, so it still draws the drawn pin.
	for k in ["fort", "watchtower", "fortified_pass", "fortified_crossing", "gate", "border_marker"]:
		_ok("T3: '%s' has its own icon in both sets and never borrows the castle (owner: Fort can't borrow)" % k,
			_file("glyph", k) and _file("painted", k) and Pin.icon_name_for("glyph", k) == k and Pin.icon_name_for("painted", k) == k)
	_ok("T3: 'citadel' (no file, no row) never borrows the castle: the drawn pin in both sets",
		Pin.icon_name_for("glyph", "citadel") == "" and Pin.icon_name_for("painted", "citadel") == "")
	for k in ["tavern", "smithy", "guild", "well", "camp", "other"]:
		_ok("T4: kind '%s' has its own glyph file (drawn code, d3560db1)" % k, _file("glyph", k) and Pin.icon_name_for("glyph", k) == k)
	for k in ["citadel", "hazard", "", "no_such_kind"]:
		_ok("T4: kind '%s' has no glyph icon (the drawn pin)" % k, Pin.icon_name_for("glyph", k) == "")
	_ok("T5: no set draws the drawn pin", Pin.icon_name_for("", "temple") == "")
	var dead: Array = []
	for k in Pin.KIND_ICON:
		if Pin.icon_name_for("glyph", String(k)) == "" and Pin.icon_name_for("painted", String(k)) == "":
			dead.append(k)
	_ok("T6: every table row has a file in at least one set", dead.is_empty(), str(dead))
	var sh := Pin.shape(20.0, "temple", "glyph")
	_ok("T7: the glyph pin is as tall as the drawn pin (within 3%: the cut leaves a 2 px margin above the head)", absf(float(sh["r"]) * (1.0 + float(sh["stem"])) / ((Pin.STEM_RATIO + 1.0) * 20.0) - 1.0) < 0.03, str(sh))
	_ok("T7: its stem ratio is the cut's 1.37 (not the drawn 2.1)", absf(float(sh["stem"]) - 1.3663) < 0.06, str(sh["stem"]))
	var fb := Pin.shape(20.0, "citadel", "glyph")
	var fortsh := Pin.shape(20.0, "fort", "painted")
	# Re-pointed 2026-11-15: the old `> 1.5` assumed the extra sheet's tip factor K (1.575 to 1.875) was the stem ratio; the PNG's
	# alpha (read independently with PIL: top row 2, widest row 103 px, height 128) gives 1.4466, longer than the glyph's 1.377.
	_ok("T7: a painted extra pin has its own, longer stem (measured per texture, %.3f; the file reads 1.447 vs glyph 1.377)" % float(fortsh["stem"]),
		absf(float(fortsh["stem"]) - 1.4466) < 0.03 and float(fortsh["stem"]) > float(sh["stem"]) + 0.03 and fortsh["icon"] == "fort")
	_ok("T8: the fallback shape (kind 'citadel', no icon) is the drawn pin", float(fb["r"]) == 20.0 and float(fb["stem"]) == Pin.STEM_RATIO and fb["icon"] == "")
	_ok("T9: the Marker style list is Pins, Circles, icons, painted", DccSettings.MARKER_STYLES == ["pins", "circles", "icons", "painted"])


# ---- R: the framebuffer ------------------------------------------------------------------------------------------

## Renders one `PinMarker.draw` call on a transparent scratch canvas and returns the image. The tip is at (150, 250).
func _render(kind: String, iset: String, angle: float, has_note: bool = false) -> Image:
	var sv := SubViewport.new()
	sv.size = Vector2i(300, 300)
	sv.transparent_bg = true
	sv.render_target_update_mode = SubViewport.UPDATE_ONCE
	var c := Control.new()
	c.size = Vector2(300, 300)
	c.draw.connect(func(): Pin.draw(c, Vector2(150, 250), 20.0, kind, has_note, 1.0, 1.0, Pin.INK, angle, iset))
	sv.add_child(c)
	add_child(sv)
	await _frames(3)
	await RenderingServer.frame_post_draw
	var img := sv.get_texture().get_image()
	sv.queue_free()
	return img


func _opaque(img: Image) -> int:
	var n := 0
	for y in img.get_height():
		for x in img.get_width():
			if img.get_pixel(x, y).a > 0.5:
				n += 1
	return n


func _alpha_at(img: Image, p: Vector2) -> float:
	return img.get_pixelv(Vector2i(clampi(int(round(p.x)), 0, img.get_width() - 1), clampi(int(round(p.y)), 0, img.get_height() - 1))).a


func _differing(a: Image, b: Image) -> int:
	var n := 0
	for y in a.get_height():
		for x in a.get_width():
			if not a.get_pixel(x, y).is_equal_approx(b.get_pixel(x, y)):
				n += 1
	return n


func _leg_r() -> void:
	print("== R. what is drawn ==")
	var drawn := await _render("temple", "", 0.0)
	for iset in ["glyph", "painted"]:
		var kind := "temple" if iset == "glyph" else "monument"
		var icon := await _render(kind, iset, 0.0)
		var ref := await _render(kind, "", 0.0)
		var sh := Pin.shape(20.0, kind, iset)
		var head := Vector2(150, 250) + Vector2(0.0, -float(sh["stem"]) * float(sh["r"]))
		_ok("R1 %s: the texture is drawn (the pixels are not the drawn pin's, %d differ)" % [iset, _differing(icon, ref)], _differing(icon, ref) > 400)
		_ok("R1 %s: its head is opaque and bigger than the drawn head (%d vs %d opaque px)" % [iset, _opaque(icon), _opaque(ref)], _alpha_at(icon, head) > 0.9 and _opaque(icon) > _opaque(ref) * 1.1)
		_ok("R2 %s: the tip pixel is on the place (opaque a few px up the axis, empty below)" % iset, _alpha_at(icon, Vector2(150, 246)) > 0.5 and _alpha_at(icon, Vector2(150, 256)) < 0.1)
		var ang := PI * 0.5
		var tilted := await _render(kind, iset, ang)
		var head_r := Vector2(150, 250) + (Vector2(0.0, -float(sh["stem"]) * float(sh["r"]))).rotated(ang)
		_ok("R3 %s: tilted, the head is where the rotation puts it" % iset, _alpha_at(tilted, head_r) > 0.9, str(head_r))
		_ok("R3 %s: tilted, nothing is left at the upright head" % iset, _alpha_at(tilted, head) < 0.1)
		_ok("R3 %s: tilted, the tip has not moved (opaque 4 px along the axis from it)" % iset,
			_alpha_at(tilted, Vector2(150, 250) + (Vector2(0.0, -4.0)).rotated(ang)) > 0.5)
		var noted := await _render(kind, iset, 0.0, true)
		_ok("R4 %s: a linked note shows as an orange badge on the head" % iset, _differing(noted, icon) > 30)
	# Re-pointed 2026-11-15: tavern and fort now have glyph files (d3560db1), so the unmapped-kind probe kind is 'citadel'.
	var tav_icon := await _render("citadel", "glyph", 0.0)
	var tav_pin := await _render("citadel", "", 0.0)
	_ok("R5: an unmapped kind draws the drawn pin exactly (the fallback), %d px differ" % _differing(tav_icon, tav_pin), _differing(tav_icon, tav_pin) == 0)
	_ok("R5: and that is not the icon's silhouette", _opaque(tav_icon) < _opaque(await _render("temple", "glyph", 0.0)))
	var fort_glyph := await _render("fort", "glyph", 0.0)
	_ok("R6: a fort in the glyph set draws its own stockade pin (glyph/fort.png, d3560db1), not the drawn pin", _differing(fort_glyph, await _render("fort", "", 0.0)) > 400)
	_ok("R6: a citadel in the glyph set (no icon, fortified) stays the drawn pin", _differing(await _render("citadel", "glyph", 0.0), await _render("citadel", "", 0.0)) == 0)
	var fort_painted := await _render("fort", "painted", 0.0)
	_ok("R6: a fort in the painted set draws its own pin", _differing(fort_painted, await _render("fort", "", 0.0)) > 400)
	var fp_sh := Pin.shape(20.0, "fort", "painted")
	var fp_tilt := await _render("fort", "painted", PI * 0.5)
	_ok("R6: the long-stemmed pin's head is at its measured place when tilted",
		_alpha_at(fp_tilt, Vector2(150, 250) + (Vector2(0.0, -float(fp_sh["stem"]) * float(fp_sh["r"]))).rotated(PI * 0.5)) > 0.9)
	_ok("R7: sanity, the drawn temple pin differs from the glyph one", _differing(drawn, await _render("temple", "glyph", 0.0)) > 400)


# ---- H: the hit footprint ----------------------------------------------------------------------------------------

func _leg_h() -> void:
	print("== H. the hit footprint ==")
	var tip := Vector2(400, 400)
	var r := 20.0
	for iset in ["glyph", "painted"]:
		var kind := "temple" if iset == "glyph" else "monument"
		var sh := Pin.shape(r, kind, iset)
		var up := Vector2(0.0, -float(sh["stem"]) * float(sh["r"]))
		_ok("H1 %s: the upright head is hit" % iset, Pin.hit_styled(tip, r, tip + up, 2.0, false, 0.0, kind, iset))
		var ang := PI * 0.5
		var head_r := tip + up.rotated(ang)
		_ok("H2 %s: a tilted textured head is hit where it was drawn" % iset, Pin.hit_styled(tip, r, head_r, 2.0, false, ang, kind, iset))
		_ok("H2 %s: and not where the upright head would be" % iset, not Pin.hit_styled(tip, r, tip + up, 2.0, false, ang, kind, iset))
		_ok("H3 %s: the stem is hit near the tip" % iset, Pin.hit_styled(tip, r, tip + up * 0.2, 2.0, false, 0.0, kind, iset))
		var out := tip + up + Vector2(float(sh["r"]) + 6.0, 0.0)
		_ok("H4 %s: 6 px outside the head is a miss" % iset, not Pin.hit_styled(tip, r, out, 2.0, false, 0.0, kind, iset))
		var edge := tip + up + Vector2(float(sh["r"]) * 0.95, 0.0)
		_ok("H5 %s: its footprint is the texture's (bigger than the drawn pin's head)" % iset,
			Pin.hit_styled(tip, r, edge, 0.0, false, 0.0, kind, iset) and not Pin.hit_styled(tip, r, edge, 0.0, false, 0.0, "citadel", iset))
		_ok("H6 %s: the rank distance is 0 on the textured axis" % iset, Pin.rank_distance(tip, r, tip + up.rotated(ang), false, ang, kind, iset) < 0.01)
	_ok("H7: a collision test uses the icon's head (two icon pins 15 px apart collide; two drawn ones do not)",
		Pin.tilt_angles([{"id": 1, "pos": Vector2(0, 0), "r": 10.0, "kind": "temple"}, {"id": 2, "pos": Vector2(15, 0), "r": 10.0, "kind": "temple"}], "glyph").size() == 2
		and Pin.tilt_angles([{"id": 1, "pos": Vector2(0, 0), "r": 10.0, "kind": "temple"}, {"id": 2, "pos": Vector2(15, 0), "r": 10.0, "kind": "temple"}], "").is_empty())


# ---- F: Ruling FC on the map -------------------------------------------------------------------------------------

func _far_spot() -> Vector2:
	var places: Array = _bridge.settlements()
	for gx in range(40, 470, 7):
		for gy in range(40, 340, 7):
			var ok := true
			for s in places:
				if Vector2(float(s["x"]), float(s["y"])).distance_to(Vector2(gx, gy)) < 6.0:
					ok = false
					break
			if ok:
				return Vector2(gx + 0.5, gy + 0.5)
	return Vector2(-1, -1)


func _hits_has(hits: Array, kind: String, id: int) -> bool:
	for h in hits:
		if String(h.get("kind", "")) == kind and int(h.get("id", -1)) == id:
			return true
	return false


## The drawn heads of the two probe POIs at the CURRENT camera zoom (the pin's radius is floored in screen px, so it moves with it).
func _heads(rect: Rect2) -> Array:
	var shape := Pin.shape(_ov._poi_pin_r(), "temple", DccSettings.marker_icon_set())
	var out: Array = []
	for k in 2:
		var pd: Dictionary = _ov._pois[k]
		var tip: Vector2 = _ov._point_to_screen(Vector2(float(pd["x"]), float(pd["y"])), rect)
		out.append(tip + Vector2(0.0, -float(shape["stem"]) * float(shape["r"])))
	return out


func _leg_f() -> void:
	print("== F. Ruling FC ==")
	var places: Array = _bridge.settlements()
	if places.is_empty():
		_ok("F0: settlements exist", false)
		return
	var s: Dictionary = places[0]
	var far := _far_spot()
	_ok("F0: a spot far from every settlement exists", far.x > 0.0)
	_ov.set_pois([
		{"index": 0, "id": 9001, "x": float(s["x"]) + 0.5 + 0.1, "y": float(s["y"]) + 0.5, "kind": "temple", "kind_label": "Temple"},
		{"index": 1, "id": 9002, "x": far.x, "y": far.y, "kind": "temple", "kind_label": "Temple"},
	])
	DccSettings.set_marker_style("icons")
	var rect: Rect2 = _ov._displayed_rect()
	var interior: Rect2 = _ov._interior_rect(rect)
	var iset := DccSettings.marker_icon_set()
	_ok("F0: the icons style is the glyph set", iset == "glyph")
	# The layer's own threshold, computed independently: the span goes under UM_FADE_FAR_KM (24 km).
	var thr := 1200.0 * _ov.size.x / (rect.size.x * 24.0)
	var z0: float = _ov._camera_zoom
	_p("threshold zoom %.1f, rect %s" % [thr, str(rect.size)])
	_ov._camera_zoom = thr * 0.5
	var heads := _heads(rect)
	_ok("F1: below the layer's zoom the town's POI is hidden", _ov._poi_town_hidden(0, rect))
	_ok("F1: a POI that belongs to no town is shown", not _ov._poi_town_hidden(1, rect))
	_ok("F2: a hidden pin cannot be picked", (_ov._hit_test_poi(heads[0], interior, rect) as Array)[0] == -1)
	_ok("F2: the free pin can, with the textured footprint", (_ov._hit_test_poi(heads[1], interior, rect) as Array)[0] == 1)
	_ok("F2: hits_at agrees (the town's pin is absent)", not _hits_has(_ov.hits_at(heads[0]), "poi", 0))
	_ov._camera_zoom = thr * 0.97
	_ok("F3: just under the threshold it is still hidden", _ov._poi_town_hidden(0, rect))
	_ov._camera_zoom = thr * 1.05
	heads = _heads(rect)
	_ok("F4: just over the layer's zoom the town's POI shows", not _ov._poi_town_hidden(0, rect))
	_ok("F4: and is picked", (_ov._hit_test_poi(heads[0], interior, rect) as Array)[0] == 0)
	_ok("F4: the free pin is unchanged", (_ov._hit_test_poi(heads[1], interior, rect) as Array)[0] == 1)
	_ov._camera_zoom = z0
	var lms: Array = _ov._landmarks
	var near := -1
	var farlm := -1
	for k in lms.size():
		var lp := Vector2(float(lms[k].get("x", 0)) + 0.5, float(lms[k].get("y", 0)) + 0.5)
		var dmin := INF
		for st in places:
			dmin = minf(dmin, lp.distance_to(Vector2(float(st["x"]) + 0.5, float(st["y"]) + 0.5)))
		if dmin <= 0.3 and near < 0:
			near = k
		if dmin > 6.0 and farlm < 0:
			farlm = k
	_p("landmarks: %d, near a town %d, free %d" % [lms.size(), near, farlm])
	if near >= 0:
		_ov._camera_zoom = thr * 0.5
		_ok("F5: a landmark inside a town's box is hidden below the zoom", _ov._lm_town_hidden(near, rect))
		_ov._camera_zoom = thr * 1.05
		_ok("F5: and shown above it", not _ov._lm_town_hidden(near, rect))
	if farlm >= 0:
		_ov._camera_zoom = thr * 0.5
		_ok("F5: a landmark outside every town box is never hidden", not _ov._lm_town_hidden(farlm, rect))
	_ov._camera_zoom = z0


# ---- V: the City Viewer and the screenshots ----------------------------------------------------------------------

func _town() -> int:
	var places: Array = _bridge.settlements()
	for k in places.size():
		var kind := String((places[k] as Dictionary).get("kind", ""))
		if kind in ["city", "capital", "town", "metropolis"]:
			var lay: Array = _bridge.urban_layouts(PackedInt32Array([k]))
			if lay.size() > 0 and (lay[0] as Dictionary).has("market"):
				return k
	return -1


func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(4)
	await RenderingServer.frame_post_draw
	DirAccess.make_dir_recursive_absolute(_out)
	_vp.get_texture().get_image().save_png(_out.path_join(name))
	_p("shot %s" % name)


func _leg_v() -> void:
	print("== V. the City Viewer and the screenshots ==")
	var si := _town()
	_ok("V0: a town with a layout exists", si >= 0)
	if si < 0:
		return
	var s: Dictionary = _bridge.settlements()[si]
	var kinds := ["temple", "tavern", "mine", "shop"]
	for k in kinds.size():
		_bridge.poi_add(float(s["x"]) + 0.5 + 0.10 + 0.12 * k, float(s["y"]) + 0.5 - 0.06 + 0.05 * k, kinds[k], "P%d" % k)
	app.viewport.refresh_annotations()
	await _frames(6)
	var pin_cell := Vector2(float(s["x"]) + 0.5, float(s["y"]) + 0.5)
	for style in ["pins", "icons", "painted"]:
		DccSettings.set_marker_style(style)
		var iset := DccSettings.marker_icon_set()
		_ov.queue_redraw()
		await _frames(6)
		app.viewport.reset_view()
		await _frames(8)
		await _shot("map_out_%s_%s.png" % [style, _form])
		var local: Vector2 = _ov._cell_to_screen(pin_cell, _ov._displayed_rect())
		var host_pt: Vector2 = (app.viewport as Control).get_global_transform_with_canvas().affine_inverse() * (_ov.get_global_transform_with_canvas() * local)
		var z := 1.0
		while z < 60.0:
			app.viewport._zoom_at(host_pt, 1.6)
			z *= 1.6
			await _frames(2)
		await _frames(30)
		await _shot("map_in_%s_%s.png" % [style, _form])
		app.viewport.reset_view()
		await _frames(6)
		app.open_city_viewer(si)
		await _frames(16)
		var cv: CityViewerWindow = app.city_viewer_window
		var pins: Array = cv._pins()
		var pr: float = cv.PIN_HEAD_R * cv._pin_unit()
		var picked := 0
		var mapped := 0
		for k in pins.size():
			var kind := String((pins[k] as Dictionary)["kind"])
			if iset == "" or Pin.icon_name_for(iset, kind) == "":
				continue
			mapped += 1
			var sh := Pin.shape(pr, kind, iset)
			var head: Vector2 = cv.pin_tip(k) + (Vector2(0.0, -float(sh["stem"]) * float(sh["r"]))).rotated(cv.pin_tilt(k))
			if cv.pin_at(head) == k:
				picked += 1
		if iset != "":
			_ok("V1 %s: the viewer picks each textured pin at its drawn head (%d of %d)" % [style, picked, mapped], mapped > 0 and picked == mapped)
		_ok("V2 %s: the viewer shows pins at the map's fit zoom (Ruling FC: always in the viewer)" % style, pins.size() > 0)
		await _shot("viewer_%s_%s.png" % [style, _form])
		cv.hide()
		await _frames(4)
