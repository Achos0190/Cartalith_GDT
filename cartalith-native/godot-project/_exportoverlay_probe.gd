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
##  7. Tile seams (batch C): a 1024 px region at 8192 with every overlay and the
##     borders, drawn as one tile and as six cut tiles stitched; then town plans
##     (forced readable by a narrow map width on the copy) cut through a town's
##     centre. Each asserts ink on the seams, then agreement -- exact or inside
##     `_segcull_probe`'s AA tolerance, printed which.
##  8. `ExportOverlay.run` end to end at 4096 (batches C and D): one tile vs
##     1000 px tiles, file against file; borders off vs on; a cancel removes the
##     file and aborts the session; the longest frame during the runs, printed.
##  9. The `SubViewport` readback ceiling behind `TILE_MAX` on this machine:
##     4096 and 8192 px square tiles (printed; `TILE_MAX` asserted).
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

## `_segcull_probe.gd`'s measured antialiasing tolerance, referenced, not
## retyped (`_cull_probe.gd` does the same): Godot's antialiased line rasteriser
## can move a pixel by 1/255 when the same visible geometry arrives as a
## different-length polyline, which per-segment culling does at a tile edge.
const _SegCullProbe := preload("res://_segcull_probe.gd")

## Pixels allowed to differ by more than the AA delta between two tilings, if
## each stands alone (no other such pixel among its 8 neighbours). The
## suspected cause is a polygon or un-antialiased stroke edge within float error
## of a pixel centre, covered or not as the tile's origin and size move the f32
## vertex arithmetic -- **a suspicion, not a measurement**: §7's origin control
## (one tile shifted by (37, 23) px, no seam) reproduces the AA class but did
## not reproduce the one such pixel seen on 2026-10-13. A seam defect -- half a
## pin, a label cut at the edge, the 13 px neatline column the integer clip
## removed -- is a connected run, which `isolated` refuses whatever the count.
## **Labelled judgement**: 4, against that 1 pixel.
const MAX_ISOLATED_TIES := 4

## `{diff, max_delta, aa, big, isolated}` over two equal-size RGBA8 (or RGB8,
## `stride` 3) buffers `w` pixels wide: differing pixels, the largest channel
## difference, how many differ by at most the AA delta, how many by more, and
## whether every one of the latter has no such neighbour (checked only when `w`
## is given; `true` otherwise).
func _stats(a: PackedByteArray, b: PackedByteArray, stride: int = 4, w: int = 0) -> Dictionary:
	var d := 0
	var md := 0
	var aa := 0
	var bigs := {}
	var big_n := 0
	var n := mini(a.size(), b.size()) / stride
	for p in n:
		var i := p * stride
		var e := 0
		for c in stride:
			e = maxi(e, absi(int(a[i + c]) - int(b[i + c])))
		if e > 0:
			d += 1
			md = maxi(md, e)
			if e <= _SegCullProbe.AA_AGREEMENT_MAX_DELTA:
				aa += 1
			else:
				big_n += 1
				if w > 0:
					bigs[p] = true
	var isolated := true
	if w > 0:
		for p: int in bigs:
			for dy in [-1, 0, 1]:
				for dx in [-1, 0, 1]:
					if (dx != 0 or dy != 0) and bigs.has(p + dy * w + dx):
						isolated = false
	return {"diff": d, "max_delta": md, "aa": aa, "big": big_n, "isolated": isolated}

## Two tilings agree: the AA-class pixels inside `_segcull_probe`'s tolerance
## (fraction of `px`), and at most `MAX_ISOLATED_TIES` larger differences, each
## a lone pixel.
func _within_aa(s: Dictionary, px: int) -> bool:
	return float(s["aa"]) / float(px) <= _SegCullProbe.AA_AGREEMENT_MAX_DIFF_FRACTION \
		and int(s["big"]) <= MAX_ISOLATED_TIES and bool(s["isolated"])

## One `r x r` tile at `(rx, ry)` against the same pixels cut out of one larger
## tile whose origin is `(37, 23)` px up and left: no seam crosses the region,
## only the tile origin moves. The control for what an origin alone changes.
func _origin_control(ex: Node, rx: int, ry: int, r: int, single: PackedByteArray) -> Dictionary:
	var big: PackedByteArray = await ex.render_tile(rx - 37, ry - 23, r + 64, r + 64)
	if big.size() != (r + 64) * (r + 64) * 4:
		return {}
	var crop := PackedByteArray()
	crop.resize(r * r * 4)
	for y in r:
		for k in r * 4:
			crop[y * r * 4 + k] = big[((y + 23) * (r + 64) + 37) * 4 + k]
	return _stats(single, crop, 4, r)

