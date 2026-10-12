extends SceneTree
## Protects: ET-E4's wide-gamut overlay half. (1) In Display P3 mode an overlay ink is the
## `render.rs::SRGB_TO_P3` encode of it, to 1/255, against values computed OUTSIDE GDScript
## (Python, from the matrix text in render.rs and the IEC 61966-2-1 transfer function, never
## from `overlay_adjust.gd`'s own constant). (2) In sRGB mode, and with the empty stack, every
## ink is the SAME value as before (whole-array `==`), and with a live stack it equals the
## pre-change `overlay_adjust.gd` (`git show HEAD:` copy at res://_oa_head.gd) value for value.
## (3) P3 is applied LAST: P3(stack(x)) with a live stack. (4) The MapOverlay hooks resolve
## road and sea inks through it. (5) An unknown space is refused and changes nothing.
## Run WINDOWED (MISTAKES.md, "Run a pixel probe"; this probe asserts colour values, not
## pixels): godot --path <scratch copy> -s res://_etp3_probe.gd

const OA := preload("res://shell/overlay_adjust.gd")
var _fail := 0
var _n := 0


func _check(ok: bool, what: String) -> void:
	_n += 1
	if not ok:
		_fail += 1
		print("FAIL: ", what)


## Independent expected values, 0..255, from Python over render.rs's SRGB_TO_P3 text.
const EXPECT := [
	[Color8(255, 0, 0), [233.959327, 51.073136, 35.332951]],
	[Color8(0, 255, 0), [116.892405, 251.242469, 76.06515]],
	[Color8(0, 0, 255), [0.0, 0.0, 244.694947]],
	[Color8(128, 64, 32), [119.648082, 67.362423, 39.284338]],
	[Color8(200, 180, 40), [196.641761, 180.710613, 70.559797]],
	[Color8(128, 128, 128), [128.0, 128.0, 128.0]],
	[Color8(35, 24, 9), [33.303393, 24.438273, 11.275697]],
]
const STACK := '{"entries":[{"kind":{"levels":{"in_black":10,"in_white":240,"gamma":1.4,"out_black":5,"out_white":250}},"enabled":true,"opacity":1.0},{"kind":{"hsl":{"hue_deg":40,"saturation":0.3,"lightness":0.1}},"enabled":true,"opacity":0.8}]}'


func _grid() -> Array:
	var out: Array = []
	for r in 17:
		for g in 17:
			for b in 17:
				out.append(Color(minf(r * 16, 255) / 255.0, minf(g * 16, 255) / 255.0, minf(b * 16, 255) / 255.0, 0.5 + 0.03 * ((r + g + b) % 10)))
	return out


func _run(src: Array) -> Array:
	var out: Array = []
	for c in src:
		out.append(OA.color(c))
	return out


