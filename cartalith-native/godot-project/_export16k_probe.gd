extends Node
## Ruling 26's one measurement: what a PNG of this renderer's output actually
## costs per pixel, measured rather than taken from a textbook figure -- and
## then, since 2026-09-06, the three things ruling 15's two new rungs need
## checked in one place: that 16384 and 32768 are offered, that the estimate
## says what an export will cost in bytes AND in memory before it starts, and
## that a size the device cannot hold is refused rather than attempted.
##
##   godot --headless --path . _export16k_probe.tscn
##
## Headless is correct here and worth saying: every number below is either a
## file length on disk or a wall clock around a CPU encode. Nothing reads a
## pixel back out of an ImageTexture, which is the case `MISTAKES.md` requires
## a windowed run for.
##
## Three seeds, not one. Map imagery's compressibility is a property of the
## world -- how much sea, how much flat interior -- so one world is one sample
## in exactly the sense a single layout measurement is.
##
## Grid is 2048 x 1311, which is `app.gd`'s own default and the grid
## `EXPORT_SCOPE.md` section 6's dimensions were derived from (8192 -> 5244,
## 16384 -> 10488, 32768 -> 20976 all follow from 1311/2048).
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed").

var bridge: Node
var fails := 0
var dir := ""

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

func _size(path: String) -> int:
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		return -1
	var n := f.get_length()
	f.close()
	return n

