extends Node
## Settlement pins probe (Ruling FF items 1 and 2, `CITY_MAP_LOOK_SCOPE.md` CL-10; `map_overlay.gd` "Settlement pins").
## Windowed (refuses --headless): the pixel frames are the evidence. Run under a scratch APPDATA (a FRESH one: bar C1 reads
## the default city look before anything is stored).
##
##   PHONE:   `-- --vp 1080x2340 --force-touch --density 2.625 --out <dir> --tag <name>`
##   DESKTOP: `-- --vp 1600x900 --out <dir> --tag <name>`
## `--circles-only` runs just the Circles frame hashes (it also works on a tree without the settlement pins, which is how
## "Circles is unchanged" is measured: the same map crop must hash identically before and after).
## Frames: `<tag>_<form>_<style>_<fit|x4>.png`. Exit 0 = held, 1 = failed, 2 = could not run.
##
## Bars, per Pins style (pins, icons, painted) unless noted (seed 1200, 512x384, 1200 km):
##   P1  a settlement of each tier on screen draws as a PIN at fit and at 4x: the pixel in the upper head differs from the
##       same point in the Circles frame, and for the drawn Pins style equals the faction colour
##   P2  picking: the head and the stem of a shown settlement pin pick it (`_hit_test_settlement`, `hits_at`, and a real
##       press raises `settlement_selected`); a point where the old disc was but the pin is not picks nothing
##   P3  capitals survive the declutter rule, even when two capitals clash (both shown, tilted apart by Ruling FA)
##   P4  every hidden settlement has a shown conflicting pin of equal or higher rank (the rank scale works), no capital or
##       city is hidden by a village-or-lower pin, and the rule hides some settlement at fit (it is live)
##   P5  a selected settlement is always shown; hidden again once deselected
##   P6  each placed name's box clears its own pin's box (labels sit beside pins), and reserves the pin for later names
##   P7  a hidden settlement is not picked and is not drawn as a pin (it is a dot)
##   C1  `DccSettings.city_look()` reads "map" on a fresh APPDATA; a stored "ink" is kept (also after a reload); a corrupt value
##       reads the default "map"
##   C2  Circles: `_settle_pins_on()` is false, no settlement is declutter-hidden, no settlement tilts, and the hit radius is the
##       disc's (a point on the old disc picks it)

const TIERS := ["metropolis", "capital", "city", "town", "village", "hamlet"]

var app: Node
var _vp: SubViewport
var _fails := 0
var _out := ""
var _tag := "real"
var _form := "desktop"
var _circles_only := false


func _p(s: String) -> void:
	print(s)


func _ok(name: String, cond: bool, detail: String = "") -> void:
	if cond:
		_p("PASS " + name)
	else:
		_fails += 1
		_p("FAIL %s  %s" % [name, detail])


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _img() -> Image:
	await _frames(5)
	await RenderingServer.frame_post_draw
	return _vp.get_texture().get_image()


func _shot(img: Image, name: String) -> void:
	if _out == "":
		return
	img.save_png(_out.path_join("%s_%s_%s" % [_tag, _form, name]))
	_p("shot %s_%s_%s" % [_tag, _form, name])


func _px(ov, img: Image, local: Vector2) -> Color:
	var g: Vector2 = ov.get_global_transform() * local
	var x := clampi(int(g.x), 0, img.get_width() - 1)
	var y := clampi(int(g.y), 0, img.get_height() - 1)
	return img.get_pixel(x, y)


func _cdist(a: Color, b: Color) -> float:
	return (absf(a.r - b.r) + absf(a.g - b.g) + absf(a.b - b.b)) * 255.0


## The map crop's hash (the status bar and docks carry clocks and counters, the map does not).
func _crop_hash(host, img: Image) -> String:
	var vr: Rect2i = Rect2i(host.get_global_rect())
	if _form == "phone":
		## the status-bar clock and the fading coach tips sit outside the clear middle of the map
		vr = Rect2i(0, 340, img.get_width(), 1000)
	vr = vr.intersection(Rect2i(Vector2i.ZERO, img.get_size()))
	var hc := HashingContext.new()
	hc.start(HashingContext.HASH_MD5)
	hc.update(img.get_region(vr).get_data())
	return hc.finish().hex_encode()


