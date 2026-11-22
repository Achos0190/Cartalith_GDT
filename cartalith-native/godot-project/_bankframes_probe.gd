extends Node
## Ruling HG (owner, 2026-11-22): a settlement's marker stands on the river bank, not in the river. Checked in the real
## shell against the pixels the map draws (`WorldGen.settlement_bank_offsets`, `map_overlay.gd::_settle_pos`).
##
##   Godot_v4.7.2-stable_win64_console.exe --path . --resolution 1600x900 _bankframes_probe.tscn -- --out <dir> [--seed N] [--mutate off] [--quick]
##
## **Windowed, never `--headless`**, from a project copy with its own `config/name` and a scratch APPDATA.
##
## What it protects (each check names itself; each pixel claim has a positive control):
##   1. THE SWITCH: `civ.bank_sites` is registered, ON for a new world, and the engine answers offsets with it on and
##      none with it off.
##   2. THE CENSUS AGREES WITH THE RASTER: the framed settlements are ones the engine's census (`bank_census`) says are
##      inside a drawn stroke; with the switch off the map's own river water is under their marker point (the positive
##      control: the frame with the engine's rivers on differs from the frame with them off at that pixel), and with it
##      on the water is under NONE of them, at x1, x4 and x16.
##   3. ONE POINT: with the switch on, the hit test finds each framed settlement at its drawn point.
##   4. SNAP TO WATER: a settlement dropped by hand with "Snap to water" whose cell is in a drawn river is drawn on
##      the bank too.
##   5. THE FRAMES: per settlement a crop round the marker, before (switch off) and after (on), at x1, x4 and x16;
##      `<out>/combined.png` is one row per settlement (the hand-dropped one last), columns x1 off, x1 on, x4 off,
##      x4 on, x16 off, x16 on.
##   6. THE ZOOM STEP: at a zoom between two steps the overlay asked the engine at a density at or below the true one
##      and within one step (a factor of 2^(1/8), 1.0905) of it, so a marker is never nearer the water than drawn.
##   7. THE RIVER CHANGED: when the shell hands the map a river again (`ViewportHost._apply_river_paint`: every repaint,
##      a look with another river width, the window the zoom law reads) the overlay asks the engine again.
## `--mutate off`: the "after" state is left off, so check 2 must FAIL. `--quick`: three settlements at x16 only and no
## hand drop (what the overlay's literal mutants are run against; the frames come from a full run).
## Result: `RESULT: PASS|FAIL (<n> failed) checks=<n>`; exit 0 pass, 1 fail, 2 bad argument or watchdog.

const ZOOMS := [1.0, 4.0, 16.0]
const TOWNS_FRAMED := 5
## A pixel is river WATER where the Rivers-on and Rivers-off frames differ by more than this (summed over RGB, 0..3).
## A difference of two frames of the same view, so it holds on either palette. A labelled judgement: the river's
## water against the ground under it measures 0.4 to 1.2 on this world, while the floodplain tint and the wet-bank
## darkening the same layer paints just OUTSIDE the edge stay under 0.15 (the first run of this probe read them as
## ink at 0.05, which is why the threshold is not lower).
const INK_DIFF := 0.25
## A framed settlement stands at least this many cells inside the map's edge, so its crop is map and not frame.
const EDGE_CELLS := 14
const CELL := Vector2i(360, 240)

var _out := ""
var _seed := 483920
var _mutate := ""
var _quick := false
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

func _shot() -> Image:
	await RenderingServer.frame_post_draw
	return _vp.get_texture().get_image()

## A frame once the view has stopped changing (deep-zoom tiles rebuild off the main thread after a layer switch):
## two shots 20 frames apart that are identical, or the last one after 16 tries.
func _settled_shot() -> Image:
	var prev := await _shot()
	for t in 16:
		await _frames(20)
		var cur := await _shot()
		if cur.get_data() == prev.get_data():
			return cur
		prev = cur
	return prev

## The marker point of settlement `k` in the SubViewport's pixels, as the overlay draws it now.
func _marker_px(k: int) -> Vector2:
	var ov = _app.viewport.overlay
	var s: Dictionary = ov._settlements[k]
	return ov.get_global_transform_with_canvas() * ov._settle_pos(s, ov.displayed_rect())

func _ink(a: Image, b: Image, p: Vector2) -> bool:
	var x := clampi(int(p.x), 0, a.get_width() - 1)
	var y := clampi(int(p.y), 0, a.get_height() - 1)
	var ca := a.get_pixel(x, y)
	var cb := b.get_pixel(x, y)
	return absf(ca.r - cb.r) + absf(ca.g - cb.g) + absf(ca.b - cb.b) > INK_DIFF

