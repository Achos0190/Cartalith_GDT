extends Node
## PIXEL VERIFY, WINDOWED: do trait badges reach the map, and does a
## below-placed label now clear them?
##
## `MISTAKES.md`: *"`ImageTexture.update()` is a no-op under `--headless`"* and
## *"a number with nothing to compare against cannot tell you whose defect it
## is"*. So this runs windowed, and every claim is a difference between two
## frames drawn by the same code path in the same frame budget:
##
##   Godot_v4.7.1-stable_win64.exe --path . _traitbadge_probe.tscn
##
## SubViewports over one fixture at a time -- a single settlement pin whose
## `above` label candidate is blocked by a manual icon, so its name is forced
## onto the `below` candidate, which is the one candidate `_civTraitDrop`
## moves:
##
##   blank     background only, so "the overlay drew ink" is falsifiable
##   base      shipping,   no traits, no name    -- pin alone
##   base_pre  PRE-CHANGE, no traits, no name    -- must equal `base`
##   badges    shipping,   traits,    no name    -- pin + badge row
##   before    PRE-CHANGE, traits,    name       -- label at drop 0
##   notraits  shipping,   no traits, name       -- must equal `before`
##   after     shipping,   traits,    name       -- label past the badges
##
## `_PreChange` is this file's own subclass of the shipping overlay with the
## two new behaviours taken back out and nothing else touched, so it renders
## exactly what this tree rendered before 2026-09-04. That is what "before"
## means here -- not a remembered screenshot.
##
## Masks are differences, so nothing has to be recognised by colour:
##   badge ink  = badges  XOR base    (the only difference is the badge row)
##   label@0    = before  XOR base    (`before` draws no badges)
##   label@drop = after   XOR badges  (both draw the same badge row)
## and the overlap the row is about is the per-pixel intersection of the badge
## ink with each label's ink.
##
## **Three fixtures, not one.** `MISTAKES.md`: *"one world is one sample, and
## panel widths are content-dependent"*. A label box is `name.length` wide and
## sits `(4+rank)*sc` from the pin, so the overlap is a function of both --
## the three fixtures vary tier and name length together, and each is reported.

## 2400 wide because the pin is sized `(4+rank) * (width/1400) * civZoomK`:
## at 900 px a city pin is 4.5 px across and its badges clamp to the 2.2
## floor, which is too small to measure anything on. At 2400 the scale is
## exactly `12/7` and a city pin is 12 px.
const W := 2400
const H := 1200
const GW := 96
const GH := 48
## Grid cell of the one settlement, and the point the blocking icon sits at.
## `_cell_to_screen` centres a cell, `_point_to_screen` does not.
const CELL := Vector2(48, 24)
const ICON_AT := Vector2(48.5, 23.5)
## The scan window, in screen pixels: from the pin's own top edge down past
## where the largest fixture's dropped label can land. Deliberately excludes
## the blocking icon above the pin -- it is identical in every arm and cancels
## in every difference, but leaving it out keeps the printed bboxes readable.
const SX := 1130
const SY := 598
const SW := 180
const SH := 110

const TRAITS := ["port", "mining", "military"]
## Four is `traits.slice(0,4)`'s cap; ALL is every key in `CIV_TRAITS`. The
## two must draw the same row, which is what pins the cap to 4 rather than to
## "however many there are".
const TRAITS_FOUR := ["port", "mining", "military", "fortified"]
const TRAITS_ALL := ["port", "mining", "military", "fortified", "trade_hub",
	"religious", "administrative"]
const GLYPHS := {"port": "⚓", "mining": "⚒", "military": "⚔",
	"fortified": "⬢", "trade_hub": "♣", "religious": "✝",
	"administrative": "♜"}

## `[name, kind]`. Tier sets the pin radius and so the badge radius and the
## drop; name length sets the label box's width.
const FIXTURES := [["Ka", "city"], ["Vandermeer", "metropolis"], ["Ost", "town"]]


## The shipping overlay with 2026-09-04's two behaviours removed and nothing
## else changed: no badge row, and lblCandidates' drop back at 0.
class _PreChange extends "res://map_overlay.gd":
	func _draw_trait_badges(_s: Dictionary, _pos: Vector2, _radius: float,
			_sc: float, _k: float, _font: Font) -> void:
		pass

	func _trait_drop(_s: Dictionary, _radius: float, _sc: float) -> float:
		return 0.0


var _fails := 0

func _chk(ok: bool, what: String) -> void:
	print(("  PASS  " if ok else "  FAIL  ") + what)
	if not ok:
		_fails += 1


static func _places(nm: String, kind: String) -> Array:
	return [{
		"x": CELL.x, "y": CELL.y, "name": nm, "population": 41000,
		"kind": kind, "faction": 1, "capital": false, "coastal": false,
		"tid": 7,
	}]


