extends Node
## Committed verification harness for the export raster + channel atlas
## (PARITY_AUDIT.md §5 item 14). Not committed. Drives the real bindings
## through EngineBridge exactly as the Data manager pane does, writes real
## files, and reads them back off disk.
##
##   godot --headless --path . _exportraster_probe.tscn
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

var bridge: Node
var fails := 0
var dir := ""

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

func _png_dims(path: String) -> Vector2i:
	## Read the IHDR straight off the file rather than through Image.load, so
	## the assertion is about the bytes on disk and not about Godot's decoder.
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		return Vector2i(-1, -1)
	var b := f.get_buffer(24)
	f.close()
	if b.size() < 24 or b[0] != 0x89 or b[1] != 0x50 or b[2] != 0x4E or b[3] != 0x47:
		return Vector2i(-1, -1)
	var w := (b[16] << 24) | (b[17] << 16) | (b[18] << 8) | b[19]
	var h := (b[20] << 24) | (b[21] << 16) | (b[22] << 8) | b[23]
	return Vector2i(w, h)

func _size(path: String) -> int:
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		return -1
	var n := f.get_length()
	f.close()
	return n

## Rasterizes `get_rivers()`'s traced polylines (`points`, grid-cell space)
## into a `w * h` byte mask, `1` within `width_cells / 2` plus a small
## antialiasing margin of a river, `0` elsewhere. Only valid where the pixel
## grid IS the cell grid -- true at every call site in this probe, which
## exports at exactly the generated grid's own resolution for this reason.
##
## Deliberately reads the engine's own polylines rather than re-detecting
## "blue-ish" pixels the way `_riverwidth_probe.gd` used to: that approach
## mistook ocean for river once already (that probe's own header), and would
## mistake it again here since the export still paints an ocean.
##
## `AA_MARGIN_CELLS` has to cover more than antialiasing. `apply_local_
## contrast` (render.rs) runs on the FINISHED raster, after the river ink is
## composited, as a box blur of radius `local_contrast_radius_frac * gw`
## (default 0.010, so ~20 cells at this probe's 2048 px width -- render.rs's
## own doc comment on `local_contrast_radius` says the same: "~20 cells at
## the app's own 2048^2"). A box blur of radius R can move a pixel up to R
## cells from the nearest actually-inked cell, because local contrast reacts
## to the luma EDGE the river creates and that reaction is smeared by the
## same blur. Measured directly: a first pass at `AA_MARGIN_CELLS = 2.0`
## left 114 739 outside-mask bytes differing (worst delta 4) -- far past the
## strict f32-prologue bound this section exists to enforce, and the tell is
## exactly render.rs's own local-contrast radius, not a wider river or a new
## defect. 24 = the ~20-cell blur radius plus a few cells' slack for the
## inner "fine" sub-blur and rounding; not derived from a runtime read
## because `local_contrast_radius_frac` isn't an exposed appearance
## parameter (only `local_contrast`, the strength, is) -- if that default
## ever moves, this margin and the ~20-cell figure above go stale together.
const AA_MARGIN_CELLS := 24.0

func _river_mask(wg: Object, w: int, h: int) -> PackedByteArray:
	var mask := PackedByteArray()
	mask.resize(w * h)
	var rivers: Array = wg.get_rivers(1)
	for river in rivers:
		var pts: PackedVector2Array = river.get("points", PackedVector2Array())
		var half: float = float(river.get("width_cells", 0.0)) * 0.5 + AA_MARGIN_CELLS
		var r := maxi(1, int(ceil(half)))
		var r2 := r * r
		for p in pts:
			var cx := int(round(p.x))
			var cy := int(round(p.y))
			for dy in range(-r, r + 1):
				var yy := cy + dy
				if yy < 0 or yy >= h:
					continue
				for dx in range(-r, r + 1):
					if dx * dx + dy * dy > r2:
						continue
					var xx := cx + dx
					if xx < 0 or xx >= w:
						continue
					mask[yy * w + xx] = 1
	return mask

