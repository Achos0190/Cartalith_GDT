extends Node
## Mutation-testing gap in `WorldGen::lod_cache_key` (`cartalith-godot/src/lib.rs`,
## grep `fn lod_cache_key`): the line folding in
## `self.params.passes.glacial_snowline.to_bits()` survived a mutation to
## `-> 0u64`. The line's own doc comment states why it is needed: three of
## LOD-D4's four new cache inputs (`peak_m`, `lapse_rate`, `planet.g`) also
## bump a pipeline-stage version through `params::invalidates`, so they would
## usually be caught twice over; `passes.glacial_snowline` does NOT
## (`params::invalidates("passes.glacial_snowline")` is `None` -- confirmed by
## reading `params.rs` for this probe), so without the explicit term a
## snowline change is invisible to the key and `WorldGen::lod_snapshot()`
## reuses a `TileFields` snapshot built at the OLD snowline.
##
## Because `lod_synthesize_tile` (the `#[func]` this probe drives) goes
## through exactly that cache (`lod_snapshot() -> lod_worker.snapshot_for
## (&key)`), the mutant is reachable from here even though `lod_cache_key` is
## a private method on a cdylib `GodotClass` (`MISTAKES.md`'s row 32/48).
##
## # What this probe finds about the "ice off" case (task step 3)
##
## `lod_cache_key`'s `glacial_snowline` term is folded in UNCONDITIONALLY --
## it does not check `appearance.ice_strength` at all -- while the RENDER
## path (`render.rs::land_color`, `glacier_on`/`cryo` locals) gates BOTH the
## glacier field and the cryo lapse correction on `ice_strength > 0.0`. So
## with ice off, changing the snowline still busts the cache (a fresh
## `TileFields` is built every time) even though the pixels it produces are
## structurally guaranteed to be identical, because the render path never
## reads `tf.glacier`/`tf.cryo` when `ice_strength <= 0.0`. That is an
## over-invalidation (wasted rebuild), not a correctness bug -- section 3
## below asserts the PIXEL claim ("does not need to"), not a cache-hit count,
## since no `#[func]` exposes the synchronous path's hit/miss counter.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x900 _glaciallodkey_probe.tscn
##
## **Not `--headless`.** `_d6async_probe.gd` section 4 established that
## comparing texel data from `lod_synthesize_tile`'s returned `ImageTexture`
## needs a real rasterizer; this probe follows the same convention rather
## than assuming `Image::create_from_data` is exempt (`MISTAKES.md`, "Run a
## pixel probe").
##
## Exit 0 pass, 1 a real assertion failed, 2 the premise could not be set up
## (no `.gdextension`, generation failed, or the world never produced a real
## cold/high land cell to anchor the pixel-level check on).

var _fail := 0
var _checks := 0
var _br: Object