## The `r x r` region at `(rx, ry)` drawn by `ex` as one tile and as tiles cut at
## `cx` (columns, absolute) and `cy` (rows), stitched. Returns `[single, stitched]`.
func _single_and_stitched(ex: Node, rx: int, ry: int, r: int, cx: Array, cy: Array) -> Array:
	var single: PackedByteArray = await ex.render_tile(rx, ry, r, r)
	var stitched := PackedByteArray()
	stitched.resize(r * r * 4)
	var xs := [rx] + cx + [rx + r]
	var ys := [ry] + cy + [ry + r]
	for j in ys.size() - 1:
		for i in xs.size() - 1:
			var tw: int = xs[i + 1] - xs[i]
			var th: int = ys[j + 1] - ys[j]
			var t: PackedByteArray = await ex.render_tile(xs[i], ys[j], tw, th)
			if t.size() != tw * th * 4:
				return [single, PackedByteArray()]
			for row in th:
				var dst: int = ((ys[j] - ry + row) * r + (xs[i] - rx)) * 4
				for k in tw * 4:
					stitched[dst + k] = t[row * tw * 4 + k]
	return [single, stitched]

## Prints up to 8 pixels where `a` and `b` (r x r RGBA8 at `(rx, ry)`) differ by
## more than the AA delta, each with its distance to the nearest seam -- the
## evidence for what a residual is, printed rather than summarised.
func _describe(a: PackedByteArray, b: PackedByteArray, r: int, rx: int, ry: int, cx: Array, cy: Array) -> void:
	var shown := 0
	for i in range(0, mini(a.size(), b.size()), 4):
		var md := 0
		for c in 4:
			md = maxi(md, absi(int(a[i + c]) - int(b[i + c])))
		if md <= _SegCullProbe.AA_AGREEMENT_MAX_DELTA:
			continue
		var x := rx + (i / 4) % r
		var y := ry + (i / 4) / r
		var dist := 1 << 30
		for c in cx:
			dist = mini(dist, absi(x - int(c)))
		for c in cy:
			dist = mini(dist, absi(y - int(c)))
		print("    (%d,%d) %d px from a seam: one tile %s, stitched %s" % [x, y, dist,
			[a[i], a[i + 1], a[i + 2], a[i + 3]], [b[i], b[i + 1], b[i + 2], b[i + 3]]])
		shown += 1
		if shown >= 8:
			return

## Covered pixels (alpha > 0) within one pixel of the seams, in the single render:
## the premise that the seams cut through drawn content at all.
func _ink_on_seams(single: PackedByteArray, r: int, rx: int, ry: int, cx: Array, cy: Array) -> int:
	var n := 0
	for y in r:
		for x in r:
			var near := false
			for c in cx:
				near = near or absi(rx + x - int(c)) <= 1
			for c in cy:
				near = near or absi(ry + y - int(c)) <= 1
			if near and single[(y * r + x) * 4 + 3] > 0:
				n += 1
	return n