func _ready() -> void:
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		push_error("export-raster probe watchdog: _ready never finished")
		print("\n==== WATCHDOG: the probe did not finish ====\n")
		get_tree().quit(2))
	bridge = load("res://shell/engine_bridge.gd").new()
	add_child(bridge)
	await get_tree().process_frame

	dir = ProjectSettings.globalize_path("user://_exportraster_probe")
	DirAccess.make_dir_recursive_absolute(dir)
	print("  scratch: %s" % dir)

	print("\n== 1. generate ==")
	bridge.world_gen.generate_sized(20260824, 1200.0, 512, 328)
	bridge.has_world = true
	_ok(bridge.world_gen.get_width() == 512, "world is 512 wide")

	print("\n== 2. the binding surface exists ==")
	for m in ["export_raster_widths", "export_raster_estimate", "export_raster_png", "export_channel_atlas"]:
		_ok(bridge.world_gen.has_method(m), "%s is bound" % m)
	var widths: PackedInt32Array = bridge.world_gen.export_raster_widths()
	print("  widths: %s" % str(widths))
	## Five rungs since 2026-09-06: `bakeRes`' own three plus the 16K/32K
	## `LARGE_ITEM_RULINGS.md` ruling 15 un-shelved. This asserted three until
	## that landed. `_export16k_probe.gd` owns the measurements behind the two
	## new rungs and the memory gate that guards them; this line only checks
	## the shell is offered what the engine will accept.
	_ok(widths.size() == 5 and widths[0] == 2048 and widths[4] == 32768, "bakeRes offers 2K/4K/8K/16K/32K")

	print("\n== 3. bakeDims, before rendering anything ==")
	for w in widths:
		var e: Dictionary = bridge.world_gen.export_raster_estimate(w)
		var got := Vector2i(int(e.get("width", 0)), int(e.get("height", 0)))
		var want_h := int(round(float(w) * 328.0 / 512.0))
		print("  %d -> %d x %d px, %.1f MP, peak %.0f MB, %d tiles"
			% [w, got.x, got.y, float(e.get("pixels", 0)) / 1e6,
				float(e.get("peak_bytes", 0)) / 1048576.0, int(e.get("tiles", 0))])
		_ok(got == Vector2i(w, want_h), "%d keeps the world's aspect ratio" % w)

	print("\n== 4. a real 2K single-file export ==")
	var t0 := Time.get_ticks_msec()
	var p2k := dir.path_join("map_2k.png")
	var r: Dictionary = bridge.world_gen.export_raster_png(p2k, 2048, false)
	print("  %s" % str(r))
	_ok(bool(r.get("ok", false)), "2K export reports ok")
	_ok(FileAccess.file_exists(p2k), "map_2k.png exists on disk")
	var d2k := _png_dims(p2k)
	print("  IHDR says %d x %d, %d bytes, %d ms wall" % [d2k.x, d2k.y, _size(p2k), Time.get_ticks_msec() - t0])
	_ok(d2k == Vector2i(2048, 1312), "the PNG's own header is 2048 x 1312")
	_ok(_size(p2k) > 500_000, "the file is a real image, not a stub (%d bytes)" % _size(p2k))

	print("\n== 5. the pixels are the map, not a flat fill ==")
	var img := Image.new()
	_ok(img.load(p2k) == OK, "Godot decodes the PNG back")
	if img.get_width() > 0:
		var seen := {}
		var lum_min := 999.0
		var lum_max := -1.0
		for i in range(4000):
			var c := img.get_pixel(randi() % img.get_width(), randi() % img.get_height())
			seen[c.to_rgba32()] = true
			var l := c.get_luminance()
			lum_min = min(lum_min, l)
			lum_max = max(lum_max, l)
		print("  %d distinct colours in 4000 samples, luminance %.3f..%.3f" % [seen.size(), lum_min, lum_max])
		_ok(seen.size() > 1000, "the raster is a real render, not a flat fill")
		_ok(lum_max - lum_min > 0.3, "it has real dynamic range")

	print("\n== 6. the export tracks the on-screen map ==")
	## A 2K export of a 512-cell world does **not** share pixels with the
	## screen, and the first draft of this probe wrongly assumed it did. The
	## mapping is `pixel p -> cell p*(GW-1)/(W-1)`, so cell `c` lands on the
	## *fractional* pixel `c*(W-1)/(GW-1)` = `c*2047/511`, an integer only at
	## `c = 0` and `c = 511`. Rounding to the nearest real pixel is worth up
	## to half a pixel, i.e. an eighth of a cell, and the material path an
	## eighth of a cell from a coastline is legitimately a different colour --
	## measured that way, 56% of cells differ, by up to 139 levels, with local
	## contrast off entirely. That is the sampling grid moving, not the render
	## disagreeing.
	##
	## Local contrast then adds a second, real difference on top: it is a box
	## blur over the *finished* raster whose radius is a fraction of the image
	## width, so the screen runs it across 512 px and the export across 2048.
	##
	## So this section asserts only that the two are the same *picture*, and
	## section 13 makes the exact statement at the one width where the
	## question is well-posed at all -- the grid's own.
	var lc_default := float((bridge.world_gen.get_appearance() as Dictionary).get("local_contrast", 0.55))
	var tex: ImageTexture = bridge.world_gen.build_color_texture()
	_ok(tex != null, "build_color_texture returns a texture")
	if tex != null and img.get_width() > 0:
		var screen2 := tex.get_image()
		var sum := 0.0
		var n := 0
		var peak := 0
		for i in range(4000):
			var sx := randi() % screen2.get_width()
			var sy := randi() % screen2.get_height()
			var px := int(round(float(sx) * float(img.get_width() - 1) / float(screen2.get_width() - 1)))
			var py := int(round(float(sy) * float(img.get_height() - 1) / float(screen2.get_height() - 1)))
			var a := screen2.get_pixel(sx, sy)
			var b := img.get_pixel(px, py)
			sum += absf(a.r - b.r) + absf(a.g - b.g) + absf(a.b - b.b)
			peak = maxi(peak, maxi(maxi(absi(a.r8 - b.r8), absi(a.g8 - b.g8)), absi(a.b8 - b.b8)))
			n += 3
		var mad := sum / float(n) * 255.0
		print("  mean |screen - export| = %.3f byte levels over %d channel samples, worst %d (local contrast %.2f)"
			% [mad, n, peak, lc_default])
		_ok(mad < 8.0, "the export is the same picture the viewport draws")

	print("\n== 7. tiled 4K ==")
	t0 = Time.get_ticks_msec()
	var tdir := dir.path_join("tiles4k")
	var rt: Dictionary = bridge.world_gen.export_raster_png(tdir, 4096, true)
	print("  ok=%s bytes=%d ms=%.0f files=%d" % [rt.get("ok"), int(rt.get("bytes", 0)),
		float(rt.get("ms", 0.0)), (rt.get("files", PackedStringArray()) as PackedStringArray).size()])
	_ok(bool(rt.get("ok", false)), "tiled 4K export reports ok")
	var files: PackedStringArray = rt.get("files", PackedStringArray())
	## 4096 x 2624 at 1024 px -> 4 cols x 3 rows = 12 tiles, plus index.json.
	_ok(files.size() == 13, "12 tiles + index.json (%d files)" % files.size())
	_ok(FileAccess.file_exists(tdir.path_join("tile_0_0.png")), "tile_0_0.png exists")
	_ok(FileAccess.file_exists(tdir.path_join("index.json")), "index.json exists")
	var corner := _png_dims(tdir.path_join("tile_0_0.png"))
	var last := _png_dims(tdir.path_join("tile_2_3.png"))
	print("  tile_0_0 %d x %d, tile_2_3 %d x %d, %d ms wall"
		% [corner.x, corner.y, last.x, last.y, Time.get_ticks_msec() - t0])
	_ok(corner == Vector2i(1024, 1024), "a full tile is 1024 x 1024")
	_ok(last == Vector2i(1024, 576), "the last row/col tile is the remainder, not padded")
	var mf := FileAccess.open(tdir.path_join("index.json"), FileAccess.READ)
	if mf != null:
		var txt := mf.get_as_text()
		mf.close()
		var j = JSON.parse_string(txt)
		_ok(j != null, "index.json is valid JSON")
		if j != null:
			print("  manifest: %d x %d, tileSize %d, %d tiles listed"
				% [int(j.get("width", 0)), int(j.get("height", 0)), int(j.get("tileSize", 0)),
					(j.get("tiles", []) as Array).size()])
			_ok(int(j.get("width", 0)) == 4096 and int(j.get("height", 0)) == 2624, "manifest carries the real dimensions")
			_ok((j.get("tiles", []) as Array).size() == 12, "manifest lists all 12 tiles")

	print("\n== 8. tiled and single are the same pixels ==")
	## Re-export 4K as one file and compare the top-left 1024 against tile_0_0.
	var p4k := dir.path_join("map_4k.png")
	bridge.world_gen.export_raster_png(p4k, 4096, false)
	var whole := Image.new()
	var tile := Image.new()
	if whole.load(p4k) == OK and tile.load(tdir.path_join("tile_0_0.png")) == OK:
		var diff := 0
		for i in range(3000):
			var x := randi() % 1024
			var y := randi() % 1024
			if whole.get_pixel(x, y) != tile.get_pixel(x, y):
				diff += 1
		print("  %d of 3000 sampled pixels differ" % diff)
		_ok(diff == 0, "the tiled and single exports are pixel-identical")

	print("\n== 9. a finer export really is finer ==")
	## The null hypothesis this has to kill is "the export is the 512-cell
	## screen image resampled up". Two independent ways of killing it, because
	## the obvious one -- comparing 4K against the co-located 2K pixel -- tests
	## nothing: the two sample mappings put those pixels at slightly different
	## world positions anyway, so they differ under *either* hypothesis.
	var lo := Image.new()
	if lo.load(p2k) == OK and whole.get_width() > 0 and tex != null:
		## The null hypothesis worth killing is "the export is the 512-cell
		## screen image resampled up". Compare the 2K export against a real
		## bilinear upscale of that screen image: under the null they would
		## be near-identical, and under a re-render every non-linear stage
		## (material thresholds, hillshade off interpolated gradients, the
		## noise grain, the paper tone) diverges from a smooth blend.
		var up := tex.get_image().duplicate() as Image
		up.resize(lo.get_width(), lo.get_height(), Image.INTERPOLATE_BILINEAR)
		var sum := 0.0
		for i in range(4000):
			var x := randi() % lo.get_width()
			var y := randi() % lo.get_height()
			var a := lo.get_pixel(x, y)
			var b := up.get_pixel(x, y)
			sum += absf(a.r - b.r) + absf(a.g - b.g) + absf(a.b - b.b)
		var mad := sum / 12000.0 * 255.0
		print("  mean |2K export - bilinear(512 screen)| = %.2f byte levels" % mad)
		_ok(mad > 4.0, "the 2K export is a render of the world, not the screen resampled")

		## And the same comparison one step finer, reported rather than
		## asserted: 4K against a bilinear upscale of 2K is a much smaller
		## number, because by 2K the export already resolves everything the
		## 512-cell fields carry and the remaining difference is only the
		## sub-cell material detail.
		var up2 := lo.duplicate() as Image
		up2.resize(whole.get_width(), whole.get_height(), Image.INTERPOLATE_BILINEAR)
		var s2 := 0.0
		for i in range(4000):
			var x := randi() % whole.get_width()
			var y := randi() % whole.get_height()
			var a := whole.get_pixel(x, y)
			var b := up2.get_pixel(x, y)
			s2 += absf(a.r - b.r) + absf(a.g - b.g) + absf(a.b - b.b)
		print("  mean |4K export - bilinear(2K export)| = %.2f byte levels (informational)"
			% (s2 / 12000.0 * 255.0))

	print("\n== 10. the channel atlas ==")
	var adir := dir.path_join("atlasout")
	var ra: Dictionary = bridge.world_gen.export_channel_atlas(adir)
	print("  ok=%s bytes=%d ms=%.0f" % [ra.get("ok"), int(ra.get("bytes", 0)), float(ra.get("ms", 0.0))])
	_ok(bool(ra.get("ok", false)), "atlas export reports ok")
	var af: PackedStringArray = ra.get("files", PackedStringArray())
	print("  %d files" % af.size())
	## habitat + settlement + 5 resource files + classes = 8 PNGs, + index.json
	_ok(af.size() == 9, "8 PNGs + index.json (%d)" % af.size())
	for name in ["habitat.png", "settlement.png", "resources_a.png", "resources_e.png", "classes.png"]:
		var p := adir.path_join("atlas").path_join(name)
		var d := _png_dims(p)
		print("    %-18s %5d x %-5d %8d bytes" % [name, d.x, d.y, _size(p)])
		_ok(d == Vector2i(512, 328), "%s is at grid resolution" % name)
	var jf := FileAccess.open(adir.path_join("atlas").path_join("index.json"), FileAccess.READ)
	if jf != null:
		var jt := jf.get_as_text()
		jf.close()
		var jj = JSON.parse_string(jt)
		_ok(jj != null, "atlas index.json is valid JSON")
		if jj != null:
			_ok(String(jj.get("kind", "")) == "cartalith-channel-atlas", "manifest kind")
			_ok(int(jj.get("width", 0)) == 512, "manifest width")
			var fl: Array = jj.get("files", [])
			_ok(fl.size() == 8, "manifest documents all 8 files (%d)" % fl.size())
			var keys := []
			for e in fl:
				for ch in (e.get("channels", {}) as Dictionary).values():
					keys.append(String(ch.get("key", "")))
			print("  documented keys: %s" % str(keys))
			_ok(keys.has("soil_fertility") and keys.has("water_access") and keys.has("carrying_capacity"), "habitat trio documented")
			_ok(keys.has("settlement_suitability"), "suitability documented")
			_ok(keys.has("biome") and keys.has("lithology") and keys.has("koppen"), "classes trio documented")
			_ok(keys.has("copper") and keys.has("alum"), "the first and last resource keys are both there")

	print("\n== 11. the atlas is data, and the data is not constant ==")
	var hab := Image.new()
	if hab.load(adir.path_join("atlas").path_join("habitat.png")) == OK:
		var rs := {}
		var gs := {}
		var bs := {}
		for i in range(3000):
			var c := hab.get_pixel(randi() % 512, randi() % 328)
			rs[int(c.r8)] = true
			gs[int(c.g8)] = true
			bs[int(c.b8)] = true
		print("  distinct levels: soil %d, water %d, carry %d" % [rs.size(), gs.size(), bs.size()])
		_ok(rs.size() > 8 and gs.size() > 8 and bs.size() > 8, "all three habitat channels carry real variation")
	var cls := Image.new()
	if cls.load(adir.path_join("atlas").path_join("classes.png")) == OK:
		var bset := {}
		var kset := {}
		for i in range(3000):
			var c := cls.get_pixel(randi() % 512, randi() % 328)
			bset[int(c.r8)] = true
			kset[int(c.b8)] = true
		print("  biome indices seen: %d, koppen channel levels: %d" % [bset.size(), kset.size()])
		_ok(bset.size() > 3, "the biome channel is a real categorical raster")
		_ok(kset.size() == 1 and kset.has(0), "the koppen channel is documented and zero, as disclosed")

	print("\n== 12. refusals, not crashes ==")
	var bad: Dictionary = bridge.world_gen.export_raster_png(dir.path_join("x.png"), 3000, false)
	_ok(not bool(bad.get("ok", true)), "an unsupported width is refused")
	print("  %s" % String(bad.get("error", "")))
	var bad2: Dictionary = bridge.world_gen.export_raster_png("", 2048, false)
	_ok(not bool(bad2.get("ok", true)), "an empty path is refused")

	print("\n== 13. at the grid's own resolution, byte for byte ==")
	## The exact statement bake_raster.rs makes as a unit test, made here
	## against the real binding, a real generated world and a real PNG on
	## disk. Needs a world whose grid *is* an offered export width, because
	## that is the only case where every output pixel lands on a whole cell
	## with no interpolation weight on its neighbours -- so this regenerates
	## at 2048 x 1312 rather than trying to make 512 and 2048 line up, which
	## is exactly the mistake section 6 documents.
	var t13 := Time.get_ticks_msec()
	bridge.world_gen.generate_sized(20260824, 1200.0, 2048, 1312)
	bridge.has_world = true
	print("  regenerated 2048 x 1312 in %.1f s" % ((Time.get_ticks_msec() - t13) / 1000.0))
	_ok(bridge.world_gen.get_width() == 2048, "world is 2048 wide")
	var pg := dir.path_join("map_gridres.png")
	var rg: Dictionary = bridge.world_gen.export_raster_png(pg, 2048, false)
	_ok(bool(rg.get("ok", false)), "grid-resolution export reports ok")
	_ok(_png_dims(pg) == Vector2i(2048, 1312), "the export is 2048 x 1312")
	var exported := Image.new()
	var gtex: ImageTexture = bridge.world_gen.build_color_texture()
	if gtex != null and exported.load(pg) == OK:
		var scr := gtex.get_image()
		_ok(scr.get_width() == 2048 and scr.get_height() == 1312, "the screen texture is the same size")
		## Compare the raw bytes rather than 2.7 M scripted get_pixel calls.
		## Both are converted to RGB8 first: the export has no alpha and the
		## viewport texture may not be RGB8 to begin with.
		scr.convert(Image.FORMAT_RGB8)
		exported.convert(Image.FORMAT_RGB8)
		var a := scr.get_data()
		var b := exported.get_data()
		_ok(a.size() == b.size(), "same byte count (%d vs %d)" % [a.size(), b.size()])
		if a.size() == b.size():
			## **Owner ruling, 2026-09-22: rivers are no longer baked into
			## `build_color_texture()`.** They are drawn as a vector stroke by
			## `map_overlay.gd::_draw_rivers()` in the lake's own colour, at
			## the channel's real world-space width -- while
			## `export_raster_png` deliberately still calls the OLD baked-ink
			## path (`WorldGen::river_ink()`), unchanged. So screen and export
			## now intentionally disagree exactly where a river runs, and only
			## there. A byte-for-byte comparison that does not know this would
			## either weaken its bound everywhere (masking a real regression
			## elsewhere) or fail permanently on the river ink alone (masking
			## nothing at all) -- both are exactly the "asserted non-emptiness
			## and shape explicitly" failure this file's own header warns
			## about. So the comparison is split: every byte gets classified
			## as river-masked or not, from the engine's own `get_rivers()`
			## polylines/widths (not from re-detecting blue pixels, which is
			## the `_riverwidth_probe.gd` mistake this project already made
			## once). The strict, historically-proven bound below is asserted
			## ONLY outside the mask -- it is exactly the bound that caught
			## the 132-level and 73-level missing-stage regressions described
			## below, unweakened. The masked region gets its own assertion:
			## it must show a real, non-trivial divergence, because if a
			## future change silently re-added the screen-side bake the two
			## images would go back to matching everywhere INCLUDING the
			## river mask -- and that must fail this probe, not pass it.
			var mask := _river_mask(bridge.world_gen, scr.get_width(), scr.get_height())
			var mask_px := 0
			for m in mask:
				if m == 1:
					mask_px += 1
			print("  river mask: %d of %d pixels (%.2f%%)"
				% [mask_px, mask.size(), 100.0 * float(mask_px) / float(maxi(1, mask.size()))])

			var bad3 := 0
			var worst := 0
			var nonriver_bytes := 0
			var river_bad := 0
			var river_worst := 0
			var river_bytes := 0
			for i in range(a.size()):
				var d: int = absi(a[i] - b[i])
				var px := int(i / 3)
				if mask[px] == 1:
					river_bytes += 1
					if d > 0:
						river_bad += 1
						river_worst = maxi(river_worst, d)
				else:
					nonriver_bytes += 1
					if d > 0:
						bad3 += 1
						worst = maxi(worst, d)
			print("  [outside river mask] %d of %d bytes differ, worst delta %d" % [bad3, nonriver_bytes, worst])
			print("  [inside river mask]  %d of %d bytes differ, worst delta %d" % [river_bad, river_bytes, river_worst])
			## Not "zero bytes differ", and the reason is worth stating where
			## someone re-running this will read it. BakeFields stores slope,
			## macro shade and meso shade as f32 -- because the reference's
			## own bake prologue stores gridSlope/gridShade/gridShadeMeso in
			## Float32Arrays, while both engines compute them in doubles for
			## the screen. So the bake and the screen agree to f32 rounding,
			## not to the bit, in this port exactly as in the original.
			## bake_raster.rs measures the same thing offline: 51% of cells
			## differ in f64 by at most 2.4e-8, and essentially none of that
			## survives quantization. The exact count moves between runs (12
			## and 17 on two) because it is a knife-edge -- a byte either
			## lands on a rounding boundary or it does not -- so what is
			## asserted is a bound, not a figure.
			##
			## The first run of this section, before the river tint was added
			## to the export, read 291,815 bytes and worst delta 132. That is
			## the difference between a rounding bound and a missing stage,
			## and it is why this asserts a bound rather than a tolerance on
			## a mean.
			##
			## It caught a second missing stage the same way, and that one is
			## worth recording because the bound is what noticed. Measured at
			## 65a8262 on 2026-09-06, this section read 199,909 bytes and
			## worst delta 73: the screen had learned to composite the stamped
			## river disc at its own intensity (58dd5b2, 2026-08-30, "Rivers
			## have a width now" -- which touched neither render.rs nor
			## export_raster.rs) while render::bake_rect still took a bare
			## chan flag and tinted every channel cell at full strength. The
			## number is the giveaway: 73 levels is far too large for f32
			## rounding, which cannot exceed 1, and far too small for a
			## different picture. Nobody re-ran the probe in between, so it
			## went unnoticed. Do not relax either assertion below to make it
			## green. (2026-09-22: this bound now applies outside the river
			## mask -- inside it, the two are SUPPOSED to differ.)
			_ok(worst <= 1, "outside the river mask, no byte is off by more than one level (worst %d)" % worst)
			_ok(bad3 * 10000 < nonriver_bytes, "outside the river mask, the f32 prologue is the only difference (%d of %d bytes)" % [bad3, nonriver_bytes])
			## Non-vacuity for the mask itself, and for the divergence it is
			## supposed to expose -- the two things a masked comparison can
			## get silently wrong. Owner ruling 2026-09-22 didn't remove
			## rivers, only where they're drawn, so a world generated from a
			## fixed seed that has always had rivers had better still have
			## some: an empty mask here would mean `_river_mask()` itself is
			## broken (wrong field name, wrong coordinate space), not that
			## the world has none. `river_worst > 10` is far above both the
			## f32 rounding ceiling (1) and any plausible antialiasing fringe
			## noise, and is the number that would go back to ~1 if a future
			## change silently re-added the screen-side river bake -- making
			## this assertion the reversion detector.
			_ok(mask_px > 0, "this world has river pixels to test the screen/export divergence against")
			_ok(river_bad > 0 and river_worst > 10,
				"the river-masked region really does differ between screen and export (export still inks rivers, screen does not) -- worst %d levels over %d differing bytes" % [river_worst, river_bad])

	print("\n==== %s so far (%d failures) ====" % ["ALL PASS" if fails == 0 else "FAILURES", fails])

	print("\n== 14. 8K, the size the estimate warns about ==")
	## The one path a unit test cannot reach: 43 MP through bake_rect, the
	## local-contrast pass' luma and its two blurs' FOUR f32 buffers, and a
	## single PNG encode, all live in one process. 943 MB peak by the
	## binding's own estimate, on a 2048 x 1312 grid -- so this also exercises
	## a 4x upsample, the ratio the whole fractional-sampling design exists
	## for. (615 MB until 2026-09-06, when PEAK_BYTES_PER_PIXEL was corrected
	## from 15 to 23: `blur_once` allocates two buffers, not one, and
	## `apply_local_contrast` runs two of them under one `rayon::join`.)
	var e8: Dictionary = bridge.world_gen.export_raster_estimate(8192)
	print("  estimate: %d x %d, %.1f MP, %.0f MB peak"
		% [int(e8.get("width", 0)), int(e8.get("height", 0)),
			float(e8.get("pixels", 0)) / 1e6, float(e8.get("peak_bytes", 0)) / 1048576.0])
	var p8 := dir.path_join("map_8k.png")
	var t8 := Time.get_ticks_msec()
	var r8: Dictionary = bridge.world_gen.export_raster_png(p8, 8192, false)
	print("  ok=%s %d bytes, %.1f s engine, %.1f s wall"
		% [r8.get("ok"), int(r8.get("bytes", 0)), float(r8.get("ms", 0.0)) / 1000.0,
			(Time.get_ticks_msec() - t8) / 1000.0])
	_ok(bool(r8.get("ok", false)), "the 8K export completes without falling over")
	_ok(_png_dims(p8) == Vector2i(8192, 5248), "the 8K PNG's own header is 8192 x 5248")
	_ok(_size(p8) > 10_000_000, "it is a real 43 MP image (%d bytes)" % _size(p8))
	var big := Image.new()
	_ok(big.load(p8) == OK, "Godot decodes 43 MP back")
	if big.get_width() > 0:
		var seen8 := {}
		for i in range(3000):
			seen8[big.get_pixel(randi() % 8192, randi() % 5248).to_rgba32()] = true
		print("  %d distinct colours in 3000 samples" % seen8.size())
		_ok(seen8.size() > 1000, "the 8K raster is a real render")

	print("\n== 15. the display space stays on the display ==")
	## export_raster.rs's module doc, asserted rather than documented and
	## hoped: **no export applies render::apply_color_space**, deliberately.
	## The space describes the monitor in front of this session (WorldGen::
	## color_space's own doc, and GUI_GAP_REGISTER.md's "display = app,
	## working space = document"), an exported PNG is a document, and
	## encode_png_rgb8 writes no ICC profile -- so P3 numbers in an untagged
	## file are misread as sRGB by every consumer rather than matching the
	## screen.
	##
	## The positive control comes first and is not optional. "The export did
	## not move" is a vacuous pass if the setting moved nothing at all, which
	## is exactly how this project has shipped an unfailable probe three
	## times.
	if bridge.world_gen.has_method("set_color_space"):
		var srgb_bytes := PackedByteArray()
		var stex: ImageTexture = bridge.world_gen.build_color_texture()
		if stex != null:
			var simg := stex.get_image()
			simg.convert(Image.FORMAT_RGB8)
			srgb_bytes = simg.get_data()
		_ok(bool(bridge.world_gen.set_color_space("Display P3")), "Display P3 is offered by this build")
		var p3_bytes := PackedByteArray()
		var ptex: ImageTexture = bridge.world_gen.build_color_texture()
		if ptex != null:
			var pimg := ptex.get_image()
			pimg.convert(Image.FORMAT_RGB8)
			p3_bytes = pimg.get_data()
		var vworst := 0
		if srgb_bytes.size() == p3_bytes.size() and srgb_bytes.size() > 0:
			for i in range(20000):
				var k := randi() % srgb_bytes.size()
				vworst = maxi(vworst, absi(srgb_bytes[k] - p3_bytes[k]))
		print("  viewport raster under P3 vs sRGB: worst %d byte levels in 20000 samples" % vworst)
		_ok(srgb_bytes.size() > 0 and p3_bytes.size() > 0 and srgb_bytes != p3_bytes,
			"positive control -- Display P3 really does move the viewport raster")

		var pp3 := dir.path_join("map_p3.png")
		var rp3: Dictionary = bridge.world_gen.export_raster_png(pp3, 2048, false)
		_ok(bool(rp3.get("ok", false)), "the export still runs with a display space selected")
		var fa := FileAccess.open(dir.path_join("map_gridres.png"), FileAccess.READ)
		var fb := FileAccess.open(pp3, FileAccess.READ)
		if fa != null and fb != null:
			var ba := fa.get_buffer(fa.get_length())
			var bb := fb.get_buffer(fb.get_length())
			fa.close()
			fb.close()
			print("  sRGB-session PNG %d bytes, P3-session PNG %d bytes" % [ba.size(), bb.size()])
			_ok(ba.size() > 500_000 and ba == bb,
				"the exported file is byte-identical with the display space moved (%d vs %d bytes)" % [ba.size(), bb.size()])
		_ok(bool(bridge.world_gen.set_color_space("sRGB")), "and the session goes back to sRGB")

	print("\n==== 8K: %s ====\n" % ["OK" if fails == 0 else "SEE ABOVE"])
	get_tree().quit(1 if fails > 0 else 0)
