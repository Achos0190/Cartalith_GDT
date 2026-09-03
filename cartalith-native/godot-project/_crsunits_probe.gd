extends SceneTree
## Verifies world_workspace.gd::_build_crs()'s new DccUnits wiring (Lane B,
## 2026-09-03) without needing a real EngineBridge/WorldGen: the exact printf
## pattern that function runs, against mock crs values, at whatever
## DccSettings.units_mode() this machine currently has saved (read-only --
## this probe never calls set_units_mode(), which persists to
## user://cartalith_settings.cfg for real).
##
##   Godot_v4.7.1-stable_win64.exe --headless --path . --script _crsunits_probe.gd

var fails := 0

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

func _initialize() -> void:
	var mode := DccSettings.units_mode()
	print("== DccSettings.units_mode() on this machine: \"%s\" (read-only, unchanged) ==" % mode)

	var crs := {
		"grid_w": 384, "grid_h": 256,
		"map_width_km": 812.0, "map_height_km": 541.0, "cell_km": 2.114,
		"lat_n": 62.3, "lat_s": 14.1, "deg_per_row": 0.1887,
	}
	# The exact expression world_workspace.gd::_build_crs() now runs.
	var text := ("%d × %d cells over %s × %s, so one cell is %s on a side. "
		+ "Rows run %.1f° to %.1f° — %.4f° of latitude per row, which is what "
		+ "the climate model integrates over.") \
		% [int(crs.get("grid_w", 0)), int(crs.get("grid_h", 0)),
			DccUnits.format(float(crs.get("map_width_km", 0.0))),
			DccUnits.format(float(crs.get("map_height_km", 0.0))),
			DccUnits.format(float(crs.get("cell_km", 0.0)), 3),
			float(crs.get("lat_n", 0.0)), float(crs.get("lat_s", 0.0)),
			float(crs.get("deg_per_row", 0.0))]
	print("  -> %s" % text)
	_ok(true, "the format string/arg-count pair did not throw at runtime")
	_ok(text.contains(DccUnits.suffix()), "output carries the live unit suffix (%s)" % DccUnits.suffix())
	_ok(text.begins_with("384 × 256 cells over"), "cell counts are untouched by the unit change")
	_ok(not text.contains(" km,") if mode != "km" else true,
		"when not in km mode, the old bare-km comma phrase is gone")

	print("\n%s (%d failure%s)" % ["ALL PASS" if fails == 0 else "FAILURES", fails, "" if fails == 1 else "s"])
	quit(1 if fails > 0 else 0)
