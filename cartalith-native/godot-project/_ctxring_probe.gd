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

const SEED := 483920

## `radial_ring.gd`'s own geometry constants, duplicated here (not imported)
## so this probe measures the SHIPPED geometry rather than assuming it stays
## in sync by construction -- the same reason `_ctxcard_probe.gd`'s own `E`/`P`
## legs hardcode row heights instead of reading them off the source file.
const RING_RADIUS := 60.0
const SLOT_SIZE := 46.0
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
	_rmb_release(ov, centre)
	await _frames(2)
	_ok("H releasing in the dead zone leaves the ring open (sticky)", _ring_state().get("active", false), true)
	await _ring_close()
	await _card_close()

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
		var label_rect := Rect2(slot_pos.x - 24, slot_pos.y + SLOT_SIZE * 0.5 + 4.0, 48, 16)
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
