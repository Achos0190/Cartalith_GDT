extends Node
## Pin declutter probe (map_overlay.gd, "Pin declutter" block; CITY_MAP_LOOK_SCOPE.md CL-12). Windowed (refuses --headless):
## a pixel frame and the real zoom path are part of the evidence. Run under a scratch APPDATA.
##
##   PHONE:   `-- --vp 1080x2340 --force-touch --density 2.625 --out <dir> --tag <name>`
##   TABLET:  `-- --vp 1700x1000 --force-touch --density=1.0 --out <dir> --tag <name>`
##   DESKTOP: `-- --vp 1600x900 --out <dir> --tag <name>`
## `--before` turns the rule off first (the baseline numbers and the "before" frames). Prints `DECL <form> ...` count lines.
## Frames: `<tag>_<form>_fit.png` and `<tag>_<form>_x4.png`. Exit 0 = held, 1 = failed, 2 = could not run.
##
## Bars (all measured on seed 1200, 512x384, 1200 km; stated here so a mutant cannot hide in a loose number):
##   B1  shown pins at the fit zoom <= SHOWN_MAX[form]  (and at least 8: the rule must not blank the map)
##   B2  no two shown pins' heads overlap at all (worst overlap fraction == 0, `1 - dist/(ra+rb)` clamped at 0)
##   B3  the shown set is identical under a 1 px pan (rect shifted, and the live camera shifted)
##   B4  zooming in 2x shows MORE pins than at the fit zoom
##   B5  a selected pin, a noted pin and a user POI are shown while an unselected, un-noted twin is not
##   B2b no tip of a shown pin lies on another shown pin's head (a vertical stack)
##   B7  every hidden pin is displaced by a shown pin that ranks at least as high and conflicts with it (the ranking works)
##   B8  Ruling FA tilts only shown pins (a hidden pin never tilts a neighbour)
##   B5b two noted pins that clash are BOTH shown (the always-keep set is never thinned by its own members)
##   B4b with the zoom law off, 2x still shows more pins (the cache key carries the zoom itself)
##   B6  a hidden pin is not picked at its own head; the same point picks it once it is shown (positive control)
##
## 2026-11-16: the Circles style and the export (the leftovers of CL-12 and `92a63f48`), each against `_brute`, an O(n^2) restatement
## of the walk with LITERAL constants (1.5 / 1.2 for pins, 1.15 for discs), so a constant moved in the overlay shows here:
##   C1  Circles at the fit zoom thins: fewer shown marks than with the rule off, and the set equals `_brute`
##   C2  no two shown Circles marks (landmark rings and settlement discs) overlap (worst overlap 0)
##   C3  the Circles hidden set is identical under a 1 px and a 37,53 px pan
##   C4  a selected / noted landmark is shown in Circles; a capital settlement is shown
##   C5  a hidden Circles landmark and a hidden Circles settlement are not picked or listed by `hits_at`; the settlement plan draws
##       every hidden settlement as the faction dot and every shown one as a disc
##   C6  switching Pins -> Circles -> Pins moves the set (the cache key carries the style) and the Pins set is the same again
##   X1  an export copy (2048 px of the fit view, every pin kind on) thins: fewer shown pins than with the rule off, and the sets equal `_brute`
##   X2  no two shown export pins overlap (heads, and no tip on a head); X3 the set does not move under a pan of the rect
##   X4  the export's settlement plan draws every hidden settlement as a dot; X5 the Circles export obeys the same bars

const SHOWN_MAX := {"phone": 160, "tablet": 160, "desktop": 160}
const VIEW_MAX := {"phone": 60, "tablet": 150, "desktop": 150}

var app: Node
var _vp: SubViewport
var _fails := 0
var _out := ""
var _tag := "real"
var _form := "desktop"
var _before := false


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


func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(4)
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png(_out.path_join("%s_%s_%s" % [_tag, _form, name]))
	_p("shot %s_%s_%s" % [_tag, _form, name])


## Sorted landmark indices the rule hides at `rect`.
func _hidden(ov, rect: Rect2) -> Array:
	var k: Array = ov._declutter_set(rect).keys()
	k.sort()
	return k


## Landmark indices that draw at `rect`: every landmark whose kind is on and not hidden by the town gate or the rule.
func _shown(ov, rect: Rect2) -> Array:
	var out: Array = []
	for i in ov._landmarks.size():
		if ov._landmark_shown(ov._landmarks[i]) and not ov._lm_hidden(i, rect):
			out.append(i)
	return out


## How many of `idx` have their tip inside the host's visible rectangle (what the user actually sees).
func _in_view(ov, host, idx: Array, rect: Rect2) -> int:
	var vr: Rect2 = host.get_global_rect()
	var n := 0
	for i in idx:
		var lm: Dictionary = ov._landmarks[i]
		var tip: Vector2 = ov._cell_to_screen(Vector2(float(lm["x"]), float(lm["y"])), rect)
		if vr.has_point(ov.get_global_transform() * tip):
			n += 1
	return n


