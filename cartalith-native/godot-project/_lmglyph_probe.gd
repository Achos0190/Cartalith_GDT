extends Node
## Ruling AL (2026-09-23): landmarks draw their own per-kind glyph inside the
## class ring, and a click on one opens the right dock's Landmark context.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x900 _lmglyph_probe.tscn -- --out <dir>
##   ... _lmglyph_probe.tscn -- --out <dir> --shot-only      (screenshots only; runs on a pre-AL overlay)
##
## Windowed, never `--headless`: section S reads pixels (MISTAKES.md, "Run a
## pixel probe"). Flags this probe reads: `--out DIR` (where PNGs go; required
## for shots), `--shot-only`. Anything else is refused.
##
## S -- synthetic, blank background, the overlay alone: same kind => near-
##      identical pixels (positive control; sub-pixel AA only), different kind
##      => far more different pixels, a kind
##      with no glyph => ring only (no ink inside the ring).
## L -- the live shell over a real generated world and a real landmark_run():
##      kind census, screenshots, and clicks pushed through `_gui_input` --
##      a landmark (dock shows exactly `bridge.landmarks()[i]`), a settlement
##      (unchanged), empty map (unchanged), and a contested pair.

var app: Node
var _fail := 0
var _out := ""
var _shot_only := false
var _sel_s: Array = []
var _sel_l: Array = []

func _p(s: String) -> void:
	print("LMGLYPH %s" % s)

