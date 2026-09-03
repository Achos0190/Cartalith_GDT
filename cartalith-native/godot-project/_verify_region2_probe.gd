extends SceneTree

# Lane B, round 2: the refusal floor at an aspect norm_region's min-8 clamp
# actually permits, plus refusal atomicity.

var g: RefCounted
var fails := 0

func _ok(cond: bool, what: String) -> void:
	if cond: print("  PASS  %s" % what)
	else: fails += 1; print("  FAIL  %s" % what)

func _dims() -> Vector2i:
	var tex: ImageTexture = g.build_color_texture()
	return Vector2i(-1, -1) if tex == null else Vector2i(tex.get_width(), tex.get_height())

func _initialize() -> void:
	g = ClassDB.instantiate("WorldGen")

	print("== norm_region floors BOTH axes at 8 -- the tightest marquee the shell can make ==")
	g.generate_sized(11, 500.0, 512, 80)
	g.region_set(0.0, 0.0, 512.0, 2.0)   # asks for h=2
	var r: Dictionary = g.region_get()
	print("  region_set(0,0,512,2) -> w=%d h=%d  (norm_region min_h = 8)" % [int(r.get("w",0)), int(r.get("h",0))])
	_ok(int(r.get("h", 0)) == 8, "h clamped to 8, so h=2 is unreachable from region_set()")

	print("\n== the sub-4 refusal, at aspect 64 with tile_size 128 ==")
	# aspect = 512/8 = 64; tile_dims -> w=128, h=max(2, round(128/64)) = 2 -> refuse
	var d_before := _dims()
	var w_before: float = g.get_map_width_km()
	var ok: bool = g.region_new_world(128, false, 0.0, 0.0)
	print("  returned %s  error=\"%s\"" % [ok, g.region_new_world_error()])
	_ok(ok == false, "REFUSED a resample whose short axis would be 2")
	_ok(String(g.region_new_world_error()).contains("4 x 4"), "reason names the 4x4 floor")
	_ok(String(g.region_new_world_error()).contains("128 x 2"), "reason reports the offending dims 128 x 2")

	print("\n== refusal atomicity: the parent world must be untouched ==")
	_ok(_dims() == d_before, "grid unchanged (%s)" % [_dims()])
	_ok(absf(g.get_map_width_km() - w_before) < 1e-12, "map_width_km unchanged (%.4f)" % g.get_map_width_km())
	_ok(not g.region_get().is_empty(), "marquee STILL SET after the refusal")
	var lb: int = g.label_create(10.0, 10.0, "SURVIVOR")
	print("  (label id %d placed after the refusal -> world is still live)" % lb)
	_ok(g.label_list().size() == 1, "world still usable after the refusal")

	print("\n== bake-finalize lock is checked BEFORE anything mutates ==")
	# not driving a real bake here; just prove the ordering by reading the source
	var src := FileAccess.get_file_as_string("res://../crates/cartalith-godot/src/ops_bridge.rs")
	if src == "":
		print("  (source not readable from res://, skipping)")
	else:
		var i_check := src.find("bake.check(cartalith_engine::bake::Mutation::Generation)")
		var i_rel := src.find("self.release_world();")
		_ok(i_check > 0 and i_rel > i_check, "bake.check precedes release_world in the source")

	print("\n== a SECOND resample of the resampled world (chain) ==")
	g.generate_sized(13, 400.0, 256, 160)
	g.region_set(0.0, 0.0, 128.0, 80.0)
	var ok1: bool = g.region_new_world(128, false, 0.0, 0.0)
	var d1 := _dims(); var w1: float = g.get_map_width_km()
	print("  1st: ok=%s dims=%s width=%.4f" % [ok1, d1, w1])
	g.region_set(0.0, 0.0, 64.0, 40.0)
	var ok2: bool = g.region_new_world(128, false, 0.0, 0.0)
	var d2 := _dims(); var w2: float = g.get_map_width_km()
	print("  2nd: ok=%s dims=%s width=%.4f" % [ok2, d2, w2])
	_ok(ok1 and ok2, "chains without refusing")
	_ok(absf(w1 - 400.0 * 128.0 / 256.0) < 1e-9, "1st width = 400*128/256 = 200 (got %.4f)" % w1)
	_ok(absf(w2 - w1 * 64.0 / 128.0) < 1e-9, "2nd width = w1*64/128 = %.4f (got %.4f)" % [w1 * 0.5, w2])

	print("\n%s (%d failure%s)" % ["ALL PASS" if fails == 0 else "FAILURES", fails, "" if fails == 1 else "s"])
	quit(1 if fails > 0 else 0)
