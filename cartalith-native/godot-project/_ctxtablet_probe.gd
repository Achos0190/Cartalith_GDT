extends Node
## CM-4 (`MAP_CONTEXT_SCOPE.md` §7.1, §11): the tablet's hold -> ring + card
## with continued tracking and slide-to-select (`shell/radial_ring.gd`,
## `shell/context_broker.gd::ring_touch_open()`, and `map_overlay.gd`'s
## `_touch_ring_active` branch of its withheld-touch-press machinery).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _ctxtablet_probe.tscn -- --vp 2560x1600 --force-touch
##
## **Windowed, not `--headless`**: leg E reads the ring/card rects against the
## real viewport size. `--vp WxH` sizes the host SubViewport (default
## 1600x900, which boots as a laptop, not a tablet -- use a tablet-sized one);
## `--force-touch` is the shell's own flag (`DccShell._ready()`), and both are
## required or the probe aborts (leg `FORM`).
##
## Every gesture is pushed through the real input path -- `ov._gui_input()`,
## the same one `_ctxcard_probe.gd`'s leg C already drives a touch hold
## through (`InputEvent.DEVICE_ID_EMULATION`, which is `map_overlay.gd`'s own
## `mb.device < 0` gate for the withheld-touch-press branch).
##
## Legs:
##   H  a still touch hold (>= 500 ms, `_TOUCH_HOLD_MS`) opens the ring AND
##      the card together, and the finger is still TRACKED afterward (the
##      withheld press is never released as a click/drag)
##   S  sliding from the hold point onto a slot and lifting arms that tool --
##      asserted through the real `app.armed_tool`, not the ring's own hover
##   Z  lifting in the dead zone (no slide) leaves BOTH the ring and the card
##      open, sticky, rather than picking anything or closing either
##   D  the dominant-hand preference flips which side the card docks on --
##      measured off `panel_rect()` against the touch point, both hands, with
##      the preference restored to whatever it was before the probe ran
##   E  the ring's own rect AND the card's own rect stay on screen at all
##      four screen edges, on the tablet touch-hold path specifically (CM-3's
##      `_ctxring_probe.gd` leg E already covers the desktop Q/RMB path)
##   G  Ruling BC (`LARGE_ITEM_RULINGS.md`) on the TOUCH-SCALED geometry --
##      the disc and the centre button both scale up alongside the already-
##      probed 96 dp ring radius (leg T), rather than staying at their
##      desktop figure while the ring around them grows
##
## Haptics (§7.1's "sample"/"tool_arm" pulses, `DccShell._haptic()`) are a
## no-op off Android/iOS by that function's own guard (`OS.has_feature
## ("mobile")`) -- this probe runs on the desktop windowed host and so cannot
## observe a vibration either way. Not tested here; not claimed either.

const SEED := 719203

## `radial_ring.gd`'s own DESKTOP geometry constants, duplicated here rather
## than imported -- `_ctxring_probe.gd`'s own header gives the reason: this
## probe measures the SHIPPED geometry, not an assumption it stays in sync.
## **Not used for the touch pad below any more** -- CM-4 residual
## (`OUTSTANDING_WORK.md`): the tablet ring's radius/slot are touch-scaled
## (`radial_ring.gd::_apply_touch_scale()`), and asserting the edge check
## against these bare desktop figures would be exactly the "assert a constant
## against itself" mistake `MISTAKES.md`'s preflight table opens with -- it
## would silently under-size `pad` and let a real off-screen ring pass leg E.
## Kept only for leg H/T's own radius-floor assertion, which explicitly
## compares the LIVE value to this desktop one.
## Ruling BC, coordinator correction 2026-09-29: track the shipped
## `radial_ring.gd::RING_RADIUS`/`SLOT_SIZE`, now the mockup's own absolute
## desktop figures (92 / 60) rather than this shell's old independently-tuned
## ones (60 / 46).
const RING_RADIUS := 92.0
const SLOT_SIZE := 60.0
const DIR_DEG := {
	"N": -90.0, "NE": -45.0, "E": 0.0, "SE": 45.0,
	"S": 90.0, "SW": 135.0, "W": 180.0, "NW": -135.0,
}