## One manual icon straddling the `above` candidate box, so the name is forced
## onto `below`. `_seed_label_occupancy` reserves ICON_BASE_RADIUS * scale
## around it; at scale 2 that is 11 px. Whether it actually blocked is
## asserted per fixture rather than assumed -- if the name stayed above the
## pin, every overlap below would read 0 for the wrong reason.
static func _blockers() -> Array:
	return [{"x": ICON_AT.x, "y": ICON_AT.y, "family": "feature",
		"slot": "mountain", "set": "", "scale": 2.0}]


func _arm(script_res) -> Array:
	var vp := SubViewport.new()
	vp.size = Vector2i(W, H)
	vp.transparent_bg = false
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	var bg := ColorRect.new()
	bg.color = Color(0.09, 0.10, 0.11)
	bg.size = Vector2(W, H)
	vp.add_child(bg)
	add_child(vp)
	if script_res == null:
		return [vp, null]
	var ov: Control = Control.new()
	ov.set_script(script_res)
	ov.size = Vector2(W, H)
	vp.add_child(ov)
	return [vp, ov]


func _push(arm: Array, traits_on: bool, nm: String, kind: String,
		keys: Array = TRAITS, glyphs: Dictionary = GLYPHS) -> void:
	var ov: Control = arm[1]
	ov.set_civ_data(_places(nm, kind), [], [], GW, GH, 0.0)
	ov.set_manual_icons(_blockers())
	ov.set_settlement_traits({7: keys} if traits_on else {}, glyphs)


func _shot(arm: Array) -> Image:
	return (arm[0] as SubViewport).get_texture().get_image()


## A per-pixel mask over the scan window: true wherever the two frames differ.
static func _mask(a: Image, b: Image) -> Array:
	var m: Array = []
	m.resize(SW * SH)
	for j in SH:
		for i in SW:
			m[j * SW + i] = a.get_pixel(SX + i, SY + j) != b.get_pixel(SX + i, SY + j)
	return m


static func _count(m: Array) -> int:
	var n := 0
	for v in m:
		if v:
			n += 1
	return n


## `[y_top, y_bottom, x_left, x_right]` in absolute screen pixels, or an empty
## array when the mask is empty -- which is a result, not a measurement
## failure, and is why every caller asserts non-emptiness first.
static func _bbox(m: Array) -> Array:
	var y0 := SH
	var y1 := -1
	var x0 := SW
	var x1 := -1
	for j in SH:
		for i in SW:
			if m[j * SW + i]:
				y0 = mini(y0, j)
				y1 = maxi(y1, j)
				x0 = mini(x0, i)
				x1 = maxi(x1, i)
	if y1 < 0:
		return []
	return [SY + y0, SY + y1, SX + x0, SX + x1]


static func _both(a: Array, b: Array) -> int:
	var n := 0
	for i in a.size():
		if a[i] and b[i]:
			n += 1
	return n


