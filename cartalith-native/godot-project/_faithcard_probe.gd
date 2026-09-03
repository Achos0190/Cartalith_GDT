extends Node
## PIXEL VERIFY: does the population-0 clause reach the screen, or only the
## return value of `_faith_lines`?
##
## `MISTAKES.md`: *"Reasoning from the scene graph proves nothing under an
## opaque overlay -- flip the flag and diff the framebuffer."* The equivalent
## flip here is the shipping `map_overlay.gd` against a subclass of itself
## whose only difference is that the clause is stripped back off the head line
## -- the exact "before" of this change, drawn by the same code path, in the
## same viewport, on the same frame budget (`_cull_probe.gd`'s own pattern).
##
## Three claims, and the third is the one the second is worthless without:
##
##   1. the two frames DIFFER -- the clause moves pixels;
##   2. the card actually grows, measured OUTSIDE this file from the accent
##      border `Color(0.878, 0.639, 0.290)` the card is stroked with -- the
##      frames are saved as PNGs and the geometry read off them;
##   3. both arms drew ink at all, against a third viewport holding the
##      background and no overlay. Two blank frames are byte-identical, so
##      claim 1 alone is satisfied by an overlay that never drew.
##
## Runs WINDOWED. Under `--headless` Godot loads the dummy display driver and
## `RenderingServer.frame_post_draw` never fires (`_cull_probe.gd` measured
## this on 2026-09-01), so a headless run hangs to the watchdog having proven
## nothing:
##
##   Godot_v4.7.1-stable_win64.exe --path . _faithcard_probe.tscn

const W := 900
const H := 600
## The card's own border, `_draw_hover_card`'s second `draw_rect`.
const BORDER := Color(0.878, 0.639, 0.290)


class _NoClause extends "res://map_overlay.gd":
	## The shipping file with one change: the population-0 clause removed
	## again. Nothing else differs, so any pixel that moves is that clause.
	func _faith_lines(s: Dictionary) -> Array:
		var out: Array = super(s)
		if not out.is_empty():
			out[0] = String(out[0]).replace(" -- no population, so no share", "")
		return out


var _fails := 0

func _chk(ok: bool, what: String) -> void:
	print(("  PASS  " if ok else "  FAIL  ") + what)
	if not ok:
		_fails += 1


static func _places() -> Array:
	## One hamlet at population 0 -- the case under test -- and one town with
	## people, so the frame is not a single pin and the control arm has
	## something to leave alone.
	return [
		{"x": 20.0, "y": 24.0, "name": "Kadzafirskadir", "population": 0, "kind": "hamlet",
			"faction": 1, "capital": false, "coastal": false, "tid": 1,
			"religion": "sun_cult", "adherents": {}},
		{"x": 44.0, "y": 30.0, "name": "Sevjuniana", "population": 17916, "kind": "town",
			"faction": 1, "capital": true, "coastal": false, "tid": 2,
			"religion": "none", "adherents": {"none": 13000, "sun_cult": 4916}},
	]


## The card's drawn width in pixels, read off its own accent border. Scans the
## upper half only: `_draw_hover_card` anchors the card above the pin and both
## fixture pins sit in the top third, so a full-frame scan would be 540 000
## `get_pixel` calls for the same answer.
##
## `0` when no card is on the frame at all -- a result, not a failure to
## measure: it is exactly what "the overlay drew nothing" looks like, which is
## why the caller asserts the CONTROL arm's width is nonzero before comparing.
static func _card_width(img: Image) -> int:
	var lo := W
	var hi := -1
	for y in range(0, H / 2):
		for x in range(0, W):
			var c := img.get_pixel(x, y)
			if absf(c.r - BORDER.r) < 0.06 and absf(c.g - BORDER.g) < 0.06 					and absf(c.b - BORDER.b) < 0.06:
				lo = mini(lo, x)
				hi = maxi(hi, x)
	return 0 if hi < 0 else hi - lo + 1


