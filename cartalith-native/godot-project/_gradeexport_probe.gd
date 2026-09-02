extends Node
## Committed verification harness: does the export raster carry the colour
## grade, and does a graded export match the graded viewport? Not committed.
##
## Why this is a separate probe from _exportraster_probe.gd: that probe's
## section 13 already compares a grid-resolution export against
## build_color_texture byte for byte, and it PASSES -- but it passes under the
## shipped default look (Natural Vibrant), whose grade is the identity. So
## apply_color_grade early-returns on both sides of that comparison and a
## missing call would have gone unseen. This probe runs the same comparison
## under Antique Parchment, the one shipped look that actually grades.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _gradeexport_probe.tscn
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

var bridge: Node
var fails := 0
var dir := ""

const GRADE_KEYS := ["grade_exposure", "grade_contrast", "grade_saturation",
	"grade_temperature", "grade_shadow_tint", "grade_highlight_tint"]

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

func _export(name: String) -> PackedByteArray:
	var p := dir.path_join(name + ".png")
	var r: Dictionary = bridge.world_gen.export_raster_png(p, 2048, false)
	if not bool(r.get("ok", false)):
		print("    export %s FAILED: %s" % [name, String(r.get("error", ""))])
		return PackedByteArray()
	var img := Image.new()
	if img.load(p) != OK:
		return PackedByteArray()
	img.convert(Image.FORMAT_RGB8)
	return img.get_data()

func _screen() -> PackedByteArray:
	var tex: ImageTexture = bridge.world_gen.build_color_texture()
	if tex == null:
		return PackedByteArray()
	var im := tex.get_image()
	im.convert(Image.FORMAT_RGB8)
	return im.get_data()

## worst per-byte delta and the fraction of bytes that moved at all.
func _cmp(a: PackedByteArray, b: PackedByteArray) -> Array:
	if a.size() == 0 or a.size() != b.size():
		return [-1, -1.0, -1.0]
	var worst := 0
	var moved := 0
	var sum := 0
	for i in range(a.size()):
		var d: int = absi(a[i] - b[i])
		if d > 0:
			moved += 1
			sum += d
			worst = maxi(worst, d)
	return [worst, float(moved) / float(a.size()), float(sum) / float(a.size())]

