extends Node
## Probe for the owner's 2026-09-22 river ruling: "keep the smoothline and use
## that to render the river ... ditch the texture bake", "the line should get
## the same look as a lake. And its width scale with the zoom".
##
## Asserts, on a real 2048 x 1312 generated world through the real shell:
##   A. the rivers layer is on by default and the tint toggle is gone
##   B. with the Rivers layer OFF the BASE texture carries no river ink --
##      measured against the grid-resolution raster export, which carries
##      rivers and so is the positive control: the export must be much
##      bluer than the screen at channel cells, and identical to it elsewhere.
##      (Until RV-5, 2026-09-28, the export's rivers were the stamped
##      `river_ink()`; since then they are the vector strokes rasterized at
##      the export's resolution -- still river, still the positive control.)
##      (Since 2026-09-27 the base texture holds no river whatever the layer
##      says: the base view's stroke is textured with `river_color_texture()`,
##      which `_rivstyle_probe.gd` measures. This section takes the texture
##      with the layer off, the state "the stamped ink is not baked" is about.)
##   C. the strokes: flip the layer and diff the framebuffer; the stroke width
##      measured on screen equals `widths[i] * px_per_cell` (RV-2's per-point
##      width; `width_cells` until 2026-09-29) at two zooms (so
##      it scales with zoom), its centre pixel is the run's own Strahler
##      `colors` entry at that render point, and
##      every on-screen centreline sample lies in ONE connected component
##      (no D8 breaks)
##   D. the LOD tiles bake no river either: with the layer OFF at deep zoom,
##      the channel centreline's blue shift (centre minus banks) is the
##      valley's own -- no more than the river-free map texture's shift at the
##      same ground points plus 4 levels -- while with the layer ON it carries
##      the ink (the leg's positive control)
##
## MUST run WINDOWED (`MISTAKES.md`, "Run a pixel probe"):
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 \
##       _riverstroke_probe.tscn -- --out C:/some/scratch/dir

var _out := "user://riverstroke/"
var _fails := 0


