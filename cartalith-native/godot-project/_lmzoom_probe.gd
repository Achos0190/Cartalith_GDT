extends Node
## The landmark zoom law and the landmark filter, measured in pixels against the
## overlay as it was before them (owner request, 2026-10-19: *"turn landmarks off
## globally or filter per type ... the landmarks themselves should also scale
## according to zoom level"*).
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _lmzoom_probe.tscn -- --head res://<copy of HEAD's map_overlay.gd>
##
## `--head` is required: the probe draws the same fixture with the CURRENT
## `map_overlay.gd` and with a copy of the pre-change script (`git show
## <base>:cartalith-native/godot-project/map_overlay.gd > _lm_head_overlay.gd`,
## a scratch file, never committed), so "byte-identical" is a comparison against
## the real previous code rather than against a number written here.
##
## What it asserts:
##   Z1  law OFF draws byte-identically to HEAD at every zoom (the switch really
##       restores the old law);
##   Z2  law ON draws byte-identically to HEAD at camera zoom 1 (the fit/export
##       scale) and at the opening view (`_lod_zoom_base()`), i.e. the default
##       look did not move;
##   Z3  law ON measures, at opening x {1, 4, 16, 64}: the drawn ring diameter
##       grows monotonically, x4 > x1, and x64 stays within the ceiling (2.5x
##       the x1 size, plus antialiasing slack), where HEAD's grows ~64x;
##   Z4  at ZOOM_MIN-like 0.4 the law keeps the zoom-1 size (the floor) while
##       HEAD shrinks;
##   F1  the filter untouched is byte-identical to HEAD at the opening view;
##   F2  hiding one kind removes exactly that kind's ring pixels and its
##       generated POI glyph, keeps the other kind's, and makes the hidden ring
##       unpickable (`_pick_mark`, `hits_at`) while the shown one still picks;
##   F3  the hand-placed POI row hides a manual POI icon, and the master switch
##       alone does not (`LANDMARK_MANUAL_POI`'s contract);
##   F4  a generated landmark label of a hidden kind reports hidden
##       (`_landmark_label_hidden`, the term `_label_hidden` ORs in), its
##       neighbour of a shown kind does not (labels are not drawn here: the
##       zoom-LOD gate would confound a pixel reading);
##   F5  an export copy (`copy_state_from` + `prepare_export`) carries the hidden
##       set and answers k = 1.
##
## Windowed only: `RenderingServer.frame_post_draw` never fires headless
## (`MISTAKES.md`). Result: `LMZOOM ok|FAIL <what>` lines, `LMZOOM ---- <n>
## checks, PASS|<n> FAILED ----`; exit 0 pass, 1 any failure, 2 bad argument /
## headless / missing --head, 3 watchdog.

const W := 900
const H := 600
## 64 x 56 cells in a 900 x 600 control: fit by height (rect 685.7 x 600), so
## the cover (opening) zoom is 900 / 685.7 = 1.3125 -- a real, >1 opening view,
## as on the shell's desktop (measured 1.36 there).
const GW := 64
const GH := 56
const BG := Color(0.03, 0.03, 0.04)

## The measured landmark sits at the centre of the view at every zoom.
const C_PEAK := Vector2i(32, 28)
const C_CAVE := Vector2i(48, 28)
const C_MANUAL := Vector2i(24, 28)

var _head_path := ""
var _fails := 0
var _checks := 0


func _p(s: String) -> void:
	print("LMZOOM %s" % s)


