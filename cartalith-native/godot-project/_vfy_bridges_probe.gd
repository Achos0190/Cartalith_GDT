extends Node
## VERIFIER probe for Lane A: does `urban_layouts()` really carry the crossings?
##
## Nobody on Lane A ran a Godot probe (the lane brief forbade GDScript), so
## this is the first time the new Rust keys are read from the shell at all.
## Graded against `target/debug/cartalith_godot.dll`, whose mtime the harness
## prints beside the two `.rs` mtimes.

const SEED := 40417
const DEDUP_M := 80.0   ## `water.rs::detect_river_crossings`' own literal, carried

var _fail := 0

func _ok(name: String, cond: bool, note: String = "") -> void:
	if not cond:
		_fail += 1
	print(("  ok   " if cond else "  FAIL ") + name + ("   -- " + note if note != "" else ""))

func _ready() -> void:
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	for _i in 3:
		await get_tree().process_frame
	DccTheme.apply_theme(false)   ## force LIGHT (this machine boots light); no
	                              ## check below is palette-bound, but forcing
	                              ## it costs nothing and removes the variable.

	var b = app.bridge
	_ok("the cdylib exports urban_layouts", b.world_gen.has_method("urban_layouts"))

	b.generate({"seed": SEED, "width_km": 2000.0, "grid_w": 256,
		"grid_h": 192, "archetype": "", "villages": true, "sea_level": 0.45})
	var waited := 0
	while b.generating and waited < 6000:
		await get_tree().process_frame
		waited += 1
	for _j in 8:
		await get_tree().process_frame
	var settlements: Array = b.settlements()
	_ok("world generated", settlements.size() > 0, "%d settlements" % settlements.size())

	# `settlement_diagnostics` is the cheap pass (no town generation), so it
	# picks the river towns and `urban_layouts` -- which runs a full generate()
	# per settlement -- is asked for a handful rather than sixty.
	var all_idx := PackedInt32Array()
	for i in settlements.size():
		all_idx.append(i)
	var diags: Array = b.settlement_diagnostics(all_idx)
	var idx := PackedInt32Array()
	var kinds := {}
	for j in diags.size():
		kinds[String((diags[j] as Dictionary).get("site_kind", "?"))] = 			int(kinds.get(String((diags[j] as Dictionary).get("site_kind", "?")), 0)) + 1
	print("  note site kinds: %s" % kinds)
	## The twelve biggest settlements, then every `riverthrough` one up to 20.
	## NOT the first N by index: the first eight river towns on this seed all
	## happen to have no crossing, and a sample that reports zero crossings
	## cannot tell an empty array from a broken one. (`MISTAKES.md`: one world
	## is one sample -- and so is one slice of one world.)
	var by_pop := []
	for i in settlements.size():
		by_pop.append({"i": i, "pop": int((settlements[i] as Dictionary).get("pop", 0))})
	by_pop.sort_custom(func(a, c): return a["pop"] > c["pop"])
	for e in by_pop:
		if idx.size() >= 12:
			break
		idx.append(int(e["i"]))
	for j in diags.size():
		if idx.size() >= 20:
			break
		if String((diags[j] as Dictionary).get("site_kind", "?")) == "riverthrough" and not idx.has(j):
			idx.append(j)
	var layouts: Array = b.urban_layouts(idx)
	_ok("layouts came back", layouts.size() > 0, "%d layouts for %d indices" % [layouts.size(), idx.size()])

	var with_key := 0
	var total_bridges := 0
	var towns_with_bridge := 0
	var fords := 0
	var dir_mismatch := 0
	var non_unit := 0
	var off_line := 0
	var stage_lines := 0
	var ford_without_bridge_pt := 0
	for d in layouts:
		var dd: Dictionary = d
		if dd.has("bridges"):
			with_key += 1
		var br: PackedVector2Array = dd.get("bridges", PackedVector2Array())
		var bd: PackedVector2Array = dd.get("bridge_dirs", PackedVector2Array())
		if br.size() != bd.size():
			dir_mismatch += 1
		total_bridges += br.size()
		if br.size() > 0:
			towns_with_bridge += 1
		for v in bd:
			if absf(v.length() - 1.0) > 1e-6:
				non_unit += 1
		# every recorded crossing must sit on the drawn river centreline
		var river: PackedVector2Array = dd.get("river", PackedVector2Array())
		for p in br:
			var near := false
			for k in range(maxi(0, river.size() - 1)):
				var a: Vector2 = river[k]
				var c: Vector2 = river[k + 1]
				var dv: Vector2 = c - a
				var t := 0.0
				if dv.length_squared() > 0.0:
					t = clampf((p - a).dot(dv) / dv.length_squared(), 0.0, 1.0)
				if p.distance_to(a + dv * t) < DEDUP_M:
					near = true
					break
			if not near:
				off_line += 1
		if dd.has("ford"):
			fords += 1
			if not dd.has("bridge_pt"):
				ford_without_bridge_pt += 1
			# the reference's `site.ford = {pt: site.bridgePt, …}`
			if dd.has("bridge_pt") and (dd["ford"] as Vector2).distance_to(dd["bridge_pt"] as Vector2) > 1e-6:
				_ok("ford == bridge_pt", false, "%s vs %s" % [dd["ford"], dd["bridge_pt"]])
			if br.size() > 0:
				_ok("ford and bridges are mutually exclusive", false, "both on one town")
		for s in dd.get("stages", PackedStringArray()):
			if String(s).begins_with("detectRiverCrossings"):
				stage_lines += 1

	print("[1] the key")
	_ok("`bridges` is present on EVERY layout, never absent",
		with_key == layouts.size(), "%d of %d" % [with_key, layouts.size()])
	_ok("`bridge_dirs` is parallel to `bridges` on every layout", dir_mismatch == 0,
		"%d mismatches" % dir_mismatch)
	_ok("every layout carries a detectRiverCrossings stage line",
		stage_lines == layouts.size(), "%d of %d" % [stage_lines, layouts.size()])

	print("[2] the measurement")
	_ok("MEASURED: real crossings on a real seed, not just on a fixture",
		total_bridges > 0 and fords > 0, "%d bridges across %d towns (%d towns have >=1); %d fords"
			% [total_bridges, layouts.size(), towns_with_bridge, fords])
	_ok("every bridge_dir is a unit vector", non_unit == 0, "%d bad" % non_unit)
	_ok("every bridge point lies on its own river centreline (<%d m)" % int(DEDUP_M),
		off_line == 0, "%d off" % off_line)
	_ok("a ford is never emitted without bridge_pt", ford_without_bridge_pt == 0)

	print("[3] the stage line agrees with the arrays")
	var mism := 0
	for d in layouts:
		var dd: Dictionary = d
		var line := ""
		for s in dd.get("stages", PackedStringArray()):
			if String(s).begins_with("detectRiverCrossings"):
				line = String(s)
		var n: int = (dd.get("bridges", PackedVector2Array()) as PackedVector2Array).size()
		if n > 0 and not line.contains("%d road crossing" % n):
			mism += 1
		elif n == 0 and dd.has("ford") and not line.contains("unbridged ford"):
			mism += 1
		elif n == 0 and not dd.has("ford") and not line.contains("none"):
			mism += 1
	_ok("the stages line names the arm the arrays show", mism == 0, "%d disagree" % mism)

	print(("=== VFY BRIDGES OK ===" if _fail == 0 else "=== VFY BRIDGES FAILED (%d) ===" % _fail))
	get_tree().quit(0 if _fail == 0 else 1)