func _ok(cond: bool, what: String) -> void:
	print(("  PASS  " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless -- run windowed.")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	for a in args:
		if a.begins_with("--") and a != "--out":
			printerr("PROBE-CANNOT-RUN: unknown argument %s (only --out DIR)" % a)
			get_tree().quit(2)
			return
	var i := args.find("--out")
	if i >= 0 and i + 1 < args.size():
		_out = args[i + 1]
		if not _out.ends_with("/"):
			_out += "/"
	DirAccess.make_dir_recursive_absolute(_out)

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var vh: Control = app.viewport
	var br: Node = app.bridge
	if vh == null or br == null:
		printerr("PROBE-CANNOT-RUN: shell exposed no viewport/bridge.")
		get_tree().quit(2)
		return
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await get_tree().process_frame

	print("generating 2048 x 1312...")
	br.generate({
		"seed": 20260824, "width_km": 1200.0, "grid_w": 2048, "grid_h": 1312,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	var spins := 0
	while br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	if br.generating:
		printerr("PROBE-CANNOT-RUN: generation never finished.")
		get_tree().quit(2)
		return
	await get_tree().create_timer(1.0).timeout
	vh.reset_view()
	vh.set_debug_layer("off")
	await _settle(5)
	var g: Vector2i = br.grid_size()
	print("grid %dx%d" % [g.x, g.y])

	# -- A ---------------------------------------------------------------------
	print("\n== A. defaults ==")
	_ok(vh.layer_visible("rivers"), "the rivers layer is on by default")
	_ok(not br.world_gen.has_method("set_suppress_river_tint"), "set_suppress_river_tint is gone from WorldGen")
	var rivers: Array = br.rivers(1)
	var with_w := 0
	var with_c := 0
	for r: Dictionary in rivers:
		with_w += int(r.has("width_cells"))
		with_c += int((r.get("colors", PackedColorArray()) as PackedColorArray).size() == (r["render_points"] as PackedVector2Array).size())
	print("  %d runs, %d with width_cells, %d with color" % [rivers.size(), with_w, with_c])
	_ok(rivers.size() > 100 and with_w > 0 and with_c == rivers.size(), "every run carries one colour per render point, and widths exist")

	# -- B ---------------------------------------------------------------------
	print("\n== B. rivers layer OFF: the base texture bakes no river ink (export = positive control) ==")
	vh.set_layer_visible("rivers", false)
	var scr: Image = br.color_texture().get_image()
	scr.convert(Image.FORMAT_RGB8)
	var ep := _out + "export_gridres.png"
	var rex: Dictionary = br.world_gen.export_raster_png(ProjectSettings.globalize_path(ep), 2048, false)
	var exp_img := Image.new()
	if not bool(rex.get("ok", false)) or exp_img.load(ProjectSettings.globalize_path(ep)) != OK:
		_ok(false, "grid-resolution export: %s" % str(rex))
	else:
		exp_img.convert(Image.FORMAT_RGB8)
		_ok(scr.get_size() == exp_img.get_size(), "screen texture and export are the same size (%s)" % str(scr.get_size()))
		var chan_shift: Array[float] = []
		var ctrl_diff: Array[float] = []
		var n_samp := 0
		var wl: Array[float] = []
		for r: Dictionary in rivers:
			wl.append(float(r.get("width_cells", 0.0)))
		wl.sort()
		print("  width_cells over %d runs: min %.2f median %.2f p90 %.2f max %.2f"
			% [wl.size(), wl[0], wl[wl.size() / 2], wl[int(wl.size() * 0.9)], wl[wl.size() - 1]])
		var w_bar: float = wl[int(wl.size() * 0.9)]
		for r: Dictionary in rivers:
			if float(r.get("width_cells", 0.0)) < w_bar:
				continue
			var pts: PackedVector2Array = r["points"]
			for k in range(0, pts.size(), 4):
				var c := Vector2i(pts[k])
				var ctl := c + Vector2i(41, 29)
				if c.x < 8 or c.y < 8 or c.x >= g.x - 8 or c.y >= g.y - 8 or ctl.x >= g.x - 8 or ctl.y >= g.y - 8:
					continue
				if String(br.sample_cell(c.x, c.y).get("water", "land")) != "land":
					continue
				var s := scr.get_pixelv(c)
				var e := exp_img.get_pixelv(c)
				chan_shift.append(((e.b - e.r) - (s.b - s.r)) * 255.0)
				var s2 := scr.get_pixelv(ctl)
				var e2 := exp_img.get_pixelv(ctl)
				ctrl_diff.append((absf(s2.r - e2.r) + absf(s2.g - e2.g) + absf(s2.b - e2.b)) * 255.0)
				n_samp += 1
				if n_samp >= 3000:
					break
			if n_samp >= 3000:
				break
		chan_shift.sort()
		ctrl_diff.sort()
		var med_shift: float = chan_shift[chan_shift.size() / 2] if chan_shift.size() > 0 else 0.0
		var med_ctrl: float = ctrl_diff[ctrl_diff.size() / 2] if ctrl_diff.size() > 0 else 99.0
		print("  %d land channel cells on runs >= %.2f cells wide (the p90)" % [n_samp, w_bar])
		print("  median blue shift export-minus-screen at channel cells: %.1f levels" % med_shift)
		print("  median |export - screen| at control cells (+41,+29): %.2f levels" % med_ctrl)
		_ok(n_samp >= 200, "enough channel cells sampled")
		_ok(med_shift > 40.0, "positive control: the export's river ink moves those cells a lot")
		_ok(med_ctrl < 1.5, "and the two paths agree away from rivers, so the shift IS the ink the screen no longer bakes")
		B_SHIFT = med_shift
	vh.set_layer_visible("rivers", true)

	# -- C ---------------------------------------------------------------------
	print("\n== C. the stroke: width on the ground, Strahler colour, no breaks ==")
	var best: Dictionary = {}
	for r: Dictionary in rivers:
		## One piece (`pieces` a single pair): a run cut at a lake is several
		## strokes, and the connectivity check below is about ONE stroke.
		if int(r.get("order", 0)) >= 3 and r.has("widths") and (r["pieces"] as PackedInt32Array).size() == 2 				and float(r.get("km", 0.0)) > float(best.get("km", 0.0)):
			best = r
	if best.is_empty():
		_ok(false, "an order>=3 run with a width exists")
	else:
		var rp: PackedVector2Array = best["render_points"]
		var mid_i := rp.size() / 2
		var mid: Vector2 = rp[mid_i]
		## RV-2: the stroke's width at THIS point (`widths`), not the run's one
		## mouth reading `width_cells` it used to be drawn at.
		var wc: float = float((best["widths"] as PackedFloat32Array)[mid_i])
		print("  run: order %d, %.1f km, width at mid %.2f cells, %d render points, mid %s"
			% [int(best["order"]), float(best["km"]), wc, rp.size(), str(mid)])
		var ov: Control = vh.overlay
		var ppc1: float = ov._displayed_rect().size.x / float(g.x)   ## control px per cell at zoom 1
		var z1: float = clampf(7.0 / (wc * ppc1), 1.0, vh._zoom_max / 2.0)
		var widths: Array[float] = []
		var wants: Array[float] = []
		var mags: Array[float] = []
		## The styled references (2026-09-27): the map WITHOUT rivers, and the
		## same map with every river composited at full coverage in its styled
		## colour -- what the drawn river's colour must be, since the style
		## reaches it now.
		var t0_img: Image = (vh.map_view.texture as Texture2D).get_image()
		var t1_tex: Texture2D = br.river_color_texture()
		var t1_img: Image = t1_tex.get_image() if t1_tex != null else null
		if t0_img != null:
			t0_img.convert(Image.FORMAT_RGB8)
		if t1_img != null:
			t1_img.convert(Image.FORMAT_RGB8)
		_ok(t0_img != null and t1_img != null, "the map texture and the river colour texture are readable")
		for z in [z1, z1 * 2.0]:
			vh.move_view_to(mid.x, mid.y)
			await _settle(3)
			vh.zoom_step(z / vh.zoom())
			vh.move_view_to(mid.x, mid.y)
			## Since 2026-09-27 the rivers are IN the deep-zoom tiles, so a
			## capture must wait for the pyramid, not for a frame count: a tile
			## still arriving shows the base map's one-texel-per-cell river.
			await _settle_lod(vh)
			## The overlay hidden for the two captures: the rivers are in the
			## MAP since 2026-09-27, under the overlay's labels, and a region
			## label lying across the river is identical on and off -- it read
			## as a break (708 of 809 at z10, every sample south of the label's
			## band). Its geometry helpers below still work while hidden.
			vh.overlay.visible = false
			vh.set_layer_visible("rivers", true)
			await _settle_lod(vh)
			var on := get_viewport().get_texture().get_image()
			vh.set_layer_visible("rivers", false)
			await _settle_lod(vh)
			var off := get_viewport().get_texture().get_image()
			vh.set_layer_visible("rivers", true)
			vh.overlay.visible = true
			await _settle_lod(vh)
			on.save_png("%sC_zoom%.1f_on.png" % [_out, vh.zoom()])
			off.save_png("%sC_zoom%.1f_off.png" % [_out, vh.zoom()])
			var xf: Transform2D = ov.get_global_transform_with_canvas()
			var rect: Rect2 = ov._displayed_rect()
			var p: Vector2 = xf * ov._point_to_screen(mid, rect)
			## Width = the SHORTEST chord through a centreline point across the
			## diff mask, over 36 directions -- independent of any tangent
			## estimate, which a cell-scale zigzag makes unreliable. Median of
			## 15 centreline points around the middle.
			var chords: Array[float] = []
			for kk in range(maxi(mid_i - 56, 0), mini(mid_i + 57, rp.size()), 8):
				var q: Vector2 = xf * ov._point_to_screen(rp[kk], rect)
				var best_c := 1e9
				for ai in 36:
					var d := Vector2.from_angle(PI * float(ai) / 36.0)
					best_c = minf(best_c, _run_len(on, off, q, d) + _run_len(on, off, q, -d) - 1.0)
				chords.append(best_c)
			chords.sort()
			var w_meas: float = chords[chords.size() / 2]
			var want: float = wc * rect.size.x / float(g.x) * vh.zoom()
			var ppc_screen: float = (xf * ov._point_to_screen(mid + Vector2(1, 0), rect) - p).length()
			print("  screen px per cell measured from the transform %.3f; overlay's own rect/_gw*zoom %.3f"
				% [ppc_screen, rect.size.x / float(g.x) * vh.zoom()])
			print("  chords (px) %s" % str(chords))
			widths.append(w_meas)
			wants.append(want)
			print("  zoom %.2f: measured stroke width %.1f px across the normal, expected widths[mid]*px_per_cell = %.2f px"
				% [vh.zoom(), w_meas, want])
			## The deep-zoom tile's magnification on screen: screen px per cell
			## over the drawn level's tile px per cell, from a real tile's size.
			var zl: int = br.lod_level_for_zoom(ppc_screen)
			var tt: Texture2D = br.lod_synthesize_tile(zl, 0, 0)
			var tile_ppc: float = float(tt.get_width() - 1) / (float(g.x - 1) / float(br.lod_tiles_per_axis(zl))) if tt != null else NAN
			var mag: float = ppc_screen / tile_ppc
			mags.append(mag)
			print("  LOD level %d: tile %.3f px/cell, magnification %.3f" % [zl, tile_ppc, mag])
			var cpx := on.get_pixelv(Vector2i(p.round()))
			var want_c: Color = (best["colors"] as PackedColorArray)[mid_i]
			var dc0 := maxf(absf(cpx.r - want_c.r), maxf(absf(cpx.g - want_c.g), absf(cpx.b - want_c.b))) * 255.0
			print("  centre pixel %s vs its palette colour %s: worst channel %.1f levels (reported, not a bar: the sheet prints the river now)" % [str(cpx), str(want_c), dc0])
			## Colour bar, from a control rather than a chosen tolerance: the
			## centre pixel against the river colour texture at the same ground
			## point must disagree by no more than the ground beside it
			## disagrees with the map texture at ITS point (tile-vs-map drift of
			## the same pipeline), plus RV-2's own 4 levels.
			var nrm := (rp[mini(mid_i + 2, rp.size() - 1)] - rp[maxi(mid_i - 2, 0)]).normalized()
			nrm = Vector2(-nrm.y, nrm.x)
			var bank_g: Vector2 = mid + nrm * (wc * 0.5 + 6.0 / ppc_screen)
			var bank_s: Vector2 = xf * ov._point_to_screen(bank_g, rect)
			var bpx := off.get_pixelv(Vector2i(bank_s.round()))
			var ref_r := _bilinear(t1_img, mid)
			var ref_b := _bilinear(t0_img, bank_g)
			var dc := maxf(absf(cpx.r - ref_r.r), maxf(absf(cpx.g - ref_r.g), absf(cpx.b - ref_r.b))) * 255.0
			var dbank := maxf(absf(bpx.r - ref_b.r), maxf(absf(bpx.g - ref_b.g), absf(bpx.b - ref_b.b))) * 255.0
			print("  centre vs river colour texture %.1f levels; bank vs map texture (control) %.1f levels" % [dc, dbank])
			## The bar was 4 levels while the overlay drew the palette colour
			## raw over the finished map. Since 2026-09-27 the river is IN the
			## map and the sheet reaches it (paper tint, vignette, grade) -- the
			## owner's request -- so the pixel is the palette colour as the
			## sheet prints it. `_rivstyle_probe.gd` measured that at 16 levels
			## per channel (median of 80 cells, Natural Vibrant); 30 is its bar.
			_ok(dc <= dbank + 4.0, "the stroke is the style's river colour (%.1f <= control %.1f + 4)" % [dc, dbank])
			# Connectivity of the on-screen stroke: every visible centreline sample
			# must be inside the diff mask AND in the one component flood-filled
			# from the mid point.
			var comp := _flood(on, off, Vector2i(p.round()))
			var vis := 0
			var hit := 0
			var sz := on.get_size()
			for q in rp:
				var sp: Vector2 = xf * ov._point_to_screen(q, rect)
				var si := Vector2i(sp.round())
				## Inside the MAP VIEWPORT, not merely the window: a centreline
				## point under a dock or the app bar is covered by opaque chrome
				## and moves no pixel whether the river is drawn or not, so it
				## read as a break. (Found 2026-09-27: 711/887 and 399/449 on two
				## builds that drew the stroke differently -- identical counts,
				## so they were measuring the chrome, not the river.)
				var vr := Rect2i(Vector2i(vh.global_position.round()) + Vector2i(4, 4), Vector2i(vh.size.round()) - Vector2i(8, 8))
				if not vr.has_point(si) or si.x >= sz.x - 4 or si.y >= sz.y - 4:
					continue
				## And inside the plate's neatline: the frame is drawn over the
				## map (`render.rs::apply_border`, the overlay's own neatline
				## clip), so a river running off the sheet is correctly hidden
				## under it and moves no pixel there. Found 2026-09-27 on the
				## post-GF-2 world, whose chosen trunk runs into the frame:
				## 1199 of 1342 at z6.5, every miss under the frame; HEAD's
				## overlay read 1200 of 1442 on the same frame.
				var inner: Rect2 = ov._interior_rect(rect)
				var i0: Vector2 = xf * inner.position
				var i1: Vector2 = xf * inner.end
				if sp.x < i0.x + 4.0 or sp.y < i0.y + 4.0 or sp.x > i1.x - 4.0 or sp.y > i1.y - 4.0:
					continue
				vis += 1
				if comp.has(si.y * sz.x + si.x):
					hit += 1
			print("  %d of %d on-screen centreline samples lie in the mid point's connected stroke" % [hit, vis])
			_ok(vis > 20 and hit >= int(vis * 0.98), "the drawn river is one unbroken stroke on screen")
		if widths.size() == 2 and widths[0] > 0.0:
			print("  width ratio %.2f for a zoom ratio 2.00 (a screen-constant stroke would read 1.00)" % (widths[1] / widths[0]))
			## First principles (2026-09-27). The chord counts the solid core,
			## W = widths[mid] * px_per_cell, PLUS a raster-dependent constant c:
			## the antialiasing fringe the threshold still sees on each side,
			## and the tile's bilinear reconstruction. Both are sized in TILE
			## pixels, so on screen they scale with the magnification, not the
			## zoom. The two zooms here differ by exactly 2x, so the pyramid
			## level moves by exactly one and the magnification (and LOD-D3's
			## morph) is the same at both -- measured below -- and c is the
			## same constant in both chords. So w2 - w1 = W2 - W1 exactly, and
			## that difference, not the ratio, is the bar: the ratio (2W+c)/(W+c)
			## equals 2 only when c = 0. The old overlay's c was its own 1 screen
			## px fringe; a tile's is 1 tile px times the magnification plus
			## the filter. c itself is bounded from the same principles: at most
			## two tile pixels a side (fringe + reconstruction), 4 * mag.
			var dw: float = widths[1] - widths[0]
			var dwant: float = wants[1] - wants[0]
			var c: float = 2.0 * widths[0] - widths[1]
			print("  magnification %.3f / %.3f; chord difference %.1f px vs width-on-ground difference %.2f px; raster constant c = %.1f px (bound 0..%.1f)"
				% [mags[0], mags[1], dw, dwant, c, 4.0 * maxf(mags[0], mags[1])])
			_ok(absf(mags[0] - mags[1]) < 0.01, "premise: the same magnification at both zooms")
			_ok(absf(dw - dwant) <= maxf(2.0, dwant * 0.2), "width scales with zoom: the chord grows by the ground width's own growth")
			_ok(c >= 0.0 and c <= 4.0 * maxf(mags[0], mags[1]), "the raster constant is within its first-principles bound")

		# -- D -----------------------------------------------------------------
		print("\n== D. LOD tiles bake no river ==")
		var zd: float = minf(vh._zoom_max, maxf(16.0, z1 * 4.0))
		vh.zoom_step(zd / vh.zoom())
		vh.move_view_to(mid.x, mid.y)
		## Both frames wait for the pyramid (`_settle_lod`: `lod_pending() == 0`
		## and the morph run), not a frame count: 90 frames left some tiles
		## mid-morph, and the reading moved by a level between runs (19.5 here,
		## 20.5 in the verifier's run, same DLL).
		vh.set_layer_visible("rivers", true)
		await _settle_lod(vh)
		var img_on := get_viewport().get_texture().get_image()
		vh.set_layer_visible("rivers", false)
		await _settle_lod(vh)
		var tiles: int = (vh._lod_tiles as Dictionary).size()
		print("  zoom %.2f  lod_active=%s  tiles=%d" % [vh.zoom(), vh.lod_active(), tiles])
		var img := get_viewport().get_texture().get_image()
		img.save_png("%sD_lod_rivers_off.png" % _out)
		img_on.save_png("%sD_lod_rivers_on.png" % _out)
		if not vh.lod_active() or tiles == 0:
			_ok(false, "LOD tiles are live at this zoom (cannot test D otherwise)")
		else:
			var xf2: Transform2D = ov.get_global_transform_with_canvas()
			var rect2: Rect2 = ov._displayed_rect()
			var off_px: float = wc * rect2.size.x / float(g.x) * vh.zoom() * 1.5 + 4.0
			var shifts: Array[float] = []
			var shifts_on: Array[float] = []
			var shifts_map: Array[float] = []
			var sz2 := img.get_size()
			## Screen px per cell here, to put the bank offset back in cells
			## for the map-texture control.
			var ppc2: float = rect2.size.x / float(g.x) * vh.zoom()
			var lakeish := 0
			var classes := {}
			for k in range(0, rp.size(), 2):
				var cell := Vector2i(rp[k].round())
				var wcls := String(br.sample_cell(cell.x, cell.y).get("water", "?"))
				classes[wcls] = int(classes.get(wcls, 0)) + 1
				var sp: Vector2 = xf2 * ov._point_to_screen(rp[k], rect2)
				var a: Vector2 = xf2 * ov._point_to_screen(rp[mini(k + 3, rp.size() - 1)], rect2)
				var b: Vector2 = xf2 * ov._point_to_screen(rp[maxi(k - 3, 0)], rect2)
				var n2 := Vector2(-(a - b).y, (a - b).x).normalized()
				var l := sp + n2 * off_px
				var rr := sp - n2 * off_px
				var ok_px := true
				for q in [sp, l, rr]:
					if q.x < 2 or q.y < 2 or q.x >= sz2.x - 2 or q.y >= sz2.y - 2:
						ok_px = false
				if not ok_px:
					continue
				var c0 := img.get_pixelv(Vector2i(sp.round()))
				var cl := img.get_pixelv(Vector2i(l.round()))
				var cr := img.get_pixelv(Vector2i(rr.round()))
				## A centreline cell that is lake or sea draws in the tile's OWN
				## water branch, not river ink -- counted and reported, and kept
				## out of the ink measure.
				if wcls != "land":
					lakeish += 1
					continue
				shifts.append(((c0.b - c0.r) - 0.5 * ((cl.b - cl.r) + (cr.b - cr.r))) * 255.0)
				var o0 := img_on.get_pixelv(Vector2i(sp.round()))
				var ol := img_on.get_pixelv(Vector2i(l.round()))
				var orr := img_on.get_pixelv(Vector2i(rr.round()))
				shifts_on.append(((o0.b - o0.r) - 0.5 * ((ol.b - ol.r) + (orr.b - orr.r))) * 255.0)
				## The same statistic on the map texture, which holds no river by
				## construction (`build_color_texture`, 2026-09-27): the valley's
				## own blue shift at these ground points -- the near-channel
				## wetness tint (`hydro_wet_strength`, target (50, 68, 74)), wet
				## ground and valley shading, all of which are real and all of
				## which the tiles draw too.
				var ng := Vector2(n2.x, n2.y) * (off_px / ppc2)
				var m0 := _bilinear(t0_img, rp[k])
				var ml := _bilinear(t0_img, rp[k] + ng)
				var mr := _bilinear(t0_img, rp[k] - ng)
				shifts_map.append(((m0.b - m0.r) - 0.5 * ((ml.b - ml.r) + (mr.b - mr.r))) * 255.0)
			shifts.sort()
			shifts_on.sort()
			shifts_map.sort()
			var med: float = shifts[shifts.size() / 2] if shifts.size() > 0 else 999.0
			var med_on: float = shifts_on[shifts_on.size() / 2] if shifts_on.size() > 0 else NAN
			var med_map: float = shifts_map[shifts_map.size() / 2] if shifts_map.size() > 0 else NAN
			print("  centreline cells by sample_cell water class: %s" % str(classes))
			print("  %d centreline tile pixels are drawn in the tile's own water colour (rivers layer OFF)" % lakeish)
			print("  %d land-drawn centreline samples; median blue shift centre-minus-banks %.1f levels (B measured the ink at %.1f)"
				% [shifts.size(), med, B_SHIFT])
			print("  same samples: layer ON %.1f levels; river-free map texture (control) %.1f levels" % [med_on, med_map])
			_ok(shifts.size() > 10, "enough land-drawn centreline samples on the tile view")
			## First principles (2026-09-27). The statistic is the valley's own
			## blue shift PLUS whatever river ink the tile carries. The old bar,
			## 0.3 x the export's ink shift, assumed the valley's share is small;
			## on the post-GF-2 world it is ~19 levels with NO river anywhere in
			## the tiles (HEAD's own tile renderer, same world, reads 19.0), so
			## that bar measured the valley, not a leak. The valley's share is
			## measured instead, on the one raster that holds no river by
			## construction, and a leak is what exceeds it: OFF may read no more
			## than the map texture's shift + 4 levels (RV-2's colour bar), which
			## catches any leak above ~6% of the ink.
			_ok(med <= med_map + 4.0, "the tiles carry no river ink with the layer off (%.1f <= control %.1f + 4)" % [med, med_map])
			## Teeth: the same measure sees the ink when it is there.
			_ok(med_on - med_map >= 0.5 * B_SHIFT, "positive control: with the layer ON the same measure sees the ink (%.1f - %.1f >= 0.5 x %.1f)" % [med_on, med_map, B_SHIFT])
		vh.set_layer_visible("rivers", true)

	print("\n_riverstroke_probe: %s (%d failures); images in %s"
		% ["ALL PASS" if _fails == 0 else "FAILURES", _fails, ProjectSettings.globalize_path(_out)])
	get_tree().quit(0 if _fails == 0 else 1)


var B_SHIFT := 0.0


## Pixels from `p` along `dir` while the on/off framebuffers differ -- the
## stroke's half-width, counting `p` itself.
## Bilinear sample of a one-texel-per-cell map image at grid point `g`
## (river space: a cell's centre is `x + 0.5`, which is texel `x`).
func _bilinear(img: Image, g: Vector2) -> Color:
	var fx := g.x - 0.5
	var fy := g.y - 0.5
	var x0 := clampi(int(floor(fx)), 0, img.get_width() - 1)
	var y0 := clampi(int(floor(fy)), 0, img.get_height() - 1)
	var x1 := mini(x0 + 1, img.get_width() - 1)
	var y1 := mini(y0 + 1, img.get_height() - 1)
	var tx := clampf(fx - floor(fx), 0.0, 1.0)
	var ty := clampf(fy - floor(fy), 0.0, 1.0)
	var a := img.get_pixel(x0, y0).lerp(img.get_pixel(x1, y0), tx)
	var b := img.get_pixel(x0, y1).lerp(img.get_pixel(x1, y1), tx)
	return a.lerp(b, ty)


func _run_len(on: Image, off: Image, p: Vector2, dir: Vector2) -> float:
	var n := 0.0
	var sz := on.get_size()
	for s in 400:
		var q := Vector2i((p + dir * float(s)).round())
		if q.x < 0 or q.y < 0 or q.x >= sz.x or q.y >= sz.y or not _diff(on, off, q):
			break
		n += 1.0
	return n


func _diff(on: Image, off: Image, q: Vector2i) -> bool:
	var a := on.get_pixelv(q)
	var b := off.get_pixelv(q)
	return absf(a.r - b.r) + absf(a.g - b.g) + absf(a.b - b.b) > 0.04


## 4-connected flood fill of the on/off diff mask from `seed`.
func _flood(on: Image, off: Image, seed: Vector2i) -> Dictionary:
	var sz := on.get_size()
	var seen := {}
	if not _diff(on, off, seed):
		return seen
	var stack: Array[Vector2i] = [seed]
	seen[seed.y * sz.x + seed.x] = true
	while not stack.is_empty():
		var c: Vector2i = stack.pop_back()
		for d in [Vector2i(1, 0), Vector2i(-1, 0), Vector2i(0, 1), Vector2i(0, -1)]:
			var q: Vector2i = c + d
			if q.x < 0 or q.y < 0 or q.x >= sz.x or q.y >= sz.y:
				continue
			var key := q.y * sz.x + q.x
			if seen.has(key) or not _diff(on, off, q):
				continue
			seen[key] = true
			stack.append(q)
	return seen


## Until the deep-zoom pyramid has caught up (`lod_pending() == 0`, a
## condition rather than a frame count) and its per-tile morph has run, capped
## at 20 s.
func _settle_lod(vh: Control) -> void:
	await _settle(6)
	var t0 := Time.get_ticks_msec()
	while vh.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 20000:
		await get_tree().process_frame
	await get_tree().create_timer(0.6).timeout
	await _settle(4)


func _settle(n: int) -> void:
	for f in n:
		await RenderingServer.frame_post_draw
