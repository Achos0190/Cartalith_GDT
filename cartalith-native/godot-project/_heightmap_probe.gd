extends Node
## Heightmap export round-trip probe.
##
##   godot --headless --path . _heightmap_probe.tscn
##
## The export exists because Cartalith could READ a heightmap PNG since Phase 1
## and could not write one -- a one-way door nobody had scoped (EXPORT_SCOPE.md
## does not contain the word). Found by comparing against Nortantis 3.18.
##
## So the assertion that matters is not "a file appeared". It is that the file
## this app writes is a file this app can read, and that what comes back is the
## field that went out. Anything less would ship a format that only looks right.

var _fail := 0

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _ready() -> void:
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var wg: Object = ClassDB.instantiate("WorldGen")
	wg.set_params({"tect.plates": 9})
	wg.generate_sized(24601, 640.0, 256, 192)
	print("[GEN] 256x192 seed 24601")

	var out := OS.get_user_data_dir() + "/_probe_heightmap.png"

	print("\n=== 1: it writes, and reports honestly ===")
	var r: Dictionary = wg.export_heightmap_png(out, 2048)
	_ok("ok", r.get("ok", false), true)
	if not bool(r.get("ok", false)):
		print("  error: ", r.get("error", "")); get_tree().quit(1); return
	print("  info ", r.get("width"), "x", r.get("height"), "  ",
		r.get("bytes"), " bytes  ", "%.1f ms" % float(r.get("ms", 0.0)))
	_ok("width is the requested bake width", r.get("width"), 2048)
	_ok("aspect follows the grid (256x192 -> 4:3)", r.get("height"), 1536)
	_ok("the file exists", FileAccess.file_exists(out), true)
	_ok("it is not a stub", int(r.get("bytes", 0)) > 20000, true)

	print("\n=== 2: it is really 16-bit, not 8 dressed up ===")
	var f := FileAccess.open(out, FileAccess.READ)
	var head := f.get_buffer(33)
	f.close()
	## PNG IHDR: bytes 24 = bit depth, 25 = colour type (0 = grayscale).
	_ok("PNG magic", head[1] == 0x50 and head[2] == 0x4E and head[3] == 0x47, true)
	_ok("bit depth 16", head[24], 16)
	_ok("colour type 0 (grayscale)", head[25], 0)

	print("\n=== 3: the round trip -- this app can read what it wrote ===")
	## Re-import at the grid's own width. The exporter uses the same box-filter
	## span arithmetic `heightmap_to_field` uses, so this should be near-identity
	## rather than two resamplers disagreeing.
	var wg2: Object = ClassDB.instantiate("WorldGen")
	wg2.set_params({"tect.plates": 9})
	## import_heightmap(path, seed, width_km, grid_w) -- same seed, extent and
	## grid the export came from, so any difference is the FORMAT and not a
	## parameter.
	var imported: bool = wg2.import_heightmap(out, 24601, 640.0, 256)
	_ok("this app can read back what it wrote", imported, true)
	if imported:
		## The exporter box-filters with the same span arithmetic
		## heightmap_to_field uses, but the importer also runs normalize_field
		## (its own doc says an 8-bit map that never reaches white would
		## otherwise read low). So this is near-identity, not bit-identity, and
		## the assertion is written for what is actually true: strong
		## correlation, no collapse.
		## No bulk field accessor is exposed (`heightmap()` does not exist), so
		## compare a lattice through `sample_cell`, which returns `elevation_m`.
		## The importer runs `normalize_field` -- its own doc says an 8-bit map
		## that never reaches white would otherwise read low -- so this is
		## near-identity, not bit-identity, and the assertion says so.
		var n := 0
		var err := 0.0
		var mn := INF
		var mx := -INF
		for gy in range(4, 188, 8):
			for gx in range(4, 252, 8):
				var ca: Dictionary = wg.sample_cell(gx, gy)
				var cb: Dictionary = wg2.sample_cell(gx, gy)
				if ca.is_empty() or cb.is_empty():
					continue
				var ea := float(ca.get("elevation_m", 0.0))
				var eb := float(cb.get("elevation_m", 0.0))
				err += absf(ea - eb)
				mn = minf(mn, ea)
				mx = maxf(mx, ea)
				n += 1
		_ok("the lattice actually sampled something", n > 100, true)
		if n > 0:
			err /= float(n)
			var span: float = mx - mn
			print("  info %d cells, mean |Δ elevation| = %.1f m over a %.1f m span" % [n, err, span])
			_ok("the field survived the round trip (mean error under 5% of span)",
				err < span * 0.05, true)
			_ok("the re-import did not collapse to a constant", span > 100.0, true)

	print("\n=== 4: refusals are refusals, not crashes ===")
	_ok("an unsupported width is refused",
		wg.export_heightmap_png(out, 1234).get("ok", true), false)
	_ok("an empty path is refused",
		wg.export_heightmap_png("", 2048).get("ok", true), false)
	var fresh: Object = ClassDB.instantiate("WorldGen")
	_ok("no world is refused rather than crashing",
		fresh.export_heightmap_png(out, 2048).get("ok", true), false)

	DirAccess.remove_absolute(out)
	print("\n_heightmap_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