func _view(k: int, z: float) -> void:
	var host = _app.viewport
	var s: Dictionary = host.overlay._settlements[k]
	host.reset_view()
	await _frames(4)
	if z > 1.0:
		host._zoom_at(host.size * 0.5, z)
		host.move_view_to(float(s["x"]), float(s["y"]))
	await _frames(45)

## The engine's river switch, for the water test only. The shell itself can no longer turn rivers off (finding F23:
## `ViewportHost.set_layer_visible("rivers", false)` is ignored), so the probe sets the engine's switch and repaints
## as that arm does. Always left on.
func _rivers(on: bool) -> void:
	var host = _app.viewport
	_app.bridge.set_rivers_in_map(on)
	host._apply_river_paint()
	host.overlay.queue_redraw()
	host.invalidate_lod_tiles()
	await _frames(10)

## The switch through the product's own path: the parameter write, which the shell answers by having the overlay ask
## again and redraw (`EngineBridge.bank_params_changed`). Nothing else is touched, so a frame that does not move
## after this is the product not following its switch.
func _set_bank(on: bool) -> void:
	_app.bridge.param_set("civ.bank_sites", on)
	await _frames(10)

func _crop(img: Image, centre: Vector2, half: Vector2i) -> Image:
	var r := Rect2i(Vector2i(centre) - half, half * 2).intersection(Rect2i(Vector2i.ZERO, img.get_size()))
	var c := img.get_region(r)
	c.resize(CELL.x, CELL.y, Image.INTERPOLATE_NEAREST)
	return c