static func _dir_vec(dir: String) -> Vector2:
	var a := deg_to_rad(float(DIR_DEG[dir]))
	return Vector2(cos(a), sin(a))

var app: Node
var _vp: SubViewport
var _fails := 0
var _checks := 0


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, got: Variant, want: Variant) -> void:
	_checks += 1
	var pass_: bool = typeof(got) == typeof(want) and got == want
	if not pass_:
		_fails += 1
	print("%s  %s  got=%s want=%s" % ["PASS" if pass_ else "FAIL", what, str(got), str(want)])


func _broker():
	return app.get("context_broker")


func _ring():
	var b = _broker()
	return b.ring if b != null else null


func _ring_state() -> Dictionary:
	var r = _ring()
	return r.debug_state() if r != null else {}


func _ring_close() -> void:
	var r = _ring()
	if r != null:
		r.close()
	await _frames(2)


func _card():
	var b = _broker()
	return b.card if b != null else null


func _card_open() -> bool:
	var c = _card()
	return c != null and c.visible


## Leg O (coordinator review, 2026-09-27): the 8 slot rects the LIVE ring is
## actually drawing at, one per `DIR_DEG` direction -- centred at
## `debug_state()`'s own `centre`, at its own live `radius`, each `slot` px
## square. Never a re-derived formula off the desktop consts: this is
## precisely the "assert a constant against itself" trap the whole CM-4
## residual exists to close, one level up (the CARD's placement, not just
## the ring's own size).
func _ring_slot_rects() -> Array[Rect2]:
	var st := _ring_state()
	var c: Vector2 = st.get("centre", Vector2.ZERO)
	var radius: float = st.get("radius", 0.0)
	var slot: float = st.get("slot", 0.0)
	var out: Array[Rect2] = []
	for dir in DIR_DEG:
		var p: Vector2 = c + _dir_vec(dir) * radius
		out.append(Rect2(p - Vector2.ONE * slot * 0.5, Vector2.ONE * slot))
	return out


## True (with a printed reason) the first time `rect` intersects any of the
## ring's own live slot rects -- so a caller gets which direction collided,
## not just a bare pass/fail.
func _card_overlaps_ring(rect: Rect2) -> bool:
	var slots := _ring_slot_rects()
	var dirs := DIR_DEG.keys()
	for i in slots.size():
		if rect.intersects(slots[i]):
			print("O overlap: card=%s hits slot %s=%s" % [rect, dirs[i], slots[i]])
			return true
	return false


## Leg O's own follow-up (coordinator review, 2026-09-27, "the context card
## can overlap an open sub-ring") -- `_ctxring_probe.gd`'s own twin helper,
## same shape: the sub-ring's own live slot rects off `debug_state()`'s
## `sub_centre`/`sub_radius`/`sub_slot_size`/`sub_count`, `[]` when no
## sub-ring is open.
func _sub_slot_rects() -> Array[Rect2]:
	var st := _ring_state()
	if not bool(st.get("sub_open", false)):
		return []
	var c: Vector2 = st.get("sub_centre", Vector2.ZERO)
	var radius: float = st.get("sub_radius", 0.0)
	var slot: float = st.get("sub_slot_size", 0.0)
	var n: int = int(st.get("sub_count", 0))
	var out: Array[Rect2] = []
	for i in n:
		var ang := deg_to_rad(-90.0 + 360.0 * float(i) / float(maxi(1, n)))
		var p: Vector2 = c + Vector2(cos(ang), sin(ang)) * radius
		out.append(Rect2(p - Vector2.ONE * slot * 0.5, Vector2.ONE * slot))
	return out


func _card_overlaps_sub(rect: Rect2) -> bool:
	var slots := _sub_slot_rects()
	for i in slots.size():
		if rect.intersects(slots[i]):
			print("O sub-overlap: card=%s hits sub slot #%d=%s" % [rect, i, slots[i]])
			return true
	return false


func _card_close() -> void:
	var c = _card()
	if c != null and c.visible:
		c.hide()
	await _frames(2)


static func _close_color(a: Color, b: Color, tol: float) -> bool:
	return absf(a.r - b.r) <= tol and absf(a.g - b.g) <= tol and absf(a.b - b.b) <= tol


