extends Node
## Does `Center landmasses` destroy the rivers?
##
##   godot --headless --path . _centerrivers_probe.tscn
##
## Found while investigating the owner's report that wrap-around / centre
## landmasses "create weird lines/streaks". It is not a streak: an adversarial
## verification pass found that `cartalith_engine::center::center_landmasses`
## does `ws.channels.take()` (center.rs:101), and `ws.channels` is written in
## exactly ONE place -- `generate_terrain` (engine/src/lib.rs:1336). Nothing
## rebuilds it, so the channel topology is gone until the next Generate.
##
## The reference does not have this. `centerLandmasses` nulls `_riverNet`, and
## `renderNow`'s own branch rebuilds it: `if(!_riverNet) _riverNet =
## buildRiverNetwork(...)`. The port drops and never rebuilds.
##
## GeoJSON is the crispest binary witness: `geojson_bridge.rs:84` gates river
## features on `(Some(order), Some(ch))`, so a dropped `channels` means exactly
## zero river features, with `stream_order` still present and correctly shifted.

var _fail := 0

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _count_rivers(gj: String) -> int:
	if gj.strip_edges() == "":
		return -1
	var parsed = JSON.parse_string(gj)
	if typeof(parsed) != TYPE_DICTIONARY:
		return -1
	var n := 0
	for f in parsed.get("features", []):
		if String((f as Dictionary).get("properties", {}).get("layer", "")) == "river":
			n += 1
	return n

func _ready() -> void:
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var wg: Object = ClassDB.instantiate("WorldGen")
	## A world whose landmass is already centred returns `offset == 0` and the
	## centring is a genuine no-op, which proves nothing either way. Hunt for a
	## seed that actually shifts before measuring anything -- the first run of
	## this probe drew seed 24601, got `offset=0`, and would have "passed"
	## while exercising none of the code under test.
	var seed := 0
	var probe_r: Dictionary = {}
	for s in [24601, 7, 1234, 99, 31337, 555, 8080, 42, 2026, 111]:
		wg.set_params({"tect.plates": 9, "world": true})
		wg.generate_sized(s, 640.0, 192, 128)
		var t: Dictionary = wg.center_landmasses()
		if int(t.get("offset", 0)) != 0:
			seed = s
			probe_r = t
			break
	if seed == 0:
		print("[FATAL] no seed produced a non-zero centring offset; cannot test.")
		get_tree().quit(1); return
	print("[SEED] ", seed, " shifts by offset=", probe_r.get("offset"),
		" -- regenerating it cleanly to measure before/after")

	wg.set_params({"tect.plates": 9, "world": true})
	wg.generate_sized(seed, 640.0, 192, 128)
	print("[GEN] 192x128 seed ", seed, " @ 640 km")

	var before := _count_rivers(wg.export_geojson())
	print("\n=== before Center landmasses ===")
	_ok("river features exist to begin with", before > 0, true)
	print("  info river features: ", before)

	var r: Dictionary = wg.center_landmasses()
	print("\n=== center_landmasses ===")
	print("  info ok=", r.get("ok"), " offset=", r.get("offset"),
		" channels_dropped=", r.get("channels_dropped"))

	var after := _count_rivers(wg.export_geojson())
	print("\n=== after Center landmasses ===")
	print("  info river features: ", after)
	## The assertion is written the way the FIXED build must answer, so this
	## probe fails today and passes once the rebuild lands. A probe that
	## asserted the broken value would have to be edited to prove the fix,
	## which is how a regression test becomes a rubber stamp.
	_ok("rivers survive centring", after > 0, true)
	if int(r.get("offset", 0)) == 0:
		print("  NOTE offset was 0 -- centring was a no-op, so this run proves nothing.")

	print("\n_centerrivers_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
