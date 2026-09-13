extends Node
## Diagnostic-only, not a pass/fail gate: measures the 13 FAILing captures
## `_cull_probe.gd` already wrote to `user://` on its most recent run, against
## `_segcull_probe.gd`'s own AA-noise tolerance (`AA_AGREEMENT_MAX_DELTA` = 10,
## `AA_AGREEMENT_MAX_DIFF_FRACTION` = 0.001 of the frame), to test one specific
## question: is `_cull_probe.gd`'s byte-equality FAIL on the dashed-way residual
## the SAME antialiasing-coverage-rounding phenomenon `_segcull_probe.gd` found
## and tolerates for per-segment culling, just measured against a stricter
## (zero-tolerance) bar that predates per-segment culling -- or is it larger /
## a different shape than that.
##
## Reads the PNGs `_cull_probe.gd` already saved this run (does not re-render
## anything itself), so this can run --headless: Image.load() is plain file
## I/O, not a live-viewport capture (the headless blocker documented on both
## probes is specifically `RenderingServer.frame_post_draw` never firing).
##
##   Godot_v4.7.1-stable_win64.exe --headless --path . _dashresidualcheck_probe.tscn

const AA_AGREEMENT_MAX_DELTA := 10          ## `_segcull_probe.gd`'s own bar.
const AA_AGREEMENT_MAX_DIFF_FRACTION := 0.001

## The 13 cases `_cull_probe.gd` reported DIFFERENT on the 2026-09-13 run this
## file is measuring. Way types drawn (from `_cull_probe.gd::_roads()`):
## highway/regional = SOLID (`WAY_STYLE.dash == 0.0`), road/track/ancient =
## DASHED. All five types are drawn together in one `set_civ_data()` call, so
## every one of these 13 frames contains BOTH kinds of way -- this probe
## cannot itself separate "caused by the dashed ways" from "caused by the
## solid ones" (that needs a same-shape capture with only one family present,
## a bigger change than this diagnostic). What it CAN measure: whether the
## existing 13 diffs are within, or well outside, the tolerance the project
## already ships for a documented cause (per-segment AA rounding).
const CASES := [
	"z1_-400_-260", "z1_300_180",
	"z2_0_0", "z2_-400_-260", "z2_-1600_-900", "z2_300_180",
	"z4_0_0", "z4_-400_-260", "z4_-1600_-900", "z4_300_180",
	"z8_0_0", "z8_-400_-260", "z8_-1600_-900",
]


func _p(s: String) -> void:
	print("DASHRESIDUAL  %s" % s)


## Exact copy of `_segcull_probe.gd::_pixel_diff` semantics (max single-channel
## delta anywhere in the frame; count of PIXELS differing in any channel).
func _pixel_diff(a: PackedByteArray, b: PackedByteArray) -> Dictionary:
	var max_delta := 0
	var diff_pixels := 0
	var n := a.size()
	var i := 0
	while i < n:
		var pixel_differs := false
		for c in 4:
			var d := absi(int(a[i + c]) - int(b[i + c]))
			if d > 0:
				pixel_differs = true
				if d > max_delta:
					max_delta = d
		if pixel_differs:
			diff_pixels += 1
		i += 4
	return {"max_delta": max_delta, "diff_pixels": diff_pixels}


func _ready() -> void:
	var missing := 0
	var within_count := 0
	var outside_count := 0
	var worst_delta := 0
	var worst_frac := 0.0
	var worst_case := ""
	for c in CASES:
		var pa := "user://cull_on_%s.png" % c
		var pb := "user://cull_off_%s.png" % c
		if not (FileAccess.file_exists(pa) and FileAccess.file_exists(pb)):
			_p("%-16s MISSING (%s / %s not found -- re-run _cull_probe.tscn first)" % [c, pa, pb])
			missing += 1
			continue
		var ia := Image.load_from_file(pa)
		var ib := Image.load_from_file(pb)
		if ia == null or ib == null or ia.get_size() != ib.get_size():
			_p("%-16s LOAD FAILED or size mismatch" % c)
			missing += 1
			continue
		var da := ia.get_data()
		var db := ib.get_data()
		var diff := _pixel_diff(da, db)
		var total_px := ia.get_width() * ia.get_height()
		var frac := float(diff["diff_pixels"]) / float(total_px)
		var within: bool = diff["max_delta"] <= AA_AGREEMENT_MAX_DELTA \
			and frac <= AA_AGREEMENT_MAX_DIFF_FRACTION
		if within:
			within_count += 1
		else:
			outside_count += 1
		if diff["max_delta"] > worst_delta or (diff["max_delta"] == worst_delta and frac > worst_frac):
			worst_delta = diff["max_delta"]
			worst_frac = frac
			worst_case = c
		_p("%-16s max_delta=%3d  diff_px=%6d/%d (%.4f%%)  within_segcull_AA_tolerance=%s"
			% [c, diff["max_delta"], diff["diff_pixels"], total_px, frac * 100.0, within])

	_p("---- summary: %d within _segcull_probe.gd's AA tolerance, %d outside it, %d missing/unreadable"
		% [within_count, outside_count, missing])
	if worst_case != "":
		_p("worst case: %s (max_delta=%d, %.4f%% of pixels)" % [worst_case, worst_delta, worst_frac * 100.0])
	get_tree().quit(0)
