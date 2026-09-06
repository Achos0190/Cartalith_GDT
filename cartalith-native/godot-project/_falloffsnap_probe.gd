extends SceneTree

## Lane Falloff — the parts of the row a `cargo test` cannot reach.
##
## `WorldGen` is a cdylib `GodotClass`, so `sculpt_set_globals`,
## `sculpt_set_grid_snap`, `sculpt_snap` and the `get_sculpt_globals_info()`
## enum row only exist behind a live Godot runtime (`MISTAKES.md`: "Test
## behaviour a unit test cannot reach → use a probe scene"). The engine-side
## coverage maths is unit-tested in `sculpt.rs`; this asserts the wiring.
##
## Asserts, does not merely print:
##   1. `get_sculpt_globals_info()` reports nine controls, exactly one of
##      which is an enum, and its `options` cover its whole ordinal range.
##   2. Each of the four falloff shapes round-trips through
##      `sculpt_set_globals` / `sculpt_get_globals`.
##   3. Each shape produces a **different committed field** — the end-to-end
##      version of the engine-side ridge measurement, through the real
##      stroke → stamp → commit path rather than through `apply_into`.
##   4. Grid snap: off by default, reported as an ABSENT key rather than a
##      zero, snaps captured points onto the lattice, and `sculpt_snap`
##      agrees with what `sculpt_add_point` stored (which is what makes the
##      stroke preview honest).

var _fail := 0

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  ok   ", what)
	else:
		_fail += 1
		print("  FAIL ", what)

