extends SceneTree
## `OUTSTANDING_WORK.md` §2.1, end to end: does a per-settlement place-editor
## override actually change the town `urban_layouts()` draws?
##
## `urban_bridge.rs` called `urban_adapter::settlement_layout()` -- the entry
## point that supplies `PlaceOverrides::default()` -- so `umWalls`/`umAge` were
## stored by ED-03, persisted by `project_bridge`, read by
## `civ_military_bridge::defences`, and then discarded on the way to the
## layout. This drives the real `#[func]` pair a player's edit travels through
## (`civ_edit_settlement` then `urban_layouts`) and asserts the **geometry**
## changed, not that an argument is threaded: a probe that only showed the
## parameter arriving would prove plumbing, not delivery.
##
## `WorldGen` is a cdylib `GodotClass` and cannot be constructed in a unit
## test, which is why this is a probe (`MISTAKES.md`).
##
##   Godot_v4.7.1-stable_win64.exe --headless --path . --script _umoverride_probe.gd

var fails := 0

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

func _one(gen: WorldGen, idx: int) -> Dictionary:
	var a: Array = gen.urban_layouts(PackedInt32Array([idx]))
	return a[0] if a.size() > 0 else {}

func _init() -> void:
	var gen := WorldGen.new()
	gen.generate_sized(24601, 812.0, 96, 64)
	gen.recompute_civilisation()
	var places: Array = gen.get_settlements()
	print("== %d settlements ==" % places.size())

	var all := PackedInt32Array()
	for i in places.size():
		all.append(i)
	var base: Array = gen.urban_layouts(all)
	print("  urban_layouts() returned %d layouts" % base.size())
	if base.size() == 0:
		print("  FAIL  no layouts at all -- nothing to override")
		quit(1)
		return

	# `umWalls` is tri-state (`set_walls`: < 0 auto, 0 off, else on), so all
	# three states are exercised on one town rather than hunting the world for
	# a fixture that happens to sit on each rung. Index 0 whatever its verdict:
	# the assertions below read the baseline rather than assuming it.
	var idx := int(base[0]["index"])
	var auto := _one(gen, idx)
	var auto_spec := String(auto["wall_spec"])
	var auto_stage := String(auto["stages"][5])
	print("  [%d] ladder's own verdict: wall_spec \"%s\", ring %s"
		% [idx, auto_spec, "present" if auto.has("wall_ring") else "absent"])

	# -- umWalls = off --------------------------------------------------------
	_ok(gen.civ_edit_settlement(idx, {"walls": 0}), "[%d] editor accepted umWalls = off" % idx)
	var off := _one(gen, idx)
	_ok(String(off["wall_spec"]) == "none" and not bool(off["walls"]) and not off.has("wall_ring"),
		"[%d] wall_spec \"%s\" -> \"%s\", circuit gone from the layout"
			% [idx, auto_spec, off["wall_spec"]])

	# -- umWalls = on ---------------------------------------------------------
	_ok(gen.civ_edit_settlement(idx, {"walls": 1}), "[%d] editor accepted umWalls = on" % idx)
	var on := _one(gen, idx)
	_ok(String(on["wall_spec"]) == "stone",
		"[%d] wall_spec \"none\" -> \"%s\" (um_wall_spec's Some(true) rung)" % [idx, on["wall_spec"]])
	_ok(bool(on["walls"]) and on.has("wall_ring"), "[%d] a circuit was actually built" % idx)
	var ring: PackedVector2Array = on.get("wall_ring", PackedVector2Array())
	var gates: PackedVector2Array = on.get("wall_gates", PackedVector2Array())
	print("    ring: %d points, %d gate points, style \"%s\"" % [ring.size(), gates.size(), on.get("wall_style", "-")])
	_ok(ring.size() >= 3, "[%d] the ring is real geometry, not an empty array" % idx)
	# Delivery, not plumbing: `generate()` reports a different town.
	_ok(String(off["stages"][5]) != String(on["stages"][5]),
		"[%d] the buildWall stage line moved with the override: \"%s\" -> \"%s\""
			% [idx, off["stages"][5], on["stages"][5]])

	# -- back to auto ---------------------------------------------------------
	gen.civ_edit_settlement(idx, {"walls": -1})
	var back := _one(gen, idx)
	_ok(String(back["wall_spec"]) == auto_spec and String(back["stages"][5]) == auto_stage,
		"[%d] umWalls back to auto restores the ladder's own \"%s\"" % [idx, auto_spec])

	# -- specialisation, the third field that travels on the same struct ------
	# "mining" is the one branch `um_place_context_with` scans the resource
	# rasters on (`um_ore_bearing`), so this also exercises the batch-built
	# `PlaceOverrides::resources` beside it. `economy` reaching
	# `assign_districts` is what a changed district assignment means.
	_ok(gen.civ_edit_settlement(idx, {"specialisation": "mining"}),
		"[%d] editor accepted specialisation = mining" % idx)
	var mined := _one(gen, idx)
	var d0: PackedStringArray = auto.get("parcel_district", PackedStringArray())
	var d1: PackedStringArray = mined.get("parcel_district", PackedStringArray())
	print("    parcel districts: %d before, %d after" % [d0.size(), d1.size()])
	_ok(d0.size() > 0 and Array(d0) != Array(d1),
		"[%d] the district assignment moved with the specialisation" % idx)
	gen.civ_edit_settlement(idx, {"specialisation": "none"})

	# -- umAge, the other half of the row -------------------------------------
	var target := idx
	var age_before := float(auto["settlement_age_years"])
	_ok(gen.civ_edit_settlement(target, {"age": 950}), "[%d] editor accepted umAge = 950" % target)
	var aged := _one(gen, target)
	_ok(is_equal_approx(float(aged["settlement_age_years"]), 950.0),
		"[%d] settlement_age_years %.1f -> %.1f (was _umInferAge's guess)"
			% [target, age_before, float(aged["settlement_age_years"])])
	gen.civ_edit_settlement(target, {"age": -1})
	_ok(is_equal_approx(float(_one(gen, target)["settlement_age_years"]), age_before),
		"[%d] umAge back to auto restores _umInferAge's %.1f" % [target, age_before])

	print("\n%s (%d failure%s)" % ["ALL PASS" if fails == 0 else "FAILURES", fails, "" if fails == 1 else "s"])
	quit(1 if fails > 0 else 0)