func _drawn(ov, i: int, rect: Rect2, interior: Rect2) -> bool:
	var s: Dictionary = ov._settlements[i]
	if ov._hidden_settlement_kinds.has(s["kind"]) or ov._settlement_hidden(s) or ov._tl_ghost_tids.has(int(s.get("tid", 0))):
		return false
	return interior.has_point(ov._cell_to_screen(Vector2(s["x"], s["y"]), rect))


func _info(ov, i: int, rect: Rect2) -> Dictionary:
	var s: Dictionary = ov._settlements[i]
	var pos: Vector2 = ov._cell_to_screen(Vector2(s["x"], s["y"]), rect)
	var sc: float = (rect.size.x / ov.PIN_SCALE_REF_PX) * ov._civ_zoom_k()
	var g: Dictionary = ov._settlement_pin_geom(i, s, pos, rect, sc)
	g["pos"] = pos
	g["sample"] = (g["head"] as Vector2) + Vector2(0.0, -0.8 * float(g["r"])).rotated(float(g["angle"]))
	g["stem"] = pos + (((g["head"] as Vector2) - pos) * 0.45)
	return g


## Settlements whose pin is alone at its sample point (no other settlement, landmark or POI footprint there), on screen.
func _isolated(ov, host, rect: Rect2, interior: Rect2) -> Dictionary:
	var by_tier := {}
	var vr: Rect2 = host.get_global_rect()
	if _form == "phone":
		## the phone's chrome (title bar, zoom buttons, tool strip, sheet) floats over the map: sample the clear middle only
		vr = Rect2(vr.position + Vector2(0.0, 340.0), vr.size - Vector2(0.0, 340.0 + 820.0))
	var gt: Transform2D = ov.get_global_transform()
	for i in ov._settlements.size():
		if not _drawn(ov, i, rect, interior) or ov._settlement_declutter_hidden(i, rect):
			continue
		var inf := _info(ov, i, rect)
		if not vr.grow(-20.0).has_point(gt * (inf["head"] as Vector2)):
			continue
		var lone := true
		for h in ov.hits_at(inf["sample"]):
			var kd := String(h.get("kind", ""))
			if (kd == "settlement" and int(h["id"]) != i) or kd == "landmark" or kd == "poi":
				lone = false
		if not lone:
			continue
		var tier := String(ov._settlements[i]["kind"])
		if not by_tier.has(tier):
			by_tier[tier] = []
		(by_tier[tier] as Array).append(i)
	return by_tier


func _rank(ov, i: int) -> float:
	return float(ov.DECLUTTER_SETTLE_RANK.get(ov._settle_display_kind(ov._settlements[i]), 3.0))