func _check(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	_p("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


static func _landmarks() -> Array:
	return [
		{"id": 1, "kind": "peak", "class": "regional", "x": C_PEAK.x, "y": C_PEAK.y, "importance": 0.8},
		{"id": 2, "kind": "cave", "class": "local", "x": C_CAVE.x, "y": C_CAVE.y, "importance": 0.4},
		## A second landmark on the cave's cell: hiding it alone must not hide
		## the cave's name (`_lm_hidden_cells` leaves cells a shown landmark holds).
		{"id": 3, "kind": "spring", "class": "local", "x": C_CAVE.x, "y": C_CAVE.y, "importance": 0.2},
	]


static func _icons() -> Array:
	return [
		## The generated POI copy of the peak (`icon_candidates`' shape).
		{"x": float(C_PEAK.x), "y": float(C_PEAK.y), "family": "poi", "slot": "mountain_peak",
			"set": "", "scale": 1.0, "origin": "generated"},
		{"x": float(C_MANUAL.x) + 0.5, "y": float(C_MANUAL.y) + 0.5, "family": "poi", "slot": "cave",
			"set": "", "scale": 1.0, "origin": "manual"},
	]


static func _labels() -> Array:
	var out: Array = []
	for c in [C_PEAK, C_CAVE]:
		out.append({"text": "Peak" if c == C_PEAK else "Cave", "x": float(c.x) + 0.5, "y": float(c.y) + 0.5,
			"generated": true, "class": "landmark", "size": 12.0, "size_mode": "fixed"})
	return out


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--head" and i + 1 < args.size():
			_head_path = args[i + 1]
			i += 2
			continue
		_p("ABORT unknown argument '%s'" % args[i])
		get_tree().quit(2)
		return
	if _head_path == "" or not ResourceLoader.exists(_head_path):
		_p("ABORT --head <res:// path of the pre-change map_overlay.gd> is required (got '%s')" % _head_path)
		get_tree().quit(2)
		return
	if DisplayServer.get_name() == "headless":
		_p("ABORT run windowed: frame_post_draw never fires headless")
		get_tree().quit(2)
		return
	var wd := get_tree().create_timer(300.0)
	wd.timeout.connect(func():
		_p("WATCHDOG")
		get_tree().quit(3))

	var cur := _make("res://map_overlay.gd")
	var head := _make(_head_path)
	var blank := _make("")
	await _settle()
	var b_img: Image = blank["vp"].get_texture().get_image()
	var zr: float = cur["ov"]._lod_zoom_base()
	_p("opening zoom (cover) = %.4f" % zr)
	_check("the fixture has a real opening zoom above 1", zr > 1.2, "zr=%.4f" % zr)

	## -- Z1/Z2/Z4: identity and floor ---------------------------------------
	var zooms := [0.4, 1.0, zr, zr * 4.0, zr * 16.0, zr * 64.0]
	var names := ["0.4", "1", "x1", "x4", "x16", "x64"]
	var diam_head := {}
	var diam_on := {}
	for zi in zooms.size():
		var z: float = zooms[zi]
		var hi: Image = await _frame(head, z)
		var on_img: Image = await _frame(cur, z)
		cur["ov"].set_landmark_zoom_law(false)
		var off_img: Image = await _frame(cur, z)
		cur["ov"].set_landmark_zoom_law(true)
		if zi == 0:
			_check("positive control: the head frame is not blank", hi.get_data() != b_img.get_data())
		_check("Z1 law OFF == HEAD at zoom %s" % names[zi], off_img.get_data() == hi.get_data())
		if names[zi] == "1" or names[zi] == "x1":
			_check("Z2 law ON == HEAD at zoom %s (default look unchanged)" % names[zi],
				on_img.get_data() == hi.get_data())
		## The radius each script says it draws, in framebuffer px, then the
		## pixels' own answer inside a window scaled to it (`_radius_px`).
		var r_head: float = head["ov"]._landmark_radius(_landmarks()[0]) * z
		var r_on: float = cur["ov"]._landmark_radius(_landmarks()[0]) * z
		diam_head[names[zi]] = 2 * _radius_px(hi, r_head)
		diam_on[names[zi]] = 2 * _radius_px(on_img, r_on)
		_p("zoom %-4s z=%7.3f  ring diameter px: HEAD %4d   law ON %4d   (k=%.4f)" % [names[zi], z,
			diam_head[names[zi]], diam_on[names[zi]], cur["ov"]._landmark_zoom_k()])
		on_img.save_png("user://lmzoom_%s_on.png" % names[zi])
		hi.save_png("user://lmzoom_%s_head.png" % names[zi])
	_check("Z3 law ON grows x1 < x4 < x16 <= x64",
		diam_on["x1"] < diam_on["x4"] and diam_on["x4"] < diam_on["x16"] and diam_on["x16"] <= diam_on["x64"],
		"%s" % [diam_on])
	## Slack: the radius is measured to the stroke's outer bright edge, which the
	## 0.30 threshold cuts ~1 px inside at x1 and ~2 px outside the radius at the
	## ceiling (the stroke scales with the law too), i.e. up to ~6 px of diameter.
	## The exponent, pinned by its own prediction rather than by the constant:
	## (4)^(1/3) = 1.587 between x1 and x4 (a labelled judgement; `0.5` gives 2.0).
	var g4 := float(diam_on["x4"]) / float(diam_on["x1"])
	_check("Z3 x1 -> x4 grows by ~4^(1/3) = 1.59 (1.45 .. 1.75)", g4 >= 1.45 and g4 <= 1.75, "ratio %.3f" % g4)
	_check("Z3 law ON x64 stays under the 2.5x ceiling (+8 px stroke/AA slack)",
		float(diam_on["x64"]) <= 2.5 * float(diam_on["x1"]) + 8.0, "x1=%d x64=%d" % [diam_on["x1"], diam_on["x64"]])
	_check("Z3 HEAD x16 really balloons (the defect the law removes, >8x x1)",
		float(diam_head["x16"]) > 8.0 * float(diam_head["x1"]), "x1=%d x16=%d" % [diam_head["x1"], diam_head["x16"]])
	_check("Z4 floor: at zoom 0.4 the law keeps the zoom-1 size, HEAD shrinks",
		absi(int(diam_on["0.4"]) - int(diam_on["1"])) <= 2 and diam_head["0.4"] < diam_head["1"],
		"on 0.4=%d 1=%d head 0.4=%d" % [diam_on["0.4"], diam_on["1"], diam_head["0.4"]])

	## -- F: the filter ------------------------------------------------------
	var base_cur: Image = await _frame(cur, zr)
	var base_head: Image = await _frame(head, zr)
	_check("F1 filter untouched == HEAD at the opening view", base_cur.get_data() == base_head.get_data())
	var ov: Control = cur["ov"]
	var rect: Rect2 = ov._displayed_rect()
	var interior: Rect2 = ov._interior_rect(rect)
	var p_peak: Vector2 = ov._cell_to_screen(Vector2(C_PEAK), rect)
	var p_cave: Vector2 = ov._cell_to_screen(Vector2(C_CAVE), rect)
	_check("F2 control: the peak picks before hiding", int(ov._pick_mark(p_peak, interior, rect)["l"]) == 0)
	var ink_peak0 := _ink(base_cur, ov, C_PEAK, zr)
	var ink_cave0 := _ink(base_cur, ov, C_CAVE, zr)
	ov.set_landmark_kind_visible("peak", false)
	var hid: Image = await _frame(cur, zr)
	var ink_peak1 := _ink(hid, ov, C_PEAK, zr)
	var ink_cave1 := _ink(hid, ov, C_CAVE, zr)
	_p("ink around peak %d -> %d, around cave %d -> %d" % [ink_peak0, ink_peak1, ink_cave0, ink_cave1])
	_check("F2 hiding 'peak' removes its ring AND its generated POI glyph (no ink left)", ink_peak0 > 0 and ink_peak1 == 0)
	_check("F2 hiding 'peak' leaves the cave exactly as it was", ink_cave1 == ink_cave0 and ink_cave0 > 0)
	_check("F2 the hidden peak no longer picks", int(ov._pick_mark(p_peak, interior, rect)["l"]) == -1)
	_check("F2 the shown cave still picks", int(ov._pick_mark(p_cave, interior, rect)["l"]) == 1)
	var hits: Array = ov.hits_at(p_peak)
	_check("F2 the context pick lists no hidden landmark",
		not hits.any(func(h): return String(h.get("kind", "")) == "landmark"), "%s" % [hits])
	var labs := _labels()
	_check("F4 the hidden peak's generated label reports hidden", ov._landmark_label_hidden(labs[0]))
	_check("F4 the shown cave's generated label does not", not ov._landmark_label_hidden(labs[1]))
	ov.set_landmark_kind_visible("spring", false)
	_check("F4 hiding 'spring' (sharing the cave's cell) leaves the cave's name", not ov._landmark_label_hidden(labs[1]))
	ov.set_landmark_kind_visible("spring", true)
	ov.set_landmark_kind_visible("peak", true)
	var back: Image = await _frame(cur, zr)
	_check("F2 showing it again restores the frame byte-for-byte", back.get_data() == base_cur.get_data())

	var ics := _icons()
	ov.set_landmarks_visible(false)
	_check("F3 the master switch alone does not hide a hand-placed POI", not ov._icon_layer_hidden(ics[1]))
	_check("F3 ... and does hide the generated POI copy", ov._icon_layer_hidden(ics[0]))
	ov.set_landmarks_visible(true)
	ov.set_landmark_kind_visible(ov.LANDMARK_MANUAL_POI, false)
	_check("F3 the hand-placed row hides the hand-placed POI", ov._icon_layer_hidden(ics[1]))
	_check("F3 ... and leaves the generated one to its landmark", not ov._icon_layer_hidden(ics[0]))
	var counts: Dictionary = ov.landmark_kind_counts()
	_check("F3 counts: peak 1, cave 1, spring 1, hand-placed 1",
		int(counts.get("peak", 0)) == 1 and int(counts.get("cave", 0)) == 1 and int(counts.get("spring", 0)) == 1
		and int(counts.get(ov.LANDMARK_MANUAL_POI, 0)) == 1,
		"%s" % [counts])
	ov.set_landmark_kind_visible(ov.LANDMARK_MANUAL_POI, true)

	## -- F5: export follows the screen ----------------------------------------
	ov.set_landmark_kind_visible("peak", false)
	var ex: Control = Control.new()
	ex.set_script(load("res://map_overlay.gd"))
	ex.size = Vector2(W * 2, H * 2)
	cur["vp"].add_child(ex)
	ex.copy_state_from(ov)
	ex.prepare_export(2.0, {"icons": true})
	_check("F5 the export copy carries the hidden set", ex.hidden_landmark_kinds() == PackedStringArray(["peak"]),
		"%s" % [ex.hidden_landmark_kinds()])
	_check("F5 the export copy answers k = 1", ex._landmark_zoom_k() == 1.0)
	_check("F5 the export copy draws no ring for the hidden kind",
		not ex._ringed_cells().has(Vector2i(C_PEAK)) and ex._ringed_cells().has(Vector2i(C_CAVE)))
	ex.queue_free()
	ov.set_landmark_kind_visible("peak", true)

	_p("---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)


## One SubViewport holding a "camera" Control (the ancestor scale ViewportHost's
## `_camera` is) and, unless `script` is empty, an overlay fed the fixture.
func _make(script: String) -> Dictionary:
	var vp := SubViewport.new()
	vp.size = Vector2i(W, H)
	vp.transparent_bg = false
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var bg := ColorRect.new()
	bg.size = Vector2(W, H)
	bg.color = BG
	vp.add_child(bg)
	var cam := Control.new()
	cam.size = Vector2(W, H)
	vp.add_child(cam)
	var out := {"vp": vp, "cam": cam, "ov": null}
	if script != "":
		var ov := Control.new()
		ov.set_script(load(script))
		ov.size = Vector2(W, H)
		cam.add_child(ov)
		ov.set_civ_data([], [], [], GW, GH, 0.0)
		ov.set_manual_icons(_icons())
		ov.set_landmarks(_landmarks())
		out["ov"] = ov
	return out


func _settle() -> void:
	for f in 4:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw


## Puts `C_PEAK` at the view centre at camera zoom `z`, waits for every cached
## layer to rebuild, and returns the frame.
func _frame(m: Dictionary, z: float) -> Image:
	var cam: Control = m["cam"]
	var ov: Control = m["ov"]
	var rect: Rect2 = ov._displayed_rect()
	var c: Vector2 = ov._cell_to_screen(Vector2(C_PEAK), rect)
	cam.scale = Vector2(z, z)
	cam.position = Vector2(W, H) * 0.5 - c * z
	ov.set_camera_zoom(z)
	ov.queue_redraw()
	for f in 6:
		await get_tree().process_frame
	var guard := 0
	while ov.pending_builds() > 0 and guard < 240:
		await get_tree().process_frame
		guard += 1
	await RenderingServer.frame_post_draw
	return m["vp"].get_texture().get_image()


## The drawn ring radius in framebuffer px: the rightmost bright (> 0.30 max
## channel) pixel on the centre row, searched out to `1.3 r + 6` from the centre
## (`r` the radius the script claims), capped at the frame edge. The window keeps
## the cave (16 cells right) out at every zoom this probe draws; a ring wider than
## the frame reads as the frame's half width, which is what the user sees.
func _radius_px(img: Image, r: float) -> int:
	var cx := W / 2
	var y := H / 2
	var lim := mini(W - 1, cx + int(ceil(1.3 * r + 6.0)))
	var last := cx
	for x in range(cx, lim + 1):
		var p := img.get_pixel(x, y)
		if maxf(p.r, maxf(p.g, p.b)) > 0.30:
			last = x
	return last - cx


## Bright pixels in a 40 px box around `cell` in a frame drawn at zoom `z`.
func _ink(img: Image, ov: Control, cell: Vector2i, z: float) -> int:
	var rect: Rect2 = ov._displayed_rect()
	var local: Vector2 = ov._cell_to_screen(Vector2(cell), rect)
	var c: Vector2 = Vector2(W, H) * 0.5 + (local - ov._cell_to_screen(Vector2(C_PEAK), rect)) * z
	var n := 0
	for y in range(maxi(0, int(c.y) - 20), mini(H, int(c.y) + 21)):
		for x in range(maxi(0, int(c.x) - 20), mini(W, int(c.x) + 21)):
			var p := img.get_pixel(x, y)
			if maxf(p.r, maxf(p.g, p.b)) > 0.30:
				n += 1
	return n