func _init() -> void:
	var wg: WorldGen = WorldGen.new()
	wg.generate_sized(24601, 1400.0, 192, 128)

	# ---- 1. the control table ----
	print("[1] globals info")
	var info: Array = wg.get_sculpt_globals_info()
	_ok(info.size() == 9, "nine brush/noise controls, got %d" % info.size())
	var enums: Array = []
	for c in info:
		var d: Dictionary = c
		if String(d.get("type", "")) == "enum":
			enums.append(d)
		else:
			_ok(not d.has("options"), "%s is not an enum and carries no options" % d.get("key", "?"))
	_ok(enums.size() == 1, "exactly one enum control, got %d" % enums.size())
	if enums.is_empty():
		_finish()
		return
	var fo: Dictionary = enums[0]
	_ok(String(fo.get("key", "")) == "falloff", "the enum control is `falloff`")
	var opts: Array = Array(fo.get("options", []))
	_ok(opts.size() == int(fo.get("max", -1)) + 1,
		"%d options span the ordinal range 0..%d" % [opts.size(), int(fo.get("max", -1))])
	_ok(float(fo.get("default", -1.0)) == 0.0, "the default shape is index 0 (Smooth)")
	print("      options=", opts)

	# ---- 2. round trip ----
	print("[2] round trip")
	for i in opts.size():
		wg.sculpt_set_globals({"falloff": float(i)})
		var back: Dictionary = wg.sculpt_get_globals()
		_ok(float(back.get("falloff", -1.0)) == float(i),
			"%s (index %d) reads back" % [String(opts[i]), i])
	wg.sculpt_set_globals({"falloff": 0.0})

	# ---- 3. each shape draws a different profile ----
	#
	# One Mountains stroke per shape, committed, then a cross-section read
	# back cell by cell through `sample_cell` — the real stroke → stamp →
	# commit path, not `apply_into` in isolation.
	#
	# Mountains rather than Ridge on purpose: Ridge's own gaussian is
	# narrower than the coverage ramp and swallows the difference, which
	# `sculpt.rs::ridge_is_immune_to_the_falloff_because_its_own_gaussian_is_narrower`
	# pins as a measured fact rather than a suspicion. Threshold is 1/255 of
	# full height, one 8-bit code point, for the same reason the engine-side
	# test uses it: below that the two controls draw the same picture.
	print("[3] committed profiles differ per shape")
	var profiles: Array = []
	for i in opts.size():
		var w2: WorldGen = WorldGen.new()
		w2.generate_sized(24601, 1400.0, 192, 128)
		w2.sculpt_set_feature("mountains")
		w2.sculpt_set_globals({"falloff": float(i), "brush_size": 16.0, "edge_noise": 0.0})
		w2.sculpt_begin_stroke()
		for k in 9:
			w2.sculpt_add_point(60.0 + float(k) * 8.0, 64.0)
		w2.sculpt_end_stroke()
		w2.sculpt_commit("falloff probe")
		# Perpendicular to the stroke, through its midpoint: 96..64+40, so
		# the section crosses the brush's whole coverage band.
		var prof := PackedFloat64Array()
		for y in range(24, 105):
			var c: Dictionary = w2.sample_cell(96, y)
			prof.append(float(c.get("elevation", 0.0)))
		profiles.append(prof)
	# A control: the section must actually contain the stamp, or every
	# comparison below would pass on flat ground and prove nothing.
	var flat_lo := 9.9
	var flat_hi := -9.9
	for v in profiles[0]:
		flat_lo = minf(flat_lo, float(v))
		flat_hi = maxf(flat_hi, float(v))
	_ok(flat_hi - flat_lo > 0.05,
		"the section crosses the stamp (relief %.4f over %d cells)" % [flat_hi - flat_lo, profiles[0].size()])
	for a in profiles.size():
		for b in range(a + 1, profiles.size()):
			var peak := 0.0
			for k in profiles[a].size():
				peak = maxf(peak, absf(float(profiles[a][k]) - float(profiles[b][k])))
			_ok(peak > 1.0 / 255.0,
				"%s vs %s: peak %.5f of height (> 1/255)" % [String(opts[a]), String(opts[b]), peak])

	# ---- 4. grid snap ----
	print("[4] grid snap")
	var steps: PackedFloat64Array = wg.get_sculpt_grid_snap_steps()
	_ok(steps.size() > 0, "the engine offers a snap ladder (%d steps)" % steps.size())
	_ok(not wg.sculpt_get_grid_snap().has("step"),
		"snapping is OFF by default, and reported as an absent key not a 0")

	# Off: the point is stored exactly as given.
	wg.sculpt_begin_stroke()
	wg.sculpt_add_point(10.4, 31.6)
	var raw: PackedFloat64Array = wg.sculpt_snap(10.4, 31.6)
	_ok(raw[0] == 10.4 and raw[1] == 31.6, "snap off leaves a point untouched")
	wg.sculpt_cancel_stroke()

	for s in steps:
		var step := float(s)
		var got := float(wg.sculpt_set_grid_snap(step))
		_ok(got == step, "set_grid_snap(%s) took effect" % step)
		var live: Dictionary = wg.sculpt_get_grid_snap()
		_ok(live.has("step") and float(live["step"]) == step, "reads back as {step: %s}" % step)
		var p: PackedFloat64Array = wg.sculpt_snap(10.4, 31.6)
		_ok(fmod(p[0], step) == 0.0 and fmod(p[1], step) == 0.0,
			"(10.4, 31.6) -> (%s, %s) is on the %s-cell lattice" % [p[0], p[1], step])
		# Idempotence — the preview readback is snapped again by
		# `sculpt_add_point`, so a second pass must be a no-op or the two
		# would drift apart over a stroke.
		var q: PackedFloat64Array = wg.sculpt_snap(p[0], p[1])
		_ok(q[0] == p[0] and q[1] == p[1], "snapping an already-snapped point is a no-op")

	_ok(float(wg.sculpt_set_grid_snap(3.7)) == 0.0, "a step off the ladder is refused")
	_ok(not wg.sculpt_get_grid_snap().has("step"), "and leaves snapping off")
	_ok(float(wg.sculpt_set_grid_snap(0.0)) == 0.0, "0 is how a shell says off")

	# A snapped stroke really does land on the lattice: two nearby taps that
	# would be distinct unsnapped collapse onto one cell at an 8-cell step.
	wg.sculpt_set_grid_snap(8.0)
	var a1: PackedFloat64Array = wg.sculpt_snap(63.4, 63.4)
	var a2: PackedFloat64Array = wg.sculpt_snap(65.9, 65.9)
	_ok(a1[0] == a2[0] and a1[1] == a2[1],
		"63.4 and 65.9 both land on %s at an 8-cell step" % a1[0])

	# ---- 5. the touch floor, at TABLET density ----
	#
	# `MISTAKES.md`: `phone_fit()` only runs on phone, so a control that is
	# fine on pointer and fine on phone can still be under the floor on a
	# tablet -- which is exactly how a 22 px field shipped. Both new controls
	# are `DccWidgets.choice`, so this measures a real one at both densities
	# rather than reading the widget factory and believing it.
	print("[5] touch floor at tablet density")
	var floor_px := 0
	for tablet in [false, true]:
		DccTheme.set_touch(tablet)
		var host := VBoxContainer.new()
		root.add_child(host)
		var falloff_ob := DccWidgets.choice(host, "Falloff", opts, 0, func(_i: int) -> void: pass)
		var snap_ob := DccWidgets.choice(host, "Grid snap", ["Off", "1 cell"], 0, func(_i: int) -> void: pass)
		host.size = Vector2(320, 200)
		await process_frame
		var fh := falloff_ob.get_combined_minimum_size().y
		var sh := snap_ob.get_combined_minimum_size().y
		if tablet:
			floor_px = DccTheme.role_px("btn_min_h")
			_ok(fh >= float(floor_px), "Falloff row %d px >= the %d px touch floor" % [int(fh), floor_px])
			_ok(sh >= float(floor_px), "Grid snap row %d px >= the %d px touch floor" % [int(sh), floor_px])
		else:
			print("      pointer density: Falloff %d px, Grid snap %d px" % [int(fh), int(sh)])
		host.queue_free()
	DccTheme.set_touch(false)

	_finish()

func _finish() -> void:
	print("")
	if _fail == 0:
		print("PROBE PASS")
	else:
		print("PROBE FAIL: %d assertion(s)" % _fail)
	quit(0 if _fail == 0 else 1)
