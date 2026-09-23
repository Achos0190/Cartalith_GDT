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

## Rasterizes `get_rivers()`'s traced polylines (`points`, grid-cell space)
## into a `w * h` byte mask, `1` within `width_cells / 2` plus a small
## antialiasing margin of a river, `0` elsewhere. Copied from
## `_exportraster_probe.gd` (see that file's own comment for the full
## reasoning) rather than shared, because these are two independent,
## uncommitted probe scripts with no shared module today and duplicating
## ~25 lines is cheaper than inventing one for two callers.
##
## Only valid where the pixel grid IS the cell grid -- true here because
## section 1 regenerates at exactly the exported width (2048 x 1312).
##
## `AA_MARGIN_CELLS` = 24, matching `_exportraster_probe.gd`'s own value and
## its own reasoning: `apply_local_contrast` (render.rs) is a box blur of
## radius ~20 cells at this probe's 2048 px width (`local_contrast_radius_
## frac` default 0.010 * gw, floored/capped -- render.rs's own doc comment
## on `local_contrast_radius`), run on the FINISHED raster after the river
## ink composites, so the river's luma edge (present in the export, absent
## on screen) can shift a pixel's colour up to that radius away from any
## cell an actual river polyline passes through. A narrower margin measured
## real outside-mask divergence there, not a defect. The stamp is a SQUARE,
## as in `_exportraster_probe.gd`, because the blur (`box_h` then `box_v`) is
## one: a disc left the kernel's diagonal corners unmasked -- that probe's
## `AA_MARGIN_CELLS` comment carries the 2026-09-23 measurement.
const AA_MARGIN_CELLS := 24.0

func _river_mask(wg: Object, w: int, h: int) -> PackedByteArray:
	var mask := PackedByteArray()
	mask.resize(w * h)
	var rivers: Array = wg.get_rivers(1)
	for river in rivers:
		var pts: PackedVector2Array = river.get("points", PackedVector2Array())
		var half: float = float(river.get("width_cells", 0.0)) * 0.5 + AA_MARGIN_CELLS
		var r := maxi(1, int(ceil(half)))
		for p in pts:
			var cx := int(round(p.x))
			var cy := int(round(p.y))
			for dy in range(-r, r + 1):
				var yy := cy + dy
				if yy < 0 or yy >= h:
					continue
				for dx in range(-r, r + 1):
					var xx := cx + dx
					if xx < 0 or xx >= w:
						continue
					mask[yy * w + xx] = 1
	return mask

