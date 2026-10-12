extends Node
## Committed verification harness for EXPORT_SCOPE.md §7 E4 batch B -- the
## export overlay drawn into one `SubViewport` tile (`shell/export_overlay.gd`,
## `map_overlay.gd::copy_state_from` / `prepare_export`), on a real generated
## world inside the real shell.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _exportoverlay_probe.tscn
##
## **Windowed, never `--headless`**: every check reads pixels back from a
## `SubViewport`, and the headless display driver rasterises nothing
## (`MISTAKES.md`, "Write `--headless` into a brief").
##
## What it checks, and against what:
##  1. Layout arithmetic: a cell's centre lands on exactly the export pixel
##     `render.rs::bake_rect` samples that cell at, `c * (W-1)/(gw-1)`, on both
##     axes, first and last cell included; content translation.
##  3. Registration at 8192 px: the westmost and eastmost isolated big
##     settlements land within 1.5 px of their `bake_rect` pixel -- at the
##     edges, where a naive texel-centred "size W, zoom 1" camera would be more
##     than 6 px off, so the check can tell the two apart.
##  4. Uniform magnification: the same map region at 8192 px carries about
##     gain^2 = 4x the covered pixels it does at 4096 px; a negative control with
##     the symbol gain withheld covers measurably less.
##  5. End to end through the batch-A session at 2048 px (one band, one tile):
##     every pixel the overlay leaves transparent is byte-identical to the
##     terrain-only `export_image`, and every fully opaque overlay pixel is the
##     overlay's own colour.
##  6. State copy is complete (`EXPORT_SCOPE.md` §5 point 5): the live overlay
##     and a `copy_state_from` copy, drawn under one identical camera, differ by
##     0 pixels -- with a coverage floor so an empty draw cannot pass, and a
##     negative control (the copy's way scale doubled) that must differ. Run
##     last: it moves the live overlay into a `SubViewport` to draw it.
##
## Result: `  PASS  <what>` / `  FAIL  <what>` per check, a final
##         `== PASS|FAIL: N failure(s) ==`; exit 1 when any failed, else 0 (a
##         watchdog quits with 2). The composited 2048 px export is left at
##         `user://_exportoverlay_probe/composite.png` for a person to look at.

const EXPORT_OVERLAY := preload("res://shell/export_overlay.gd")
const OVERLAY := preload("res://map_overlay.gd")

var fails := 0

func _ok(cond: bool, what: String) -> void:
	print(("  PASS  %s" if cond else "  FAIL  %s") % what)
	if not cond:
		fails += 1

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _find_script(n: Node, file: String) -> Node:
	for c in n.get_children():
		if c.get_script() != null and String(c.get_script().resource_path).ends_with(file):
			return c
		var r := _find_script(c, file)
		if r != null:
			return r
	return null

## A transparent SubViewport of `size` holding `ctl` under an identity `Node2D`.
func _host(ctl: Control, size: Vector2i) -> SubViewport:
	var vp := SubViewport.new()
	vp.transparent_bg = true
	vp.disable_3d = true
	vp.size = size
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var cam := Node2D.new()
	vp.add_child(cam)
	if ctl.get_parent() != null:
		ctl.get_parent().remove_child(ctl)
	cam.add_child(ctl)
	ctl.position = Vector2.ZERO
	return vp

func _grab(vp: SubViewport) -> Image:
	var img := vp.get_texture().get_image()
	if img != null and img.get_format() != Image.FORMAT_RGBA8:
		img.convert(Image.FORMAT_RGBA8)
	return img

## [differing pixels, pixels with any alpha in a, pixels with any alpha in b].
func _diff(a: PackedByteArray, b: PackedByteArray) -> Array:
	var d := 0
	var ca := 0
	var cb := 0
	for i in range(0, mini(a.size(), b.size()), 4):
		if a[i + 3] > 0:
			ca += 1
		if b[i + 3] > 0:
			cb += 1
		if a[i] != b[i] or a[i + 1] != b[i + 1] or a[i + 2] != b[i + 2] or a[i + 3] != b[i + 3]:
			d += 1
	return [d, ca, cb]

## Pixels of `rgba` (w x h) with alpha above 127.
func _covered(rgba: PackedByteArray) -> int:
	var c := 0
	for i in range(3, rgba.size(), 4):
		if rgba[i] > 127:
			c += 1
	return c