## The declutter rule's keep-out test between two pins (head radius and tip), restated independently of the rule.
func _clash(ta: Vector2, ra: float, tb: Vector2, rb: float, stem: float) -> bool:
	var ha := ta + Vector2(0.0, -stem * ra)
	var hb := tb + Vector2(0.0, -stem * rb)
	return ha.distance_to(hb) < 1.5 * (ra + rb) or ta.distance_to(hb) < 1.2 * rb or tb.distance_to(ha) < 1.2 * ra


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
		elif a == "--out":
			_out = String(args[i + 1])
			i += 2
		elif a == "--tag":
			_tag = String(args[i + 1])
			i += 2
		elif a == "--density":
			i += 2
		elif a == "--circles-only":
			_circles_only = true
			i += 1
		elif a == "--force-touch" or a.begins_with("--density="):
			i += 1
		else:
			_p("ABORT unknown argument %s" % a)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		_p("ABORT run windowed")
		get_tree().quit(2)
		return
	var wd := get_tree().create_timer(600.0)
	wd.timeout.connect(func():
		_p("WATCHDOG")
		get_tree().quit(3))
	## C1 first: nothing has stored a look yet on a fresh APPDATA.
	var fresh_look := DccSettings.city_look()
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
	_form = "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() and DccTheme.is_touch() else "desktop")
	_p("form: %s, viewport %s" % [_form, str(vp_size)])
	var bridge = app.bridge
	bridge.generate({"seed": 1200, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	_ok("a world generated", bridge.has_world)
	var ov = app.viewport.overlay
	var host = app.viewport
	await bridge.landmark_run()
	await _frames(6)
	var t0 := Time.get_ticks_msec()
	while app.busy_overlay != null and app.busy_overlay.visible and Time.get_ticks_msec() - t0 < 30000:
		await _frames(10)
	await _frames(20)
	app.select_domain("cartography")
	await _frames(8)
	app.arm_tool("inspect")
	await _frames(4)
	var pins_build: bool = ov.has_method("_settle_pins_on")
	_p("PINSBUILD %s settlements=%d" % [str(pins_build), ov._settlements.size()])

	## ---- the Circles frames (the baseline for P1, and the hash C2 compares across trees) ----
	DccSettings.set_marker_style("circles")
	ov.queue_redraw()
	host.reset_view()
	await _frames(10)
	var rect: Rect2 = ov.displayed_rect()
	var interior: Rect2 = ov._interior_rect(rect)
	var circ_fit := await _img()
	await _shot(circ_fit, "circles_fit.png")
	_p("CIRCLEHASH %s fit %s" % [_form, _crop_hash(host, circ_fit)])
	host._zoom_at(host.size * 0.5, 4.0)
	await _frames(12)
	await get_tree().create_timer(1.5).timeout   ## deep-zoom tiles finish: the hash must not depend on timing
	var circ_x4 := await _img()
	await _shot(circ_x4, "circles_x4.png")
	_p("CIRCLEHASH %s x4 %s" % [_form, _crop_hash(host, circ_x4)])
	if _circles_only or not pins_build:
		## The "before" frames on a tree without settlement pins: the default Pins style, settlements still discs.
		DccSettings.set_marker_style("pins")
		ov.queue_redraw()
		host.reset_view()
		await _frames(10)
		await _shot(await _img(), "pins_fit.png")
		host._zoom_at(host.size * 0.5, 4.0)
		await _frames(12)
		await _shot(await _img(), "pins_x4.png")
		get_tree().quit(0)
		return
	## C2
	var rc: Rect2 = ov.displayed_rect()
	_ok("C2 Circles: pins are off", not ov._settle_pins_on())
	var any_hidden := false
	for k in ov._settlements.size():
		if ov._settlement_declutter_hidden(k, rc):
			any_hidden = true
	_ok("C2 Circles: no settlement is declutter-hidden", not any_hidden)
	_ok("C2 Circles: no settlement tilts", (ov._fan(rc)["s_a"] as Dictionary).is_empty())
	var ic: Rect2 = ov._interior_rect(rc)
	var disc_ok := 0
	var disc_n := 0
	for k in ov._settlements.size():
		if not _drawn(ov, k, rc, ic) or ov._settlement_below_lod(String(ov._settlements[k]["kind"])):
			continue
		var s: Dictionary = ov._settlements[k]
		var ps: Vector2 = ov._cell_to_screen(Vector2(s["x"], s["y"]), rc)
		disc_n += 1
		if ov._hit_test_settlement(ps + Vector2(1.0, 0.0), ic, rc) >= 0:
			disc_ok += 1
		if disc_n >= 6:
			break
	_ok("C2 Circles: a point on the disc picks a settlement (%d of %d)" % [disc_ok, disc_n], disc_n > 0 and disc_ok == disc_n)

	## ---- C1 ----
	_ok("C1 a fresh APPDATA reads the city look 'map' (got '%s')" % fresh_look, fresh_look == "map")
	DccSettings.set_city_look("ink")
	_ok("C1 a stored 'ink' is read back", DccSettings.city_look() == "ink")
	DccSettings._loaded = false
	_ok("C1 and survives a reload of the settings file", DccSettings.city_look() == "ink")
	DccSettings._ensure_loaded()
	DccSettings._cfg.set_value("city_look", "style", "bogus")
	_ok("C1 a corrupt value reads the default 'map'", DccSettings.city_look() == "map")
	DccSettings.set_city_look("map")
	_ok("C1 'map' stays selectable", DccSettings.city_look() == "map")
	DccSettings.set_city_look("ink")
	_ok("C1 'ink' stays selectable", DccSettings.city_look() == "ink")
	DccSettings.set_city_look("map")

	for style in ["pins", "icons", "painted"]:
		await _run_style(style, circ_fit, circ_x4)
	_p("== done: %d failed ==" % _fails)
	get_tree().quit(1 if _fails > 0 else 0)


func _run_style(style: String, circ_fit: Image, circ_x4: Image) -> void:
	var ov = app.viewport.overlay
	var host = app.viewport
	DccSettings.set_marker_style(style)
	ov.queue_redraw()
	host.reset_view()
	await _frames(10)
	for zoom_name in ["fit", "x4"]:
		if zoom_name == "x4":
			## Zoom about a drawn capital, so settlements are on screen (the map centre may hold none).
			var pivot: Vector2 = host.size * 0.5
			var r0: Rect2 = ov.displayed_rect()
			for k in ov._settlements.size():
				if String(ov._settle_display_kind(ov._settlements[k])) == "capital" and _drawn(ov, k, r0, ov._interior_rect(r0)):
					var st: Dictionary = ov._settlements[k]
					pivot = host.get_global_transform().affine_inverse() * (ov.get_global_transform() * ov._cell_to_screen(Vector2(st["x"], st["y"]), r0))
					break
			host._zoom_at(pivot, 4.0)
			await _frames(12)
			await get_tree().create_timer(1.5).timeout   ## the cached settlement layer rebuilds in the background
		var rect: Rect2 = ov.displayed_rect()
		var interior: Rect2 = ov._interior_rect(rect)
		var img := await _img()
		await _shot(img, "%s_%s.png" % [style, zoom_name])
		var base: Image = circ_fit if zoom_name == "fit" else circ_x4
		var tag := "%s/%s/%s" % [_form, style, zoom_name]
		var min_n := 3 if zoom_name == "fit" else 1   ## at 4x few pins stand alone: one is enough (the fit frame carries the tiers)
		var iso := _isolated(ov, host, rect, interior)
		var tiers_seen := iso.keys()
		_p("TOWNPINS %s tiers_isolated=%s shown_settlements=%d hidden=%d" % [tag, str(tiers_seen),
			_count_shown(ov, rect, interior), (ov._declutter_hidden_settlements(rect) as Dictionary).size()])
		## P1
		var checked := 0
		var drew := 0
		var hamlet_n := 0   ## the reference draws hamlets and villages as LOD dots at the fit zoom: a pin must replace the dot
		var hamlet_drew := 0
		var faction_ok := 0
		var faction_n := 0
		for tier in iso:
			for i in (iso[tier] as Array).slice(0, 2):
				var inf := _info(ov, i, rect)
				var got := _px(ov, img, inf["sample"])
				var was := _px(ov, base, inf["sample"])
				checked += 1
				if _cdist(got, was) > 40.0:
					drew += 1
				if tier == "hamlet" or tier == "village":
					hamlet_n += 1
					if _cdist(got, was) > 40.0:
						hamlet_drew += 1
				if style == "pins":
					faction_n += 1
					if _cdist(got, ov._faction_color(int(ov._settlements[i]["faction"]))) < 30.0:
						faction_ok += 1
					else:
						_p("  faction mismatch %s i=%d kind=%s got=%s want=%s r=%.2f" % [tag, i, tier, str(got), str(ov._faction_color(int(ov._settlements[i]["faction"]))), float(inf["r"])])
		_ok("P1 %s settlements draw as pins (%d of %d sampled heads differ from the Circles frame)" % [tag, drew, checked],
			checked >= min_n and drew * 4 >= checked * 3)
		if style == "pins":
			_ok("P1 %s the drawn pin is the faction colour (%d of %d)" % [tag, faction_ok, faction_n], faction_n >= min_n and faction_ok * 4 >= faction_n * 3)
		## P2
		var picked := 0
		var pick_n := 0
		var stem_ok := 0
		var hits_ok := 0
		for tier in iso:
			for i in (iso[tier] as Array).slice(0, 2):
				var inf := _info(ov, i, rect)
				pick_n += 1
				if ov._hit_test_settlement(inf["sample"], interior, rect) == i:
					picked += 1
				if ov._hit_test_settlement(inf["stem"], interior, rect) == i:
					stem_ok += 1
				for h in ov.hits_at(inf["sample"]):
					if String(h.get("kind", "")) == "settlement" and int(h["id"]) == i:
						hits_ok += 1
						break
		_ok("P2 %s the head picks its settlement (%d of %d)" % [tag, picked, pick_n], pick_n >= min_n and picked == pick_n)
		_ok("P2 %s the stem picks it too (%d of %d)" % [tag, stem_ok, pick_n], stem_ok == pick_n)
		_ok("P2 %s hits_at lists it (%d of %d)" % [tag, hits_ok, pick_n], hits_ok == pick_n)
		## a point beside the pin (the old disc's neighbourhood far from head and stem) picks nothing
		var far_ok := 0
		var far_n := 0
		for tier in iso:
			var i0: int = (iso[tier] as Array)[0]
			var inf0 := _info(ov, i0, rect)
			var side: Vector2 = (inf0["head"] as Vector2) + Vector2(3.5 * float(inf0["r"]), 0.0)
			far_n += 1
			if ov._hit_test_settlement(side, interior, rect) != i0:
				far_ok += 1
		_ok("P2 %s a point 3.5 radii beside the head does not pick it (%d of %d)" % [tag, far_ok, far_n], far_n > 0 and far_ok == far_n)
		## a real press
		var pressed := [-2]
		var cb := func(_d, idx: int) -> void: pressed[0] = idx
		ov.settlement_selected.connect(cb)
		var any_tier := String(iso.keys()[0]) if not iso.is_empty() else ""
		if any_tier != "":
			var i1: int = (iso[any_tier] as Array)[0]
			if _form == "phone":
				## A press on the phone also opens the Sample sheet over the map: take the pick the press would use instead.
				var mk: Dictionary = ov.marker_at((_info(ov, i1, rect)["sample"] as Vector2))
				_ok("P2 %s marker_at (the pick a tap uses) returns the pin's settlement (got %s)" % [tag, str(mk)],
					String(mk.get("kind", "")) == "settlement" and int(mk.get("id", -1)) == i1)
			else:
				var ev := InputEventMouseButton.new()
				ev.button_index = MOUSE_BUTTON_LEFT
				ev.pressed = true
				ev.position = (_info(ov, i1, rect)["sample"] as Vector2)
				## The press also raises `element_clicked` (CL-3's card); keep the shell's listener off for the probe's press.
				ov.element_clicked.disconnect(app._on_element_clicked)
				ov._gui_input(ev)
				ov.element_clicked.connect(app._on_element_clicked)
				_ok("P2 %s a press on the pin selects it (got %d, want %d)" % [tag, int(pressed[0]), i1], int(pressed[0]) == i1)
				var rel := InputEventMouseButton.new()
				rel.button_index = MOUSE_BUTTON_LEFT
				rel.pressed = false
				rel.position = ev.position
				ov._gui_input(rel)
				ov.set_selected_settlement(-1)
				ov.settlement_selected.emit(null, -1)   ## the shell's card or panel for the pick goes away again
				await _frames(6)
		ov.settlement_selected.disconnect(cb)
		## P3 / P4 / P7
		var hidden: Dictionary = ov._declutter_hidden_settlements(rect)
		var cap_hidden := 0
		for k in ov._settlements.size():
			var d: String = ov._settle_display_kind(ov._settlements[k])
			if (d == "capital" or d == "metropolis") and hidden.has(k):
				cap_hidden += 1
		_ok("P3 %s no capital is hidden by the declutter rule (%d)" % [tag, cap_hidden], cap_hidden == 0)
		if zoom_name == "fit":
			_ok("P4 %s the rule thins some settlement at fit (%d)" % [tag, hidden.size()], hidden.size() > 0)
		var unexplained := 0
		var low_beat_high := 0
		var canon := Rect2(Vector2.ZERO, rect.size)
		for h in hidden:
			var sh: Dictionary = ov._settlements[int(h)]
			var th: Vector2 = ov._cell_to_screen(Vector2(sh["x"], sh["y"]), canon)
			var rh: float = ov._settlement_head_r(String(sh["kind"]), canon)
			var found := false
			for k in ov._settlements.size():
				if k == int(h) or hidden.has(k):
					continue
				var s2: Dictionary = ov._settlements[k]
				if not _drawn(ov, k, rect, Rect2(Vector2(-1e7, -1e7), Vector2(2e7, 2e7))):
					continue
				var t2: Vector2 = ov._cell_to_screen(Vector2(s2["x"], s2["y"]), canon)
				if _clash(th, rh, t2, ov._settlement_head_r(String(s2["kind"]), canon), ov.PinMarker.STEM_RATIO) \
						and _rank(ov, k) >= _rank(ov, int(h)):
					found = true
					break
			if not found:
				## a landmark of lower class could never be the reason; a landmark of higher class may be, but then
				## its class rank must beat the settlement's.
				for li in ov._landmarks.size():
					var lm: Dictionary = ov._landmarks[li]
					if not ov._landmark_shown(lm) or ov._lm_hidden(li, rect):
						continue
					var tl: Vector2 = ov._cell_to_screen(Vector2(float(lm["x"]), float(lm["y"])), canon)
					var cr := float({"continental": 3, "regional": 2, "cultural": 1, "local": 0}.get(String(lm.get("class", "local")), 0))
					if _clash(th, rh, tl, ov._landmark_pin_r(lm), ov.PinMarker.STEM_RATIO) and (cr >= _rank(ov, int(h)) or ov.landmark_has_note(li)):
						found = true
						break
			if not found:
				unexplained += 1
			var dk: String = ov._settle_display_kind(sh)
			if (dk == "city") and _blocked_by_minor(ov, int(h), rect):
				low_beat_high += 1
		_ok("P4 %s every hidden settlement (%d) has a shown, conflicting pin of equal or higher rank (%d without)" % [tag, hidden.size(), unexplained], unexplained == 0)
		_ok("P4 %s no city is hidden by a village or hamlet pin (%d)" % [tag, low_beat_high], low_beat_high == 0)
		## P7: a hidden settlement is not picked at its own head
		var leak := 0
		var lc := 0
		for h in hidden:
			if lc >= 8:
				break
			lc += 1
			var sh2: Dictionary = ov._settlements[int(h)]
			var p2: Vector2 = ov._cell_to_screen(Vector2(sh2["x"], sh2["y"]), rect)
			var hd: Vector2 = p2 + Vector2(0.0, -ov._settlement_head_r(String(sh2["kind"]), rect) * ov.PinMarker.STEM_RATIO)
			if interior.has_point(p2) and ov._hit_test_settlement(hd, interior, rect) == int(h):
				leak += 1
		_ok("P7 %s a hidden settlement is not picked at its own head (%d leaks)" % [tag, leak], leak == 0)
		## P6: labels beside the pin
		var L: Dictionary = ov._layers["settlements"]
		var plan = L["plan"][L["front"]]
		var overlaps := 0
		var labelled := 0
		var pins_n := 0
		if plan != null:
			for e in (plan["entries"] as Array):
				if not e.has("pin"):
					continue
				pins_n += 1
				if e["box"] != null:
					labelled += 1
					if (e["box"] as Rect2).intersects((e["pin"]["bbox"] as Rect2)):
						overlaps += 1
		_ok("P6 %s no name box overlaps its own pin (%d labelled of %d pins, %d overlaps)" % [tag, labelled, pins_n, overlaps],
			pins_n >= 3 and labelled >= 1 and overlaps == 0)
	## P1b: far below the fit zoom the reference's tier LOD would draw hamlets and villages as dots; in the Pins styles every
	## settlement the declutter rule keeps is a pin, and in Circles (the positive control) the dots are still there.
	if style == "pins":
		var zsave: float = ov._camera_zoom
		ov._camera_zoom = float(ov._lod_zoom_base()) * 0.9   ## below the hamlet tier's 1.4 threshold, near the fit view
		var lr: Rect2 = ov.displayed_rect()
		var plan_p: Dictionary = ov._settlement_plan_begin(lr, [])
		for st in range(ov.SETTLE_PLAN_CHUNKS):
			ov._settlement_plan_step(plan_p, lr, ov._interior_rect(lr), st, ov.SETTLE_PLAN_CHUNKS)
		var pins_n2 := 0
		var dots_n2 := 0
		for e in (plan_p["entries"] as Array):
			if e.has("pin"):
				if String(ov._settlements[int(e["i"])]["kind"]) == "hamlet":
					pins_n2 += 1
			elif bool(e["dot"]) and not ov._settlement_declutter_hidden(int(e["i"]), lr):
				dots_n2 += 1
		_ok("P1b %s below the hamlet LOD threshold a kept hamlet is a pin, not a dot (%d hamlet pins, %d LOD dots)" % [style, pins_n2, dots_n2], pins_n2 > 0 and dots_n2 == 0)
		DccSettings.set_marker_style("circles")
		var plan_c: Dictionary = ov._settlement_plan_begin(lr, [])
		for st in range(ov.SETTLE_PLAN_CHUNKS):
			ov._settlement_plan_step(plan_c, lr, ov._interior_rect(lr), st, ov.SETTLE_PLAN_CHUNKS)
		var dots_c := 0
		for e in (plan_c["entries"] as Array):
			if bool(e["dot"]):
				dots_c += 1
		_ok("P1b control: Circles still draws the LOD dots at that zoom (%d)" % dots_c, dots_c > 0)
		DccSettings.set_marker_style("pins")
		ov._camera_zoom = zsave
	## P5 and the clashing-capitals fixture, at the fit zoom
	host.reset_view()
	await _frames(10)
	var rect2: Rect2 = ov.displayed_rect()
	var hid: Dictionary = ov._declutter_hidden_settlements(rect2)
	var hk := -1
	for h in hid:
		hk = int(h)
		break
	if hk >= 0:
		ov.set_selected_settlement(hk)
		_ok("P5 %s a selected settlement is shown" % style, not ov._settlement_declutter_hidden(hk, rect2))
		ov.set_selected_settlement(-1)
		_ok("P5 %s and hidden again once deselected" % style, ov._settlement_declutter_hidden(hk, rect2))
	else:
		_ok("P5 %s found a hidden settlement to select" % style, false)
	var cap := -1
	for k in ov._settlements.size():
		if String(ov._settle_display_kind(ov._settlements[k])) == "capital" and _drawn(ov, k, rect2, ov._interior_rect(rect2)):
			cap = k
			break
	if cap >= 0:
		var saved: Array = ov._settlements
		var twin: Dictionary = (ov._settlements[cap] as Dictionary).duplicate()
		twin["x"] = int(twin["x"]) + 1
		twin["tid"] = 99999
		var two: Array = saved.duplicate()
		two.append(twin)
		ov._settlements = two
		ov._fan_rev += 1
		var ni: int = two.size() - 1
		var nrect: Rect2 = ov.displayed_rect()
		_ok("P3 %s two clashing capitals are both shown" % style,
			not ov._settlement_declutter_hidden(cap, nrect) and not ov._settlement_declutter_hidden(ni, nrect))
		var ta: float = ov._tilt_s(cap, nrect)
		var tb: float = ov._tilt_s(ni, nrect)
		_p("TILT %s %.1f %.1f" % [style, rad_to_deg(ta), rad_to_deg(tb)])
		_ok("P3 %s the two capitals tilt apart by Ruling FA (%.1f, %.1f deg)" % [style, rad_to_deg(ta), rad_to_deg(tb)],
			absf(ta) > 0.1 and absf(tb) > 0.1 and ta * tb < 0.0)
		## P5b: a SELECTED small place beside a capital stays shown although the capital (always kept, higher rank) clashes
		## with it: only the forced flag, not the walk order, can save it.
		var hut: Dictionary = (ov._settlements[cap] as Dictionary).duplicate()
		hut["x"] = int(hut["x"]) + 1
		hut["tid"] = 99998
		hut["kind"] = "hamlet"
		hut["capital"] = false
		hut["population"] = 5
		var withhut: Array = saved.duplicate()
		withhut.append(hut)
		ov._settlements = withhut
		ov._fan_rev += 1
		var hi: int = withhut.size() - 1
		var hrect: Rect2 = ov.displayed_rect()
		_ok("P5b %s an unselected hamlet beside a capital is thinned" % style, ov._settlement_declutter_hidden(hi, hrect))
		ov.set_selected_settlement(hi)
		_ok("P5b %s the same hamlet, selected, is shown beside the capital" % style, not ov._settlement_declutter_hidden(hi, hrect))
		ov.set_selected_settlement(-1)
		ov._settlements = saved
		ov._fan_rev += 1


func _count_shown(ov, rect: Rect2, interior: Rect2) -> int:
	var n := 0
	for i in ov._settlements.size():
		if _drawn(ov, i, rect, interior) and not ov._settlement_declutter_hidden(i, rect):
			n += 1
	return n


## True when a shown village or hamlet pin conflicts with hidden settlement `h` (it would be the reason a city was thinned).
func _blocked_by_minor(ov, h: int, rect: Rect2) -> bool:
	var canon := Rect2(Vector2.ZERO, rect.size)
	var sh: Dictionary = ov._settlements[h]
	var th: Vector2 = ov._cell_to_screen(Vector2(sh["x"], sh["y"]), canon)
	var rh: float = ov._settlement_head_r(String(sh["kind"]), canon)
	for k in ov._settlements.size():
		var dk: String = ov._settle_display_kind(ov._settlements[k])
		if k == h or (dk != "village" and dk != "hamlet") or ov._settlement_declutter_hidden(k, rect):
			continue
		var s2: Dictionary = ov._settlements[k]
		var t2: Vector2 = ov._cell_to_screen(Vector2(s2["x"], s2["y"]), canon)
		if _clash(th, rh, t2, ov._settlement_head_r(String(s2["kind"]), canon), ov.PinMarker.STEM_RATIO):
			return true
	return false