## Same as `_cmp`, split into "outside the river mask" and "inside it".
## **Owner ruling, 2026-09-22: rivers are no longer baked into
## `build_color_texture()`** -- they're drawn as a vector stroke by
## `map_overlay.gd::_draw_rivers()` instead, while `export_raster_png` still
## calls the old baked-ink path unchanged. So every screen/export comparison
## in this probe now has an expected divergence exactly where a river runs,
## and the strict bound this probe exists to enforce (does the export carry
## the same colour grade as the screen) must be asserted on the part of the
## image where the two are still supposed to agree.
func _cmp_masked(a: PackedByteArray, b: PackedByteArray, mask: PackedByteArray) -> Dictionary:
	var out := {
		"nonriver_worst": -1, "nonriver_frac": -1.0, "nonriver_mean": -1.0, "nonriver_bytes": 0,
		"river_worst": -1, "river_frac": -1.0, "river_mean": -1.0, "river_bytes": 0,
	}
	if a.size() == 0 or a.size() != b.size() or mask.size() * 3 != a.size():
		return out
	var nr_worst := 0
	var nr_moved := 0
	var nr_sum := 0
	var nr_n := 0
	var r_worst := 0
	var r_moved := 0
	var r_sum := 0
	var r_n := 0
	for i in range(a.size()):
		var d: int = absi(a[i] - b[i])
		if mask[int(i / 3)] == 1:
			r_n += 1
			if d > 0:
				r_moved += 1
				r_sum += d
				r_worst = maxi(r_worst, d)
		else:
			nr_n += 1
			if d > 0:
				nr_moved += 1
				nr_sum += d
				nr_worst = maxi(nr_worst, d)
	out.nonriver_worst = nr_worst
	out.nonriver_frac = float(nr_moved) / float(maxi(1, nr_n))
	out.nonriver_mean = float(nr_sum) / float(maxi(1, nr_n))
	out.nonriver_bytes = nr_n
	out.river_worst = r_worst
	out.river_frac = float(r_moved) / float(maxi(1, r_n))
	out.river_mean = float(r_sum) / float(maxi(1, r_n))
	out.river_bytes = r_n
	return out

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

	## Rivers don't move when the look/grade changes -- only a regenerate
	## does that -- so one mask, built here, covers every screen/export
	## comparison below. See `_cmp_masked`'s own doc comment for why this is
	## needed at all since 2026-09-22.
	var river_mask := _river_mask(bridge.world_gen, 2048, 1312)
	var mask_px := 0
	for m in river_mask:
		if m == 1:
			mask_px += 1
	print("  river mask: %d of %d pixels (%.2f%%)"
		% [mask_px, river_mask.size(), 100.0 * float(mask_px) / float(maxi(1, river_mask.size()))])
	_ok(mask_px > 0, "this world has river pixels to test the screen/export divergence against")

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
	var c := _cmp_masked(vib_screen, vib_export, river_mask)
	print("  [outside river mask] worst %d levels, %.4f %% of bytes moved, mean %.4f"
		% [c.nonriver_worst, c.nonriver_frac * 100.0, c.nonriver_mean])
	print("  [inside river mask]  worst %d levels, %.4f %% of bytes moved, mean %.4f (%d bytes)"
		% [c.river_worst, c.river_frac * 100.0, c.river_mean, c.river_bytes])
	_ok(c.nonriver_worst >= 0 and c.nonriver_worst <= 1,
		"Vibrant: outside the river mask, no byte is off by more than the f32 prologue's one level")

	print("\n== 4. Antique Parchment: the graded export == the graded screen ==")
	## THE assertion this probe exists for. Antique grades (temperature 0.26,
	## saturation -0.10, contrast 0.08, shadow tint 0.18); if the export path
	## skipped apply_color_grade this comparison would blow up, exactly the way
	## the missing river tint did at 291,815 bytes / worst 132.
	_ok(bridge.world_gen.set_look("Antique Parchment"), "set_look(Antique Parchment)")
	var ant_screen := _screen()
	var ant_export := _export("antique")
	var c2 := _cmp_masked(ant_screen, ant_export, river_mask)
	print("  [outside river mask] worst %d levels, %.4f %% of bytes moved (%d bytes), mean %.4f"
		% [c2.nonriver_worst, c2.nonriver_frac * 100.0, int(c2.nonriver_frac * float(c2.nonriver_bytes)), c2.nonriver_mean])
	_ok(c2.nonriver_worst >= 0 and c2.nonriver_worst <= 2,
		"Antique: outside the river mask, the graded export matches the graded viewport (worst %d levels)" % c2.nonriver_worst)
	_ok(c2.nonriver_frac < 0.001, "Antique: fewer than 0.1 %% of non-river bytes differ at all")
	print("  [inside river mask]  worst %d levels, %.4f %% of bytes moved (%d bytes), mean %.4f"
		% [c2.river_worst, c2.river_frac * 100.0, int(c2.river_frac * float(c2.river_bytes)), c2.river_mean])
	## Non-vacuity: the export still inks rivers (the old baked path,
	## deliberately untouched), the screen no longer does at all -- if this
	## region came back near-identical it would mean either the mask missed
	## the rivers or a future change silently re-added the screen-side bake.
	## `10` is far above the f32-prologue ceiling of `1` and far below a real
	## colour swap; either regression would fail this line.
	_ok(c2.river_worst > 10 and c2.river_frac > 0.0,
		"the river-masked region really does differ between screen and export (worst %d levels, %.4f %% moved)"
			% [c2.river_worst, c2.river_frac * 100.0])

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
	var c6 := _cmp_masked(ungraded_screen, ungraded_export, river_mask)
	print("  [outside river mask] Antique WITHOUT the grade, export vs screen: worst %d, %.4f %% moved (%d bytes)"
		% [c6.nonriver_worst, c6.nonriver_frac * 100.0, int(c6.nonriver_frac * float(c6.nonriver_bytes))])
	_ok(c6.nonriver_worst >= 0 and c6.nonriver_worst <= 1,
		"ungraded Antique is back to the f32 prologue's one level outside the river mask")
	_ok(c2.nonriver_worst <= c6.nonriver_worst + 1,
		"the graded pair is the ungraded pair plus at most one level of grade gain (outside the river mask)")

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