## `_ctxring_probe.gd`'s own leg-L helper, duplicated rather than imported
## (same reason every geometry const in this file is duplicated, not
## shared) -- see that copy's header for the median-luminance reasoning.
static func _label_inside_circle(img: Image, centre: Vector2, radius: float) -> Dictionary:
	var steps := 40
	var radii := [radius + 2.0, radius + 4.0, radius + 6.0]
	var samples: Array = []
	for i in steps:
		var ang := TAU * float(i) / float(steps)
		for rr in radii:
			var p: Vector2 = centre + Vector2(cos(ang), sin(ang)) * float(rr)
			var pi := Vector2i(int(p.x), int(p.y))
			if pi.x < 0 or pi.y < 0 or pi.x >= img.get_width() or pi.y >= img.get_height():
				continue
			var px := img.get_pixel(pi.x, pi.y)
			samples.append({"p": p, "px": px, "lum": px.get_luminance()})
	if samples.is_empty():
		return {"ok": true}
	var lums: Array = []
	for s in samples:
		lums.append(s["lum"])
	lums.sort()
	var median: float = lums[lums.size() / 2]
	for s in samples:
		if absf(float(s["lum"]) - median) > 0.10:
			return {"ok": false, "at": s["p"], "px": (s["px"] as Color).to_html(false)}
	return {"ok": true}


## `InputEvent.DEVICE_ID_EMULATION`: `map_overlay.gd`'s own `mb.device < 0`
## gate for the withheld-touch-press branch -- the same marker
## `_ctxcard_probe.gd`'s leg C and `_ctxbroker_probe.gd` already use to drive a
## touch gesture on a desktop test host with no real touchscreen.
func _touch_press(ov: Control, pos: Vector2) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = true
	ev.position = pos
	ev.device = InputEvent.DEVICE_ID_EMULATION
	ov._gui_input(ev)


func _touch_release(ov: Control, pos: Vector2) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = false
	ev.position = pos
	ev.device = InputEvent.DEVICE_ID_EMULATION
	ov._gui_input(ev)


func _touch_move(ov: Control, pos: Vector2) -> void:
	var mv := InputEventMouseMotion.new()
	mv.position = pos
	mv.button_mask = MOUSE_BUTTON_MASK_LEFT
	mv.device = InputEvent.DEVICE_ID_EMULATION
	ov._gui_input(mv)


## Press, then wait past `_TOUCH_HOLD_MS` (500) and settle -- the point at
## which `map_overlay.gd`'s own timer has already fired and the ring+card are
## open with the finger still tracked (`_touch_ring_active`).
func _touch_hold(ov: Control, pos: Vector2) -> void:
	_touch_press(ov, pos)
	await get_tree().create_timer(0.6).timeout
	await _frames(2)


