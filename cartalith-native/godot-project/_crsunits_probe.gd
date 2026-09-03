extends SceneTree
## Verifies `world_workspace.gd::_build_crs()`'s `DccUnits` wiring (Lane B,
## 2026-09-03) by **running that function**, in all three unit modes.
##
## **Why it drives the real panel rather than the expression.** The first
## version of this probe copied `_build_crs()`'s format string into itself and
## asserted the copy. A verifier measured what that was worth: changing the
## panel's `DccUnits.format(cell_km, 3)` to `, 0)` SURVIVED, and reverting the
## panel outright to a hardcoded `str(int(...)) + " km"` SURVIVED too -- a copy
## cannot see a mutation in the original. So this builds a `WorldWorkspace`,
## hands it a real `EngineBridge` over a real generated world, calls
## `_build_crs()` and reads the Label it produced.
##
## **And it exercises mi and nautical miles, not just km.** The old probe never
## called `set_units_mode()`, so on a km-mode machine (this one) every
## conversion in the function under test was the identity and the whole point
## of the fix went untested. The three expectations below are computed from the
## *defining* factors -- 1 mi = 1 609.344 m, 1 NM = 1 852 m -- and a literal
## decimal count, never from `DccUnits`, so an assertion cannot pass by
## agreeing with the thing it is checking.
##
## The saved units mode is read first and restored last: `set_units_mode()`
## writes `user://cartalith_settings.cfg` for real, and a probe must not leave
## a machine in a mode its owner did not choose.
##
##   Godot_v4.7.1-stable_win64.exe --headless --path . --script _crsunits_probe.gd

## Exact definitions, not `DccUnits.KM_PER_MI`/`KM_PER_NMI`: asserting a
## constant against itself holds for every value of it (`MISTAKES.md`).
const MI_M := 1609.344
const NMI_M := 1852.0

var fails := 0

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

## Every Label under `n`, in tree order.
func _labels(n: Node, out: Array) -> Array:
	for c in n.get_children():
		if c is Label:
			out.append(String((c as Label).text))
		_labels(c, out)
	return out

func _init() -> void:
	var saved := DccSettings.units_mode()
	print("== saved DccSettings.units_mode() = \"%s\" (restored at exit) ==" % saved)

	var bridge: EngineBridge = EngineBridge.new()
	get_root().add_child(bridge)
	bridge.world_gen.generate_sized(24601, 812.0, 96, 64)
	var crs: Dictionary = bridge.world_crs()
	if crs.is_empty():
		print("  FAIL  world_crs() is empty -- no world to build the panel over")
		quit(1)
		return
	var w_km := float(crs.get("map_width_km", 0.0))
	var h_km := float(crs.get("map_height_km", 0.0))
	var c_km := float(crs.get("cell_km", 0.0))
	var gw := int(crs.get("grid_w", 0))
	var gh := int(crs.get("grid_h", 0))
	print("  world_crs(): %d x %d cells, %.4f x %.4f km, cell %.6f km" % [gw, gh, w_km, h_km, c_km])

	var ws := WorldWorkspace.new()
	ws.bridge = bridge
	var host := VBoxContainer.new()
	get_root().add_child(host)

	var seen := {}
	for mode in ["km", "mi", "nmi"]:
		DccSettings.set_units_mode(mode)
		ws._build_crs(host)

		# The one note that carries the three converted figures.
		var line := ""
		for t in _labels(host, []):
			if t.contains("cells over"):
				line = t
				break
		if line.is_empty():
			_ok(false, "[%s] _build_crs() drew no \"cells over\" note at all" % mode)
			continue
		print("  [%s] -> %s" % [mode, line])
		seen[mode] = line

		# Independent expectations: metres per unit, straight from the
		# definition, and the decimal counts the panel is supposed to use.
		var per_km := 1.0
		var suffix := "km"
		match mode:
			"mi":
				per_km = MI_M / 1000.0
				suffix = "mi"
			"nmi":
				per_km = NMI_M / 1000.0
				suffix = "nm"
		var want_w := "%.0f %s" % [w_km / per_km, suffix]
		var want_h := "%.0f %s" % [h_km / per_km, suffix]
		var want_cell := "%.3f %s" % [c_km / per_km, suffix]

		_ok(line.begins_with("%d × %d cells over " % [gw, gh]),
			"[%s] the cell counts are untouched by the unit change" % mode)
		_ok(line.contains("cells over %s × %s," % [want_w, want_h]),
			"[%s] map extent reads \"%s × %s\"" % [mode, want_w, want_h])
		_ok(line.contains("one cell is %s on a side" % want_cell),
			"[%s] cell size reads \"%s\" -- three decimals, converted" % [mode, want_cell])
		if mode != "km":
			_ok(not line.contains(" km"),
				"[%s] no bare kilometre figure survives in the line" % mode)

	# A hardcoded-km panel would print one identical line in all three modes;
	# so would any change that dropped the conversion. This says nothing about
	# WHICH numbers are right -- the three checks above do -- only that the
	# function is reading the mode at all.
	_ok(seen.size() == 3
			and seen["km"] != seen["mi"]
			and seen["mi"] != seen["nmi"]
			and seen["km"] != seen["nmi"],
		"the three modes produce three different lines")

	DccSettings.set_units_mode(saved)
	_ok(DccSettings.units_mode() == saved, "units mode restored to \"%s\"" % saved)

	print("\n%s (%d failure%s)" % ["ALL PASS" if fails == 0 else "FAILURES", fails, "" if fails == 1 else "s"])
	quit(1 if fails > 0 else 0)
