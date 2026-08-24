extends SceneTree

## Headless save/reload probe for `GUI_GAP_REGISTER.md` FI-01, run through the
## real GDExtension rather than through `cargo test`: generate a world, write
## it with `WorldGen::save_project`, open the file with a *fresh* `WorldGen`,
## and check what actually came back.
##
## Three independent checks, because each one can pass while another fails:
##
## 1. **The file holds the world that was generated.** `heightmap.f32` is
##    decoded here, in GDScript, and compared against the same cells read
##    through `sample_cell` on the live world — so the writer is checked
##    against the engine's own readout, not against the reader that shares
##    its code.
## 2. **Nothing is lost through the loader.** Save, reopen, save again, and
##    compare all six field entries byte-for-byte. A field the loader dropped
##    or reordered cannot survive that.
## 3. **The generation parameters come back**, all of them, plus the seed,
##    scale and dimensions the shell reads.
##
## Deliberately does *not* compare rendered pixels: `build_color_texture`
## renders a loaded save through a different path (no flow field, no
## lithology — the format stores neither, see `load_save`'s own comments), so
## a pixel diff would measure a documented property of the loader rather than
## anything about the writer. It does assert the reopened world still renders.
##
## Same shape and lifetime as `smoke_test.gd`: a `SceneTree` script, non-zero
## exit on failure, no scene involved.

const ENTRIES := ["heightmap.f32", "temperature.f32", "rainfall.f32",
	"volcanic_field.f32", "impact_field.f32", "strahler_order.bin"]

var _fails: Array[String] = []

func _check(ok: bool, what: String) -> void:
	if ok:
		print("  ok   ", what)
	else:
		print("  FAIL ", what)
		_fails.append(what)

func _entries_of(path: String) -> Dictionary:
	var out := {}
	var z := ZIPReader.new()
	if z.open(path) != OK:
		return out
	for name in z.get_files():
		out[String(name)] = z.read_file(String(name))
	z.close()
	return out

func _initialize() -> void:
	var wg: WorldGen = WorldGen.new()
	if not wg.has_method("save_project"):
		print("FAIL: this extension has no save_project")
		quit(1)
		return

	## Parameters deliberately off their defaults, so the parameter block in
	## the file is proven real rather than the defaults coming back.
	wg.set_params({"tect.plates": 9, "climate.lat_n": 62.0, "volc.count": 24})
	wg.generate_sized(24601, 640.0, 48, 30)

	var gw: int = wg.get_width()
	var gh: int = wg.get_height()
	var before := {"w": gw, "h": gh, "seed": wg.get_seed(), "km": wg.get_map_width_km()}
	var before_params: Dictionary = wg.get_params()

	var path := OS.get_user_data_dir().path_join("_save_probe.zip")
	_check(wg.save_project(path), "save_project wrote %s" % path)
	_check(FileAccess.file_exists(path), "the file exists")

	# -- 1. the file holds the world that was generated ------------------------
	var written := _entries_of(path)
	for name in ENTRIES:
		_check(written.has(name), "the archive carries %s" % name)
	_check(written.has("params.json"), "the archive carries params.json")
	var heights: PackedByteArray = written.get("heightmap.f32", PackedByteArray())
	_check(heights.size() == gw * gh * 4,
		"heightmap.f32 is %d bytes, expected %d (gw*gh*4, no header)" % [heights.size(), gw * gh * 4])

	var drift := 0
	var relief := 0
	if heights.size() == gw * gh * 4:
		for y in gh:
			for x in gw:
				var stored := heights.decode_float((y * gw + x) * 4)
				var live := float(wg.sample_cell(x, y).get("elevation", -1.0))
				if stored > 0.001:
					relief += 1
				## Exact: `.f32` is a byte dump of the same `f32` the sampler
				## rounds a `f64` readout from, so the only allowance is that
				## one round trip through 32 bits.
				if not is_equal_approx(stored, float(str(live).to_float())) and absf(stored - live) > 1e-6:
					drift += 1
	_check(relief > 100, "the saved heightmap is a real world (%d cells above zero)" % relief)
	_check(drift == 0, "%d of %d saved cells differ from the live world" % [drift, gw * gh])

	# -- 2. nothing is lost through the loader ---------------------------------
	var reopened: WorldGen = WorldGen.new()
	_check(reopened.load_save(path), "a fresh WorldGen reopened it")

	var again := OS.get_user_data_dir().path_join("_save_probe_2.zip")
	_check(reopened.save_project(again), "the reopened world saved again")
	var rewritten := _entries_of(again)
	for name in ENTRIES:
		_check(written.get(name, PackedByteArray()) == rewritten.get(name, PackedByteArray()),
			"%s survived save -> load -> save byte-for-byte" % name)

	# -- 3. the settings came back --------------------------------------------
	_check(reopened.get_width() == before["w"], "width %d == %d" % [reopened.get_width(), before["w"]])
	_check(reopened.get_height() == before["h"], "height %d == %d" % [reopened.get_height(), before["h"]])
	_check(reopened.get_seed() == before["seed"], "seed %d == %d" % [reopened.get_seed(), before["seed"]])
	_check(is_equal_approx(reopened.get_map_width_km(), before["km"]), "map width km")

	var after_params: Dictionary = reopened.get_params()
	var drifted: Array[String] = []
	for key in before_params:
		if str(before_params[key]) != str(after_params.get(key, "<missing>")):
			drifted.append("%s: %s -> %s" % [key, before_params[key], after_params.get(key, "<missing>")])
	_check(drifted.is_empty(), "all %d parameters restored (%s)" % [before_params.size(), ", ".join(drifted)])
	_check(int(after_params.get("tect.plates", 0)) == 9, "tect.plates came back as 9")
	_check(is_equal_approx(float(after_params.get("climate.lat_n", 0.0)), 62.0), "climate.lat_n came back as 62")

	## The reference app reads `state.tect.seed` and `state.mapWidthKm`; the
	## Open-project gallery reads the seed out of the same place.
	var parsed = JSON.parse_string(written["params.json"].get_string_from_utf8())
	_check(parsed is Dictionary, "params.json parses")
	if parsed is Dictionary:
		var state: Dictionary = (parsed as Dictionary).get("state", {})
		_check(int((state.get("tect", {}) as Dictionary).get("seed", 0)) == 24601, "state.tect.seed is the reference's own key")
		_check(int((state.get("tect", {}) as Dictionary).get("plates", 0)) == 9, "state.tect.plates uses the reference's vocabulary")
		_check(is_equal_approx(float(state.get("mapWidthKm", 0.0)), 640.0), "state.mapWidthKm")
		_check(state.has("cartalith"), "state.cartalith carries this port's own parameter block")

	var after_img: Image = (reopened.build_color_texture() as ImageTexture).get_image()
	_check(after_img.get_size() == Vector2i(before["w"], before["h"]),
		"the reopened world renders at %s" % after_img.get_size())

	DirAccess.remove_absolute(path)
	DirAccess.remove_absolute(again)

	if _fails.is_empty():
		print("PASS")
		quit(0)
	else:
		print("FAILED: ", _fails)
		quit(1)