func _ok(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if cond:
		print("  ok   %s%s" % [name, ("  [%s]" % detail) if detail != "" else ""])
	else:
		_fail += 1
		printerr("  FAIL %s%s" % [name, ("  [%s]" % detail) if detail != "" else ""])

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## Max absolute per-channel diff between two same-sized RGBA byte buffers, at
## pixel (x, y) of an image `w` pixels wide.
func _pixel_diff(a: PackedByteArray, b: PackedByteArray, w: int, x: int, y: int) -> int:
	var i := (y * w + x) * 4
	if i + 3 >= a.size() or i + 3 >= b.size():
		return -1
	var d := 0
	for c in 4:
		d = maxi(d, absi(int(a[i + c]) - int(b[i + c])))
	return d

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless. Comparing lod_synthesize_tile's texel data needs a")
		printerr("  real rasterizer (MISTAKES.md, \"Run a pixel probe\"; the same reason _d6async_probe.gd requires windowed).")
		get_tree().quit(2)
		return
	if not ClassDB.class_exists("WorldGen"):
		printerr("PROBE-CANNOT-RUN: the .gdextension did not load.")
		get_tree().quit(2)
		return

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await _frames(30)
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(15)
	_br = app.bridge
	if _br == null or _br.world_gen == null:
		printerr("PROBE-CANNOT-RUN: the shell exposed no bridge/world_gen.")
		get_tree().quit(2)
		return
	print("[BOOT] shell up")

	if not _br.has_method("lod_synthesize_tile"):
		printerr("PROBE-CANNOT-RUN: the loaded extension has no lod_synthesize_tile forwarder.")
		get_tree().quit(2)
		return

	# --- 1. a deliberately cold, high, glacial-pass-on world ---------------
	#
	# `passes.glacial_snowline` does not itself change the heightmap or
	# temperature raster (`params::invalidates` returns `None` for it -- read
	# in `params.rs` for this probe, not assumed), so the same generated
	# world can be re-rendered at two different snowlines with no regenerate
	# in between. What has to be real is COLD, HIGH ground with a real
	# catchment under it, which is a property of generation, set once here.
	print("\n=== 1: a cold, high, glacial-pass-on world ===")
	var wg = _br.world_gen
	wg.set_params({
		"peak_m": 6000.0,
		"climate.pole_temp": -35.0,
		"climate.lat_n": 75.0,
		"climate.lat_s": -75.0,
		"passes.glacial": true,
		"passes.glacial_snowline": 0.55,
	})
	_br.generate({
		"seed": 1337, "width_km": 800.0, "grid_w": 512, "grid_h": 384,
		"sea_level": 0.42, "villages": false,
	})
	var spins := 0
	while _br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	if _br.generating:
		printerr("PROBE-CANNOT-RUN: generation never finished.")
		get_tree().quit(2)
		return
	await get_tree().create_timer(1.0).timeout
	_ok("world present", _br.has_world)
	if not _br.has_world:
		print("[ABORT] no world -- nothing else here can run")
		get_tree().quit(2)
		return

	# --- 2. find a real cold, high LAND cell to anchor the pixel check -----
	#
	# The pivot is found from the world's own fields (elevation, temperature)
	# the same way `_lodsweep_probe.gd`'s Aletsch sheet does -- the coldest
	# high land cell -- and it is independent of `glacial_snowline`, so it can
	# be found once, before either render.
	print("\n=== 2: locating a cold, high land cell ===")
	var g: Vector2i = _br.grid_size()
	var stride: int = maxi(1, g.x / 128)
	var best_score := -1e30
	var best := Vector2i(-1, -1)
	var best_t := 999.0
	var y := 0
	while y < g.y:
		var x := 0
		while x < g.x:
			var s: Dictionary = _br.sample_cell(x, y)
			if not s.is_empty() and s.get("water", "land") == "land":
				var t: float = float(s.get("temperature_c", 99.0))
				var e: float = float(s.get("elevation", 0.0))
				var score: float = e * 100.0 - t
				if score > best_score:
					best_score = score
					best = Vector2i(x, y)
					best_t = t
			x += stride
		y += stride
	_ok("found a cold, high land cell", best.x >= 0 and best_t < 0.0,
		"cell=%s temperature_c=%.2f" % [str(best), best_t])
	if best.x < 0 or best_t >= 0.0:
		print("[ABORT] this seed/config never produced freezing land -- not a real failure of the probe's target")
		get_tree().quit(2)
		return
	print("  info pivot cell %s, temperature_c=%.2f" % [str(best), best_t])

	# --- 3. ice ON: two renders of the SAME tile, only snowline changed ----
	print("\n=== 3: ice ON -- same tile, only glacial_snowline changed ===")
	var applied: int = _br.set_appearance({"ice_strength": 1.0})
	_ok("ice_strength recognised by set_appearance (teeth check)", applied == 1, "applied=%d" % applied)

	var tex_a: Texture2D = _br.lod_synthesize_tile(0, 0, 0)
	_ok("tile (0,0,0) synthesised at snowline 0.55", tex_a != null)
	if tex_a == null:
		print("[ABORT] no tile -- nothing else here can run")
		get_tree().quit(2)
		return
	var img_a := tex_a.get_image()
	var data_a := img_a.get_data()
	var w := img_a.get_width()
	var h := img_a.get_height()

	wg.set_params({"passes.glacial_snowline": 0.15})
	var tex_b: Texture2D = _br.lod_synthesize_tile(0, 0, 0)
	_ok("tile (0,0,0) synthesised at snowline 0.15", tex_b != null)
	var img_b := tex_b.get_image()
	var data_b := img_b.get_data()

	_ok("the two renders are the same size (a precondition for the diffs below)",
		img_b.get_width() == w and img_b.get_height() == h,
		"%dx%d vs %dx%d" % [w, h, img_b.get_width(), img_b.get_height()])

	# Whole-tile diff: not just "differs somewhere" -- the count of visibly
	# differing pixels, and where the single biggest change landed.
	var differing := 0
	var max_diff := -1
	var max_at := Vector2i(-1, -1)
	for py in h:
		for px in w:
			var d := _pixel_diff(data_a, data_b, w, px, py)
			if d > 10:
				differing += 1
			if d > max_diff:
				max_diff = d
				max_at = Vector2i(px, py)

	_ok("a real region of pixels changed, not a stray one or two (ice ON)",
		differing >= 8, "differing=%d of %d px" % [differing, w * h])
	_ok("the single biggest change is a real visible step, not rounding noise",
		max_diff >= 20, "max_diff=%d at %s" % [max_diff, str(max_at)])

	# Where the biggest change landed should itself be cold land -- ties the
	# measured diff back to the glacier/snow region rather than trusting that
	# a diff anywhere in the tile must mean the right thing changed.
	if max_at.x >= 0:
		var mgx: int = clampi(int(round(float(max_at.x) / float(w) * float(g.x))), 0, g.x - 1)
		var mgy: int = clampi(int(round(float(max_at.y) / float(h) * float(g.y))), 0, g.y - 1)
		var ms: Dictionary = _br.sample_cell(mgx, mgy)
		var mt: float = float(ms.get("temperature_c", 99.0))
		var mwater: String = ms.get("water", "land")
		_ok("the biggest change sits on cold land (ties the pixel diff to the ice/snow region)",
			mwater == "land" and mt < 5.0,
			"grid=(%d,%d) water=%s temperature_c=%.2f" % [mgx, mgy, mwater, mt])

	# --- 4. ice OFF: the companion assertion (task step 3) -----------------
	#
	# `lod_cache_key` folds in `glacial_snowline` UNCONDITIONALLY -- confirmed
	# by reading the function, not assumed -- so this still forces a fresh
	# `TileFields` build on both calls below. The claim under test is the
	# PIXEL one: `render.rs`'s `glacier_on`/`cryo` locals gate on
	# `ice_strength > 0.0`, so with ice off the fresh glacier field is built
	# but never read, and the two renders must be byte-identical despite the
	# cache having been busted.
	print("\n=== 4: ice OFF -- same snowline change must move NO pixel ===")
	var applied_off: int = _br.set_appearance({"ice_strength": 0.0})
	_ok("ice_strength off recognised (teeth check)", applied_off == 1, "applied=%d" % applied_off)

	wg.set_params({"passes.glacial_snowline": 0.55})
	var tex_c: Texture2D = _br.lod_synthesize_tile(0, 0, 0)
	_ok("tile (0,0,0) synthesised, ice off, snowline 0.55", tex_c != null)
	var data_c := tex_c.get_image().get_data() if tex_c != null else PackedByteArray()

	wg.set_params({"passes.glacial_snowline": 0.15})
	var tex_d: Texture2D = _br.lod_synthesize_tile(0, 0, 0)
	_ok("tile (0,0,0) synthesised, ice off, snowline 0.15", tex_d != null)
	var data_d := tex_d.get_image().get_data() if tex_d != null else PackedByteArray()

	_ok("with ice off, the same snowline change moves NO pixel (structurally does not need to bust the cache)",
		tex_c != null and tex_d != null and data_c == data_d,
		"%d bytes compared" % data_c.size())

	print("\nRESULT %d checks, %d failed" % [_checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