func _mk(script_res) -> Array:
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
	ov.set_civ_data(_places(), [], [], 96, 64, 0.0)
	return [vp, ov]


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG -- probe did not finish")
		get_tree().quit(2))
	wd.start()

	if DisplayServer.get_name() == "headless":
		print("ABORT: this probe measures pixels and cannot run headless -- ")
		print("  Godot_v4.7.1-stable_win64.exe --path . _faithcard_probe.tscn")
		get_tree().quit(2)
		return

	var blank := _mk(null)
	var ship := _mk(load("res://map_overlay.gd"))
	var ctrl := _mk(_NoClause)

	## Hover the population-0 settlement in both arms. Index 0 of `_places()`.
	ship[1]._hover_index = 0
	ctrl[1]._hover_index = 0
	ship[1].queue_redraw()
	ctrl[1].queue_redraw()
	for f in 4:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw

	var img_blank: Image = blank[0].get_texture().get_image()
	var img_ship: Image = ship[0].get_texture().get_image()
	var img_ctrl: Image = ctrl[0].get_texture().get_image()
	var d_blank := img_blank.get_data()
	var d_ship := img_ship.get_data()
	var d_ctrl := img_ctrl.get_data()

	_chk(not d_blank.is_empty(), "the blank reference frame captured something at all")
	_chk(d_ship != d_blank, "the shipping overlay drew ink (not a blank frame)")
	_chk(d_ctrl != d_blank, "the control arm drew ink too")

	## The card geometry is measured OUTSIDE this file. `Image.get_pixel` over
	## 900x600 twice is a GDScript loop long enough to look like a hang, and a
	## PNG on disk is a re-checkable artefact rather than a number in a log.
	var dir := "user://faithcard"
	DirAccess.make_dir_recursive_absolute(dir)
	img_ship.save_png(dir + "/ship.png")
	img_ctrl.save_png(dir + "/ctrl.png")
	img_blank.save_png(dir + "/blank.png")
	print("IMAGES: ", ProjectSettings.globalize_path(dir))
	var diff := 0
	for i in mini(d_ship.size(), d_ctrl.size()):
		if d_ship[i] != d_ctrl[i]:
			diff += 1
	print("bytes differing between the two frames: %d of %d" % [diff, d_ship.size()])
	_chk(diff > 0, "the clause moves pixels -- the two frames are not identical (%d bytes)"
		% diff)

	var t0 := Time.get_ticks_msec()
	var w_ship := _card_width(img_ship)
	var w_ctrl := _card_width(img_ctrl)
	print("card width with the clause: %d px ; without: %d px (scan %d ms)"
		% [w_ship, w_ctrl, Time.get_ticks_msec() - t0])
	_chk(w_ctrl > 0, "the control arm actually drew a card to compare against (%d px)" % w_ctrl)
	_chk(w_ship > w_ctrl,
		"and the card is wider with the clause: %d px vs %d px (+%d)"
		% [w_ship, w_ctrl, w_ship - w_ctrl])

	## The one settlement in this fixture that HAS people must be unaffected --
	## otherwise "wider" could be a global restyle rather than this clause.
	ship[1]._hover_index = 1
	ctrl[1]._hover_index = 1
	ship[1].queue_redraw()
	ctrl[1].queue_redraw()
	for f in 4:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	var p_ship: PackedByteArray = ship[0].get_texture().get_image().get_data()
	var p_ctrl: PackedByteArray = ctrl[0].get_texture().get_image().get_data()
	_chk(p_ship != d_blank, "the populated-settlement card drew ink")
	_chk(p_ship == p_ctrl,
		"and the populated settlement's card is byte-identical between the two arms")

	print("")
	print("FAITHCARD RESULT: ", "ALL PASS" if _fails == 0 else "%d FAILURES" % _fails)
	get_tree().quit(0 if _fails == 0 else 1)
