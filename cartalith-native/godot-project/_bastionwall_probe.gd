extends SceneTree
## Lane URBAN Part B, Gate B1: does `urban_layouts()` actually emit a
## bastioned wall's STYLE and its GEOMETRY (the star-fort trace/ravelins the
## reference draws) once a settlement is marked Fortified through the SAME
## bridge call path the Place Editor's trait pill uses
## (`civ_settlement_toggle_trait`)?
##
## Pure data -- no drawing, so headless is correct here (MISTAKES.md: windowed
## is for pixels; this reads a Dictionary).
##
##   Godot_v4.7.1-stable_win64.exe --headless --path . --script _bastionwall_probe.gd

var fails := 0

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

func _fort_like_keys(d: Dictionary) -> Array:
	var out := []
	for k in d.keys():
		var ks := String(k)
		if ks.begins_with("wall") or ks.begins_with("fort") or ks.contains("bastion") or ks.contains("ravelin"):
			out.append(ks)
	out.sort()
	return out

func _init() -> void:
	var gen := WorldGen.new()
	gen.generate_sized(24601, 96.0, 96, 64)
	gen.recompute_civilisation()
	var settlements: Array = gen.get_settlements()
	print("BASTIONWALL %d settlements" % settlements.size())

	# The largest settlement, so `pop_target` clears FORT_MIN (2500) without
	# the test needing to depend on this seed already having a metropolis.
	var target_idx := -1
	var best_pop := -1
	for i in settlements.size():
		var s: Dictionary = settlements[i]
		var pop := int(s.get("population", 0))
		if pop > best_pop:
			best_pop = pop
			target_idx = i
	if target_idx < 0:
		print("  FAIL  no settlement generated -- cannot test")
		quit(2)
		return
	var t: Dictionary = settlements[target_idx]
	print("BASTIONWALL target #%d \"%s\" pop=%d (before any override)" % [
		target_idx, String(t.get("name", "?")), best_pop])

	# The SAME bridge call the Place Editor's population field uses -- forces
	# `pop_target` well clear of FORT_MIN (2500) so the test does not depend
	# on this seed's largest town already being one.
	var edit_ok: bool = gen.civ_edit_settlement(target_idx, {"population": 8000})
	_ok(edit_ok, "civ_edit_settlement raised population to 8000")

	# -- BEFORE: unfortified layout, for comparison --------------------------
	var before: Array = gen.urban_layouts(PackedInt32Array([target_idx]))
	_ok(before.size() == 1, "layout generated before toggling the trait")
	if before.size() == 1:
		var lb: Dictionary = before[0]
		var ring_b: PackedVector2Array = lb.get("wall_ring", PackedVector2Array())
		print("BASTIONWALL before: wall_style=%s wall_ring=%d verts wall/fort keys=%s" % [
			String(lb.get("wall_style", "<absent>")), ring_b.size(), _fort_like_keys(lb)])

	# -- the exact call the Place Editor's trait pill makes -------------------
	var toggled: bool = gen.civ_settlement_toggle_trait(target_idx, "fortified")
	_ok(toggled, "civ_settlement_toggle_trait(idx, \"fortified\") returned true")

	# -- AFTER: urban_layouts() re-derives PlaceOverrides.fortified_trait live -
	var after: Array = gen.urban_layouts(PackedInt32Array([target_idx]))
	_ok(after.size() == 1, "layout generated after toggling the trait")
	if after.size() != 1:
		quit(1)
		return
	var la: Dictionary = after[0]
	var ring_a: PackedVector2Array = la.get("wall_ring", PackedVector2Array())
	var style_a := String(la.get("wall_style", "<absent>"))
	print("BASTIONWALL after:  wall_style=%s wall_ring=%d verts wall/fort keys=%s" % [
		style_a, ring_a.size(), _fort_like_keys(la)])
	print("BASTIONWALL after: ALL %d KEYS = %s" % [la.keys().size(), la.keys()])

	_ok(style_a == "bastioned", "wall_style flips to \"bastioned\" after the toggle")
	_ok(ring_a.size() > 0, "wall_ring is still present (the gorge polygon) after the toggle")

	# Gate B1's actual question: does ANY key carrying the star-fort's own
	# drawable geometry (trace / bastions / ravelins) cross the bridge?
	var has_fort_geom := la.has("fort_trace") or la.has("wall_trace") \
		or la.has("wall_ravelins") or la.has("fort_ravelins") or la.has("bastions")
	_ok(has_fort_geom, "a fort-trace/ravelin key is present in the layout dictionary")

	print("BASTIONWALL %s (%d failed)" % ["ALL PASS" if fails == 0 else "SOME FAILED", fails])
	quit(1 if fails > 0 else 0)