func _check(name: String, cond: bool, detail: String = "") -> void:
	_p("%s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _collect(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append((c as Label).text)
		_collect(c, out)

func _dock_texts() -> Array:
	var out: Array = []
	_collect(app.right_dock_body, out)
	return out

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--out":
				_out = args[i + 1]
				i += 1
			"--shot-only":
				_shot_only = true
			_:
				_p("ABORT: unknown argument '%s'" % args[i])
				get_tree().quit(2)
				return
		i += 1
	if DisplayServer.get_name() == "headless":
		_p("ABORT: pixel probe, run windowed")
		get_tree().quit(2)
		return
	var wd := get_tree().create_timer(600.0)
	wd.timeout.connect(func():
		_p("WATCHDOG")
		get_tree().quit(3))
	if not _shot_only:
		await _synthetic()
	await _live()
	_p("---- %s ----" % ("PASS" if _fail == 0 else "%d FAILED" % _fail))
	get_tree().quit(1 if _fail > 0 else 0)

# -- S: synthetic -------------------------------------------------------------

const SW := 900
const SH := 400
const HALF := 26

func _synthetic() -> void:
	var vp := SubViewport.new()
	vp.size = Vector2i(SW, SH)
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var bg := ColorRect.new()
	bg.size = Vector2(SW, SH)
	bg.color = Color(0.03, 0.03, 0.04)
	vp.add_child(bg)
	var ov := Control.new()
	ov.set_script(load("res://map_overlay.gd"))
	ov.size = Vector2(SW, SH)
	vp.add_child(ov)
	ov.set_civ_data([], [], [], 64, 28, 0.0)
	var mk := func(id: int, kind: String, cls: String, x: int, y: int) -> Dictionary:
		return {"id": id, "kind": kind, "class": cls, "x": x, "y": y, "importance": 0.6,
			"elevation": 100.0, "score": 0.5, "causal": []}
	var cells := [Vector2i(8, 8), Vector2i(24, 8), Vector2i(40, 8), Vector2i(8, 20), Vector2i(24, 20), Vector2i(40, 20)]
	ov.set_landmarks([
		mk.call(1, "waterfall", "regional", cells[0].x, cells[0].y),
		mk.call(2, "cave", "regional", cells[1].x, cells[1].y),
		mk.call(3, "waterfall", "regional", cells[2].x, cells[2].y),
		mk.call(4, "zz_no_glyph_kind", "local", cells[3].x, cells[3].y),
		mk.call(5, "shrine", "local", cells[4].x, cells[4].y),
		mk.call(6, "volcanic_feature", "regional", cells[5].x, cells[5].y),
	])
	await _frames(3)
	await RenderingServer.frame_post_draw
	var img: Image = vp.get_texture().get_image()
	var rect: Rect2 = ov.displayed_rect()
	var crops: Array = []
	var inner: Array = []
	for c: Vector2i in cells:
		var sp: Vector2 = ov._cell_to_screen(Vector2(c.x, c.y), rect)
		crops.append(img.get_region(Rect2i(int(sp.x) - HALF, int(sp.y) - HALF, HALF * 2, HALF * 2)).get_data())
		## Bright pixels well inside the ring (r >= 6.75 px for local): a box of
		## half-width 3 at the centre, which the ring stroke never reaches.
		var n := 0
		for y in range(int(sp.y) - 3, int(sp.y) + 4):
			for x in range(int(sp.x) - 3, int(sp.x) + 4):
				var px := img.get_pixel(x, y)
				if maxf(px.r, maxf(px.g, px.b)) > 0.30:
					n += 1
		inner.append(n)
	if _out != "":
		img.save_png(_out.path_join("lmglyph_synthetic.png"))
	_p("S inner-ink counts (waterfall, cave, waterfall, no-glyph, shrine, volcano): %s" % [inner])
	## Same kind at two positions differs only by sub-pixel antialiasing (the
	## cell pitch is 14.06 px, not an integer), so "same" is measured as a
	## byte distance, against the distance between two different kinds.
	var same := _dist(crops[0], crops[2])
	var diff := _dist(crops[0], crops[1])
	_p("S byte distance: waterfall~waterfall=%d  waterfall~cave=%d" % [same, diff])
	_check("S1 positive control: same kind is far closer than a different kind", same * 4 < diff)
	_check("S2 waterfall vs cave (same class) draw DIFFERENT marks", diff > 0)
	_check("S3 shrine vs no-glyph kind (both local) differ", crops[4] != crops[3])
	_check("S4 glyph kinds put ink inside the ring", inner[0] > 0 and inner[1] > 0 and inner[4] > 0 and inner[5] > 0)
	_check("S5 a kind with no glyph falls back to ring only (no ink inside)", inner[3] == 0, "inner=%d" % inner[3])
	vp.queue_free()

func _dist(a: PackedByteArray, b: PackedByteArray) -> int:
	var t := 0
	for i in a.size():
		t += absi(a[i] - b[i])
	return t

# -- L: live ------------------------------------------------------------------

func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	img.save_png(_out.path_join(name))
	_p("shot %s  %dx%d" % [name, img.get_width(), img.get_height()])

func _press(ov: Control, local: Vector2) -> void:
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = local
		ev.device = 0
		ov._gui_input(ev)
	await _frames(2)

func _live() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	## A pinned world, so a before (pre-AL) run and an after run draw the SAME
	## landmarks -- the shell's own Generate picks a fresh seed per boot.
	app.bridge.generate({"seed": 234476, "width_km": 800.0, "grid_w": 1024, "grid_h": 656,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var waited := 0
	while app.bridge.generating and waited < 3000:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	if not app.bridge.has_world:
		_check("a world generated", false)
		return
	app.bridge.landmark_run()
	waited = 0
	while app.bridge.generating and waited < 3000:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	var lms: Array = app.bridge.landmarks()
	var ov: Control = app.viewport.overlay
	_check("L0 landmarks placed and handed to the overlay", lms.size() > 0 and ov._landmarks.size() == lms.size(),
		"bridge=%d overlay=%d" % [lms.size(), ov._landmarks.size()])
	var kinds := {}
	var noglyph := {}
	for lm: Dictionary in lms:
		var k := String(lm["kind"])
		kinds[k] = int(kinds.get(k, 0)) + 1
		if DccIcons.landmark_glyph(k) == "":
			noglyph[k] = true
	_p("L kinds placed (%d distinct, %d landmarks): %s" % [kinds.size(), lms.size(), kinds])
	_p("L kinds with NO glyph in this build: %s" % [noglyph.keys()])
	await _shot("lmglyph_live_fit.png")

	## A contact sheet: one crop per distinct kind, framebuffer coordinates
	## through the overlay's own canvas transform.
	var rect: Rect2 = ov._displayed_rect()
	var interior: Rect2 = ov._interior_rect(rect)
	var xf: Transform2D = ov.get_global_transform_with_canvas()
	var fb := get_viewport().get_texture().get_image()
	var sheet := Image.create(8 * 96, 96, false, fb.get_format())
	var seen := {}
	var col := 0
	for lm: Dictionary in lms:
		var k := String(lm["kind"])
		if seen.has(k) or col >= 8:
			continue
		var lp: Vector2 = ov._cell_to_screen(Vector2(lm["x"], lm["y"]), rect)
		if not interior.has_point(lp):
			continue
		var sp: Vector2 = xf * lp
		var r := Rect2i(int(sp.x) - 24, int(sp.y) - 24, 48, 48)
		if r.position.x < 0 or r.position.y < 0 or r.end.x > fb.get_width() or r.end.y > fb.get_height():
			continue
		var crop := fb.get_region(r)
		crop.resize(96, 96, Image.INTERPOLATE_NEAREST)
		sheet.blit_rect(crop, Rect2i(0, 0, 96, 96), Vector2i(col * 96, 0))
		_p("L sheet[%d] = %s (%s, imp %.2f)" % [col, k, lm["class"], float(lm["importance"])])
		seen[k] = true
		col += 1
	if _out != "":
		sheet.save_png(_out.path_join("lmglyph_live_sheet.png"))
	if _shot_only:
		return

	app.viewport.landmark_selected.connect(func(d, i): _sel_l.append([d, i]))
	app.viewport.settlement_selected.connect(func(d, i): _sel_s.append([d, i]))

	## (c) a landmark nothing else contests.
	var li := -1
	for i in lms.size():
		var lp: Vector2 = ov._cell_to_screen(Vector2(lms[i]["x"], lms[i]["y"]), rect)
		if interior.has_point(lp) and ov._pick_mark(lp, interior, rect)["l"] == i:
			li = i
			break
	_check("L1 found an uncontested landmark to click", li >= 0)
	if li >= 0:
		_sel_s.clear(); _sel_l.clear()
		await _press(ov, ov._cell_to_screen(Vector2(lms[li]["x"], lms[li]["y"]), rect))
		var truth: Dictionary = app.bridge.landmarks()[li]
		_check("L2 landmark_selected carried index %d" % li, _sel_l.size() == 1 and int(_sel_l[0][1]) == li, "%s" % [_sel_l])
		_check("L3 settlement_selected said null for the same click", _sel_s.size() == 1 and _sel_s[0][0] == null)
		_check("L4 right dock is in the Landmark context", app.right_dock_ctrl._context == "landmark",
			"context=%s" % app.right_dock_ctrl._context)
		var t := _dock_texts()
		var want := [
			app.right_dock_ctrl._landmark_label(String(truth["kind"])),
			String(truth["class"]).capitalize(),
			app.right_dock_ctrl._m_text(float(truth["elevation"])),
			"%d%%" % roundi(100.0 * float(truth["importance"])),
			"%d%%" % roundi(100.0 * float(truth["score"])),
		]
		var causal: Array = Array(truth["causal"])
		for j in causal.size():
			want.append("%d. %s" % [j + 1, String(causal[j])])
		var missing: Array = []
		for w in want:
			if not t.has(w):
				missing.append(w)
		_p("L truth[%d] = kind=%s class=%s elev=%.1f imp=%.4f score=%.4f causal=%s" % [li, truth["kind"],
			truth["class"], float(truth["elevation"]), float(truth["importance"]), float(truth["score"]), causal])
		_p("L dock texts: %s" % [t.slice(0, 18)])
		_check("L5 dock shows every field of bridge.landmarks()[%d] (%d strings)" % [li, want.size()], missing.is_empty(),
			"missing=%s" % [missing])
		_check("L5b causal chain non-empty in this real world", causal.size() > 0, "n=%d" % causal.size())
		await _shot("lmglyph_live_selected.png")

	## (d) a settlement click is unchanged.
	var sets: Array = ov._settlements
	var si := -1
	for i in sets.size():
		var sp: Vector2 = ov._cell_to_screen(Vector2(sets[i]["x"], sets[i]["y"]), rect)
		if interior.has_point(sp) and ov._pick_mark(sp, interior, rect)["s"] == i:
			si = i
			break
	_check("L6 found a clickable settlement", si >= 0)
	if si >= 0:
		_sel_s.clear(); _sel_l.clear()
		await _press(ov, ov._cell_to_screen(Vector2(sets[si]["x"], sets[si]["y"]), rect))
		_check("L7 settlement click selects that settlement", _sel_s.size() == 1 and int(_sel_s[0][1]) == si)
		_check("L8 ... and no landmark", _sel_l.size() == 1 and _sel_l[0][0] == null)
		_check("L9 dock is in the Settlement context", app.right_dock_ctrl._context == "settlement")

	## (d) empty map.
	var empty := Vector2(-1, -1)
	for gy in range(20):
		for gx in range(20):
			var pt := interior.position + interior.size * Vector2((gx + 0.5) / 20.0, (gy + 0.5) / 20.0)
			var pk: Dictionary = ov._pick_mark(pt, interior, rect)
			if pk["s"] == -1 and pk["l"] == -1:
				empty = pt
				break
		if empty.x >= 0:
			break
	_sel_s.clear(); _sel_l.clear()
	await _press(ov, empty)
	_check("L10 empty-map click: settlement null, landmark null", _sel_s.size() == 1 and _sel_s[0][0] == null
		and _sel_l.size() == 1 and _sel_l[0][0] == null)
	## Sample, or River when the empty point sits on a channel: `right_dock.gd`'s
	## `_on_map_clicked_river` picks rivers on a miss, which is pre-AL behaviour.
	var ctx: String = app.right_dock_ctrl._context
	_check("L11 dock left the Landmark/Settlement context (Sample or a river pick)", ctx == "sample" or ctx == "river",
		"context=%s" % ctx)

	## Contested: a synthetic landmark one cell east of a real settlement.
	if si >= 0:
		var s: Dictionary = sets[si]
		var saved: Array = ov._landmarks
		## Offset by enough cells to put the centres ~6 px apart at this zoom
		## (a cell here is well under a pixel at fit).
		var off := maxi(1, ceili(6.0 / (rect.size.x / float(ov._gw))))
		ov.set_landmarks([{"id": 99, "kind": "shrine", "class": "cultural", "x": int(s["x"]) + off, "y": int(s["y"]),
			"importance": 0.5, "elevation": 0.0, "score": 0.5, "causal": ["probe"]}])
		var spt: Vector2 = ov._cell_to_screen(Vector2(s["x"], s["y"]), rect)
		var lpt: Vector2 = ov._cell_to_screen(Vector2(int(s["x"]) + off, s["y"]), rect)
		var both_hit: bool = ov._hit_test_settlement(lpt, interior, rect) == si
		_p("L contest: settlement centre %s, landmark centre %s (%.1f px apart); settlement pin also under the landmark centre: %s"
			% [spt, lpt, spt.distance_to(lpt), both_hit])
		_check("L12 pointing at the settlement's centre picks the settlement", ov._pick_mark(spt, interior, rect)["s"] == si)
		_check("L13 pointing at the landmark's centre picks the landmark", ov._pick_mark(lpt, interior, rect)["l"] == 0)
		ov.set_landmarks(saved)