## §7. Tile seams (E4 batch C): the same region, one tile against tiles cut
## through it and stitched. (a) the full content at 8192 around a capital --
## labels, pins, ways, icons, borders; (b) town plans, forced readable by giving
## the copy a narrow map width, with the cut through a town's centre. Each case
## asserts the seams cross ink (else the comparison is vacuous) and that the
## stitch agrees -- exactly, or inside the AA tolerance, and says which.
func _seams(app: Node, ov: Control, bridge: Node, W: int, H: int, first_px: Vector2) -> void:
	print("\n== 7. tile seams ==")
	var borders := [app.viewport.territory_view, app.viewport.province_view]
	var content := {"settlements": true, "settlement_tier": "all", "ways": true, "routes": true,
		"labels": true, "icons": true, "territory": true}
	var ex := EXPORT_OVERLAY.new()
	add_child(ex)
	_ok(ex.setup(ov, W, H, content, borders) == "", "a full-content copy at %d px" % W)
	var R := 1024
	var c := first_px if first_px.x >= 0.0 else Vector2(W / 2, H / 2)
	var rx := clampi(int(c.x) - R / 2, 0, W - R)
	var ry := clampi(int(c.y) - R / 2, 0, H - R)
	var cx := [rx + 389, rx + 702]
	var cy := [ry + 611]
	var pair: Array = await _single_and_stitched(ex, rx, ry, R, cx, cy)
	_ok(pair[0].size() == R * R * 4 and pair[1].size() == R * R * 4, "the region reads back whole and in 6 tiles")
	if pair[1].size() == R * R * 4:
		var ink := _ink_on_seams(pair[0], R, rx, ry, cx, cy)
		var s := _stats(pair[0], pair[1], 4, R)
		print("  region (%d,%d) %dx%d cut at x %s, y %s: %d inked px on the seams; stitched vs single: %s"
			% [rx, ry, R, R, cx, cy, ink, s])
		_describe(pair[0], pair[1], R, rx, ry, cx, cy)
		print("  shifted origin, no seam: %s" % await _origin_control(ex, rx, ry, R, pair[0]))
		_ok(ink > 200, "the seams cut through drawn content (%d px)" % ink)
		_ok(_within_aa(s, R * R), "full content: the stitched tiles agree with one tile (%s; %d AA px, %d lone ties)"
			% ["exact" if int(s["diff"]) == 0 else "within tolerance", s["aa"], s["big"]])
	ex.queue_free()

	## (b) Town plans. The live map width is narrowed only while the copy copies
	## it, and restored at once; the copy then draws every town as a plan (box at
	## least `EXPORT_URBAN_MIN_BOX_PX`).
	var km: float = ov.get("_map_width_km")
	ov.set("_map_width_km", 120.0)
	var eu := EXPORT_OVERLAY.new()
	add_child(eu)
	var why: String = eu.setup(ov, W, H, {"settlements": true, "settlement_tier": "town"}, [])
	ov.set("_map_width_km", km)
	_ok(why == "", "a town-plan copy at %d px" % W)
	var need: PackedInt32Array = eu.overlay.export_urban_needs()
	var fetched: String = await eu._fetch_urban(bridge)
	var layouts: Dictionary = eu.overlay.get("_urban_layouts")
	var got := 0
	var pick := -1
	for i in need:
		if layouts.get(i) != null:
			got += 1
			if pick < 0:
				pick = i
	print("  town plans: %d needed, %d laid out (%s)" % [need.size(), got, "ok" if fetched == "" else fetched])
	_ok(need.size() > 0 and got > 0, "the copy asks for town plans and the bridge lays them out (%d of %d)" % [got, need.size()])
	_ok(eu.overlay.export_urban_needs().is_empty(), "after the fetch nothing is still needed")
	if pick >= 0:
		var s0: Dictionary = (eu.overlay.get("_settlements") as Array)[pick]
		var lay: Dictionary = EXPORT_OVERLAY.layout(W, H, int(ov.get("_gw")), int(ov.get("_gh")))
		var tp: Vector2 = EXPORT_OVERLAY.texture_to_export(Vector2(int(s0["x"]), int(s0["y"])) + Vector2(0.5, 0.5), lay)
		var R2 := 512
		var ux := clampi(int(tp.x) - R2 / 2, 0, W - R2)
		var uy := clampi(int(tp.y) - R2 / 2, 0, H - R2)
		var ucx := [int(tp.x) + 3]
		var ucy := [int(tp.y) - 2]
		var up: Array = await _single_and_stitched(eu, ux, uy, R2, ucx, ucy)
		var revealed: Dictionary = eu.overlay.get("_urban_revealed")
		_ok(revealed.has(pick), "the picked town is drawn as its plan (revealed: %d towns)" % revealed.size())
		if up[1].size() == R2 * R2 * 4:
			var uink := _ink_on_seams(up[0], R2, ux, uy, ucx, ucy)
			var us := _stats(up[0], up[1], 4, R2)
			print("  town %s at %s, cut through its centre: %d inked px on the seams; stitched vs single: %s"
				% [s0.get("name", "?"), tp, uink, us])
			_describe(up[0], up[1], R2, ux, uy, ucx, ucy)
			print("  shifted origin, no seam: %s" % await _origin_control(eu, ux, uy, R2, up[0]))
			_ok(uink > 100, "the seams cut through the town plan (%d px)" % uink)
			_ok(_within_aa(us, R2 * R2), "town plans: the stitched tiles agree with one tile (%s; %d AA px, %d lone ties)"
				% ["exact" if int(us["diff"]) == 0 else "within tolerance", us["aa"], us["big"]])
	eu.queue_free()

	## (c) Borders alone, against the grid: every pixel's alpha must be the
	## territory texture's alpha at the cell whose footprint holds the pixel
	## (`ExportOverlay.BORDER_SHADER`'s rule, computed here in f64), except the
	## pixels within 1e-3 px of a cell edge, where f32 on the GPU may round the
	## other way (counted and printed). Content is `territory` only, so the
	## overlay itself draws nothing.
	var tv: TextureRect = app.viewport.territory_view
	var timg: Image = tv.texture.get_image() if tv.texture != null else null
	var eb := EXPORT_OVERLAY.new()
	add_child(eb)
	_ok(eb.setup(ov, W, H, {"territory": true}, [tv]) == "", "a borders-only copy")
	var bx := W / 3 + 7
	var by := H / 3 + 5
	var bt: PackedByteArray = await eb.render_tile(bx, by, 512, 512)
	if timg != null and bt.size() == 512 * 512 * 4:
		timg.convert(Image.FORMAT_RGBA8)
		var td := timg.get_data()
		var tw := timg.get_width()
		var q: Vector2 = EXPORT_OVERLAY.layout(W, H, int(ov.get("_gw")), int(ov.get("_gh")))["q"]
		var bad := 0
		var ties := 0
		var lit := 0
		for y in 512:
			var fy := (by + y + 0.5) / q.y + 0.5
			var cyi := int(floor(fy))
			var yt := absf(fy - round(fy)) * q.y < 1e-3
			for x in 512:
				var fx := (bx + x + 0.5) / q.x + 0.5
				var cxi := int(floor(fx))
				var want := td[(cyi * tw + cxi) * 4 + 3]
				var galpha := bt[(y * 512 + x) * 4 + 3]
				if galpha > 0:
					lit += 1
				if absi(int(galpha) - int(round(float(want) * tv.modulate.a))) > 1:
					if yt or absf(fx - round(fx)) * q.x < 1e-3:
						ties += 1
					else:
						bad += 1
		print("  borders vs the grid: %d px with wash, %d off by more than 1 level, %d at a cell-edge tie" % [lit, bad, ties])
		_ok(lit > 1000, "the region carries territory wash (%d px)" % lit)
		_ok(bad == 0, "every border pixel is its cell's texel (%d differ)" % bad)
	else:
		_ok(false, "the territory texture and a borders tile read back")
	eb.queue_free()