## Worst head overlap fraction among `idx`, on the canonical rect (the rule's own frame): 1 - dist/(ra+rb), clamped at 0.
func _worst_overlap(ov, idx: Array, rect: Rect2) -> float:
	var canon := Rect2(Vector2.ZERO, rect.size)
	var heads: Array = []
	for i in idx:
		var lm: Dictionary = ov._landmarks[i]
		var r: float = ov._landmark_pin_r(lm)
		var tip: Vector2 = ov._cell_to_screen(Vector2(float(lm["x"]), float(lm["y"])), canon)
		heads.append([tip + Vector2(0.0, -ov.PinMarker.STEM_RATIO * r), r])
	var worst := 0.0
	for a in heads.size():
		for b in range(a + 1, heads.size()):
			var d: float = (heads[a][0] as Vector2).distance_to(heads[b][0])
			var rr: float = float(heads[a][1]) + float(heads[b][1])
			worst = maxf(worst, 1.0 - d / rr)
	return worst


## The rule's rank tuple for landmark `i`, higher = better (class, importance, score, then lower index): an independent
## restatement of `_declutter_order`, so a reversed or dropped term shows.
func _rank(ov, i: int) -> Array:
	var lm: Dictionary = ov._landmarks[i]
	var cr := {"continental": 3, "regional": 2, "cultural": 1, "local": 0}
	return [int(cr.get(String(lm.get("class", "local")), 0)), float(lm.get("importance", 0.0)), float(lm.get("score", 0.0)), -i]


func _rank_ge(a: Array, b: Array) -> bool:
	for k in 4:
		if a[k] != b[k]:
			return a[k] > b[k]
	return true


## True when a shown settlement pin (not declutter-hidden) touches landmark `i`'s keep-out and ranks at least its class.
func _settlement_displaces(ov, i: int, rect: Rect2) -> bool:
	if not ov.has_method("_settle_pins_on"):
		return false
	var canon := Rect2(Vector2.ZERO, rect.size)
	var la: Dictionary = ov._landmarks[i]
	var ra: float = ov._landmark_pin_r(la)
	var ta: Vector2 = ov._cell_to_screen(Vector2(float(la["x"]), float(la["y"])), canon)
	var ha := ta + Vector2(0.0, -ov.PinMarker.STEM_RATIO * ra)
	var cr: float = float({"continental": 3, "regional": 2, "cultural": 1, "local": 0}.get(String(la.get("class", "local")), 0))
	var hidden: Dictionary = ov._declutter_hidden_settlements(rect)
	for k in ov._settlements.size():
		var st: Dictionary = ov._settlements[k]
		if hidden.has(k) or ov._hidden_settlement_kinds.has(st["kind"]) or ov._settlement_hidden(st):
			continue
		var rb: float = ov._settlement_head_r(String(st["kind"]), canon)
		var tb: Vector2 = ov._cell_to_screen(Vector2(float(st["x"]), float(st["y"])), canon)
		var hb := tb + Vector2(0.0, -ov.PinMarker.STEM_RATIO * rb)
		if (ha.distance_to(hb) < 1.5 * (ra + rb) or ta.distance_to(hb) < 1.2 * rb or tb.distance_to(ha) < 1.2 * ra) 				and float(ov.DECLUTTER_SETTLE_RANK.get(ov._settle_display_kind(st), 3.0)) >= cr:
			return true
	return false


## Shown pins that touch pin `i`'s keep-out (heads closer than 1.5x the radii sum, or one tip on the other's head).
func _clashers(ov, i: int, pool: Array, rect: Rect2) -> Array:
	var canon := Rect2(Vector2.ZERO, rect.size)
	var out: Array = []
	var la: Dictionary = ov._landmarks[i]
	var ra: float = ov._landmark_pin_r(la)
	var ta: Vector2 = ov._cell_to_screen(Vector2(float(la["x"]), float(la["y"])), canon)
	var ha := ta + Vector2(0.0, -ov.PinMarker.STEM_RATIO * ra)
	for j in pool:
		var lb: Dictionary = ov._landmarks[j]
		var rb: float = ov._landmark_pin_r(lb)
		var tb: Vector2 = ov._cell_to_screen(Vector2(float(lb["x"]), float(lb["y"])), canon)
		var hb := tb + Vector2(0.0, -ov.PinMarker.STEM_RATIO * rb)
		if ha.distance_to(hb) < 1.5 * (ra + rb) or ta.distance_to(hb) < 1.2 * rb or tb.distance_to(ha) < 1.2 * ra:
			out.append(j)
	return out


