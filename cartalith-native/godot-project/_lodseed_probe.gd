extends SceneTree

## `OUTSTANDING_WORK.md` §2.11, "The stored LOD pyramid is written and never
## read back". End to end through the real `#[func]`s, which the Rust tests
## in `lod_worker.rs` cannot reach (`WorldGen` is a cdylib `GodotClass`).
##
## RUN WINDOWED, not `--headless`: section C reads tile pixels back through
## `ImageTexture.get_image()`, and the headless dummy renderer keeps none.
##
## What it proves, in order:
##  A. A project whose tiles were drawn from the SAME world the reopened session
##     draws is SEEDED: every stored tile asked for is served from the archive
##     (`seeded_served` == tiles asked) and NOTHING is synthesised
##     (`synthesized` == 0). This is the check that fails if a reopened project
##     re-synthesises tiles it carries.
##  B. The worker path (`lod_prepare` / `lod_request_tile` /
##     `lod_take_ready_tiles`) serves a seeded tile the same way.
##  C. The seeded tiles are byte-identical to what synthesis draws for the
##     same reopened world -- a second session that opens the same world with
##     no stored tiles synthesises every one of them, and they are compared.
##  D. A GENERATED world's stored tiles ARE seeded into its reopened self
##     (owner Ruling AR, 2026-09-24). Until then they were refused, correctly:
##     the reopened world had no flow and no lithology (`lod_snapshot_inputs`'
##     `Loaded` arm), so it drew different tiles. A project now carries the
##     world substrate (`SAVEFILE_COMPAT.md` §8.3) and reopens as the complete
##     `WorldState`, so every sample tile is served from the archive, nothing
##     is synthesised, and a no-seed session over the same reopened world
##     draws exactly the stored pixels.
##  E. A different colour space refuses the seed.

const LEVEL_MAX := 4  # lod_bridge::SAVE_PYRAMID_MAX_LEVEL

var fails := 0

func _fail(msg: String) -> void:
	print("  FAIL: ", msg)
	fails += 1

func _tiles(wg: WorldGen) -> Array:
	var out := []
	for z in range(LEVEL_MAX + 1):
		var n: int = wg.lod_tiles_per_axis(z)
		for col in range(n):
			for row in range(n):
				out.append(Vector3i(z, col, row))
	return out

func _bytes(wg: WorldGen, t: Vector3i) -> PackedByteArray:
	var tex: Texture2D = wg.lod_synthesize_tile(t.x, t.y, t.z)
	if tex == null:
		return PackedByteArray()
	var img := tex.get_image()
	return img.get_data() if img != null else PackedByteArray()

func _save(wg: WorldGen, path: String, tiles: bool) -> bool:
	var r: Dictionary = wg.project_save_with_documents(path, {}, PackedByteArray(), tiles)
	if not bool(r.get("ok", false)):
		_fail("save %s -> %s" % [path, r.get("error", "?")])
		return false
	return true

func _open(path: String) -> WorldGen:
	var wg := WorldGen.new()
	var o: Dictionary = wg.project_open(path)
	if not bool(o.get("ok", false)):
		_fail("open %s -> %s" % [path, o.get("error", "?")])
	return wg