## Records the longest gap between two process frames while `on` is true.
class _FrameWatch extends Node:
	var on := false
	var worst_ms := 0.0
	var over_50 := 0
	var frames := 0
	var _last := 0
	func _process(_d: float) -> void:
		var now := Time.get_ticks_usec()
		if on and _last > 0:
			var ms := (now - _last) / 1000.0
			worst_ms = maxf(worst_ms, ms)
			frames += 1
			if ms > 50.0:
				over_50 += 1
		_last = now

## §8. `ExportOverlay.run` end to end at 4096 px (E4 batches C and D): one tile
## against 1000 px tiles (12 of them), whose output files must agree as the
## stitched region did; borders off against on (the territory wash must reach
## the file); a cancel mid-run must remove the partial file; and the longest
## frame while a run is in progress is reported (Ruling AZ: no UI freeze).
func _run_end_to_end(app: Node, ov: Control, bridge: Node, dir: String) -> void:
	print("\n== 8. run(): whole exports through the session ==")
	var borders := [app.viewport.territory_view, app.viewport.province_view]
	_ok(app.viewport.territory_view.texture != null, "premise: the live map has a territory texture")
	var content := {"settlements": true, "settlement_tier": "town", "ways": true, "labels": true,
		"icons": true, "territory": true}
	var runner := EXPORT_OVERLAY.new()
	add_child(runner)
	var watch := _FrameWatch.new()
	add_child(watch)
	var paths := [dir.path_join("run_one_tile.png"), dir.path_join("run_tiled.png"), dir.path_join("run_no_borders.png")]
	var results := []
	for k in 3:
		var c2 := content.duplicate()
		if k == 2:
			c2["territory"] = false
		watch.on = true
		var t0 := Time.get_ticks_msec()
		var r: Dictionary = await runner.run(bridge, paths[k], {"width": 4096, "content": c2}, ov, borders, 1000 if k == 1 else 4096)
		watch.on = false
		print("  run %d (%s): %s in %d ms" % [k, ["one tile", "1000 px tiles", "no borders"][k],
			"ok %d bands %d bytes" % [r.get("bands", 0), r.get("bytes", 0)] if r.get("ok", false) else r.get("error", "?"),
			Time.get_ticks_msec() - t0])
		print("    longest main-thread step of each kind (ms): %s" % runner.step_ms)
		_ok(bool(r.get("ok", false)), "run %d writes its file" % k)
		results.append(r)
	print("  longest frame during the three runs: %.1f ms over %d frames (%d over 50 ms)" % [watch.worst_ms, watch.frames, watch.over_50])
	var imgs := []
	for p in paths:
		var im := Image.load_from_file(p)
		if im != null:
			im.convert(Image.FORMAT_RGB8)
		imgs.append(im)
	if imgs[0] != null and imgs[1] != null and imgs[2] != null:
		var px: int = imgs[0].get_width() * imgs[0].get_height()
		var s := _stats(imgs[0].get_data(), imgs[1].get_data(), 3, imgs[0].get_width())
		print("  one tile vs 1000 px tiles (of %d px): %s" % [px, s])
		_ok(imgs[0].get_size() == imgs[1].get_size() and _within_aa(s, px),
			"the tiled export is the one-tile export (%s; %d AA px, %d lone ties)"
			% ["exact" if int(s["diff"]) == 0 else "within tolerance", s["aa"], s["big"]])
		var sb := _stats(imgs[0].get_data(), imgs[2].get_data(), 3)
		print("  borders on vs off: %d px differ" % sb["diff"])
		_ok(int(sb["diff"]) > px / 50, "the territory wash reaches the file (%d px move)" % sb["diff"])
	else:
		_ok(false, "the three run outputs load back")
	## Cancel: stop the run after its first band's poll, and the file must go.
	var cancel_path := dir.path_join("run_cancelled.png")
	runner.progress.connect(func(_f: float, _t: String) -> void: runner.cancel(), CONNECT_ONE_SHOT)
	var rc: Dictionary = await runner.run(bridge, cancel_path, {"width": 4096, "content": content}, ov, borders)
	_ok(not bool(rc.get("ok", true)) and bool(rc.get("cancelled", false)), "a cancelled run reports it: %s" % rc)
	_ok(not FileAccess.file_exists(cancel_path), "a cancelled run leaves no file")
	_ok(String(bridge.world_gen.export_session_state()) == "aborted", "the session is aborted, not left open")
	runner.queue_free()
	watch.queue_free()