## Everything the walk draws as a mark at `rect`: `[{"kind": "lm"|"st", "i": index, "tip": Vector2, "head": Vector2, "r": float,
## "forced": bool}]` for each generated landmark and each settlement NOT thinned by the rule (`hidden_lm`, `hidden_st` are the sets to
## skip). `circles` picks the disc footprint (head = tip) or the pin's (head above the tip). Geometry only, on the canonical rect.
func _marks(ov, rect: Rect2, circles: bool, hidden_lm: Dictionary, hidden_st: Dictionary) -> Array:
	var canon := Rect2(Vector2.ZERO, rect.size)
	var town_shown: bool = ov._town_layer_zoom_reached(rect)
	var members: Dictionary = ov._town_pin_membership()["lm"]
	var out: Array = []
	if ov._landmarks_visible:
		for i in ov._landmarks.size():
			var lm: Dictionary = ov._landmarks[i]
			if not ov._landmark_shown(lm) or (not town_shown and members.has(i)) or hidden_lm.has(i):
				continue
			var r: float = ov._landmark_radius(lm) if circles else ov._landmark_pin_r(lm)
			var tip: Vector2 = ov._cell_to_screen(Vector2(float(lm.get("x", 0)), float(lm.get("y", 0))), canon)
			var head := tip if circles else tip + Vector2(0.0, -ov.PinMarker.STEM_RATIO * r)
			out.append({"kind": "lm", "i": i, "tip": tip, "head": head, "r": r,
				"forced": i == ov._lm_pinned or ov.landmark_has_note(i)})
	if ov._show_settlements:
		for i in ov._settlements.size():
			var st: Dictionary = ov._settlements[i]
			if ov._hidden_settlement_kinds.has(st["kind"]) or ov._settlement_hidden(st) \
					or ov._tl_ghost_tids.has(int(st.get("tid", 0))) or hidden_st.has(i):
				continue
			if circles and ov._settlement_below_lod(String(st["kind"])):
				continue
			var r2: float = ov._settlement_pin_radius(String(st["kind"]), canon) if circles else ov._settlement_head_r(String(st["kind"]), canon)
			var tip2: Vector2 = ov._cell_to_screen(Vector2(float(st["x"]), float(st["y"])), canon)
			var dk: String = ov._settle_display_kind(st)
			out.append({"kind": "st", "i": i, "tip": tip2, "head": tip2 if circles else tip2 + Vector2(0.0, -ov.PinMarker.STEM_RATIO * r2),
				"r": r2, "forced": i == ov._settle_pinned or dk == "capital" or dk == "metropolis"})
	return out


## True when marks `a` and `b` conflict by the rule's LITERAL constants (not the overlay's).
func _conflict(a: Dictionary, b: Dictionary, circles: bool) -> bool:
	var ra: float = a["r"]
	var rb: float = b["r"]
	if circles:
		return (a["head"] as Vector2).distance_to(b["head"]) < 1.15 * (ra + rb)
	return (a["head"] as Vector2).distance_to(b["head"]) < 1.5 * (ra + rb) \
		or (a["tip"] as Vector2).distance_to(b["head"]) < 1.2 * rb or (b["tip"] as Vector2).distance_to(a["head"]) < 1.2 * ra


## The independent walk: the overlay's own ranked order (`_declutter_order`, asserted elsewhere by B7), then a plain O(n^2) keep-unless
## -clash loop over `_marks`. Returns `{"lm": {index: true}, "st": {index: true}}` of the marks it thins.
func _brute(ov, rect: Rect2, circles: bool) -> Dictionary:
	var all := _marks(ov, rect, circles, {}, {})
	var by_id := {}
	for m in all:
		by_id[(3000000 + int(m["i"])) if m["kind"] == "st" else int(m["i"])] = m
	var kept: Array = []
	var res := {"lm": {}, "st": {}}
	for id in ov._declutter_order():
		if not by_id.has(int(id)):
			continue
		var m: Dictionary = by_id[int(id)]
		var clash := false
		for k in kept:
			if _conflict(m, k, circles):
				clash = true
				break
		if clash and not bool(m["forced"]):
			(res["st"] if m["kind"] == "st" else res["lm"])[int(m["i"])] = true
		else:
			kept.append(m)
	return res


## Worst overlap (1 - distance / radii sum, clamped at 0; pins also count a tip inside another head as overlap 1) among the SHOWN marks.
func _worst(ov, rect: Rect2, circles: bool) -> float:
	var shown := _marks(ov, rect, circles, ov._declutter_set(rect), ov._declutter_hidden_settlements(rect))
	var worst := 0.0
	for a in shown.size():
		for b in range(a + 1, shown.size()):
			var ma: Dictionary = shown[a]
			var mb: Dictionary = shown[b]
			var rr: float = float(ma["r"]) + float(mb["r"])
			worst = maxf(worst, 1.0 - (ma["head"] as Vector2).distance_to(mb["head"]) / rr)
			if not circles:
				if (ma["tip"] as Vector2).distance_to(mb["head"]) < float(mb["r"]) or (mb["tip"] as Vector2).distance_to(ma["head"]) < float(ma["r"]):
					worst = maxf(worst, 1.0)
	return worst


func _count_shown(ov, rect: Rect2, circles: bool) -> Array:
	var shown := _marks(ov, rect, circles, ov._declutter_set(rect), ov._declutter_hidden_settlements(rect))
	var lm := 0
	for m in shown:
		if m["kind"] == "lm":
			lm += 1
	return [lm, shown.size() - lm]


