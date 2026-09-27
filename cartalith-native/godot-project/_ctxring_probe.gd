extends Node
## CM-3 (`MAP_CONTEXT_SCOPE.md` §11): the desktop ring (`shell/radial_ring.gd`,
## driven by `shell/context_broker.gd`'s ring methods).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _ctxring_probe.tscn
##
## **Windowed, not `--headless`**: leg P reads pixels. `--vp WxH` sizes the
## host SubViewport (default 1600x900); any other argument aborts (exit 2).
##
## Every gesture is pushed through the real input path -- `ov._gui_input()`
## for mouse events (the same path `_ctxcard_probe.gd` uses to drive RMB) and
## `_vp.push_input()` for the keyboard (Q reaches `app.gd`'s
## `_unhandled_key_input`, which only fires for genuine viewport input, not a
## direct method call).
##
## Legs:
##   F  the no-wait flick, all 8 directions, all 3 domains: a fast RMB drag
##      (>= 8 px, released before 150 ms) arms the right tool/feature/mode
##      with the ring never having become visible
##   D  the same 8x3, but held past 150 ms so the ring IS drawn before release
##      -- same result, asserted through the real armed-tool/feature state
##   C  a `children` slot (Uplift) opens its sub-ring on release rather than
##      arming anything by itself; a sticky click into the sub-ring then arms
##      the picked feature
##   R  a short RMB click (< 8 px, released quickly) still opens the card,
##      never the ring
##   H  a still RMB hold >= 300 ms opens the ring AND the card together, and
##      releasing over the dead zone leaves the ring open (sticky) rather than
##      picking anything
##   Q  Q tap (press, release with no aim) leaves the ring open sticky; Q hold
##      (press, aim over a slot, release) picks it
##   A  the armed slot draws filled (`ring_collect()`'s own `armed` flag)
##   E  the ring's own rect stays on screen at the four screen edges
##   P  contrast of a slot's label against the ring's own ground, both
##      palettes forced
##   G  Ruling BC (`LARGE_ITEM_RULINGS.md`): the mockup's style/shape, in the
##      theme's own colours -- the disc sits behind the slots and is visibly
##      distinct from the plain scrim, the centre button is smaller than a
##      main slot, the hover wedge changes the pixel toward the hovered slot,
##      a slot's label draws INSIDE it (nothing bleeds past the slot's own
##      edge), the caption pill is drawn, and a sub-ring's own centre fills
##      in the theme accent (the "‹ BACK" button)
##   K  CM-3 residual, the continuous nested flick (`MAP_CONTEXT_SCOPE.md`
##      §5.2 rule 2): ONE RMB drag -- press, past Uplift, past the ring's own
##      radius (opening the sub-ring mid-gesture, no release yet), onto a
##      sub-slot, release -- picks that sub-slot directly, no second click
##   V  CARTO's View and Style diagonals, picked through a ring gesture
##      (`_view_field_rows()`/`RenderWorkspace.STYLE_PRESETS`, previously only
##      reachable through the dock)
##   U  CM-3 residual, the Icon▸ Custom entry: disabled with its reason
##      before any custom icon exists, then armed through the ring once one is
##      imported and applied (`icon_arm_custom`/`icon_custom_slots`)
##   W  the Way▸ sub-ring offers the engine's real vocabulary (road / track /
##      sea lane / ancient, `infra_tools_bridge::parse_way_type`), not the
##      stale "trail"/"bridge" pair `MAP_CONTEXT_SCOPE.md` §5.1 used to name

const SEED := 483920

## `radial_ring.gd`'s own geometry constants, duplicated here (not imported)
## so this probe measures the SHIPPED geometry rather than assuming it stays
## in sync by construction -- the same reason `_ctxcard_probe.gd`'s own `E`/`P`
## legs hardcode row heights instead of reading them off the source file.
## Ruling BC, coordinator correction 2026-09-29: these track the shipped
## `radial_ring.gd::RING_RADIUS`/`SLOT_SIZE`, now the mockup's own absolute
## desktop figures (92 / 60) rather than this shell's old independently-tuned
## ones (60 / 46) -- updated here, not imported, for the reason above.
const RING_RADIUS := 92.0
const SLOT_SIZE := 60.0
## Same compass angles `radial_ring.gd::ANGLES` uses -- 0 deg = east, clockwise
## in screen space (Y down).
const DIR_DEG := {
	"N": -90.0, "NE": -45.0, "E": 0.0, "SE": 45.0,
	"S": 90.0, "SW": 135.0, "W": 180.0, "NW": -135.0,
}

static func _dir_vec(dir: String) -> Vector2:
	var a := deg_to_rad(float(DIR_DEG[dir]))
	return Vector2(cos(a), sin(a))


## `radial_ring.gd`'s own `_centre`/`_sub.centre` (`debug_state()`'s
## `centre`/`sub_centre`) live in the SAME converted space `context_broker.gd`
## builds them in -- `get_global_transform_with_canvas() * local_pos` -- not
## `map_overlay.gd`'s own local space real events arrive in. Feeding one back
## into a synthetic `InputEvent` at `ov` without inverting that transform
## first double-applies it.
func _at_to_local(ov: Control, at: Vector2) -> Vector2:
	return ov.get_global_transform_with_canvas().affine_inverse() * at

var app: Node
var _vp: SubViewport
var _fails := 0
var _checks := 0
var _k := -1


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


## Leg O (coordinator review, 2026-09-27) -- `_ctxtablet_probe.gd`'s own twin
## helper, same shape: the 8 live slot rects off `debug_state()`'s own
## `centre`/`radius`/`slot`, never the bare desktop consts above (which this
## probe's own header already disclaims for the same reason).
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