## The pin drawn nearest the centre of a w x h tile, measured by its dark
## outline ring (`MARKER_OUTLINE`, drawn at the pin's radius): from the centre,
## walk left, right, up and down to the first opaque dark pixel, and take the
## midpoints. Symmetric by construction, so the glyph inside, the name beside it,
## trait badges below and the plate frame's clip cannot bias it the way a colour
## bounding box was biased (an edge capital's ring was clipped by the frame).
## `(-1, -1)` when any walk finds no outline within `reach`.
func _pin_centre(rgba: PackedByteArray, w: int, h: int, reach: int) -> Vector2:
	var cx := w / 2
	var cy := h / 2
	var hits := []
	for dir: Vector2i in [Vector2i(-1, 0), Vector2i(1, 0), Vector2i(0, -1), Vector2i(0, 1)]:
		var found := -1.0
		for t in range(1, reach):
			var x := cx + dir.x * t
			var y := cy + dir.y * t
			if x < 0 or y < 0 or x >= w or y >= h:
				break
			var i := (y * w + x) * 4
			if rgba[i + 3] >= 250 and rgba[i] + rgba[i + 1] + rgba[i + 2] < 3 * 70:
				found = float(t)
				break
		if found < 0.0:
			return Vector2(-1, -1)
		hits.append(found)
	return Vector2(cx + (hits[1] - hits[0]) * 0.5, cy + (hits[3] - hits[2]) * 0.5)

