extends SceneTree

# Adversarial verification of Lane A1: units in all three modes.
# Replicates right_dock.gd's converted call sites verbatim, plus the
# surviving hard-coded ones, at km / mi / nmi.

func _init() -> void:
	var orig := DccSettings.units_mode()
	print("ORIGINAL units_mode = ", orig)
	var fail := 0

	# --- conversion-before-rounding check -------------------------------
	# 100.0 km at 0 decimals: mi = 62.137 -> "62 mi"; nmi = 53.996 -> "54 nm".
	# If the site rounded km first (100 -> "100") then converted, mi would be
	# 62 too, so use a value where rounding-first and converting-first differ:
	# 1.4 km. round-first -> 1 km -> 0.62 mi -> "1 mi"; convert-first ->
	# 0.8699 mi -> "1 mi". Use 0.9 km at 1 dp instead:
	#   convert-first: 0.9/1.609344 = 0.5592 -> "0.6 mi"
	#   round-first:   0.9 -> "0.9" -> /1.609 = 0.559 -> "0.6" (same)
	# The real discriminator is the SUFFIX and magnitude at 0 dp.
	DccSettings.set_units_mode("km")
	var km_a := DccUnits.format(100.0)
	DccSettings.set_units_mode("mi")
	var mi_a := DccUnits.format(100.0)
	DccSettings.set_units_mode("nmi")
	var nm_a := DccUnits.format(100.0)
	print("format(100.0, 0):  km=%s  mi=%s  nmi=%s" % [km_a, mi_a, nm_a])
	if km_a != "100 km": print("  FAIL km"); fail += 1
	if mi_a != "62 mi": print("  FAIL mi (expected 62 mi)"); fail += 1
	if nm_a != "54 nm": print("  FAIL nmi (expected 54 nm)"); fail += 1

	# rounding order: 1000.0 km. convert-first nmi = 539.956 -> "540.0" at 1dp.
	DccSettings.set_units_mode("nmi")
	var r1 := DccUnits.format(1000.0, 1)
	print("format(1000.0, 1) nmi = ", r1, "  (convert-first => 540.0 nm)")
	if r1 != "540.0 nm": print("  FAIL rounding order"); fail += 1

	# --- the six MEASURE readout arms, as right_dock writes them --------
	# Faithful copies of right_dock.gd's expressions.
	var mr := {
		"total_km": 250.0, "radius_km": 250.0, "length_km": 250.0,
		"delta_m": 300.0, "projected_km2": 1000.0,
		"straight_line_km": 200.0, "total_km_3d": 260.0,
		"horizontal_km": 250.0, "distance_3d_km": 260.0,
		"perimeter_km": 250.0, "bbox_w_km": 250.0, "bbox_h_km": 250.0,
		"diameter_km": 500.0, "circumference_km": 1570.0,
		"length_3d_km": 260.0, "above_2000m_km": 40.0,
	}
	for mode in ["km", "mi", "nmi"]:
		DccSettings.set_units_mode(mode)
		print("--- units_mode = %s ---" % mode)
		# CONVERTED (claimed):
		print("  [conv] readout distance     : %s" % DccUnits.format(float(mr["total_km"]), 1))
		print("  [conv] Total length         : %s" % DccUnits.format(float(mr["total_km"])))
		print("  [conv] segment km           : %s" % DccUnits.format(float(mr["total_km"])))
		print("  [conv] bearing Distance     : %s" % DccUnits.format(float(mr["total_km"]), 1))
		print("  [conv] Straight line        : %s" % DccUnits.format(float(mr["straight_line_km"]), 1))
		print("  [conv] derived 3D length    : %s" % DccUnits.format(float(mr["total_km_3d"]), 1))
		print("  [conv] Region Extent label  : Extent (%s)" % DccUnits.suffix())
		print("  [conv] Region Extent value  : %s x %s" % [DccUnits.format(400.0), DccUnits.format(300.0)])
		# SURVIVING HARD-CODED km, same panel (_measure_readout's own siblings):
		print("  [RAW ] readout radius arm   : %s" % ("r %.0f km" % float(mr["radius_km"])))
		print("  [RAW ] readout section arm  : %s" % ("%.0f km section" % float(mr["length_km"])))
		print("  [RAW ] Radius panel Radius  : %s" % ("%.0f km" % float(mr["radius_km"])))
		print("  [RAW ] Vertical horiz dist  : %s" % ("%.1f km" % float(mr["horizontal_km"])))
		print("  [RAW ] Section Length       : %s" % ("%.0f km" % float(mr["length_km"])))
		print("  [RAW ] Area Perimeter       : %s" % ("%.0f km" % float(mr["perimeter_km"])))
		print("  [RAW ] Route Length (1165)  : %s" % ("%.1f km" % 250.0))

	DccSettings.set_units_mode(orig)
	print("RESTORED units_mode = ", DccSettings.units_mode())
	print("UNITS PROBE FAILURES = ", fail)
	quit()