func _ready() -> void:
	get_tree().create_timer(1500.0).timeout.connect(func() -> void:
		push_error("bankframes probe watchdog")
		get_tree().quit(2))
	if DisplayServer.get_name() == "headless":
		push_error("_bankframes_probe must be windowed")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--out": _out = args[i + 1]
			"--seed": _seed = int(args[i + 1])
			"--mutate": _mutate = args[i + 1]
			"--quick":
				_quick = true
				i -= 1
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
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.0).timeout
	var bridge = _app.bridge
	bridge.generate({"seed": _seed, "width_km": 800.0, "grid_w": 384, "grid_h": 288, "archetype": "", "villages": true})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)
	var host = _app.viewport
	var ov = host.overlay
	var on_state := _mutate != "off"

	print("\n== 1. the switch ==")
	print("  MAP VIEW %d x %d px (the census's --view), fit %.3f px per cell" % [int(host.size.x), int(host.size.y), ov.label_px_per_cell()])
	_ok(not bridge.param_info("civ.bank_sites").is_empty(), "civ.bank_sites is a registered parameter")
	_ok(bool(bridge.param_get("civ.bank_sites")), "civ.bank_sites is ON in the app's parameters (a new world)")
	var ppc4: float = ov.label_px_per_cell() * 4.0
	var on_off: PackedVector2Array = bridge.settlement_bank_offsets(ppc4, -1.0)
	var moved := 0
	for v in on_off:
		moved += 1 if v.length() > 0.0 else 0
	_ok(on_off.size() == bridge.settlements().size() and moved >= 10, "with the switch on the engine answers an offset per settlement, %d of %d non-zero at x4" % [moved, on_off.size()])
	bridge.param_set("civ.bank_sites", false)
	_ok(bridge.settlement_bank_offsets(ppc4, -1.0).is_empty(), "with the switch off the engine answers none (every marker at its cell centre)")
	bridge.param_set("civ.bank_sites", true)

	print("\n== 2-3. the census against the raster ==")
	var rows: Array = bridge.world_gen.bank_census(host.size.x, host.size.y, 2.0)
	var in1 := 0
	var in4 := 0
	var left := 0
	var cand: Array = []
	for r in rows:
		in1 += int(r["in_x1"])
		in4 += int(r["in_x4"])
		left += int(r["left_x1"]) + int(r["left_x4"])
		var edge: bool = mini(mini(int(r["x"]), int(r["y"])), mini(bridge.grid_size().x - int(r["x"]), bridge.grid_size().y - int(r["y"]))) < EDGE_CELLS
		if r["in_x1"] and r["in_x4"] and r.has("w") and not edge:
			cand.append(r)
	print("  engine census in this view: %d settlements, inside a drawn stroke x1 %d, x4 %d; still inside at their bank point %d" % [rows.size(), in1, in4, left])
	_ok(in1 > 0 and in4 > 0, "the census has settlements inside a drawn river (x1 %d, x4 %d): not an empty measurement" % [in1, in4])
	_ok(left == 0, "the engine's bank point leaves none inside (%d)" % left)
	## The widest rivers first; a generated (not village-flagged) place before a village of the same width.
	cand.sort_custom(func(a, b): return (float(a["w"]) + (0.0 if a["village"] else 1.0)) > (float(b["w"]) + (0.0 if b["village"] else 1.0)))
	var towns: Array = []
	for r in cand:
		if towns.size() < TOWNS_FRAMED:
			towns.append(int(r["i"]))
	_ok(towns.size() == TOWNS_FRAMED, "%d settlements inside the stroke at both zooms to frame (%d)" % [TOWNS_FRAMED, towns.size()])

	## 4. the hand drop: the first cell beside a wide drawn river, clear of every settlement, whose snapped cell is in the stroke.
	print("\n== 4. snap to water ==")
	var snap_k := -1
	var runs: Array = bridge.rivers(1)
	var places: Array = bridge.settlements()
	var tries := 0
	if _quick:
		runs = []
		towns.resize(3)
	for run in runs:
		if snap_k >= 0 or tries > 60:
			break
		if not run.has("widths") or run.has("parallel_of"):
			continue
		var rp: PackedVector2Array = run["render_points"]
		var ws: PackedFloat32Array = run["widths"]
		for j in range(4, rp.size() - 4, 5):
			if snap_k >= 0 or tries > 60:
				break
			if ws[j] < 0.25 or minf(minf(rp[j].x, rp[j].y), minf(bridge.grid_size().x - rp[j].x, bridge.grid_size().y - rp[j].y)) < EDGE_CELLS:
				continue
			var far := true
			for s in bridge.settlements():
				if Vector2(float(s["x"]), float(s["y"])).distance_to(rp[j]) < 6.0:
					far = false
			if not far:
				continue
			tries += 1
			var before_n: int = bridge.settlements().size()
			var k: int = bridge.civ_drop_settlement(rp[j].x, rp[j].y, "town", 0, "Snapford", true)
			if k < 0 or bridge.settlements().size() == before_n:
				continue
			var row: Dictionary = bridge.world_gen.bank_census(host.size.x, host.size.y, 2.0)[k]
			if row["in_x4"]:
				snap_k = k
				print("  dropped 'Snapford' with Snap to water at a click on the river at (%.1f, %.1f): it landed on cell (%d, %d), inside the stroke at x1 %s, x4 %s" % [rp[j].x, rp[j].y, row["x"], row["y"], row["in_x1"], row["in_x4"]])
	if not _quick:
		_ok(snap_k >= 0, "a hand drop with Snap to water landed on a cell inside a drawn river (%d tries): the case the ruling names" % tries)
	## The hand drop is a new settlement: the roster is handed over as the shell hands it after a drop.
	host.refresh()
	await _frames(20)
	if snap_k >= 0:
		towns.append(snap_k)

	print("\n== 2, 3, 5. the pixels and the frames ==")
	var sheet: Array = []
	var ink_off := {1.0: 0, 4.0: 0, 16.0: 0}
	var ink_on := {1.0: 0, 4.0: 0, 16.0: 0}
	var picked := 0
	var picks := 0
	var toggled := 0
	var crop_at := Vector2i.ZERO
	for k in towns:
		var s: Dictionary = ov._settlements[k]
		var cells: Array = []
		for z in ([16.0] if _quick else ZOOMS):
			await _view(k, z)
			## The map's own river water: Rivers on against Rivers off, with the settlements and the ways hidden for
			## both (roads meet at a settlement's cell, and a road drawn over the river there reads as "no water": the
			## third run of this probe counted two such towns as clear).
			for layer in ["settlements", "roads", "sea_routes"]:
				host.set_layer_visible(layer, false)
			await _frames(10)
			var with_r := await _settled_shot()
			await _rivers(false)
			var no_r := await _settled_shot()
			await _rivers(true)
			for layer in ["settlements", "roads", "sea_routes"]:
				host.set_layer_visible(layer, true)
			await _frames(10)
			var half := Vector2i(60, 40) if z < 2.0 else Vector2i(90, 60)
			for state in [false, true]:
				await _set_bank(state and on_state)
				var p := _marker_px(k)
				## Both states are cropped round the cell centre's point, so the marker is seen to move.
				var c0 := p if not state else Vector2(crop_at)
				crop_at = Vector2i(c0)
				var inked := _ink(with_r, no_r, p)
				if state:
					ink_on[z] += int(inked)
					## 3. one point: a press at the drawn point finds this settlement.
					if z == (16.0 if _quick else 4.0):
						picks += 1
						var local: Vector2 = ov.get_global_transform_with_canvas().affine_inverse() * p
						var hit: int = ov._hit_test_settlement(local, ov._interior_rect(ov.displayed_rect()), ov.displayed_rect())
						picked += int(hit == k)
						if hit != k:
							print("  NOTE: the press at #%d's point found #%d" % [k, hit])
				else:
					ink_off[z] += int(inked)
					## The control for the ink test itself: the Rivers switch moved pixels in this crop.
					var moved_px := 0
					for yy in range(crop_at.y - half.y, crop_at.y + half.y, 2):
						for xx in range(crop_at.x - half.x, crop_at.x + half.x, 2):
							moved_px += int(_ink(with_r, no_r, Vector2(xx, yy)))
					toggled += int(moved_px > 20)
				var img := await _settled_shot()
				var crop := _crop(img, c0, half)
				crop.save_png(_out.path_join("s%d_x%d_%s.png" % [k, int(z), "on" if state else "off"]))
				cells.append(crop)
				print("  #%d %s (%d,%d) x%d %s: marker at %s px, %s the river's water" % [k, s["kind"], int(s["x"]), int(s["y"]), int(z), "on " if state else "off", str(Vector2i(p)), "IN" if inked else "clear of"])
		sheet.append(cells)
	await _set_bank(true)
	var n := towns.size()
	var crops := n * (1 if _quick else ZOOMS.size())
	_ok(toggled == crops, "control: the Rivers switch moved pixels in every crop (%d of %d): the water test can see a river" % [toggled, crops])
	for z in ZOOMS:
		print("  x%d: marker point in the river's water, switch off %d of %d, on %d of %d" % [int(z), ink_off[z], n, ink_on[z], n])
	_ok(ink_off[1.0] + ink_off[4.0] + ink_off[16.0] >= 3, "control: with the switch off the map's river water is under the marker point (x1 %d, x4 %d of %d each): the census and the raster agree" % [ink_off[1.0], ink_off[4.0], n])
	_ok(ink_on[1.0] == 0 and ink_on[4.0] == 0 and ink_on[16.0] == 0, "ON THE BANK: with the switch on no marker point is in the river's water (x1 %d, x4 %d, x16 %d)" % [ink_on[1.0], ink_on[4.0], ink_on[16.0]])
	_ok(picks > 0 and picked == picks, "ONE POINT: a press at the drawn point finds the settlement (%d of %d)" % [picked, picks])

	print("
== 6. the zoom step ==")
	host.reset_view()
	await _frames(4)
	host._zoom_at(host.size * 0.5, 5.3)
	await _frames(30)
	ov._settle_pos(ov._settlements[towns[0]], ov.displayed_rect())
	var true_ppc: float = ov.label_px_per_cell() * ov._camera_zoom
	print("  zoom %.3f: the map is drawn at %.4f px per cell, the engine was asked at %.4f" % [ov._camera_zoom, true_ppc, ov._bank_ppc])
	_ok(ov._bank_ppc <= true_ppc * 1.0001 and ov._bank_ppc > true_ppc / 1.0905 * 0.9999, "THE ZOOM STEP: asked at or below the drawn density and within one step of it (%.4f against %.4f)" % [ov._bank_ppc, true_ppc])
	_ok(absf(ov._bank_ppc - true_ppc) > true_ppc * 0.001, "control: this zoom is between two steps (the bound above is not met by equality)")

	print("
== 7. the river changed ==")
	var asked := [0]
	ov.set_bank_resolver(func(ppc: float, gap: float) -> PackedVector2Array:
		asked[0] += 1
		return bridge.settlement_bank_offsets(ppc, gap))
	ov._settle_pos(ov._settlements[towns[0]], ov.displayed_rect())
	var before_asks: int = asked[0]
	ov._settle_pos(ov._settlements[towns[0]], ov.displayed_rect())
	_ok(before_asks == 1 and asked[0] == 1, "control: with nothing changed the overlay asks the engine once, not per marker (%d, then %d)" % [before_asks, asked[0]])
	host._apply_river_paint()
	ov._settle_pos(ov._settlements[towns[0]], ov.displayed_rect())
	_ok(asked[0] == 2, "THE RIVER CHANGED: after the shell hands the map its river again the overlay asks again (%d asks)" % asked[0])
	ov.set_bank_resolver(bridge.settlement_bank_offsets)

	if not sheet.is_empty() and not _quick:
		var out_img := Image.create(CELL.x * 6, CELL.y * sheet.size(), false, Image.FORMAT_RGBA8)
		out_img.fill(Color.WHITE)
		for ri in sheet.size():
			for ci in (sheet[ri] as Array).size():
				var im: Image = sheet[ri][ci]
				im.convert(Image.FORMAT_RGBA8)
				out_img.blit_rect(im, Rect2i(Vector2i.ZERO, CELL), Vector2i(ci * CELL.x, ri * CELL.y))
		out_img.save_png(_out.path_join("combined.png"))
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