func _ready() -> void:
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		push_error("export-overlay probe watchdog: _ready never finished")
		get_tree().quit(2))
	var dir := ProjectSettings.globalize_path("user://_exportoverlay_probe")
	DirAccess.make_dir_recursive_absolute(dir)
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge: Node = app.bridge
	bridge.generate({"seed": 9137, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(6)
	var ov: Control = _find_script(app, "map_overlay.gd")
	_ok(ov != null and int(ov.get("_gw")) > 1, "the live overlay holds a world")
	if ov == null:
		print("== FAIL: %d failure(s) ==" % (fails + 1))
		get_tree().quit(1)
		return
	var gw := int(ov.get("_gw"))
	var gh := int(ov.get("_gh"))

	print("\n== 1. layout arithmetic ==")
	var W := 8192
	var H := int(round(float(W) * gh / gw))   ## `render.rs::bake_dims`
	var lay: Dictionary = EXPORT_OVERLAY.layout(W, H, gw, gh)
	var worst := 0.0
	for c in [Vector2i(0, 0), Vector2i(17, 33), Vector2i(gw - 1, gh - 1)]:
		var got: Vector2 = EXPORT_OVERLAY.texture_to_export(Vector2(c) + Vector2(0.5, 0.5), lay)
		var want := Vector2(c.x * float(W - 1) / (gw - 1), c.y * float(H - 1) / (gh - 1))
		worst = maxf(worst, got.distance_to(want))
	_ok(worst < 1e-3, "a cell centre lands on bake_rect's sample pixel, first and last cell included (worst %s px)" % str(worst))
	_ok(EXPORT_OVERLAY.layout(1, 1, gw, gh).is_empty() and EXPORT_OVERLAY.layout(W, H, 1, gh).is_empty(),
		"a degenerate image or grid has no layout")
	_ok(EXPORT_OVERLAY.overlay_content({"settlements": true})["settlement_tier"] == "all"
		and EXPORT_OVERLAY.overlay_content({"settlement_tier": "town"})["settlement_tier"] == "town"
		and EXPORT_OVERLAY.overlay_content({})["settlement_tier"] == "",
		"content: settlements with no tier is every tier; no settlements key is none")

	print("\n== 3. registration at %dx%d ==" % [W, H])
	var setts: Array = ov.get("_settlements")
	var big: Array = []
	for i in setts.size():
		var s: Dictionary = setts[i]
		if not (String(s["kind"]) in ["city", "capital", "metropolis"]):
			continue
		var x := int(s["x"])
		var y := int(s["y"])
		if x < 8 or y < 8 or x > gw - 9 or y > gh - 9:
			continue
		var lonely := true
		for t in setts:
			if t == s or String(t["kind"]) in ["village", "hamlet"]:
				continue
			if absi(int(t["x"]) - x) <= 5 and absi(int(t["y"]) - y) <= 5:
				lonely = false
				break
		if lonely:
			big.append(s)
	## Westmost first and eastmost last: the corner-aligned and naive mappings
	## differ most at the edges (≈ half a cell, ~10 px at 8192 on this grid) and
	## least mid-map, so the edges are where registration is tested hardest.
	big.sort_custom(func(p: Dictionary, q: Dictionary) -> bool: return int(p["x"]) < int(q["x"]))
	_ok(big.size() >= 2, "two isolated big settlements to register against (%d found)" % big.size())
	## Settlements only: on this world a province label crosses the eastern
	## capital's pin, and its dark glyph outline inside the ring misled the walk.
	var reg := EXPORT_OVERLAY.new()
	add_child(reg)
	var why: String = reg.setup(ov, W, H, {"settlements": true, "settlement_tier": "town"})
	_ok(why == "", "setup at %d px: %s" % [W, why if why != "" else "ok"])
	var first_px := Vector2(-1, -1)
	if big.size() >= 2 and why == "":
		for s: Dictionary in [big[0], big[big.size() - 1]]:
			var want: Vector2 = EXPORT_OVERLAY.texture_to_export(Vector2(int(s["x"]), int(s["y"])) + Vector2(0.5, 0.5), lay)
			var naive := (Vector2(int(s["x"]), int(s["y"])) + Vector2(0.5, 0.5)) * float(W) / float(gw)
			var tx := int(round(want.x)) - 128
			var ty := int(round(want.y)) - 128
			var rgba: PackedByteArray = await reg.render_tile(tx, ty, 256, 256)
			_ok(rgba.size() == 256 * 256 * 4, "a 256x256 tile reads back")
			if rgba.size() != 256 * 256 * 4:
				continue
			var c := _pin_centre(rgba, 256, 256, 120) + Vector2(tx, ty)
			print("  %s (%d,%d) %s: drawn at %s, bake_rect pixel %s, naive %s" % [s.get("name", "?"), s["x"], s["y"], s["kind"], c, want, naive])
			_ok(c.x >= float(tx) and c.distance_to(want) < 1.5, "%s lands within 1.5 px of its bake_rect pixel (%.2f)" % [s.get("name", "?"), c.distance_to(want)])
			_ok(want.distance_to(naive) > 6.0 and c.distance_to(naive) > want.distance_to(naive) - 1.5,
				"...where the naive camera would be %.2f px off, it is %.2f px from that" % [want.distance_to(naive), c.distance_to(naive)])
			if first_px.x < 0.0:
				first_px = want
	reg.queue_free()

	print("\n== 4. uniform magnification ==")
	var ex := EXPORT_OVERLAY.new()
	add_child(ex)
	ex.setup(ov, W, H, {"settlements": true, "settlement_tier": "town", "ways": true, "labels": true, "icons": true})
	if first_px.x >= 0.0:
		var W1 := 4096
		var H1 := int(round(float(W1) * gh / gw))
		var ex1 := EXPORT_OVERLAY.new()
		add_child(ex1)
		ex1.setup(ov, W1, H1, {"settlements": true, "settlement_tier": "town", "ways": true, "labels": true, "icons": true})
		var r := Vector2i(int(first_px.x * 0.5) - 192, int(first_px.y * 0.5) - 192)
		var t1: PackedByteArray = await ex1.render_tile(r.x, r.y, 384, 384)
		var t2: PackedByteArray = await ex.render_tile(r.x * 2, r.y * 2, 768, 768)
		var c1 := _covered(t1)
		var c2 := _covered(t2)
		var ratio := float(c2) / maxf(1.0, float(c1))
		print("  region at 4096: %d px covered; at 8192: %d px; ratio %.3f (gain^2 = %.3f)" % [c1, c2, ratio,
			pow(float(ex.overlay.get("_sym_gain")) / float(ex1.overlay.get("_sym_gain")), 2.0)])
		_ok(c1 > 500, "the region holds overlay to measure (%d px)" % c1)
		_ok(ratio > 3.4 and ratio < 4.6, "twice the export width covers ~4x the pixels: symbols magnify with the map (%.3f)" % ratio)
		## Negative control: the 8192 copy with the 4096 copy's symbol gain and way scale.
		ex.overlay.set("_sym_gain", float(ex1.overlay.get("_sym_gain")))
		ex.overlay.set("_way_scale", float(ex1.overlay.get("_way_scale")))
		var t3: PackedByteArray = await ex.render_tile(r.x * 2, r.y * 2, 768, 768)
		var ratio3 := float(_covered(t3)) / maxf(1.0, float(c1))
		print("  negative control, gain withheld: ratio %.3f" % ratio3)
		_ok(ratio3 < ratio - 0.3, "withholding the symbol gain covers measurably less (%.3f < %.3f)" % [ratio3, ratio])
		ex1.queue_free()
	ex.queue_free()

	print("\n== 5. one tile through the session at 2048 px ==")
	var wg = bridge.world_gen
	var W2 := 2048
	var content := {"settlements": true, "settlement_tier": "town", "ways": true, "labels": true, "icons": true}
	var base_path := dir.path_join("terrain.png")
	var comp_path := dir.path_join("composite.png")
	var rb: Dictionary = wg.export_image(base_path, {"width": W2})
	_ok(bool(rb.get("ok", false)), "terrain-only export_image: %s" % rb.get("error", "ok"))
	var r0: Dictionary = wg.export_session_begin(comp_path, {"width": W2, "content": content})
	_ok(bool(r0.get("ok", false)) and int(r0.get("bands", 0)) == 1, "the session opens as one band: %s" % r0)
	var H2 := int(r0.get("height", 0))
	var ex2 := EXPORT_OVERLAY.new()
	add_child(ex2)
	var why2: String = ex2.setup(ov, W2, H2, content)
	var tile: PackedByteArray = await ex2.render_tile(0, 0, W2, H2) if why2 == "" else PackedByteArray()
	_ok(tile.size() == W2 * H2 * 4, "the whole 2048 px overlay is one tile (%d bytes)" % tile.size())
	var sub: Dictionary = wg.export_session_submit_tile(0, 0, 0, W2, H2, tile)
	_ok(bool(sub.get("ok", false)) and bool(sub.get("band_complete", false)), "the tile completes band 0: %s" % sub)
	var fin: Dictionary = wg.export_session_finish()
	_ok(bool(fin.get("ok", false)), "the session finishes: %s" % fin.get("error", "ok"))
	var base := Image.load_from_file(base_path)
	var comp := Image.load_from_file(comp_path)
	if base != null and comp != null and tile.size() == W2 * H2 * 4:
		base.convert(Image.FORMAT_RGB8)
		comp.convert(Image.FORMAT_RGB8)
		var bb := base.get_data()
		var cc := comp.get_data()
		var clear_bad := 0
		var solid_bad := 0
		var solid := 0
		var clear := 0
		for p in W2 * H2:
			var al := tile[p * 4 + 3]
			if al == 0:
				clear += 1
				if cc[p * 3] != bb[p * 3] or cc[p * 3 + 1] != bb[p * 3 + 1] or cc[p * 3 + 2] != bb[p * 3 + 2]:
					clear_bad += 1
			elif al == 255:
				solid += 1
				if cc[p * 3] != tile[p * 4] or cc[p * 3 + 1] != tile[p * 4 + 1] or cc[p * 3 + 2] != tile[p * 4 + 2]:
					solid_bad += 1
		print("  %d clear px, %d opaque px" % [clear, solid])
		_ok(solid > 2000, "the overlay covers something (%d opaque px)" % solid)
		_ok(clear_bad == 0, "every pixel the overlay leaves clear is the terrain export's (%d differ)" % clear_bad)
		_ok(solid_bad == 0, "every opaque overlay pixel is the overlay's colour (%d differ)" % solid_bad)
	else:
		_ok(false, "both exports load back")
	print("  composite: %s" % comp_path)

	print("\n== 6. the copy is the live overlay, pixel for pixel ==")
	ov.set_camera_zoom(1.0)
	await _frames(4)
	var vsize := Vector2i(int(ov.size.x), int(ov.size.y))
	var cp: Control = OVERLAY.new()
	cp.copy_state_from(ov)
	var vp_cp := _host(cp, vsize)
	cp.size = ov.size
	print("  live before: size %s rect %s visible %s" % [ov.size, ov.displayed_rect(), ov.is_visible_in_tree()])
	var vp_live := _host(ov, vsize)
	ov.size = cp.size
	await _frames(2)
	print("  live after: size %s rect %s visible %s" % [ov.size, ov.displayed_rect(), ov.is_visible_in_tree()])
	ov.queue_redraw()
	cp.queue_redraw()
	for _i in 3:
		await RenderingServer.frame_post_draw
	var a := _grab(vp_live)
	var b := _grab(vp_cp)
	var d := _diff(a.get_data(), b.get_data())
	print("  live %d px covered, copy %d, differing %d (%dx%d)" % [d[1], d[2], d[0], vsize.x, vsize.y])
	_ok(int(d[1]) > 2000, "the live overlay drew something to compare (%d px)" % d[1])
	_ok(int(d[0]) == 0, "the copy differs from the live overlay by 0 pixels (%d)" % d[0])
	cp.set("_way_scale", float(cp.get("_way_scale")) * 2.0)
	cp.queue_redraw()
	for _i in 3:
		await RenderingServer.frame_post_draw
	var d2 := _diff(a.get_data(), _grab(vp_cp).get_data())
	_ok(int(d2[0]) > 100, "negative control: doubling the copy's way scale moves pixels (%d)" % d2[0])
	vp_cp.queue_free()

	print("\n== %s: %d failure(s) ==" % ["PASS" if fails == 0 else "FAIL", fails])
	get_tree().quit(0 if fails == 0 else 1)