func _run() -> void:
	var ov: Control = app.viewport.overlay
	var vis: Rect2 = app.get_viewport().get_visible_rect()
	var centre := vis.size * 0.5

	# -- H: a still touch hold opens the ring AND the card together -------------
	app.select_domain("world")
	await _frames(4)
	app.arm_tool("inspect")
	await _frames(2)
	await _touch_hold(ov, centre)
	_ok("H tablet hold >= 500ms: the ring is visible", _ring_state().get("visible", false), true)
	_ok("H tablet hold >= 500ms: the card opened alongside it", _card_open(), true)
	## The withheld press must not have resolved as a plain tap/click either --
	## nothing armed changed just from opening the surfaces.
	_ok("H opening the ring+card armed nothing by itself", app.armed_tool, "inspect")
	## CM-4 residual (`OUTSTANDING_WORK.md`; `MAP_CONTEXT_SCOPE.md` §7.2:
	## "Slots sit at 96 dp on touch (60 px on desktop)"). Measured from the
	## LIVE node (`debug_state()`'s `radius`, `radial_ring.gd::_r_ring`) rather
	## than re-declaring the constant and asserting it against itself
	## (`MISTAKES.md`'s preflight rule) -- this is the whole reason CM-4's
	## residual row asked for it.
	var live_radius: float = _ring_state().get("radius", 0.0)
	print("T touch ring radius: live=%.2f px  desktop_const=%.1f px  spec_floor=96 dp" %
		[live_radius, RING_RADIUS])
	_ok("T touch ring radius is >= 96 dp", live_radius >= 96.0, true)
	_ok("T touch ring radius actually grew off the desktop 60 px figure",
		live_radius > RING_RADIUS, true)
	_touch_release(ov, centre)
	await _frames(2)

	# -- Z: lifting in the dead zone (no slide) leaves both open, sticky --------
	_ok("Z dead-zone lift: the ring stays open", _ring_state().get("active", false), true)
	_ok("Z dead-zone lift: the ring is sticky", _ring_state().get("sticky", false), true)
	_ok("Z dead-zone lift: the card stays open", _card_open(), true)
	_ok("Z dead-zone lift: nothing armed", app.armed_tool, "inspect")
	await _ring_close()
	await _card_close()
	app.arm_tool("inspect")
	await _frames(2)

	# -- S: sliding onto a slot and lifting arms that tool -----------------------
	## W is Region on every domain (F3, `ring_cardinals`'s global cardinals) --
	## `_ctxring_probe.gd` leg F already established this for the desktop
	## flick; here the same slot is reached by a touch SLIDE after the hold,
	## continued tracking rather than a fresh gesture.
	await _touch_hold(ov, centre)
	_touch_move(ov, centre + _dir_vec("W") * 40.0)
	await _frames(2)
	_ok("S slide: hovering W before lift", _ring_state().get("hover", ""), "W")
	_touch_release(ov, centre + _dir_vec("W") * 40.0)
	await _frames(4)
	_ok("S slide-to-select: lifting on W armed Region", app.armed_tool, "region")
	_ok("S slide-to-select: the ring closed (a real pick, not sticky)", _ring_state().get("active", false), false)
	await _card_close()
	app.arm_tool("inspect")
	await _frames(2)

	# -- D: handedness flips which side the card docks on ------------------------
	## `anchor` is the SAME point `context_broker.gd::present()` docks against
	## (`get_global_transform_with_canvas() * screen_pos`), which is NOT the
	## same number as `centre` here whenever the map viewport sits under a
	## dock/rail offset -- comparing `panel_rect()` to `centre` directly
	## measured the wrong reference point the first time this leg ran.
	var anchor: Vector2 = ov.get_global_transform_with_canvas() * centre
	var orig_hand := DccSettings.dominant_hand()
	DccSettings.set_dominant_hand("right")
	await _touch_hold(ov, centre)
	var pr_right_handed: Rect2 = _card().panel_rect()
	print("D right-handed: touch=%s anchor=%s card=%s" % [centre, anchor, pr_right_handed])
	_ok("D right-handed (default): the card docks LEFT of the anchor",
		pr_right_handed.end.x <= anchor.x + 1.0, true)
	_ok("O right-handed: the card does not overlap any ring slot",
		_card_overlaps_ring(pr_right_handed), false)
	## CM-4 residual: one framebuffer PNG per hand, tablet size -- the ring and
	## card as actually drawn, not a re-derived rect.
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png("res://_ctxtablet_ring_right.png")
	_touch_release(ov, centre)
	await _ring_close()
	await _card_close()

	DccSettings.set_dominant_hand("left")
	await _touch_hold(ov, centre)
	var pr_left_handed: Rect2 = _card().panel_rect()
	print("D left-handed: touch=%s anchor=%s card=%s" % [centre, anchor, pr_left_handed])
	_ok("D left-handed: the card docks RIGHT of the anchor",
		pr_left_handed.position.x >= anchor.x - 1.0, true)
	_ok("O left-handed: the card does not overlap any ring slot",
		_card_overlaps_ring(pr_left_handed), false)
	## `context_broker.gd::present()`'s own dock choice is read straight off
	## `DccSettings.dominant_hand()` (its own header comment names the exact
	## line) -- these two D checks, both against the live setting rather than
	## a mocked value, are what "the ring's edge flipping/handedness actually
	## reads the setting" means for the card half. The ring itself has no
	## handedness-dependent geometry of its own (§7.2 names only screen-edge
	## clamping for the ring; handedness picks the CARD's side), so there is
	## nothing further to flip on the ring beyond what `_clamp_centre()`
	## already does and leg E already covers.
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png("res://_ctxtablet_ring_left.png")
	_touch_release(ov, centre)
	await _ring_close()
	await _card_close()
	DccSettings.set_dominant_hand(orig_hand)
	app.arm_tool("inspect")
	await _frames(2)

	# -- O (sub-ring follow-up, both hands): a sub-ring opened WHILE the card
	# is already showing must not overlap it either -- the defect the
	# "context card can overlap an open sub-ring" backlog row named. Leg D
	# above only ever checked the TOP-level ring's own footprint; a sub-ring
	# re-centres on its own parent slot (`radial_ring.gd::_open_sub()`),
	# reaching past that. `world`'s own NW slot (Uplift) has `children` on
	# every domain the D-table above already established.
	for hand in ["right", "left"]:
		DccSettings.set_dominant_hand(hand)
		await _touch_hold(ov, centre)
		_ok("O %s-handed: the ring opened" % hand, _ring_state().get("visible", false), true)
		_ok("O %s-handed: the card opened alongside it" % hand, _card_open(), true)
		var r_o: float = _ring_state().get("radius", 0.0)
		var nw_pos_o := centre + _dir_vec("NW") * r_o
		_touch_move(ov, nw_pos_o)
		await _frames(2)
		_touch_release(ov, nw_pos_o)   ## Uplift has children -> release() opens its sub-ring.
		await _frames(4)
		_ok("O %s-handed: release opened Uplift's sub-ring" % hand,
			_ring_state().get("sub_open", false), true)
		_ok("O %s-handed: nothing armed yet (a children slot only opens a sub-ring)" % hand,
			app.armed_tool, "inspect")
		var card_rect_o: Rect2 = _card().panel_rect() if _card() != null else Rect2()
		print("O %s-handed: card=%s" % [hand, card_rect_o])
		_ok("O %s-handed: the card still does not overlap the MAIN ring's slots" % hand,
			_card_overlaps_ring(card_rect_o), false)
		_ok("O %s-handed: the card does not overlap the SUB-ring's own slots" % hand,
			_card_overlaps_sub(card_rect_o), false)
		await _ring_close()
		await _card_close()
		app.arm_tool("inspect")
		await _frames(2)
	DccSettings.set_dominant_hand(orig_hand)

	# -- E: the ring's AND the card's own rects stay on screen at the edges -----
	## Aimed at the PLATE's own edges (`_interior_rect()`), not the raw
	## viewport's: at this grid's aspect ratio the displayed map is
	## letterboxed, so a touch at the bare viewport edge lands on paper with
	## no world coordinate under it (`request_at()` returns `{}`) and the card
	## never opens there at all -- measured the first time this leg ran, where
	## every edge printed the SAME stale card rect left over from leg D. The
	## ring still opens regardless (it needs no grid pick), which is why
	## `_ctxring_probe.gd`'s own desktop leg E can use the raw viewport edges.
	## Live, touch-scaled geometry -- NOT `RING_RADIUS`/`SLOT_SIZE` above.
	## `radial_ring.gd::_clamp_centre()` pads by its own live `_r_ring`/
	## `_r_slot`, which on a tablet are the touch-scaled figures
	## `_apply_touch_scale()` computes, larger than the bare desktop consts.
	## Padding this check with the smaller desktop figures would under-size
	## `ring_rect` and let a real off-screen ring pass -- exactly the "assert a
	## constant against itself" failure this residual exists to close.
	var live: Dictionary = _ring_state()
	var pad: float = float(live.get("radius", RING_RADIUS)) \
		+ float(live.get("slot", SLOT_SIZE)) * 0.5 + 40.0
	var disp: Rect2 = ov.displayed_rect()
	var inter: Rect2 = ov._interior_rect(disp)
	var inset := 24.0
	var edge_pts := [
		Vector2(inter.position.x + inset, inter.get_center().y),
		Vector2(inter.end.x - inset, inter.get_center().y),
		Vector2(inter.get_center().x, inter.position.y + inset),
		Vector2(inter.get_center().x, inter.end.y - inset)]
	var names := ["left", "right", "top", "bottom"]
	for i in 4:
		await _touch_hold(ov, edge_pts[i])
		var c: Vector2 = _ring_state().get("centre", Vector2.ZERO)
		var ring_rect := Rect2(c - Vector2.ONE * pad, Vector2.ONE * pad * 2.0)
		_ok("E tablet-%s: the touch landed on a real grid cell (card can open)" % names[i],
			_card_open(), true)
		var card_rect: Rect2 = _card().panel_rect() if _card() != null else Rect2()
		print("E tablet-%s aim=%s ring_centre=%s ring_rect=%s card_rect=%s viewport=%s" %
			[names[i], edge_pts[i], c, ring_rect, card_rect, vis])
		_ok("E tablet-%s: the ring's own rect is on screen" % names[i], vis.grow(1.0).encloses(ring_rect), true)
		_ok("E tablet-%s: the card's own rect is on screen" % names[i], vis.grow(1.0).encloses(card_rect), true)
		_touch_release(ov, edge_pts[i])
		await _ring_close()
		await _card_close()

	# -- G: Ruling BC geometry scales with the touch ring ------------------------
	app.select_domain("world")
	await _frames(4)
	app.arm_tool("inspect")
	await _frames(2)
	await _touch_hold(ov, centre)
	var st_g := _ring_state()
	var disc_r: float = st_g.get("disc_radius", 0.0)
	var live_r: float = st_g.get("radius", 0.0)
	var live_slot: float = st_g.get("slot", 0.0)
	var centre_sz: float = st_g.get("centre_size", 0.0)
	print("G touch disc_radius=%.2f ring_radius=%.2f slot=%.2f centre_size=%.2f" %
		[disc_r, live_r, live_slot, centre_sz])
	_ok("G touch disc reaches past the (already 96 dp+) touch ring",
		disc_r > live_r + live_slot * 0.5, true)
	_ok("G touch centre button grew off the desktop 46/60 figure (not left at desktop size)",
		centre_sz > SLOT_SIZE * (46.0 / 60.0), true)
	_touch_release(ov, centre)
	await _ring_close()
	await _card_close()
	app.arm_tool("inspect")
	await _frames(2)

	## One screenshot with a sub-ring ACTUALLY open on the touch-scaled
	## geometry (Ruling BC evidence -- desktop's own `_ctxring_probe.gd`
	## already saves a plain-ring pair; this is the tablet's sub-ring pair).
	##
	## Coordinator correction 2026-09-29: the first pass here moved onto NW
	## and screenshotted WITHOUT releasing, so the ring only ever showed the
	## hover WEDGE toward Uplift, never an opened sub-ring -- `release()`
	## (`radial_ring.gd`) is what actually opens a `children` slot's sub-ring
	## (mirrors `_held_drag()`'s desktop path, which does release). The aim
	## point is also now read fresh from `debug_state()` on each pass rather
	## than a distance computed once before the loop, so a theme change that
	## ever altered geometry could not aim short of the live slot.
	for dark_g in [true, false]:
		var was_g := DccTheme.is_dark()
		if was_g != dark_g:
			DccTheme.apply_theme(dark_g)
			app.rebuild_theme(was_g)
			await _frames(8)
		await _touch_hold(ov, centre)
		var r_shot: float = _ring_state().get("radius", 0.0)
		var nw_pos: Vector2 = centre + _dir_vec("NW") * r_shot
		_touch_move(ov, nw_pos)
		await _frames(2)
		_touch_release(ov, nw_pos)   ## Uplift has children -> release() opens its sub-ring.
		await _frames(4)
		await RenderingServer.frame_post_draw
		await RenderingServer.frame_post_draw
		print("G sub-ring screenshot (%s): sub_open=%s" %
			[("dark" if dark_g else "light"), _ring_state().get("sub_open", false)])
		_vp.get_texture().get_image().save_png(
			"res://_ctxtablet_ring_sub_%s.png" % ("dark" if dark_g else "light"))
		await _ring_close()
		await _card_close()
		app.arm_tool("inspect")
		await _frames(2)

	# -- L: every label sits inside its own circle, on the TOUCH geometry -------
	## Coordinator review, 2026-09-27 (overlap follow-up): the card is no
	## longer closed before sampling. It used to be -- the tablet's touch-hold
	## opens the ring AND the card together (leg H), and the card's own dock
	## offset (`ring_clear`) was sized to the TOP-level ring's own footprint
	## only, so it could sit over a slot's label once a SUB-ring opened in a
	## different direction (measured once, sub #4 "Cliff / Escarpment": the
	## "failing" pixel was the card's own corner). `context_broker.gd::
	## _sync_card_ring_clear()` now re-docks the card the moment a sub-ring
	## opens beside it (`_ctxring_probe.gd`'s own leg O, and this file's leg O
	## above, both assert the card never overlaps a live sub-slot rect), so
	## the card can stay open here and still not touch a label.
	await _touch_hold(ov, centre)
	var st_l := _ring_state()
	var l_centre: Vector2 = st_l.get("centre", Vector2.ZERO)
	var l_radius: float = st_l.get("radius", 0.0)
	var l_slot: float = st_l.get("slot", 0.0)
	_touch_release(ov, centre)   ## dead-zone release: sticky, hover stays "" -- no wedge.
	await _frames(4)
	await _frames(2)
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	var img_l: Image = _vp.get_texture().get_image()
	for dir in DIR_DEG:
		var pos: Vector2 = l_centre + _dir_vec(dir) * l_radius
		var res := _label_inside_circle(img_l, pos, l_slot * 0.5)
		if not res.get("ok", true):
			print("L touch main %s FAIL at=%s px=%s" % [dir, res["at"], res["px"]])
		_ok("L touch main slot %s: its label stays inside its own circle" % dir, res.get("ok", false), true)
	await _ring_close()
	await _card_close()
	app.arm_tool("inspect")
	await _frames(2)

	await _touch_hold(ov, centre)
	var r_l2: float = _ring_state().get("radius", 0.0)
	var nw_pos_l: Vector2 = centre + _dir_vec("NW") * r_l2
	_touch_move(ov, nw_pos_l)
	await _frames(2)
	_touch_release(ov, nw_pos_l)
	await _frames(4)
	await _frames(2)
	var st_l2 := _ring_state()
	if bool(st_l2.get("sub_open", false)):
		var sub_c_l: Vector2 = st_l2.get("sub_centre", Vector2.ZERO)
		var sub_r_l: float = st_l2.get("sub_radius", 0.0)
		var sub_slot_l: float = st_l2.get("sub_slot_size", 0.0)
		var sub_n_l: int = int(st_l2.get("sub_count", 0))
		await RenderingServer.frame_post_draw
		await RenderingServer.frame_post_draw
		var img_l2: Image = _vp.get_texture().get_image()
		for i in sub_n_l:
			var ang := deg_to_rad(-90.0 + 360.0 * float(i) / float(maxi(1, sub_n_l)))
			var pos2: Vector2 = sub_c_l + Vector2(cos(ang), sin(ang)) * sub_r_l
			var res2 := _label_inside_circle(img_l2, pos2, sub_slot_l * 0.5)
			if not res2.get("ok", true):
				print("L touch sub #%d FAIL at=%s px=%s" % [i, res2["at"], res2["px"]])
			_ok("L touch sub slot #%d: its label stays inside its own circle" % i, res2.get("ok", false), true)
	else:
		_ok("L touch: Uplift opened its own sub-ring (precondition for the sub-slot label check)",
			st_l2.get("sub_open", false), true)
	await _ring_close()
	await _card_close()
	app.arm_tool("inspect")
	await _frames(2)


func _ready() -> void:
	var vp_size := Vector2i(1600, 900)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a: String = args[i]
		if a == "--vp" and i + 1 < args.size():
			var wh := args[i + 1].split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
			continue
		if a == "--force-touch":
			i += 1
			continue
		print("### CTXTABLET ABORT: unknown argument '%s' ###" % a)
		get_tree().quit(2)
		return
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)

	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.0).timeout

	var bridge = app.bridge
	bridge.generate({
		"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)

	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("FORM booted as %s at %s" % [form, _vp.size])
	if form != "tablet":
		print("### CTXTABLET ABORT: CM-4 needs the tablet form -- booted as %s. Use --force-touch and a tablet-sized --vp (e.g. 2560x1600) ###" % form)
		get_tree().quit(2)
		return
	await _run()
	print("### CTXTABLET %s  %d/%d checks passed ###" % [
		"GREEN" if _fails == 0 else "RED", _checks - _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