func _initialize() -> void:
	var head = load("res://_oa_head.gd")
	_check(head != null, "HEAD copy of overlay_adjust.gd loads")
	var grid := _grid()
	OA.clear()
	_check(not OA.display_is_p3() and not OA.is_active(), "default: sRGB, inactive")
	# (2) identity, empty stack, sRGB: whole array equal.
	var base := _run(grid)
	_check(base == grid, "sRGB + empty stack: all %d inks identical" % grid.size())
	# (1) P3 literals, to 1/255.
	var rev0: int = OA.revision
	_check(OA.set_display_space("Display P3"), "set P3 accepted")
	_check(OA.revision == rev0 + 1 and OA.is_active() and not OA.is_live(), "P3: revision moved, active, stack still not live")
	_check(OA.set_display_space("Display P3") and OA.revision == rev0 + 1, "re-setting the same space does not bump revision")
	var worst := 0.0
	for e in EXPECT:
		var c: Color = e[0]
		var o: Color = OA.color(c)
		for i in 3:
			worst = maxf(worst, absf(o[i] * 255.0 - float(e[1][i])))
		_check(is_equal_approx(o.a, c.a), "alpha kept")
	_check(worst <= 1.0, "P3 inks match the Python/Rust values within 1/255 (worst %.6f of 255)" % worst)
	var red: Color = OA.color(Color8(255, 0, 0))
	_check(roundi(red.r * 255.0) == 234 and roundi(red.g * 255.0) == 51 and roundi(red.b * 255.0) == 35, "red rounds to the well-known P3 (234,51,35)")
	# (3) stack then P3.
	if head != null:
		head.set_stack_json(STACK)
		_check(OA.set_stack_json(STACK)["ok"], "stack accepted in P3 mode")
		var bad := 0
		for c in grid:
			var s: Color = head.color(c)
			var v := PackedFloat64Array([clampf(s.r, 0.0, 1.0) * 255.0, clampf(s.g, 0.0, 1.0) * 255.0, clampf(s.b, 0.0, 1.0) * 255.0])
			var p: PackedFloat64Array = OA.srgb255_to_p3(v)
			var o: Color = OA.color(c)
			if absf(o.r * 255.0 - clampf(p[0], 0.0, 255.0)) > 1e-3 or absf(o.g * 255.0 - clampf(p[1], 0.0, 255.0)) > 1e-3 or absf(o.b * 255.0 - clampf(p[2], 0.0, 255.0)) > 1e-3:
				bad += 1
		_check(bad == 0, "P3 mode + live stack == P3(HEAD stack(ink)), %d of %d differ" % [bad, grid.size()])
		var moved := 0
		for c in grid:
			if OA.color(c) != head.color(c):
				moved += 1
		_check(moved > 0, "positive control: P3 moves live-stack inks (%d of %d)" % [moved, grid.size()])
		# back to sRGB: equals HEAD over the whole array.
		_check(OA.set_display_space("sRGB") and not OA.display_is_p3(), "back to sRGB")
		var now := _run(grid)
		var ref: Array = []
		for c in grid:
			ref.append(head.color(c))
		_check(now == ref, "sRGB + live stack: all %d inks equal HEAD's value for value" % grid.size())
		OA.clear()
		head.clear()
	# empty stack after a P3 round trip is identity again.
	OA.set_display_space("sRGB")
	_check(_run(grid) == grid, "after a P3 round trip, empty stack is identity again")
	# (5) refusal.
	var rev1: int = OA.revision
	_check(not OA.set_display_space("Rec2020") and not OA.display_is_p3() and OA.revision == rev1, "unknown space refused, nothing changed")
	# (4) MapOverlay hooks.
	var ov = load("res://map_overlay.gd").new()
	var row_srgb: Dictionary = ov.way_style_for("highway")
	_check(row_srgb["over"] == Color(0.824, 0.569, 0.216, 0.98), "overlay sRGB: highway over ink is the shipped literal")
	_check(ov.set_display_space("Display P3") and ov.overlay_adjust_live(), "overlay: P3 on, live")
	var row_p3: Dictionary = ov.way_style_for("highway")
	var ro: Color = row_p3["over"]
	_check(absf(ro.r * 255.0 - 200.539909) <= 1.0 and absf(ro.g * 255.0 - 147.868780) <= 1.0 and absf(ro.b * 255.0 - 72.895939) <= 1.0 and is_equal_approx(ro.a, 0.98), "overlay P3: highway over ink = independent P3 value, alpha kept")
	var sea: Dictionary = ov.sea_style()
	_check(sea["under"] != Color(0.039, 0.118, 0.235, 0.4), "overlay P3: sea lane ink moved")
	ov.set_display_space("sRGB")
	_check(ov.way_style_for("highway")["over"] == Color(0.824, 0.569, 0.216, 0.98) and not ov.overlay_adjust_live(), "overlay back to sRGB: shipped literal again")
	ov.free()
	print("etp3 probe: %d checks, %d failed" % [_n, _fail])
	quit(1 if _fail > 0 else 0)
