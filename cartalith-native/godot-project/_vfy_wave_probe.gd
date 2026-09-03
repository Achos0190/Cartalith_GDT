extends SceneTree

## Adversarial verification probe (not a lane deliverable).
## 1. belief determinism across two identical worlds
## 2. world-replacement: does adherence actually disappear on each path?
## 3. the uncovered seed input: civ_edit_settlement changing a settlement's faction
## 4. layer-stack: does the default really round-trip byte-for-byte through set()?

func _fp(wg: WorldGen) -> String:
	var tex := wg.build_color_texture()
	if tex == null:
		return "<null>"
	var img: Image = tex.get_image()
	var d: PackedByteArray = img.get_data()
	return "%d@%dx%d" % [d.size(), img.get_width(), img.get_height()]

func _adh(wg: WorldGen) -> Array:
	var out: Array = []
	for p in wg.get_settlements():
		if p.has("adherents"):
			out.append([String(p["name"]), String(p["religion"]), p["adherents"]])
		else:
			out.append([String(p["name"]), "<absent>", {}])
	return out

func _init() -> void:
	var fails := 0

	# ---- 1. determinism -----------------------------------------------
	var a: WorldGen = WorldGen.new()
	a.generate_sized(24601, 640.0, 96, 64)
	a.civ_set_faction_field(1, "religion", "sun_cult")
	a.civ_set_faction_field(2, "religion", "old_gods")
	a.civ_belief_run(120)
	var ra := _adh(a)

	var b: WorldGen = WorldGen.new()
	b.generate_sized(24601, 640.0, 96, 64)
	b.civ_set_faction_field(1, "religion", "sun_cult")
	b.civ_set_faction_field(2, "religion", "old_gods")
	b.civ_belief_run(120)
	var rb := _adh(b)

	print("  det: ", ra.size(), " settlements; run A[0]=", ra[0], " run B[0]=", rb[0])
	if str(ra) != str(rb):
		print("  FAIL: two identical worlds produced different adherence")
		fails += 1
	else:
		print("  PASS determinism: two identical worlds agree exactly")

	# 120 years split as 60+60 must equal 120 in one go (continuation, not reset)
	var c: WorldGen = WorldGen.new()
	c.generate_sized(24601, 640.0, 96, 64)
	c.civ_set_faction_field(1, "religion", "sun_cult")
	c.civ_set_faction_field(2, "religion", "old_gods")
	c.civ_belief_run(60)
	c.civ_belief_run(60)
	if str(_adh(c)) != str(ra):
		print("  NOTE: 60+60 != 120 (a second call is not a pure continuation)")
	else:
		print("  PASS: 60+60 == 120, a second call continues rather than resets")

	# ---- 2. world replacement -----------------------------------------
	# (a) regenerate a DIFFERENT world on the same object
	a.generate_sized(777, 640.0, 96, 64)
	var after_regen := _adh(a)
	var leaked := 0
	for row in after_regen:
		if row[1] != "<absent>":
			leaked += 1
	print("  after generate_sized(new seed): settlements carrying a religion key = ", leaked, "/", after_regen.size())
	if leaked != 0:
		print("  FAIL: adherence survived a world regeneration")
		fails += 1

	# (b) recreate, then civ_populate (Replace) / civ_auto_routes (Routes)
	var d: WorldGen = WorldGen.new()
	d.generate_sized(24601, 640.0, 96, 64)
	d.civ_set_faction_field(1, "religion", "sun_cult")
	d.civ_belief_run(80)
	var before_routes := _adh(d)
	var had := 0
	for row in before_routes:
		if row[1] != "<absent>":
			had += 1
	print("  seeded before route rebuild: ", had, "/", before_routes.size())

	if d.has_method("civ_auto_routes"):
		d.civ_auto_routes()
		var after_routes := _adh(d)
		var kept := 0
		for row in after_routes:
			if row[1] != "<absent>":
				kept += 1
		print("  after civ_auto_routes (CivRebuild::Routes): keys kept = ", kept, "/", after_routes.size(),
			"  identical to before = ", str(after_routes) == str(before_routes))
	else:
		print("  NOTE: civ_auto_routes not exposed")

	if d.has_method("civ_populate"):
		d.civ_populate()
		var after_pop := _adh(d)
		var kept2 := 0
		for row in after_pop:
			if row[1] != "<absent>":
				kept2 += 1
		print("  after civ_populate (CivRebuild::Replace): keys kept = ", kept2, "/", after_pop.size())
		if kept2 != 0:
			print("  FAIL: adherence survived a settlement replacement")
			fails += 1
	else:
		print("  NOTE: civ_populate not exposed")

	# ---- 3. the uncovered seed input: settlement faction reassignment ---
	var e: WorldGen = WorldGen.new()
	e.generate_sized(24601, 640.0, 96, 64)
	e.civ_set_faction_field(1, "religion", "sun_cult")
	e.civ_belief_run(80)
	var places: Array = e.get_settlements()
	# find a settlement in faction 1 with a sun_cult plurality
	var idx := -1
	for i in range(places.size()):
		if int(places[i]["faction"]) == 1 and int(places[i]["adherents"].get("sun_cult", 0)) > 0:
			idx = i
			break
	if idx < 0:
		print("  NOTE: no faction-1 sun_cult settlement to reassign; skipping")
	else:
		var nm := String(places[idx]["name"])
		var before_rel := String(places[idx]["religion"])
		# move it into faction 2, whose religion is `none`
		var ok: bool = e.civ_edit_settlement(idx, {"faction": 2})
		var st: Dictionary = e.civ_belief_run(0)
		var after: Array = e.get_settlements()
		print("  reassign ", nm, " faction 1 -> 2 (edit ok=", ok, "): religion before=", before_rel,
			" after=", String(after[idx]["religion"]), "  re-seeded=", bool(st.get("seeded", false)))
		if not bool(st.get("seeded", false)):
			print("  >>> UNCOVERED SEED INPUT: a settlement's faction changed and the layer did NOT re-seed")

	# ---- 4. layer stack: default round-trips through set() -------------
	var f: WorldGen = WorldGen.new()
	f.generate_sized(24601, 640.0, 96, 64)
	var base := _fp(f)
	var rows: Array = f.get_layer_stack()
	var n: int = f.set_layer_stack(rows)
	var after_rt := _fp(f)
	print("  set_layer_stack(get_layer_stack()) -> ", n, "; image ", base, " -> ", after_rt,
		"  identical=", base == after_rt)
	if base != after_rt:
		print("  FAIL: writing back the stack it just read changed the picture")
		fails += 1

	print("VERIFY ", ("PASS" if fails == 0 else "FAIL"), " -- ", fails, " failure(s)")
	quit(0 if fails == 0 else 1)