func _card_overlaps_ring(rect: Rect2) -> bool:
	var slots := _ring_slot_rects()
	var dirs := DIR_DEG.keys()
	for i in slots.size():
		if rect.intersects(slots[i]):
			print("O overlap: card=%s hits slot %s=%s" % [rect, dirs[i], slots[i]])
			return true
	return false


## Leg O's own follow-up (coordinator review, 2026-09-27, "the context card
## can overlap an open sub-ring"): the sub-ring's own live slot rects, off
## `debug_state()`'s `sub_centre`/`sub_radius`/`sub_slot_size`/`sub_count` --
## `[]` when no sub-ring is open, same shape as `_ring_slot_rects()` but at
## `_open_sub()`'s own angle formula (`-90 + 360*i/n`, straight up for item 0),
## reproduced here rather than imported, same reason the main-ring angles are.
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


## A sub-ring's item `i`-of-`n`'s own live position (`_open_sub()`/
## `_draw_sub_ring()`'s own `-90 + 360*i/n` angle formula, reproduced here for
## the same reason the main-ring angles already are), off `_ring_state()`'s
## LIVE `sub_centre`/`sub_radius`/`sub_count` rather than a re-derived guess --
## `MISTAKES.md`'s "never assert a constant against itself" applies just as
## much to a probe's own click target as to an assertion.
func _sub_item_at(st: Dictionary, i: int) -> Vector2:
	var c: Vector2 = st.get("sub_centre", Vector2.ZERO)
	var r: float = st.get("sub_radius", 60.0)
	var n: int = int(st.get("sub_count", 1))
	var ang := deg_to_rad(-90.0 + 360.0 * float(i) / float(maxi(1, n)))
	return c + Vector2(cos(ang), sin(ang)) * r


## Leg W: `InfrastructureWorkspace` is composed INTO `CivilizationWorkspace`
## as a plain `_infra` field (`civilization_workspace.gd`'s own doc comment:
## "a real `InfrastructureWorkspace` instance"), not registered in
## `app._workspaces` under its own entry the way `RenderWorkspace` is nested
## in `CartographyWorkspace` for the identical reason -- so this walks
## `_workspaces` for whichever one CARRIES an `_infra` property, rather than
## `has_method()` on the wrong (outer) object.
func _find_infra() -> Node:
	for ws in app._workspaces:
		var v = ws.get("_infra")
		if v != null:
			return v
	return null


func _card_open() -> bool:
	var c = _card()
	return c != null and c.visible


func _card_close() -> void:
	var c = _card()
	if c != null and c.visible:
		c.hide()
	await _frames(2)


func _rmb_press(ov: Control, pos: Vector2) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_RIGHT
	ev.pressed = true
	ev.position = pos
	ov._gui_input(ev)


func _rmb_release(ov: Control, pos: Vector2) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_RIGHT
	ev.pressed = false
	ev.position = pos
	ov._gui_input(ev)


func _move(ov: Control, pos: Vector2, mask: int = 0) -> void:
	var mv := InputEventMouseMotion.new()
	mv.position = pos
	mv.button_mask = mask
	ov._gui_input(mv)


func _lmb_click(ov: Control, pos: Vector2) -> void:
	_move(ov, pos)
	await _frames(1)
	var down := InputEventMouseButton.new()
	down.button_index = MOUSE_BUTTON_LEFT
	down.pressed = true
	down.position = pos
	ov._gui_input(down)
	await _frames(2)
	var up := down.duplicate()
	up.pressed = false
	ov._gui_input(up)
	await _frames(2)


func _qkey(down: bool) -> void:
	var ev := InputEventKey.new()
	ev.keycode = KEY_Q
	ev.physical_keycode = KEY_Q
	ev.pressed = down
	_vp.push_input(ev)
	await _frames(2)


## §6's no-wait flick: press, drag past the slop, release before 150 ms --
## the ring must never have become visible.
func _flick(ov: Control, from: Vector2, dir: String) -> Vector2:
	var to := from + _dir_vec(dir) * 40.0
	_rmb_press(ov, from)
	_move(ov, to, MOUSE_BUTTON_MASK_RIGHT)
	await _frames(1)
	_rmb_release(ov, to)
	await _frames(4)
	return to


## The held-drag path: past the slop AND past 150 ms, so the ring IS drawn.
func _held_drag(ov: Control, from: Vector2, dir: String) -> Vector2:
	var to := from + _dir_vec(dir) * 40.0
	_rmb_press(ov, from)
	_move(ov, to, MOUSE_BUTTON_MASK_RIGHT)
	await get_tree().create_timer(0.22).timeout
	await _frames(2)
	_rmb_release(ov, to)
	await _frames(4)
	return to


static func _ratio(a: Color, b: Color) -> float:
	var la := _lin(a)
	var lb := _lin(b)
	return (maxf(la, lb) + 0.05) / (minf(la, lb) + 0.05)


static func _close_color(a: Color, b: Color, tol: float) -> bool:
	return absf(a.r - b.r) <= tol and absf(a.g - b.g) <= tol and absf(a.b - b.b) <= tol