func _init() -> void:
	var dir := OS.get_user_data_dir()
	var p_gen := dir.path_join("_lodseed_generated_tiles.ctl")
	var p_plain := dir.path_join("_lodseed_plain.ctl")
	var p_loaded_tiles := dir.path_join("_lodseed_loaded_tiles.ctl")

	var gen := WorldGen.new()
	gen.generate_sized(24601, 400.0, 96, 64)
	if not (_save(gen, p_gen, true) and _save(gen, p_plain, false)):
		quit(1)
		return

	# The stored tiles for A-C are drawn by a session that has the reopened
	# world -- open the plain save, then save it again with tiles.
	var loaded := _open(p_plain)
	if not _save(loaded, p_loaded_tiles, true):
		quit(1)
		return

	# -- A. reopened, seeded, nothing synthesised --------------------------
	var seeded := WorldGen.new()
	var o: Dictionary = seeded.project_open(p_loaded_tiles)
	var held: int = int(o.get("lod_tiles_held", -1))
	var asked := _tiles(seeded)
	print("  A. held %d stored tiles; asking for %d (levels 0..%d)" % [held, asked.size(), LEVEL_MAX])
	if held != asked.size():
		_fail("the archive should carry every tile of levels 0..%d (%d), held %d" % [LEVEL_MAX, asked.size(), held])
	var seeded_bytes := {}
	for t in asked:
		seeded_bytes[t] = _bytes(seeded, t)
	var s: Dictionary = seeded.lod_worker_stats()
	print("     seeded_served=%d synthesized=%d" % [s.get("seeded_served", -1), s.get("synthesized", -1)])
	if int(s.get("synthesized", -1)) != 0:
		_fail("a reopened project re-synthesised %d tiles it carries" % int(s.get("synthesized", -1)))
	if int(s.get("seeded_served", -1)) != asked.size():
		_fail("only %d of %d tiles were served from the archive" % [int(s.get("seeded_served", -1)), asked.size()])
	# A tile the archive does not carry is still synthesised.
	var deep := _bytes(seeded, Vector3i(LEVEL_MAX + 1, 0, 0))
	if deep.is_empty() or int(seeded.lod_worker_stats().get("synthesized", -1)) != 1:
		_fail("a level-%d tile (not stored) should be synthesised exactly once" % (LEVEL_MAX + 1))

	# -- B. the worker path ------------------------------------------------
	var state := 0
	for i in range(3000):
		state = seeded.lod_prepare()
		if state != 2:
			break
		OS.delay_msec(2)
	if state != 1:
		_fail("lod_prepare never became ready (state %d)" % state)
	else:
		var before: int = int(seeded.lod_worker_stats().get("synthesized", -1))
		if not seeded.lod_request_tile(2, 1, 1):
			_fail("lod_request_tile refused a seeded tile")
		var ready: Array = seeded.lod_take_ready_tiles(8)
		var after: int = int(seeded.lod_worker_stats().get("synthesized", -1))
		print("  B. worker path: %d ready at once, synthesized %d -> %d" % [ready.size(), before, after])
		if ready.size() != 1 or after != before:
			_fail("the worker path should hand a seeded tile over at once with nothing synthesised")
		elif (ready[0]["tex"] as Texture2D).get_image().get_data() != seeded_bytes[Vector3i(2, 1, 1)]:
			_fail("the worker path's seeded tile differs from the synchronous path's")

	# -- C. seeded == synthesised, byte for byte ---------------------------
	var fresh := _open(p_plain)
	var differ := 0
	var empty := 0
	for t in asked:
		var want := _bytes(fresh, t)
		if want.is_empty() or (seeded_bytes[t] as PackedByteArray).is_empty():
			empty += 1
		elif want != seeded_bytes[t]:
			differ += 1
	var fs: Dictionary = fresh.lod_worker_stats()
	print("  C. %d tiles compared against a no-seed session (it synthesised %d): %d differ, %d empty" % [asked.size(), fs.get("synthesized", -1), differ, empty])
	if int(fs.get("synthesized", -1)) != asked.size():
		_fail("the comparison session should have synthesised every tile")
	if differ != 0 or empty != 0:
		_fail("seeded tiles are not what synthesis draws")

	# -- D. a generated world's tiles over its reopened self --------------
	var from_gen := _open(p_gen)
	var sample := [Vector3i(0, 0, 0), Vector3i(1, 0, 0), Vector3i(2, 1, 2), Vector3i(3, 5, 3)]
	for t in sample:
		_bytes(from_gen, t)
	var gs: Dictionary = from_gen.lod_worker_stats()
	print("  D. generated-world pyramid reopened: held %d, served %d, synthesized %d" % [gs.get("seeded_held", -1), gs.get("seeded_served", -1), gs.get("synthesized", -1)])
	if int(gs.get("seeded_held", 0)) <= 0:
		_fail("the generated world's pyramid should be held (it is the same heightmap)")
	if int(gs.get("seeded_served", -1)) != sample.size() or int(gs.get("synthesized", -1)) != 0:
		_fail("a generated world's reopened self should serve all %d sample tiles from the archive and synthesise none" % sample.size())
	# The stored pixels against what the reopened world draws with no seed.
	var drawn := _open(p_plain)
	var zr := ZIPReader.new()
	if zr.open(p_gen) == OK:
		var moved := 0
		var total := 0
		for t in sample:
			var stored: PackedByteArray = zr.read_file("cartography/tiles/%d/%d/%d.u8" % [t.x, t.y, t.z])
			var live := _bytes(drawn, t)
			var n := mini(stored.size() / 3, live.size() / 4)
			for i in range(n):
				total += 1
				if stored[i * 3] != live[i * 4] or stored[i * 3 + 1] != live[i * 4 + 1] or stored[i * 3 + 2] != live[i * 4 + 2]:
					moved += 1
		zr.close()
		print("     stored vs reopened-synthesised: %d of %d pixels differ over %d sample tiles" % [moved, total, sample.size()])
		if total == 0:
			_fail("no pixels compared")
		elif moved != 0:
			_fail("the reopened world draws %d pixels differently from the world that stored them" % moved)

	# -- E. another colour space refuses the seed --------------------------
	var p3 := WorldGen.new()
	p3.project_open(p_loaded_tiles)
	if p3.has_method("set_color_space"):
		if not p3.set_color_space("Display P3"):
			_fail("set_color_space refused \"Display P3\"")
		_bytes(p3, Vector3i(1, 0, 0))
		var ps: Dictionary = p3.lod_worker_stats()
		print("  E. display-p3: served %d, synthesized %d" % [ps.get("seeded_served", -1), ps.get("synthesized", -1)])
		if int(ps.get("seeded_served", -1)) != 0:
			_fail("a tile stored in sRGB was served into a Display P3 session")
	else:
		print("  E. skipped: no set_color_space binding")

	# -- F. a re-save writes the held tiles back (2026-09-24) ---------------
	# Open the seeded project and save it again with tiles, asking for no tile
	# in between: `LodWorker::pyramid_masks` must reuse every held mask, so
	# nothing is synthesised and every stored entry is byte-identical to the
	# archive it was opened from. Reads zip bytes only, so this section is
	# valid headless as well.
	var p_resave := dir.path_join("_lodseed_resaved.ctl")
	var re := _open(p_loaded_tiles)
	if _save(re, p_resave, true):
		var rs: Dictionary = re.lod_worker_stats()
		print("  F. re-save: seeded_served=%d synthesized=%d" % [rs.get("seeded_served", -1), rs.get("synthesized", -1)])
		if int(rs.get("synthesized", -1)) != 0:
			_fail("a re-save synthesised %d tiles the project already held" % int(rs.get("synthesized", -1)))
		if int(rs.get("seeded_served", -1)) != asked.size():
			_fail("a re-save reused %d of %d held tiles" % [int(rs.get("seeded_served", -1)), asked.size()])
		var za := ZIPReader.new()
		var zb := ZIPReader.new()
		if za.open(p_loaded_tiles) == OK and zb.open(p_resave) == OK:
			var same := 0
			for t in asked:
				var e := "cartography/tiles/%d/%d/%d.u8" % [t.x, t.y, t.z]
				var a := za.read_file(e)
				if not a.is_empty() and a == zb.read_file(e):
					same += 1
			print("     %d of %d stored tiles byte-identical after the re-save" % [same, asked.size()])
			if same != asked.size():
				_fail("the re-saved pyramid differs from the one it was opened with")
		else:
			_fail("could not reopen the two archives to compare")
		za.close()
		zb.close()

	print("lod-seed probe: ", "PASS" if fails == 0 else "%d FAILURE(S)" % fails)
	quit(1 if fails > 0 else 0)