func _settle() -> void:
	for f in 4:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG -- probe did not finish")
		get_tree().quit(2))
	wd.start()

	if DisplayServer.get_name() == "headless":
		print("ABORT: this probe measures pixels and cannot run headless -- ")
		print("  Godot_v4.7.1-stable_win64.exe --path . _traitbadge_probe.tscn")
		get_tree().quit(2)
		return

	var ship = load("res://map_overlay.gd")
	var blank := _arm(null)
	var base := _arm(ship)
	var base_pre := _arm(_PreChange)
	var badges := _arm(ship)
	var before := _arm(_PreChange)
	var notraits := _arm(ship)
	var after := _arm(ship)
	var extra := _arm(ship)

	var pin_y := int(round((CELL.y + 0.5) / float(GH) * float(H)))
	var pin_x := int(round((CELL.x + 0.5) / float(GW) * float(W)))
	var dir := "user://traitbadge"
	DirAccess.make_dir_recursive_absolute(dir)

	for fx in FIXTURES:
		var nm: String = fx[0]
		var kind: String = fx[1]
		_push(base, false, "", kind)
		_push(base_pre, false, "", kind)
		_push(badges, true, "", kind)
		_push(before, true, nm, kind)
		_push(notraits, false, nm, kind)
		_push(after, true, nm, kind)
		await _settle()

		var i_blank := _shot(blank)
		var i_base := _shot(base)
		var i_base_pre := _shot(base_pre)
		var i_badges := _shot(badges)
		var i_before := _shot(before)
		var i_notraits := _shot(notraits)
		var i_after := _shot(after)
		i_after.save_png("%s/after_%s.png" % [dir, kind])
		i_before.save_png("%s/before_%s.png" % [dir, kind])

		print("")
		print("================ fixture: \"%s\", a %s ================" % [nm, kind])
		_chk(not i_blank.get_data().is_empty(), "the blank reference frame captured something")
		_chk(i_base.get_data() != i_blank.get_data(), "the overlay drew ink over the background")

		print("-- rule 3: a settlement with no traits is untouched by this change --")
		_chk(i_base.get_data() == i_base_pre.get_data(),
			"pin-only frame is BYTE-IDENTICAL to the pre-change build")
		_chk(i_notraits.get_data() == i_before.get_data(),
			"pin+label frame with no traits is BYTE-IDENTICAL to the pre-change build")

		print("-- row 1: the badge row reaches the map --")
		var m_badge := _mask(i_badges, i_base)
		var n_badge := _count(m_badge)
		var b_badge := _bbox(m_badge)
		_chk(n_badge > 0, "pushing traits moves pixels -- badges are drawn (%d px)" % n_badge)
		if b_badge.is_empty():
			print("  no badge ink; nothing further is measurable for this fixture")
			continue
		print("badge bbox: y %d..%d  x %d..%d" % [b_badge[0], b_badge[1], b_badge[2], b_badge[3]])
		_chk(b_badge[0] > pin_y, "the row sits BELOW the pin centre (top %d vs pin %d)"
			% [b_badge[0], pin_y])
		var row_mid: float = float(int(b_badge[2]) + int(b_badge[3])) / 2.0
		_chk(absf(row_mid - float(pin_x)) <= 1.0,
			"and is centred on the pin (row mid %.1f vs pin x %d)" % [row_mid, pin_x])

		print("-- row 2: the below-label's overlap with the badges, before and after --")
		var m_lbl_before := _mask(i_before, i_base)
		var m_lbl_after := _mask(i_after, i_badges)
		var b_lbl_before := _bbox(m_lbl_before)
		var b_lbl_after := _bbox(m_lbl_after)
		_chk(not b_lbl_before.is_empty(), "the pre-change label drew ink (%d px)" % _count(m_lbl_before))
		_chk(not b_lbl_after.is_empty(), "the shipping label drew ink (%d px)" % _count(m_lbl_after))
		if b_lbl_before.is_empty() or b_lbl_after.is_empty():
			print("  a label arm drew nothing; overlap is not measurable for this fixture")
			continue
		_chk(b_lbl_before[0] > pin_y, "the name really was forced BELOW the pin (top %d vs pin %d)"
			% [b_lbl_before[0], pin_y])
		print("label top y   before %d   after %d   (moved %d px down)"
			% [b_lbl_before[0], b_lbl_after[0], int(b_lbl_after[0]) - int(b_lbl_before[0])])
		var ov_before := _both(m_badge, m_lbl_before)
		var ov_after := _both(m_badge, m_lbl_after)
		print("OVERLAP with the badge row:   before %d px   after %d px" % [ov_before, ov_after])
		_chk(ov_before > 0,
			"the pre-change label really did overprint the badges (%d px) -- without this the fix has nothing to fix"
			% ov_before)
		_chk(ov_after == 0, "and the shipping label overlaps them by %d px" % ov_after)
		_chk(b_lbl_after[0] > b_badge[1],
			"the label now starts below the badge row's last pixel (%d > %d)"
			% [b_lbl_after[0], b_badge[1]])
		_chk(b_lbl_after[0] > b_lbl_before[0],
			"positive control: the label MOVED (%d px down)"
			% (int(b_lbl_after[0]) - int(b_lbl_before[0])))

		print("-- a trait with no vocabulary entry draws nothing, and the cap is 4 --")
		_push(extra, true, "", kind, TRAITS, {})
		await _settle()
		_chk(_shot(extra).get_data() == i_base.get_data(),
			"three traits with an EMPTY glyph vocabulary draw no disc and no glyph")
		_push(extra, true, "", kind, TRAITS_FOUR)
		await _settle()
		var b_four := _bbox(_mask(_shot(extra), i_base))
		_push(extra, true, "", kind, TRAITS_ALL)
		await _settle()
		var b_seven := _bbox(_mask(_shot(extra), i_base))
		_chk(not b_four.is_empty() and not b_seven.is_empty(),
			"both cap arms drew a row to compare")
		if not b_four.is_empty() and not b_seven.is_empty():
			var w4: int = int(b_four[3]) - int(b_four[2])
			var w7: int = int(b_seven[3]) - int(b_seven[2])
			print("row width: 3 traits %d px, 4 traits %d px, 7 traits %d px"
				% [int(b_badge[3]) - int(b_badge[2]), w4, w7])
			_chk(w4 == w7, "a 7-trait settlement draws the same row as a 4-trait one (%d vs %d px)"
				% [w4, w7])
			_chk(w4 > int(b_badge[3]) - int(b_badge[2]),
				"and a 4-trait row IS wider than a 3-trait one, so the cap is 4 and not 3")

	print("")
	print("IMAGES: ", ProjectSettings.globalize_path(dir))
	print("TRAITBADGE RESULT: ", "ALL PASS" if _fails == 0 else "%d FAILURES" % _fails)
	get_tree().quit(0 if _fails == 0 else 1)