func _ready() -> void:
	get_tree().create_timer(3600.0).timeout.connect(func() -> void:
		push_error("export-16k probe watchdog: _ready never finished")
		print("\n==== WATCHDOG: the probe did not finish ====\n")
		get_tree().quit(2))
	bridge = load("res://shell/engine_bridge.gd").new()
	add_child(bridge)
	await get_tree().process_frame

	dir = ProjectSettings.globalize_path("user://_export16k_probe")
	DirAccess.make_dir_recursive_absolute(dir)
	print("  scratch: %s" % dir)

	var seeds := [20260906, 7, 991733]
	var widths := [2048, 4096, 8192]
	var totals := {}
	for w in widths:
		totals[w] = []

	for s in seeds:
		print("\n======== seed %d ========" % s)
		var tg := Time.get_ticks_msec()
		bridge.world_gen.generate_sized(s, 1200.0, 2048, 1311)
		bridge.has_world = true
		print("  generated 2048 x 1311 in %.1f s" % ((Time.get_ticks_msec() - tg) / 1000.0))
		_ok(bridge.world_gen.get_width() == 2048, "world is 2048 wide")

		## The estimate at every rung, including the two this probe does not
		## itself run -- 16K and 32K are minutes of wall clock across three
		## seeds, and `_exportbig_probe.gd` is where they are actually
		## exported. `export_raster_estimate` never consulted BAKE_WIDTHS and
		## still does not, so it prices a width whether or not it is offered.
		for w in [2048, 4096, 8192, 16384, 32768]:
			var e: Dictionary = bridge.world_gen.export_raster_estimate(w)
			print("  estimate %6d -> %6d x %-6d %8.1f MP   peak %8.2f MB   %d tiles"
				% [w, int(e.get("width", 0)), int(e.get("height", 0)),
					float(e.get("pixels", 0)) / 1e6,
					float(e.get("peak_bytes", 0)) / 1048576.0, int(e.get("tiles", 0))])

		for w in widths:
			var p := dir.path_join("s%d_%d.png" % [s, w])
			var t0 := Time.get_ticks_msec()
			var r: Dictionary = bridge.world_gen.export_raster_png(p, w, false)
			var wall := (Time.get_ticks_msec() - t0) / 1000.0
			if not bool(r.get("ok", false)):
				_ok(false, "%d export: %s" % [w, String(r.get("error", "?"))])
				continue
			var px := int(r.get("width", 0)) * int(r.get("height", 0))
			var bytes := _size(p)
			var bpp := float(bytes) / float(px)
			(totals[w] as Array).append(bpp)
			print("  RUN      %6d -> %6d x %-6d %10d bytes  %.4f bytes/px  %.1f s"
				% [w, int(r.get("width", 0)), int(r.get("height", 0)), bytes, bpp, wall])
			_ok(bytes > 100000, "%d wrote a real image" % w)
			## Free the file: three seeds at 2K/4K/8K is a lot of disk to leave
			## behind, and nothing below re-reads them.
			DirAccess.remove_absolute(p)

	print("\n======== bytes per pixel, by export width ========")
	for w in widths:
		var a: Array = totals[w]
		if a.is_empty():
			continue
		var lo: float = a[0]
		var hi: float = a[0]
		var sum := 0.0
		for v in a:
			lo = minf(lo, v)
			hi = maxf(hi, v)
			sum += v
		a.sort()
		print("  %6d px wide  (%.0fx the grid):  median %.4f  (%.4f .. %.4f) bytes/px over %d worlds"
			% [w, float(w) / 2048.0, a[a.size() / 2], lo, hi, a.size()])

	print("\n======== the ladder, after ruling 15 ========")
	var widths_bound: PackedInt32Array = bridge.world_gen.export_raster_widths()
	print("  export_raster_widths() = %s" % str(widths_bound))
	_ok(widths_bound.size() == 5 and widths_bound[3] == 16384 and widths_bound[4] == 32768,
		"the binding offers 16384 and 32768")
	var still_bad: Dictionary = bridge.world_gen.export_raster_png(dir.path_join("x.png"), 3000, false)
	_ok(not bool(still_bad.get("ok", true)), "a width off the ladder is still refused, not rounded")
	print("  3000: %s" % String(still_bad.get("error", "")))

	print("\n======== ruling 26: the estimate informs before the run ========")
	## The estimate's `file_bytes` against a file that was actually written, at
	## the largest width this probe runs. `_ok` on the residual rather than on
	## the number, because the model is fitted across three worlds and this is
	## one of them -- see `FILE_BYTES_AT_GRID_WIDTH`'s own doc.
	var e8: Dictionary = bridge.world_gen.export_raster_estimate(8192)
	print("  estimate keys: %s" % str(e8.keys()))
	for k in ["width", "height", "pixels", "peak_bytes", "file_bytes", "heightmap_peak_bytes"]:
		_ok(e8.has(k), "the estimate carries %s" % k)
	var p8 := dir.path_join("estimate_check_8192.png")
	bridge.world_gen.export_raster_png(p8, 8192, false)
	var real8 := _size(p8)
	var pred8 := int(e8.get("file_bytes", 0))
	var resid := float(pred8 - real8) / float(real8)
	print("  file_bytes predicted %d, actually wrote %d, %.1f%% out" % [pred8, real8, resid * 100.0])
	_ok(absf(resid) < 0.10, "the pre-run file-size estimate lands within 10% of the file")
	print("  peak_bytes %.0f MB, heightmap_peak_bytes %.0f MB"
		% [float(e8.get("peak_bytes", 0)) / 1048576.0, float(e8.get("heightmap_peak_bytes", 0)) / 1048576.0])
	DirAccess.remove_absolute(p8)

	## `memory_available` and `affordable` are **omitted** on a platform that
	## reports no budget, so this asserts the pair moves together rather than
	## asserting either is present -- `MISTAKES.md`'s omit-don't-fake rule
	## checked from the consumer's side.
	_ok(e8.has("memory_available") == e8.has("affordable"),
		"memory_available and affordable are present together or not at all")
	if e8.has("memory_available"):
		print("  memory_available %.1f GB, affordable=%s"
			% [float(e8.get("memory_available", 0)) / 1073741824.0, str(e8.get("affordable"))])
		_ok(int(e8.get("memory_available", 0)) > 0, "a reported budget is a real number, never 0 or -1")
	else:
		print("  this platform reports no memory budget -- both keys correctly absent")

	print("\n======== the refusal fires on a size this device cannot hold ========")
	## Exercised honestly rather than by pretending the machine is smaller: a
	## tall, narrow grid makes `bake_dims` return a huge HEIGHT for the same
	## width, so 32768 px across a 512 x 1311 world is 2.7 gigapixels -- past
	## any desktop's budget while every constant stays real.
	bridge.world_gen.generate_sized(20260906, 1200.0, 512, 1311)
	bridge.has_world = true
	var tall: Dictionary = bridge.world_gen.export_raster_estimate(32768)
	print("  tall world: %d x %d, %.2f Gpx, peak %.1f GB, affordable=%s"
		% [int(tall.get("width", 0)), int(tall.get("height", 0)),
			float(tall.get("pixels", 0)) / 1e9, float(tall.get("peak_bytes", 0)) / 1073741824.0,
			str(tall.get("affordable", "(absent)"))])
	var refused: Dictionary = bridge.world_gen.export_raster_png(dir.path_join("tall.png"), 32768, false)
	_ok(not bool(refused.get("ok", true)), "an unaffordable export is refused rather than attempted")
	print("  %s" % String(refused.get("error", "")))
	_ok(String(refused.get("error", "")).contains("memory"), "and the refusal says why")
	_ok(not FileAccess.file_exists(dir.path_join("tall.png")), "nothing was written")

	print("\n==== %s (%d failures) ====\n" % ["ALL PASS" if fails == 0 else "FAILURES", fails])
	get_tree().quit(1 if fails > 0 else 0)