## §9. The `SubViewport` ceiling behind `TILE_MAX`, measured on this machine:
## square tiles of 4096 and 8192 px read back (or not), and how long a tile
## takes. A measurement, printed; the one assertion is that `TILE_MAX` itself
## reads back. A 16384 px square also read back on 2026-10-13 (276-282 ms); it is
## left out of the default run as a 1 GB render target for one printed number.
func _tile_ceiling(ov: Control) -> void:
	print("\n== 9. tile size ceiling (this machine) ==")
	var gw := int(ov.get("_gw"))
	var gh := int(ov.get("_gh"))
	var W := 32768
	var H := int(round(float(W) * gh / gw))
	var ex := EXPORT_OVERLAY.new()
	add_child(ex)
	ex.setup(ov, W, H, {"settlements": true, "settlement_tier": "town", "ways": true, "labels": true})
	for side: int in [4096, 8192]:
		var t0 := Time.get_ticks_msec()
		var t: PackedByteArray = await ex.render_tile(0, 0, side, side, 16384)
		var ok: bool = t.size() == side * side * 4
		print("  %5d px square: %s in %d ms" % [side, "reads back" if ok else "NULL / wrong size", Time.get_ticks_msec() - t0])
		if side == EXPORT_OVERLAY.TILE_MAX:
			_ok(ok, "a TILE_MAX (%d px) tile reads back" % side)
	ex.queue_free()

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

	await _seams(app, ov, bridge, W, H, first_px)
	await _run_end_to_end(app, ov, bridge, dir)
	await _tile_ceiling(ov)

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