## Every settlement the plan would draw: `{index: dot?}` from the real planner, so "hidden means the faction dot" is read from what
## the draw pass is told, not from the predicate.
func _plan_dots(ov, rect: Rect2) -> Dictionary:
	var interior: Rect2 = ov._interior_rect(rect)
	var plan: Dictionary = ov._settlement_plan_begin(rect, [])
	var steps: int = ov.SETTLE_PLAN_CHUNKS
	for st in range(0, steps):
		ov._settlement_plan_step(plan, rect, interior, st, steps)
	var out := {}
	for e in plan["entries"]:
		out[int(e["i"])] = bool(e["dot"])
	return out


func _sorted_keys(d: Dictionary) -> Array:
	var k: Array = d.keys()
	k.sort()
	return k


## The Circles leg. `before` = the rule off: print the baseline numbers and the frame only.
func _circles_leg(ov, host, before: bool) -> void:
	DccSettings.set_marker_style("circles")
	host.reset_view()
	await _frames(8)
	var rect: Rect2 = ov.displayed_rect()
	var n := _count_shown(ov, rect, true)
	_p("DECL %s circles shown_landmarks=%d shown_settlements=%d worst_overlap=%.4f declutter=%s" % [_form, n[0], n[1], _worst(ov, rect, true), str(not before)])
	await _shot("circles_fit.png")
	if before:
		return
	## baseline with the rule off, same view
	ov.declutter_enabled = false
	var base := _count_shown(ov, rect, true)
	var base_worst := _worst(ov, rect, true)
	ov.declutter_enabled = true
	_p("DECL %s circles_baseline landmarks=%d settlements=%d worst_overlap=%.4f" % [_form, base[0], base[1], base_worst])
	_ok("C1 Circles baseline overlaps (%.3f) so there is something to thin" % base_worst, base_worst > 0.0)
	_ok("C1 Circles shows fewer marks than with the rule off (%d+%d < %d+%d)" % [n[0], n[1], base[0], base[1]], n[0] + n[1] < base[0] + base[1])
	_ok("C1 Circles map is not blanked (%d landmarks, %d settlements)" % [n[0], n[1]], n[0] >= 8 and n[1] >= 1)
	var br := _brute(ov, rect, true)
	var ho: Dictionary = ov._declutter_set(rect)
	var hs: Dictionary = ov._declutter_hidden_settlements(rect)
	_ok("C1 landmark set equals the independent walk (%d hidden vs %d)" % [ho.size(), (br["lm"] as Dictionary).size()], _sorted_keys(ho) == _sorted_keys(br["lm"]))
	_ok("C1 settlement set equals the independent walk (%d hidden vs %d)" % [hs.size(), (br["st"] as Dictionary).size()], _sorted_keys(hs) == _sorted_keys(br["st"]))
	var w := _worst(ov, rect, true)
	_ok("C2 no two shown Circles marks overlap (worst %.4f)" % w, w <= 0.0)
	var h1 := _sorted_keys(ov._declutter_set(Rect2(rect.position + Vector2(1.0, 0.0), rect.size)))
	var h2 := _sorted_keys(ov._declutter_set(Rect2(rect.position + Vector2(-37.0, 53.0), rect.size)))
	_ok("C3 Circles hidden set identical under 1 px and 37,53 px pans", h1 == _sorted_keys(ho) and h2 == _sorted_keys(ho))
	## C4: forced marks
	var hid: Array = _sorted_keys(ho)
	_ok("C4 some Circles landmark is hidden (%d)" % hid.size(), hid.size() >= 2)
	if hid.size() >= 2:
		var sel: int = hid[0]
		ov.set_selected_landmark(sel)
		_ok("C4 a selected Circles landmark is shown", not ov._lm_hidden(sel, rect))
		ov.set_selected_landmark(-1)
		_ok("C4 and hidden again once deselected", ov._lm_hidden(sel, rect))
		var ids: Array = []
		for k in ov._landmarks.size():
			ids.append(1000 + k)
		var noted: int = hid[1]
		ov.set_pin_note_source(func() -> Dictionary:
			return {"linked": {"landmark:%d" % (1000 + noted): true}, "landmark_ids": ids})
		ov.invalidate_pin_notes()
		await _frames(2)
		_ok("C4 a noted Circles landmark is shown", not ov._lm_hidden(noted, rect), "has_note=%s town=%s dc=%s" % [str(ov.landmark_has_note(noted)), str(ov._lm_town_hidden(noted, rect)), str(ov._lm_declutter_hidden(noted, rect))])
		ov.set_pin_note_source(func() -> Dictionary:
			return {"linked": {}, "landmark_ids": ids})
		ov.invalidate_pin_notes()
		await _frames(2)
		_ok("C4 and hidden again once the note is gone", ov._lm_hidden(noted, rect))
		## C5: a hidden landmark is not picked at its ring centre; a shown one is (positive control)
		var interior := Rect2(Vector2(-1e6, -1e6), Vector2(2e6, 2e6))
		var tip: Vector2 = ov._cell_to_screen(Vector2(float(ov._landmarks[sel]["x"]), float(ov._landmarks[sel]["y"])), rect)
		var got: Array = ov._hit_test_landmark(tip, interior, rect)
		_ok("C5 hidden Circles landmark %d is not picked at its centre (got %d)" % [sel, int(got[0])], int(got[0]) != sel)
		var leak := false
		for h in ov.hits_at(tip):
			if String(h.get("kind", "")) == "landmark" and int(h.get("id", -1)) == sel:
				leak = true
		_ok("C5 hits_at does not list the hidden Circles landmark", not leak)
		ov.set_selected_landmark(sel)
		var got2: Array = ov._hit_test_landmark(tip, interior, rect)
		_ok("C5 control: once selected the same point picks it (got %d)" % int(got2[0]), int(got2[0]) == sel)
		ov.set_selected_landmark(-1)
	## C4 capital, C5 settlements
	var cap_ok := true
	var ncap := 0
	for i in ov._settlements.size():
		if ov._settle_display_kind(ov._settlements[i]) == "capital":
			ncap += 1
			if hs.has(i):
				cap_ok = false
	_ok("C4 every capital settlement is shown in Circles (%d capitals)" % ncap, ncap > 0 and cap_ok)
	var dots := _plan_dots(ov, rect)
	var dot_bad := 0
	var disc_bad := 0
	for i in hs.keys():
		if dots.has(i) and not bool(dots[i]):
			dot_bad += 1
		if not dots.has(i):
			dot_bad += 1
	for i in dots.keys():
		if not hs.has(i) and bool(dots[i]) and not ov._settlement_below_lod(String(ov._settlements[i]["kind"])):
			disc_bad += 1
	_ok("C5 the plan draws all %d thinned settlements as dots (%d not)" % [hs.size(), dot_bad], dot_bad == 0 and (hs.size() > 0 or _form != "desktop"))   ## the touch forms' 19 disc settlements clear each other at the fit zoom: nothing to thin there
	_ok("C5 and no shown settlement disc as a dot (%d)" % disc_bad, disc_bad == 0)
	var interior2 := Rect2(Vector2(-1e6, -1e6), Vector2(2e6, 2e6))
	if hs.size() > 0:
		var si: int = int(_sorted_keys(hs)[0])
		var sp: Vector2 = ov._cell_to_screen(Vector2(float(ov._settlements[si]["x"]), float(ov._settlements[si]["y"])), rect)
		_ok("C5 a thinned Circles settlement is not picked at its centre (got %d)" % int(ov._hit_test_settlement(sp, interior2, rect)), int(ov._hit_test_settlement(sp, interior2, rect)) != si)
		var sleak := false
		for h in ov.hits_at(sp):
			if String(h.get("kind", "")) == "settlement" and int(h.get("id", -1)) == si:
				sleak = true
		_ok("C5 hits_at does not list the thinned Circles settlement", not sleak)
	## C6: the style is part of the cache key
	var circ_hidden := _sorted_keys(ov._declutter_set(rect))
	DccSettings.set_marker_style("pins")
	await _frames(2)
	var pins_hidden := _sorted_keys(ov._declutter_set(rect))
	var pins_brute := _sorted_keys(_brute(ov, rect, false)["lm"])
	_ok("C6 the Pins set differs from the Circles set (%d vs %d) and equals its own walk" % [pins_hidden.size(), circ_hidden.size()], pins_hidden != circ_hidden and pins_hidden == pins_brute)
	DccSettings.set_marker_style("circles")
	await _frames(2)
	_ok("C6 and back to Circles gives the Circles set again", _sorted_keys(ov._declutter_set(rect)) == circ_hidden)