func _ready() -> void:
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		push_error("grade-export probe watchdog: _ready never finished")
		get_tree().quit(2))
	bridge = load("res://shell/engine_bridge.gd").new()
	add_child(bridge)
	await get_tree().process_frame

	dir = ProjectSettings.globalize_path("user://_gradeexport_probe")
	DirAccess.make_dir_recursive_absolute(dir)
	print("  scratch: %s" % dir)

	print("\n== 1. a world whose grid IS an offered export width ==")
	## The only case where "the export equals the screen" is a well-posed
	## byte-for-byte question at all -- _exportraster_probe.gd section 6
	## documents why 512-vs-2048 is not.
	var t0 := Time.get_ticks_msec()
	bridge.world_gen.generate_sized(20260824, 1200.0, 2048, 1312)
	bridge.has_world = true
	print("  generated 2048 x 1312 in %.1f s" % ((Time.get_ticks_msec() - t0) / 1000.0))
	_ok(bridge.world_gen.get_width() == 2048, "world is 2048 wide")

	print("\n== 2. the looks, and which of them actually grade ==")
	var looks: PackedStringArray = bridge.world_gen.list_looks()
	print("  looks: %s, open on '%s'" % [str(looks), String(bridge.world_gen.get_look())])
	_ok(String(bridge.world_gen.get_look()) == "Natural Vibrant", "opens on Natural Vibrant")
	for lk in looks:
		bridge.world_gen.set_look(lk)
		var ap: Dictionary = bridge.world_gen.get_appearance()
		var g := []
		for k in GRADE_KEYS:
			g.append("%s=%.2f" % [k.replace("grade_", ""), float(ap.get(k, 0.0))])
		print("    %-18s %s" % [lk, " ".join(g)])

	print("\n== 3. the shipped default: export == screen, and the grade is at rest ==")
	_ok(bridge.world_gen.set_look("Natural Vibrant"), "set_look(Natural Vibrant)")
	var vib_screen := _screen()
	var vib_export := _export("vibrant")
	var c := _cmp(vib_screen, vib_export)
	print("  worst %d levels, %.4f %% of bytes moved, mean %.4f" % [c[0], c[1] * 100.0, c[2]])
	_ok(c[0] >= 0 and c[0] <= 1, "Vibrant: no byte is off by more than the f32 prologue's one level")

	print("\n== 4. Antique Parchment: the graded export == the graded screen ==")
	## THE assertion this probe exists for. Antique grades (temperature 0.26,
	## saturation -0.10, contrast 0.08, shadow tint 0.18); if the export path
	## skipped apply_color_grade this comparison would blow up, exactly the way
	## the missing river tint did at 291,815 bytes / worst 132.
	_ok(bridge.world_gen.set_look("Antique Parchment"), "set_look(Antique Parchment)")
	var ant_screen := _screen()
	var ant_export := _export("antique")
	var c2 := _cmp(ant_screen, ant_export)
	print("  worst %d levels, %.4f %% of bytes moved (%d bytes), mean %.4f"
		% [c2[0], c2[1] * 100.0, int(c2[1] * float(ant_export.size())), c2[2]])
	_ok(c2[0] >= 0 and c2[0] <= 2, "Antique: the graded export matches the graded viewport (worst %d levels)" % c2[0])
	_ok(c2[1] < 0.001, "Antique: fewer than 0.1 %% of bytes differ at all")

	print("\n== 5. and the two exports are not the same picture ==")
	## Non-vacuity: if set_look never reached the export, sections 3 and 4
	## would both pass on identical bytes.
	var c3 := _cmp(vib_export, ant_export)
	print("  Vibrant vs Antique export: worst %d, %.2f %% moved, mean %.2f" % [c3[0], c3[1] * 100.0, c3[2]])
	_ok(c3[1] > 0.5, "the look reaches the export at all")

	print("\n== 6. the grade alone, isolated inside the export ==")
	## Antique with its four grade axes forced to rest, everything else the
	## same look. The difference between this and section 4's export is the
	## colour grade and nothing else, measured through the real binding.
	var rest := {}
	for k in GRADE_KEYS:
		rest[k] = 0.0
	var n: int = bridge.world_gen.set_appearance(rest)
	print("  set_appearance zeroed %d grade keys" % n)
	_ok(n == GRADE_KEYS.size(), "all six grade axes reached the engine")
	var ungraded_export := _export("antique_ungraded")
	var ungraded_screen := _screen()
	var c4 := _cmp(ant_export, ungraded_export)
	print("  graded vs ungraded EXPORT: worst %d, %.2f %% moved, mean %.2f levels" % [c4[0], c4[1] * 100.0, c4[2]])
	var c5 := _cmp(ant_screen, ungraded_screen)
	print("  graded vs ungraded SCREEN: worst %d, %.2f %% moved, mean %.2f levels" % [c5[0], c5[1] * 100.0, c5[2]])
	_ok(c4[1] > 0.5, "the grade moves the export measurably (%.2f %%)" % (c4[1] * 100.0))
	_ok(c4[2] > 2.0, "and by a real amount, not a rounding wobble (mean %.2f levels)" % c4[2])
	## The export must feel the grade the same amount the screen does. Not
	## byte-identical deltas -- local contrast runs over the same 2048 px on
	## both here, so at grid resolution they should be very close indeed.
	_ok(absf(c4[2] - c5[2]) < 1.0,
		"the export feels the grade as strongly as the screen (%.2f vs %.2f levels)" % [c4[2], c5[2]])

	print("\n== 6b. why section 4's worst is 2 and not 1 ==")
	## Section 3 (Vibrant, grade at rest) reads worst 1 -- the f32 bake
	## prologue, exactly as bake_raster.rs and _exportraster_probe.gd section
	## 13 document it. Section 4 (Antique, grade live) reads worst 2. The
	## question is whether that second level is a second defect or the same
	## one amplified, and the way to settle it is to run the SAME look with
	## the grade zeroed: if the extra level is the grade's gain acting on a
	## one-level input difference, this pair must come back at worst 1.
	##
	## Antique's contrast is +0.08, i.e. a slope of 1/(1 - 0.06) = 1.064
	## about mid-grey, and the temperature and tint shifts add their own
	## local gain on top. A one-level pre-grade difference passed through a
	## gain above 1 and re-quantized lands two levels apart whenever both
	## sides straddle a floor boundary -- which is a handful of bytes, not a
	## population.
	var c6 := _cmp(ungraded_screen, ungraded_export)
	print("  Antique WITHOUT the grade, export vs screen: worst %d, %.4f %% moved (%d bytes)"
		% [c6[0], c6[1] * 100.0, int(c6[1] * float(ungraded_export.size()))])
	_ok(c6[0] >= 0 and c6[0] <= 1, "ungraded Antique is back to the f32 prologue's one level")
	_ok(c2[0] <= c6[0] + 1, "the graded pair is the ungraded pair plus at most one level of grade gain")

	print("\n== 7. an eyeball crop, graded vs ungraded vs screen ==")
	## A 512x512 strip of each, side by side, so the numbers above can be
	## checked against a human looking at them.
	var strip := Image.create(512 * 3, 512, false, Image.FORMAT_RGB8)
	var srcs := {"screen": ant_screen, "export": ant_export, "ungraded": ungraded_export}
	var col := 0
	for key in ["screen", "export", "ungraded"]:
		var d: PackedByteArray = srcs[key]
		if d.size() > 0:
			var whole := Image.create_from_data(2048, 1312, false, Image.FORMAT_RGB8, d)
			var crop := whole.get_region(Rect2i(700, 400, 512, 512))
			strip.blit_rect(crop, Rect2i(0, 0, 512, 512), Vector2i(col * 512, 0))
		col += 1
	var sp := dir.path_join("_grade_strip.png")
	_ok(strip.save_png(sp) == OK, "wrote the side-by-side strip")
	print("  %s  (left: graded screen | middle: graded export | right: ungraded export)" % sp)

	print("\n==== %s (%d failures) ====\n" % ["ALL PASS" if fails == 0 else "FAILURES", fails])
	get_tree().quit(1 if fails > 0 else 0)