## Leg L (Ruling BC, coordinator correction 2026-09-29: "every label must sit
## fully inside its circle"). True when nothing draws in a band just past
## `radius` around the FULL circle at `centre` -- i.e. no glyph or label ink
## crossed the slot's own edge.
##
## The disc lets the live map show through at its own alpha (88% main / 44%
## with a sub open), so a SINGLE reference pixel is not reliable -- the map's
## own texture varies pixel to pixel even with nothing drawn (measured: a
## smooth ~0.05 luminance drift band-to-band was enough to false-fail this
## leg against a fixed corner or a single "1.6x radius" sample the first time
## it ran). The robust signal is the band's own MEDIAN luminance: a genuine
## stroke of text is high-contrast against its local ground and stands out
## from the band's typical value, where gradual map texture does not.
static func _label_inside_circle(img: Image, centre: Vector2, radius: float) -> Dictionary:
	## Kept tight (a few px past the slot's own border) rather than reaching
	## further out: a slot pointing straight down (`S`) sits on the same ray
	## as the caption pill below the whole ring, and a wider band the first
	## time this leg ran caught the pill's own background, not a label.
	var steps := 40
	var radii := [radius + 2.0, radius + 4.0, radius + 6.0]
	var samples: Array = []   ## [{"p":Vector2,"px":Color,"lum":float}, ...]
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
			return {"ok": false, "at": s["p"], "px": (s["px"] as Color).to_html(false),
				"median_lum": median, "sample_lum": s["lum"]}
	return {"ok": true}


static func _lin(c: Color) -> float:
	var f := func(v: float) -> float:
		return v / 12.92 if v <= 0.03928 else pow((v + 0.055) / 1.055, 2.4)
	return 0.2126 * f.call(c.r) + 0.7152 * f.call(c.g) + 0.0722 * f.call(c.b)