## The export leg: an export copy of the fit view at `px` wide, under the current marker style. Saves the overlay over grey as
## `<tag>_<form>_export_<style>.png` when `--out` is given.
func _export_leg(ov, host, before: bool, style: String, px: int) -> void:
	DccSettings.set_marker_style(style)
	var circles := style == "circles"
	host.reset_view()
	await _frames(8)
	var py := int(round(px * 0.75))
	var eo := ExportOverlay.new()
	add_child(eo)
	var err := eo.setup(ov, px, py, {"icons": true, "labels": false, "ways": false, "settlement_tier": "all"})
	_ok("X0 the export copy builds (%s)" % err, err == "")
	if err != "":
		eo.queue_free()
		return
	var cp = eo.overlay
	await _frames(3)
	var rect: Rect2 = cp.displayed_rect()
	var n := _count_shown(cp, rect, circles)
	_p("DECL %s export_%s px=%d landmarks=%d/%d settlements=%d/%d worst_overlap=%.4f declutter=%s" % [_form, style, px, n[0],
		cp._landmarks.size(), n[1], cp._settlements.size(), _worst(cp, rect, circles), str(not before)])
	var data: PackedByteArray = await eo.render_tile(0, 0, px, py, 4096)
	if _out != "" and data.size() == px * py * 4:
		var img := Image.create_from_data(px, py, false, Image.FORMAT_RGBA8, data)
		var bg := Image.create(px, py, false, Image.FORMAT_RGBA8)
		bg.fill(Color(0.55, 0.58, 0.55))
		for yy in py:
			for xx in px:
				var c := img.get_pixel(xx, yy)
				if c.a > 0.0:
					## premultiplied readback: out = src + dst * (1 - a)
					var d := bg.get_pixel(xx, yy)
					bg.set_pixel(xx, yy, Color(c.r + d.r * (1.0 - c.a), c.g + d.g * (1.0 - c.a), c.b + d.b * (1.0 - c.a), 1.0))
		bg.save_png(_out.path_join("%s_%s_export_%s.png" % [_tag, _form, style]))
		_p("shot %s_%s_export_%s.png" % [_tag, _form, style])
	if not before:
		cp.declutter_enabled = false
		var base := _count_shown(cp, rect, circles)
		cp.declutter_enabled = true
		_p("DECL %s export_%s_baseline landmarks=%d settlements=%d" % [_form, style, base[0], base[1]])
		_ok("X1 [%s] the export thins (%d+%d < %d+%d)" % [style, n[0], n[1], base[0], base[1]], n[0] + n[1] < base[0] + base[1])
		_ok("X1 [%s] the export is not blanked (%d landmarks, %d settlements)" % [style, n[0], n[1]], n[0] >= 8 and n[1] >= 1)
		var br := _brute(cp, rect, circles)
		_ok("X1 [%s] landmark set equals the independent walk" % style, _sorted_keys(cp._declutter_set(rect)) == _sorted_keys(br["lm"]))
		_ok("X1 [%s] settlement set equals the independent walk" % style, _sorted_keys(cp._declutter_hidden_settlements(rect)) == _sorted_keys(br["st"]))
		var w := _worst(cp, rect, circles)
		_ok("X2 [%s] no two shown export marks overlap (worst %.4f)" % [style, w], w <= 0.0)
		var ho := _sorted_keys(cp._declutter_set(rect))
		_ok("X3 [%s] the export set is the same under a pan of the rect" % style,
			_sorted_keys(cp._declutter_set(Rect2(rect.position + Vector2(311.0, -97.0), rect.size))) == ho)
		var hs: Dictionary = cp._declutter_hidden_settlements(rect)
		var dots := _plan_dots(cp, rect)
		var bad := 0
		var interior_x: Rect2 = cp._interior_rect(rect)
		for i in hs.keys():
			var spos: Vector2 = cp._cell_to_screen(Vector2(float(cp._settlements[i]["x"]), float(cp._settlements[i]["y"])), rect)
			## a settlement off the plate has no entry at all (the planner's own interior cull); every other thinned one is a dot
			if (dots.has(i) and not bool(dots[i])) or (not dots.has(i) and interior_x.has_point(spos)):
				bad += 1
		_ok("X4 [%s] the export draws all %d thinned settlements as dots (%d not)" % [style, hs.size(), bad], bad == 0 and hs.size() > 0)
		## export scale: the walk's radii are OUTPUT pixels: a landmark head is the unfloored class radius times the gain
		var lm0: Dictionary = cp._landmarks[0]
		var want: float = cp._landmark_radius(lm0)
		_ok("X6 [%s] an export landmark head takes no screen floor (%.2f)" % [style, cp._landmark_pin_r(lm0)], is_equal_approx(cp._landmark_pin_r(lm0), want))
	eo.queue_free()
	await _frames(2)


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
		elif a == "--before":
			_before = true
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
	await bridge.landmark_run()
	await _frames(6)
	var t0 := Time.get_ticks_msec()
	while app.busy_overlay != null and app.busy_overlay.visible and Time.get_ticks_msec() - t0 < 30000:
		await _frames(10)
	await _frames(20)
	DccSettings.set_marker_style("pins")
	app.select_domain("cartography")
	await _frames(8)
	app.arm_tool("inspect")
	await _frames(4)
	if _before:
		ov.declutter_enabled = false
		ov.queue_redraw()
	var host = app.viewport
	host.reset_view()
	await _frames(8)
	var rect: Rect2 = ov.displayed_rect()
	var n_all: int = ov._landmarks.size()
	var shown_fit := _shown(ov, rect)
	_p("DECL %s landmarks=%d shown_fit=%d in_view=%d zoom=%.3f declutter=%s" % [_form, n_all, shown_fit.size(),
		_in_view(ov, host, shown_fit, rect), float(ov._camera_zoom), str(not _before)])
	await _shot("fit.png")
	if _before:
		## Baseline run: also frame the 4x view, then stop (the assertions below are about the rule).
		host._zoom_at(host.size * 0.5, 4.0)
		await _frames(10)
		await _shot("x4.png")
		_p("DECL %s shown_x4=%d" % [_form, _shown(ov, ov.displayed_rect()).size()])
		host.reset_view()
		await _frames(6)
		await _circles_leg(ov, host, true)
		await _export_leg(ov, host, true, "pins", 2048)
		await _export_leg(ov, host, true, "circles", 2048)
		get_tree().quit(0)
		return
	_ok("landmarks exist (%d)" % n_all, n_all > 40)
	## B1
	var cap: int = SHOWN_MAX[_form]
	_ok("B1 shown at fit %d <= %d" % [shown_fit.size(), cap], shown_fit.size() <= cap)
	var vcap: int = VIEW_MAX[_form]
	var inv := _in_view(ov, host, shown_fit, rect)
	_ok("B1 pins in the visible window %d <= %d" % [inv, vcap], inv <= vcap)
	_ok("B1 shown at fit %d >= 8 (the map is not blanked)" % shown_fit.size(), shown_fit.size() >= 8)
	## B2
	var worst := _worst_overlap(ov, shown_fit, rect)
	_p("DECL %s worst_head_overlap=%.4f" % [_form, worst])
	_ok("B2 no two shown heads overlap (worst %.4f)" % worst, worst <= 0.0)
	## B2b: no tip of a shown pin inside another shown head
	var canon0 := Rect2(Vector2.ZERO, rect.size)
	var stacked := 0
	for a in shown_fit:
		var la: Dictionary = ov._landmarks[a]
		var ta: Vector2 = ov._cell_to_screen(Vector2(float(la["x"]), float(la["y"])), canon0)
		for b in shown_fit:
			if a == b:
				continue
			var lb: Dictionary = ov._landmarks[b]
			var rb: float = ov._landmark_pin_r(lb)
			var tb: Vector2 = ov._cell_to_screen(Vector2(float(lb["x"]), float(lb["y"])), canon0)
			if ta.distance_to(tb + Vector2(0.0, -ov.PinMarker.STEM_RATIO * rb)) < 1.0 * rb:
				stacked += 1
	_ok("B2b no shown tip lies on another shown head (%d)" % stacked, stacked == 0)
	## B7: each hidden candidate has a shown, at-least-as-good, conflicting pin
	var unexplained := 0
	var checked := 0
	for h in ov._declutter_set(rect).keys():
		checked += 1
		var ok := false
		for c in _clashers(ov, int(h), shown_fit, rect):
			if _rank_ge(_rank(ov, int(c)), _rank(ov, int(h))) or ov.landmark_has_note(int(c)):
				ok = true
				break
		## Ruling FF item 1: a shown SETTLEMENT pin joins the walk and outranks minor landmarks, so it may be what displaced
		## this landmark: accept a shown settlement pin that clashes and whose tier rank is at least the landmark's class rank.
		if not ok and _settlement_displaces(ov, int(h), rect):
			ok = true
		if not ok:
			unexplained += 1
	_ok("B7 every hidden pin (%d) has a conflicting shown pin of equal or higher rank (%d without)" % [checked, unexplained], checked > 0 and unexplained == 0)
	## B8: tilts only among shown pins
	var tilted_hidden := 0
	for k in (ov._fan(rect)["lm_a"] as Dictionary).keys():
		if ov._lm_declutter_hidden(int(k), rect):
			tilted_hidden += 1
	_ok("B8 no hidden pin is tilted by Ruling FA (%d)" % tilted_hidden, tilted_hidden == 0)
	## B3
	var h0 := _hidden(ov, rect)
	var h1 := _hidden(ov, Rect2(rect.position + Vector2(1.0, 0.0), rect.size))
	var h2 := _hidden(ov, Rect2(rect.position + Vector2(-37.0, 53.0), rect.size))
	_ok("B3 hidden set identical under a 1 px rect pan", h0 == h1)
	_ok("B3 hidden set identical under a 37,53 px rect pan", h0 == h2)
	host._camera.position += Vector2(1.0, 0.0)
	await _frames(4)
	_ok("B3 hidden set identical after the live camera moved 1 px", _hidden(ov, ov.displayed_rect()) == h0)
	host._camera.position -= Vector2(1.0, 0.0)
	await _frames(2)
	## B5 (needs a hidden pin at fit, and a twin we do not touch)
	var hid: Array = h0.duplicate()
	_ok("some pin is hidden at the fit zoom (%d)" % hid.size(), hid.size() >= 2)
	if hid.size() >= 2:
		var sel: int = hid[0]
		var noted: int = hid[1]
		var twin: int = hid[hid.size() - 1] if hid.size() > 2 else -1
		ov.set_selected_landmark(sel)
		_ok("B5 a selected pin is shown", not ov._lm_hidden(sel, rect))
		ov.set_selected_landmark(-1)
		_ok("B5 and hidden again once deselected", ov._lm_hidden(sel, rect))
		var ids: Array = []
		for k in n_all:
			ids.append(1000 + k)
		ov.set_pin_note_source(func() -> Dictionary:
			return {"linked": {"landmark:%d" % (1000 + noted): true}, "landmark_ids": ids})
		ov.invalidate_pin_notes()
		await _frames(2)
		_ok("B5 a pin with a linked note is shown", not ov._lm_hidden(noted, rect))
		if twin >= 0 and twin != noted and twin != sel:
			_ok("B5 an un-noted twin stays hidden", ov._lm_hidden(twin, rect))
		## B5b: a hidden pin and the shown pin that displaced it, both noted
		var both := -1
		var partner := -1
		for h in hid:
			if h == noted or h == sel:
				continue
			var cl := _clashers(ov, int(h), _shown(ov, rect), rect)
			if not cl.is_empty():
				both = int(h)
				partner = int(cl[0])
				break
		if both >= 0:
			ov.set_pin_note_source(func() -> Dictionary:
				return {"linked": {"landmark:%d" % (1000 + both): true, "landmark:%d" % (1000 + partner): true}, "landmark_ids": ids})
			ov.invalidate_pin_notes()
			await _frames(2)
			_ok("B5b two clashing noted pins (%d, %d) are both shown" % [both, partner],
				not ov._lm_hidden(both, rect) and not ov._lm_hidden(partner, rect))
		else:
			_fails += 1
			_p("FAIL B5b found no hidden pin with a shown clasher")
		ov.set_pin_note_source(func() -> Dictionary:
			return {"linked": {"landmark:%d" % (1000 + noted): true}, "landmark_ids": ids})
		ov.invalidate_pin_notes()
		await _frames(2)
		## a user POI at a hidden pin's cell is picked, whatever the rule does to landmarks
		var lmx: Dictionary = ov._landmarks[sel]
		var wk: float = ov._map_width_km
		ov.set_pois([{"id": 7, "x": float(lmx["x"]) + 0.5, "y": float(lmx["y"]) + 0.5, "kind": "tavern", "name": "T"}])
		await _frames(2)
		var pt: Vector2 = ov._point_to_screen(Vector2(float(lmx["x"]) + 0.5, float(lmx["y"]) + 0.5), rect)
		var interior := Rect2(Vector2(-1e6, -1e6), Vector2(2e6, 2e6))
		var pr: float = ov._poi_pin_r()
		var ph: Array = ov._hit_test_poi(pt + Vector2(0.0, -2.1 * pr), interior, rect)
		_ok("B5 a user POI is hit at its head (index %d)" % int(ph[0]), int(ph[0]) == 0 or ov._poi_town_hidden(0, rect))
		ov.set_pois([])
		## B6: a hidden pin is not picked at its head; shown pin is (positive control, by zooming in)
		var target := -1
		for c in hid:
			if c != noted and c != sel:
				target = c
				break
		var lm: Dictionary = ov._landmarks[target]
		var r: float = ov._landmark_pin_r(lm)
		var tip: Vector2 = ov._cell_to_screen(Vector2(float(lm["x"]), float(lm["y"])), rect)
		var head := tip + Vector2(0.0, -ov.PinMarker.STEM_RATIO * r)
		var got: Array = ov._hit_test_landmark(head, interior, rect)
		_ok("B6 hidden pin %d is not picked at its own head (got %d)" % [target, int(got[0])], int(got[0]) != target)
		var hits: Array = ov.hits_at(head)
		var leak := false
		for h in hits:
			if String(h.get("kind", "")) == "landmark" and int(h.get("id", -1)) == target:
				leak = true
		_ok("B6 hits_at does not list the hidden pin", not leak)
		## B4 and the positive control: zoom 2x about the centre
		var n1 := _shown(ov, rect).size()
		host._zoom_at(host.size * 0.5, 2.0)
		await _frames(10)
		var rect2: Rect2 = ov.displayed_rect()
		var n2 := _shown(ov, rect2).size()
		_p("DECL %s shown_x2=%d (fit %d)" % [_form, n2, n1])
		_ok("B4 2x zoom shows more pins (%d -> %d)" % [n1, n2], n2 > n1)
		_ok("B2 at 2x too: no head overlap (%.4f)" % _worst_overlap(ov, _shown(ov, rect2), rect2), _worst_overlap(ov, _shown(ov, rect2), rect2) <= 0.0)
		if not ov._lm_hidden(target, rect2):
			var lm2: Dictionary = ov._landmarks[target]
			var r2: float = ov._landmark_pin_r(lm2)
			var tip2: Vector2 = ov._cell_to_screen(Vector2(float(lm2["x"]), float(lm2["y"])), rect2)
			var head2 := tip2 + Vector2(0.0, -ov.PinMarker.STEM_RATIO * r2)
			var got2: Array = ov._hit_test_landmark(head2, interior, rect2)
			_ok("B6 control: once shown (2x) pin %d is picked at its head (got %d)" % [target, int(got2[0])], int(got2[0]) == target)
		else:
			_p("note: target %d still hidden at 2x; control skipped" % target)
	## B4b: with the zoom law off the pin size no longer follows `_landmark_zoom_k`, so only the zoom itself can move the set
	ov.set_landmark_zoom_law(false)
	host.reset_view()
	await _frames(8)
	var za := _shown(ov, ov.displayed_rect()).size()
	host._zoom_at(host.size * 0.5, 2.0)
	await _frames(10)
	var zb := _shown(ov, ov.displayed_rect()).size()
	_ok("B4b law off: 2x zoom shows more pins (%d -> %d)" % [za, zb], zb > za)
	ov.set_landmark_zoom_law(true)
	## frames and the 4x count
	host.reset_view()
	await _frames(8)
	host._zoom_at(host.size * 0.5, 4.0)
	await _frames(10)
	var r4: Rect2 = ov.displayed_rect()
	_p("DECL %s shown_x4=%d" % [_form, _shown(ov, r4).size()])
	await _shot("x4.png")
	host.reset_view()
	await _frames(6)
	await _circles_leg(ov, host, false)
	await _export_leg(ov, host, false, "pins", 2048)
	await _export_leg(ov, host, false, "circles", 2048)
	_p("== done: %d failed ==" % _fails)
	get_tree().quit(1 if _fails > 0 else 0)