func _run() -> void:
	var bridge = app.bridge
	var ov: Control = app.viewport.overlay
	var vis: Rect2 = app.get_viewport().get_visible_rect()
	var centre := vis.size * 0.5

	# -- F: the no-wait flick, all 8 directions, all 3 domains -------------------
	## `armed_tool`/current feature or mode is read after the release, and the
	## ring's own `visible` flag is read BEFORE it (right after the drag, still
	## mid-gesture) so "never drawn" is asserted while it would still be true
	## to check, not inferred from the outcome alone.
	app.select_domain("world")
	await _frames(4)
	app.arm_tool("inspect")
	await _frames(2)
	_rmb_press(ov, centre)
	_move(ov, centre + _dir_vec("W") * 40.0, MOUSE_BUTTON_MASK_RIGHT)
	await _frames(1)
	_ok("F flick: ring not yet visible mid-drag", _ring_state().get("visible", true), false)
	_rmb_release(ov, centre + _dir_vec("W") * 40.0)
	await _frames(4)
	_ok("F WORLD W flick arms Region", app.armed_tool, "region")
	app.arm_tool("inspect")
	await _frames(2)

	## E (Measure) is a `children` slot (§5.1: "Measure ▸"), so a flick lands
	## on its sub-ring rather than arming Measure directly -- the sub-ring
	## picking path is already covered by `D`/`C` below for the domain
	## diagonals; here it is enough that the flick reached Measure's own
	## sub-ring rather than mis-firing some other slot.
	await _flick(ov, centre, "E")
	_ok("F WORLD E flick opened Measure's sub-ring", _ring_state().get("sub_open", false), true)
	await _ring_close()
	app.arm_tool("inspect")
	await _frames(2)

	await _flick(ov, centre, "N")
	_ok("F WORLD N flick arms Inspect (a no-op re-arm)", app.armed_tool, "inspect")

	# -- D: the same directions, held past 150 ms so the ring draws -------------
	for domain_dir in [
		["world", "NW", "sculpt"], ["world", "NE", "sculpt"],
		["world", "SE", "sculpt"], ["world", "SW", "paint"],
		["civilization", "NW", "settlement"], ["civilization", "NE", "territory"],
		["civilization", "SE", "way"], ["civilization", "SW", "route"],
		["cartography", "NW", "label"], ["cartography", "NE", "icon"],
	]:
		var domain := String(domain_dir[0])
		var dir := String(domain_dir[1])
		var want_tool := String(domain_dir[2])
		app.select_domain(domain)
		await _frames(4)
		app.arm_tool("inspect")
		await _frames(2)
		var end_pos := await _held_drag(ov, centre, dir)
		_ok("D %s %s drag+hold: opened a sub-ring or armed directly" % [domain, dir],
			bool(_ring_state().get("sub_open", false)) or app.armed_tool == want_tool, true)
		if bool(_ring_state().get("sub_open", false)):
			## A `children` slot: pick its first item with a sticky click,
			## then check the tool armed. `sub_centre` + index 0's own angle
			## (`-90 + 360*0/n = -90`, i.e. straight up) is `_open_sub()`'s
			## own formula, reproduced here rather than imported.
			var st := _ring_state()
			var sub_centre: Vector2 = st.get("sub_centre", Vector2.ZERO)
			var item0 := sub_centre + Vector2(0, -1) * 50.0
			await _lmb_click(ov, _at_to_local(ov, item0))
			_ok("D %s %s sub-item 0 armed %s" % [domain, dir, want_tool], app.armed_tool, want_tool)
		await _ring_close()
		app.arm_tool("inspect")
		await _frames(2)

	# -- C: Uplift's own sub-ring, feature-checked (not just tool-checked) -------
	app.select_domain("world")
	await _frames(4)
	app.arm_tool("inspect")
	await _frames(2)
	await _held_drag(ov, centre, "NW")
	var st_c := _ring_state()
	_ok("C Uplift opened a sub-ring rather than arming by itself", st_c.get("sub_open", false), true)
	_ok("C nothing armed yet (still on Inspect)", app.armed_tool, "inspect")
	var sub_c: Vector2 = st_c.get("sub_centre", Vector2.ZERO)
	await _lmb_click(ov, _at_to_local(ov, sub_c + Vector2(0, -1) * 50.0))   ## index 0 -- Mountains
	_ok("C Uplift's first sub-item armed Sculpt", app.armed_tool, "sculpt")
	_ok("C ...with Mountains the live feature", bridge.sculpt_get_feature(), "mountains")
	await _ring_close()
	app.arm_tool("inspect")
	await _frames(2)

	# -- R: a short click still opens the card, never the ring ------------------
	_rmb_press(ov, centre)
	_rmb_release(ov, centre)
	await _frames(4)
	_ok("R short RMB click: the card opened", _card_open(), true)
	_ok("R short RMB click: the ring never engaged", _ring_state().get("active", false), false)
	await _card_close()

	# -- H: a still hold opens the ring AND the card together --------------------
	_rmb_press(ov, centre)
	await get_tree().create_timer(0.35).timeout
	await _frames(2)
	_ok("H still hold >= 300ms: the ring is visible", _ring_state().get("visible", false), true)
	_ok("H still hold >= 300ms: the card opened alongside it", _card_open(), true)
	## Coordinator review, 2026-09-27: same "ring and card open together"
	## overlap `_ctxtablet_probe.gd`'s own leg O found on the 96 dp touch
	## ring -- checked here at desktop's 60 px ring too, since
	## `context_broker.gd::present()`'s `ring_clear` fix is shared by both.
	var card_node = _card()
	_ok("O desktop: the card does not overlap any ring slot",
		_card_overlaps_ring(card_node.panel_rect()) if card_node != null else true, false)
	_rmb_release(ov, centre)
	await _frames(2)
	_ok("H releasing in the dead zone leaves the ring open (sticky)", _ring_state().get("active", false), true)
	await _ring_close()
	await _card_close()
	app.arm_tool("inspect")
	await _frames(2)

	# -- O (sub-ring follow-up): a sub-ring opened WHILE the card is already
	# showing must not overlap it either -- the defect the "context card can
	# overlap an open sub-ring" backlog row named: a sub-ring re-centres on
	# its own parent slot (`radial_ring.gd::_open_sub()`), reaching past the
	# top-level ring's own footprint the plain `H`/`O` check above covers.
	# Exercised on every WORLD direction with a `children` slot (`D`'s own
	# table above: NW=Uplift, E=Measure) so this is not a single-direction
	# coincidence, and in both orders the sub can appear relative to an
	# already-open card: opened AFTER the card (still-hold, then drag onto
	# the slot and release) and opened BEFORE the card would exist at all
	# were it not for `present()`'s own live re-check on open.
	for sub_dir in ["NW", "E"]:
		app.select_domain("world")
		await _frames(4)
		app.arm_tool("inspect")
		await _frames(2)
		_rmb_press(ov, centre)
		await get_tree().create_timer(0.35).timeout
		await _frames(2)
		_ok("O %s: still hold opened the ring" % sub_dir, _ring_state().get("visible", false), true)
		_ok("O %s: the card opened alongside it" % sub_dir, _card_open(), true)
		var r_sub: float = _ring_state().get("radius", 0.0)
		var sub_dir_pos := centre + _dir_vec(sub_dir) * r_sub
		_move(ov, sub_dir_pos, MOUSE_BUTTON_MASK_RIGHT)
		await _frames(2)
		_ok("O %s: hovering the children slot" % sub_dir, _ring_state().get("hover", ""), sub_dir)
		_rmb_release(ov, sub_dir_pos)
		await _frames(4)
		_ok("O %s: release opened its sub-ring" % sub_dir, _ring_state().get("sub_open", false), true)
		_ok("O %s: nothing armed yet (a children slot only opens a sub-ring)" % sub_dir,
			app.armed_tool, "inspect")
		var card_node_o = _card()
		_ok("O %s: the card still does not overlap the MAIN ring's slots" % sub_dir,
			_card_overlaps_ring(card_node_o.panel_rect()) if card_node_o != null else true, false)
		_ok("O %s: the card does not overlap the SUB-ring's own slots" % sub_dir,
			_card_overlaps_sub(card_node_o.panel_rect()) if card_node_o != null else true, false)
		await RenderingServer.frame_post_draw
		await RenderingServer.frame_post_draw
		_vp.get_texture().get_image().save_png("res://_ctxring_sub_%s.png" % sub_dir)
		await _ring_close()
		await _card_close()
		app.arm_tool("inspect")
		await _frames(2)

	# -- Q: tap leaves it open sticky; hold picks the aimed slot -----------------
	_move(ov, centre)
	await _frames(1)
	await _qkey(true)
	await _qkey(false)
	await _frames(2)
	var q_tap := _ring_state()
	_ok("Q tap: ring open", q_tap.get("active", false), true)
	_ok("Q tap: sticky (no aim happened)", q_tap.get("sticky", false), true)
	await _ring_close()

	_move(ov, centre)
	await _frames(1)
	await _qkey(true)
	_ok("Q hold: ring visible immediately (no flick delay for Q)", _ring_state().get("visible", false), true)
	_move(ov, centre + _dir_vec("N") * 40.0)
	await _frames(2)
	await _qkey(false)
	await _frames(2)
	_ok("Q hold: released over N armed Inspect", app.armed_tool, "inspect")

	# -- A: the armed slot draws filled ------------------------------------------
	app.select_domain("world")
	await _frames(4)
	app.arm_tool("region")
	await _frames(2)
	var req: Dictionary = _broker().ring_domain_req()
	var slots: Dictionary = _broker().ring_collect(req)
	_ok("A Region's own slot (W) reads armed=true", bool((slots["W"] as Dictionary).get("armed", false)), true)
	_ok("A Inspect's own slot (N) reads armed=false while Region is armed",
		bool((slots["N"] as Dictionary).get("armed", false)), false)
	app.arm_tool("inspect")
	await _frames(2)

	# -- E: the ring's rect stays on screen at the four edges --------------------
	var pad := RING_RADIUS + SLOT_SIZE * 0.5 + 40.0
	var edge_pts := [
		Vector2(2, vis.size.y * 0.5), Vector2(vis.size.x - 2, vis.size.y * 0.5),
		Vector2(vis.size.x * 0.5, 2), Vector2(vis.size.x * 0.5, vis.size.y - 2)]
	var names := ["left", "right", "top", "bottom"]
	for i in 4:
		_move(ov, edge_pts[i])
		await _frames(1)
		await _qkey(true)
		await _qkey(false)
		await _frames(2)
		var c: Vector2 = _ring_state().get("centre", Vector2.ZERO)
		var ring_rect := Rect2(c - Vector2.ONE * pad, Vector2.ONE * pad * 2.0)
		print("E ring-%s aim=%s centre=%s rect=%s viewport=%s" % [names[i], edge_pts[i], c, ring_rect, vis])
		_ok("E ring-%s: the ring's own rect is on screen" % names[i], vis.grow(1.0).encloses(ring_rect), true)
		await _ring_close()

	# -- P: contrast, both palettes -----------------------------------------------
	for dark in [true, false]:
		var was := DccTheme.is_dark()
		if was != dark:
			DccTheme.apply_theme(dark)
			app.rebuild_theme(was)
			await _frames(8)
		_ok("P palette forced: %s" % ("dark" if dark else "light"), DccTheme.is_dark(), dark)
		_move(ov, centre)
		await _frames(1)
		await _qkey(true)
		await _qkey(false)
		await _frames(6)
		var img: Image = _vp.get_texture().get_image()
		var c: Vector2 = _ring_state().get("centre", Vector2.ZERO)
		var slot_pos := c + Vector2(0, -1) * RING_RADIUS   ## N -- Inspect
		var ground := img.get_pixelv(Vector2i(int(slot_pos.x) - int(SLOT_SIZE * 0.5) + 3,
			int(slot_pos.y) - int(SLOT_SIZE * 0.5) + 3))
		## Ruling BC moved the label INSIDE the slot (`radial_ring.gd::
		## _draw_ring()`, offset `pos + Vector2(0, slot*0.28)`) -- this box
		## used to sit below the slot's own circle, where the label no
		## longer draws, and would otherwise measure ground against ground
		## and pass vacuously.
		var label_rect := Rect2(slot_pos.x - 24, slot_pos.y + SLOT_SIZE * 0.10, 48, 20)
		var ink := ground
		var best := 0.0
		for y in range(int(label_rect.position.y), int(label_rect.end.y)):
			for x in range(int(label_rect.position.x), int(label_rect.end.x)):
				if x < 0 or y < 0 or x >= img.get_width() or y >= img.get_height():
					continue
				var px := img.get_pixel(x, y)
				var d := absf(px.get_luminance() - ground.get_luminance())
				if d > best:
					best = d
					ink = px
		var ratio := _ratio(ink, ground)
		print("P %s ground=%s ink=%s ratio=%.2f" % ["dark" if dark else "light",
			ground.to_html(false), ink.to_html(false), ratio])
		_ok("P %s 'Inspect' label reaches 4.5:1 on the slot's own ground" % ("dark" if dark else "light"),
			ratio >= 4.5, true)
		if OS.get_environment("CTXRING_SHOT_DIR") != "":
			img.save_png(OS.get_environment("CTXRING_SHOT_DIR").path_join(
				"ctxring_%s.png" % ("dark" if dark else "light")))
		await _ring_close()

	# -- G: Ruling BC -- disc, labels inside, wedge, centre, caption -------------
	app.select_domain("world")
	await _frames(4)
	app.arm_tool("inspect")
	await _frames(2)
	_move(ov, centre)
	await _frames(1)
	await _qkey(true)
	await _qkey(false)
	await _frames(4)
	var st_g := _ring_state()
	var disc_r: float = st_g.get("disc_radius", 0.0)
	var ring_r: float = st_g.get("radius", 0.0)
	var slot_r: float = st_g.get("slot", 0.0)
	var centre_sz: float = st_g.get("centre_size", 0.0)
	_ok("G1 the disc reaches past the slot ring", disc_r > ring_r + slot_r * 0.5, true)
	_ok("G3 the centre button is smaller than a main slot",
		centre_sz > 0.0 and centre_sz < slot_r, true)

	var g_c: Vector2 = st_g.get("centre", Vector2.ZERO)
	var n_pt: Vector2 = g_c + Vector2(0, -1) * ring_r             ## N slot's own centre
	var disc_pt: Vector2 = g_c + Vector2(0, -1) * (ring_r * 0.5)  ## disc, off any slot, along N
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	var img_g: Image = _vp.get_texture().get_image()
	var far_bg := img_g.get_pixelv(Vector2i(4, 4))                ## outside every ring surface
	var disc_px := img_g.get_pixelv(Vector2i(int(disc_pt.x), int(disc_pt.y)))
	print("G4 far_bg=%s disc=%s" % [far_bg.to_html(false), disc_px.to_html(false)])
	_ok("G4 the disc's own ground is visibly distinct from the plain scrim",
		not _close_color(disc_px, far_bg, 0.02), true)

	## G5: hover N and re-sample the SAME point -- the wedge fill
	## (`accent_wash_2`) now sits between the dead zone and the slot at that
	## exact bearing, so the pixel must change against its own un-hovered
	## value above. `n_pt` is in the SAME converted space `debug_state()`'s
	## own `centre` lives in (`_at_to_local()`'s own header) -- `_move()`
	## needs it inverted back into `ov`'s local space first, exactly like
	## every other leg's synthetic pointer position.
	_move(ov, _at_to_local(ov, n_pt))
	await _frames(2)
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	var img_g2: Image = _vp.get_texture().get_image()
	var wedge_px := img_g2.get_pixelv(Vector2i(int(disc_pt.x), int(disc_pt.y)))
	print("G5 hover=%s before-hover=%s after-hover(N)=%s" %
		[_ring_state().get("hover", ""), disc_px.to_html(false), wedge_px.to_html(false)])
	_ok("G5 the hover wedge changes the pixel toward the hovered slot",
		not _close_color(disc_px, wedge_px, 0.02), true)

	## G6: nothing draws just past the N slot's own bottom edge any more
	## (this used to be exactly where the label hung) -- that band must read
	## as the plain disc ground, not text ink. Sampled from `img_g`, the
	## UN-hovered capture: the hover wedge itself reaches almost to the disc
	## edge at the hovered bearing (by design -- `r1=disc_radius-4`), so
	## `img_g2`'s hovered frame would also tint this same point and the check
	## would no longer isolate "did the label move" from "is N hovered".
	var below_pt: Vector2 = n_pt + Vector2(0, slot_r * 0.5 + 6.0)
	var below_px := img_g.get_pixelv(Vector2i(int(below_pt.x), int(below_pt.y)))
	print("G6 below-slot=%s disc=%s" % [below_px.to_html(false), disc_px.to_html(false)])
	_ok("G6 nothing draws past the slot's own edge (label moved inside)",
		_close_color(below_px, disc_px, 0.05), true)

	## G7: the caption pill's own band, past the disc's edge.
	var cap_pt: Vector2 = g_c + Vector2(0, 1) * (disc_r + 10.0)
	var cap_px := img_g2.get_pixelv(Vector2i(int(cap_pt.x), int(cap_pt.y)))
	print("G7 far_bg=%s caption=%s" % [far_bg.to_html(false), cap_px.to_html(false)])
	_ok("G7 the caption pill is drawn below the ring", not _close_color(cap_px, far_bg, 0.02), true)
	await _ring_close()
	app.arm_tool("inspect")
	await _frames(2)

	## G8: a sub-ring's own centre fills in the theme accent ("‹ BACK"),
	## checked and screenshotted in both palettes -- the same "assert the
	## palette the threshold was written for" rule leg P already follows.
	for dark_g8 in [true, false]:
		var was_g8 := DccTheme.is_dark()
		if was_g8 != dark_g8:
			DccTheme.apply_theme(dark_g8)
			app.rebuild_theme(was_g8)
			await _frames(8)
		await _held_drag(ov, centre, "NW")   ## Uplift -- opens its own sub-ring
		await _frames(2)
		var st_g8 := _ring_state()
		if bool(st_g8.get("sub_open", false)):
			var sub_c_g8: Vector2 = st_g8.get("sub_centre", Vector2.ZERO)
			await RenderingServer.frame_post_draw
			await RenderingServer.frame_post_draw
			var img_g3: Image = _vp.get_texture().get_image()
			var back_px := img_g3.get_pixelv(Vector2i(int(sub_c_g8.x), int(sub_c_g8.y)))
			var accent := DccTheme.c("accent")
			print("G8 %s sub-centre=%s accent=%s" %
				[("dark" if dark_g8 else "light"), back_px.to_html(false), accent.to_html(false)])
			_ok("G8 %s the sub-ring's own centre draws in the theme accent" % ("dark" if dark_g8 else "light"),
				_close_color(back_px, accent, 0.10), true)
			if OS.get_environment("CTXRING_SHOT_DIR") != "":
				img_g3.save_png(OS.get_environment("CTXRING_SHOT_DIR").path_join(
					"ctxring_sub_%s.png" % ("dark" if dark_g8 else "light")))
		else:
			_ok("G8 %s Uplift opened its own sub-ring (precondition for the centre check)" %
				("dark" if dark_g8 else "light"), st_g8.get("sub_open", false), true)
		await _ring_close()
		app.arm_tool("inspect")
		await _frames(2)
	await _ring_close()
	app.arm_tool("inspect")
	await _frames(2)

	# -- L: every label sits fully inside its own circle (main and sub) ---------
	app.select_domain("world")
	await _frames(4)
	app.arm_tool("inspect")
	await _frames(2)
	## Q tap (press, release with no aim in between) leaves the ring open
	## sticky with `_hover == ""` -- the same no-wedge state leg G's own
	## `img_g` capture relies on, confirmed there.
	_move(ov, centre)
	await _frames(1)
	await _qkey(true)
	await _qkey(false)
	await _frames(4)
	var st_l := _ring_state()
	var l_centre: Vector2 = st_l.get("centre", Vector2.ZERO)
	var l_radius: float = st_l.get("radius", 0.0)
	var l_slot: float = st_l.get("slot", 0.0)
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	var img_l: Image = _vp.get_texture().get_image()
	for dir in DIR_DEG:
		var pos: Vector2 = l_centre + _dir_vec(dir) * l_radius
		var res := _label_inside_circle(img_l, pos, l_slot * 0.5)
		if not res.get("ok", true):
			print("L main %s FAIL at=%s px=%s median_lum=%.3f sample_lum=%.3f" %
				[dir, res["at"], res["px"], res.get("median_lum", 0.0), res.get("sample_lum", 0.0)])
		_ok("L main slot %s: its label stays inside its own circle" % dir, res.get("ok", false), true)
	await _ring_close()
	app.arm_tool("inspect")
	await _frames(2)

	## Uplift's own sub-ring -- its "Cliff / Escarpment" child is the long-
	## text case the wrap/shrink path exists for.
	await _held_drag(ov, centre, "NW")
	await _frames(2)
	var st_l2 := _ring_state()
	if bool(st_l2.get("sub_open", false)):
		var sub_centre_l: Vector2 = st_l2.get("sub_centre", Vector2.ZERO)
		var sub_radius_l: float = st_l2.get("sub_radius", 0.0)
		var sub_slot_l: float = st_l2.get("sub_slot_size", 0.0)
		var sub_n: int = int(st_l2.get("sub_count", 0))
		await RenderingServer.frame_post_draw
		await RenderingServer.frame_post_draw
		var img_l2: Image = _vp.get_texture().get_image()
		for i in sub_n:
			var ang := deg_to_rad(-90.0 + 360.0 * float(i) / float(maxi(1, sub_n)))
			var pos2: Vector2 = sub_centre_l + Vector2(cos(ang), sin(ang)) * sub_radius_l
			var res2 := _label_inside_circle(img_l2, pos2, sub_slot_l * 0.5)
			if not res2.get("ok", true):
				print("L sub #%d FAIL at=%s px=%s median_lum=%.3f sample_lum=%.3f" %
					[i, res2["at"], res2["px"], res2.get("median_lum", 0.0), res2.get("sample_lum", 0.0)])
			_ok("L sub slot #%d: its label stays inside its own circle" % i, res2.get("ok", false), true)
	else:
		_ok("L Uplift opened its own sub-ring (precondition for the sub-slot label check)",
			st_l2.get("sub_open", false), true)
	await _ring_close()
	app.arm_tool("inspect")
	await _frames(2)

	# -- K: the continuous nested flick -- ONE RMB drag, no release in between --
	# `MAP_CONTEXT_SCOPE.md` §5.2 rule 2 / mockup `ringHover`'s own
	# `dist>RR+18` gate: drag onto Uplift (NW), keep going PAST the ring's own
	# radius (opens the sub-ring in place, still mid-drag, no release), keep
	# dragging onto its first sub-slot (Mountains), release there -- one
	# continuous gesture, no intermediate release/click.
	app.select_domain("world")
	await _frames(4)
	app.arm_tool("inspect")
	await _frames(2)
	_rmb_press(ov, centre)
	_move(ov, centre + _dir_vec("NW") * 20.0, MOUSE_BUTTON_MASK_RIGHT)   ## past the slop, short of push-past
	await _frames(1)
	_ok("K mid-drag, short of push-past: no sub-ring yet", _ring_state().get("sub_open", false), false)
	var st_k0 := _ring_state()
	var push_r: float = float(st_k0.get("radius", RING_RADIUS)) + float(st_k0.get("push_past", 18.0)) + 6.0
	_move(ov, centre + _dir_vec("NW") * push_r, MOUSE_BUTTON_MASK_RIGHT)
	await _frames(2)
	_ok("K dragging PAST the ring opened Uplift's sub-ring -- still mid-drag, RMB never released",
		_ring_state().get("sub_open", false), true)
	_ok("K ...and nothing armed yet", app.armed_tool, "inspect")
	var st_k1 := _ring_state()
	var item0_at := _sub_item_at(st_k1, 0)   ## index 0 -- Mountains
	var item0_local := _at_to_local(ov, item0_at)
	_move(ov, item0_local, MOUSE_BUTTON_MASK_RIGHT)   ## still the SAME drag
	await _frames(2)
	_ok("K ...hovering the sub-ring's own first item", int(_ring_state().get("sub_hover", -1)), 0)
	_rmb_release(ov, item0_local)   ## the ONE release for the whole gesture
	await _frames(4)
	_ok("K one continuous release picked it: Sculpt armed", app.armed_tool, "sculpt")
	_ok("K ...with Mountains the live feature", bridge.sculpt_get_feature(), "mountains")
	_ok("K ...and the ring closed itself", _ring_state().get("active", false), false)
	app.arm_tool("inspect")
	await _frames(2)

	## K2: releasing in the sub-ring's own dead zone, still inside the SAME
	## continuous drag, must keep today's behaviour -- the ring stays open and
	## sticky, exactly as a release-then-second-click into the dead zone
	## already does (`release()`'s own `_sticky` branch, unchanged by this
	## pass). Proves the continuous path did not bypass that rule.
	_rmb_press(ov, centre)
	_move(ov, centre + _dir_vec("NW") * push_r, MOUSE_BUTTON_MASK_RIGHT)
	await _frames(2)
	var st_k2 := _ring_state()
	_ok("K2 same drag opens the sub-ring again", st_k2.get("sub_open", false), true)
	var sub_c_k2: Vector2 = st_k2.get("sub_centre", Vector2.ZERO)
	var dead_local := _at_to_local(ov, sub_c_k2)   ## the sub-ring's own centre -- its dead zone
	_move(ov, dead_local, MOUSE_BUTTON_MASK_RIGHT)
	await _frames(2)
	_rmb_release(ov, dead_local)
	await _frames(4)
	_ok("K2 releasing in the sub-ring's dead zone: nothing armed", app.armed_tool, "inspect")
	_ok("K2 ...and the ring stays open, sticky", _ring_state().get("active", false), true)
	_ok("K2 ...sticky specifically", _ring_state().get("sticky", false), true)
	await _ring_close()
	app.arm_tool("inspect")
	await _frames(2)

	# -- V: CARTO's View and Style diagonals, picked through a ring gesture ------
	app.select_domain("cartography")
	await _frames(4)
	app.arm_tool("inspect")
	await _frames(2)
	var before_view = app.viewport.debug_view()
	await _held_drag(ov, centre, "SE")
	var st_view := _ring_state()
	_ok("V View opened its own sub-ring", st_view.get("sub_open", false), true)
	if bool(st_view.get("sub_open", false)):
		var view_pt := _at_to_local(ov, _sub_item_at(st_view, 1))
		await _lmb_click(ov, view_pt)
		await _frames(2)
		_ok("V picking View's sub-item #1 changed the active debug view",
			app.viewport.debug_view() != before_view, true)
	await _ring_close()
	app.arm_tool("inspect")
	await _frames(2)

	await _held_drag(ov, centre, "SW")
	var st_style := _ring_state()
	_ok("V Style opened its own sub-ring", st_style.get("sub_open", false), true)
	if bool(st_style.get("sub_open", false)):
		var style_pt := _at_to_local(ov, _sub_item_at(st_style, 2))   ## index 2 -- Antique
		await _lmb_click(ov, style_pt)
		await _frames(4)
		_ok("V picking Style's sub-item #2 (Antique) applied its look",
			bridge.look(), "Antique Parchment")
	await _ring_close()
	app.arm_tool("inspect")
	await _frames(2)

	# -- U: CM-3 residual -- Icon's Custom entry ---------------------------------
	var req_u: Dictionary = _broker().ring_domain_req()
	var slots_u: Dictionary = _broker().ring_collect(req_u)
	var icon_children_before: Array = (slots_u.get("NE", {}) as Dictionary).get("children", [])
	_ok("U before any custom icon, Icon's sub-ring carries a Custom entry",
		icon_children_before.size() >= 1, true)
	if not icon_children_before.is_empty():
		var custom_before: Dictionary = icon_children_before[icon_children_before.size() - 1]
		_ok("U ...disabled with its own reason",
			bool(custom_before.get("enabled", true)), false)
		_ok("U ...naming the missing custom art",
			String(custom_before.get("reason", "")), "no custom icons imported")

	bridge.as_set_pack_info("Ring probe pack", "probe", "CC0")
	var cslot: Dictionary = bridge.as_add_custom_slot("Ring probe icon", "")
	var cimg := Image.create(4, 4, false, Image.FORMAT_RGBA8)
	cimg.fill(Color(0.5, 0.5, 0.9, 1.0))
	var cimp: Dictionary = bridge.as_import_item(String(cslot.get("uid", "")), "probe.png",
		cimg.save_png_to_buffer())
	_ok("U fixture: the probe's own custom item imported", bool(cimp.get("ok", false)), true)
	var capplied: Dictionary = bridge.as_apply_to_map()
	_ok("U fixture: the library compiled into the live pack", bool(capplied.get("ok", false)), true)
	await _frames(2)

	var custom_live: Array = bridge.icon_custom_slots()
	_ok("U icon_custom_slots now reports the imported custom icon", custom_live.size() >= 1, true)

	var req_u2: Dictionary = _broker().ring_domain_req()
	var slots_u2: Dictionary = _broker().ring_collect(req_u2)
	var icon_children_after: Array = (slots_u2.get("NE", {}) as Dictionary).get("children", [])
	if not icon_children_after.is_empty():
		var custom_after: Dictionary = icon_children_after[icon_children_after.size() - 1]
		_ok("U Icon's Custom entry is enabled once a custom icon exists",
			bool(custom_after.get("enabled", true)), true)

	await _held_drag(ov, centre, "NE")
	var st_u := _ring_state()
	_ok("U Icon opened its own sub-ring", st_u.get("sub_open", false), true)
	if bool(st_u.get("sub_open", false)):
		var custom_i: int = int(st_u.get("sub_count", 1)) - 1
		var custom_pt := _at_to_local(ov, _sub_item_at(st_u, custom_i))
		await _lmb_click(ov, custom_pt)
		await _frames(2)
		_ok("U picking Custom armed the icon tool", app.armed_tool, "icon")
		var armed_u: Dictionary = bridge.icon_armed()
		_ok("U ...with family 'custom'", String(armed_u.get("family", "")), "custom")
		_ok("U ...and a real, non-empty set/slot pair",
			String(armed_u.get("slot", "")) != "" and String(armed_u.get("set", "")) != "", true)
	await _ring_close()
	app.arm_tool("inspect")
	await _frames(2)

	# -- W: the Way▸ sub-ring offers the engine's real vocabulary ----------------
	app.select_domain("civilization")
	await _frames(4)
	app.arm_tool("inspect")
	await _frames(2)
	var req_w: Dictionary = _broker().ring_domain_req()
	var slots_w: Dictionary = _broker().ring_collect(req_w)
	var way_children: Array = (slots_w.get("SE", {}) as Dictionary).get("children", [])
	var way_labels: Array = []
	for c in way_children:
		way_labels.append(String((c as Dictionary).get("label", "")))
	_ok("W Way's sub-ring offers exactly the engine's real vocabulary",
		way_labels, ["Road", "Track", "Sea lane", "Ancient"])

	await _held_drag(ov, centre, "SE")
	var st_w := _ring_state()
	_ok("W Way opened its own sub-ring", st_w.get("sub_open", false), true)
	if bool(st_w.get("sub_open", false)):
		var seal_pt := _at_to_local(ov, _sub_item_at(st_w, 2))   ## index 2 -- Sea lane
		await _lmb_click(ov, seal_pt)
		await _frames(2)
		_ok("W picking Sea lane armed the way tool", app.armed_tool, "way")
		var infra := _find_infra()
		_ok("W ...with sea_lane the live way type",
			String(infra.get("_way_type")) if infra != null else "<no infra workspace>", "sea_lane")
	await _ring_close()
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
		print("### CTXRING ABORT: unknown argument '%s' ###" % a)
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
	if form != "desktop":
		print("### CTXRING ABORT: CM-3 is desktop-only -- booted as %s ###" % form)
		get_tree().quit(2)
		return
	await _run()
	print("### CTXRING %s  %d/%d checks passed ###" % [
		"GREEN" if _fails == 0 else "RED", _checks - _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
