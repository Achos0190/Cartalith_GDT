extends Node
## CM-6 (`MAP_CONTEXT_SCOPE.md` §8.2, §11; Ruling AX F2): the phone's thumb
## fan -- `shell/tool_fan.gd`. Drives the real state machine through
## `ToolFan.press()`/`.pointer()`/`.release()`/`.click_at()`, the same
## global-position probe seam `context_broker.gd::ring_press()` etc. give
## `RadialRing`, and reads back through the real armed-tool state
## (`app.armed_tool`) rather than the fan's own hover, the same rule
## `_ctxtablet_probe.gd`'s own leg S already follows.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _ctxfan_probe.tscn -- --vp 1080x2340 --force-touch
##
## **Windowed, not `--headless`**: leg R reads the fan's real global rect
## against the real viewport. `--vp WxH` sizes the host SubViewport and
## `--force-touch` is the shell's own flag (`DccShell._ready()`); both are
## required or the probe aborts (leg FORM).
##
## Legs:
##   T  a 200 ms press-and-release (no slide) is a tap: the fan never opens,
##      and it toggles Inspect <-> the previously-armed tool
##   O  a press held past 260 ms without moving opens the fan
##   M  moving > 12 dp before 260 ms opens the fan immediately (the slop path)
##   A  dragging onto N (Inspect) and releasing arms it, checked through the
##      real `app.armed_tool`
##   A2 the same for E (Measure) and W (Region) -- the ring's own global
##      cardinals (§5.1), reused rather than a second table
##   Z  releasing with no hover leaves the fan open, sticky (§6's own rule,
##      reimplemented here for the half-fan)
##   K  once sticky, a tap on a slot (via `click_at()`, the overlay's own
##      path) arms it
##   X  once sticky, a tap outside every slot closes the fan (dismissal)
##   R  the fan's bounding rect stays inside the phone viewport, measured in
##      both px and dp
##   C  regression: CM-5's own probe, `_ctxphone_probe.gd`, still passes
##      31/31 -- run separately by the caller, not from inside this file

const SEED := 552017

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


func _fan():
	return app._tool_fan


func _run() -> void:
	app.select_domain("civilization")
	await _frames(4)
	var fan = _fan()
	var pill_rect: Rect2 = fan.pill_global_rect()
	print("FIXTURE pill rect=%s" % [pill_rect])
	_ok("the pill has a real, non-empty rect", pill_rect.size.x > 0.0 and pill_rect.size.y > 0.0, true)
	var centre: Vector2 = pill_rect.get_center()

	# -- T: a quick tap toggles Inspect, never opens the fan ---------------------
	app.arm_tool("measure")
	await _frames(2)
	fan.press(centre)
	await _frames(2)
	fan.release()
	await _frames(2)
	_ok("T a quick tap: the fan never opened", fan.is_open(), false)
	_ok("T a quick tap: Inspect is now armed", app.armed_tool, "inspect")
	fan.press(centre)
	await _frames(2)
	fan.release()
	await _frames(2)
	_ok("T a second quick tap: back to the previous tool", app.armed_tool, "measure")
	app.arm_tool("inspect")
	await _frames(2)

	# -- O: a still hold past 260 ms opens the fan --------------------------------
	fan.press(centre)
	await get_tree().create_timer(0.1).timeout
	_ok("O 100ms still: not open yet", fan.is_open(), false)
	await get_tree().create_timer(0.25).timeout
	await _frames(2)
	_ok("O 350ms still: the fan opened", fan.is_open(), true)
	fan.release()
	await _frames(2)
	_ok("O released with no hover: the fan stayed open (sticky)", fan.is_open(), true)
	_ok("O sticky flag is set", fan.is_sticky(), true)
	## Close it (tap the dead zone, sticky -> cancel) before leg M.
	fan.click_at(centre)
	await _frames(2)
	_ok("cleanup before M: closed", fan.is_open(), false)

	# -- M: moving past the 12 dp slop opens the fan immediately -----------------
	var scale: float = app.phone_scale()
	fan.press(centre)
	await _frames(1)
	fan.pointer(centre + Vector2(0.0, -20.0 * scale))
	await _frames(2)
	_ok("M moved past slop before 260ms: the fan is already open", fan.is_open(), true)
	fan.release()
	await _frames(2)
	fan.click_at(centre)  ## cleanup: cancel the sticky fan
	await _frames(2)

	# -- A / A2: dragging onto a cardinal arms it, checked through armed_tool ----
	## N = Inspect, E = Measure, W = Region -- `GlobalTools.ring_cardinals()`,
	## the SAME provider CM-3's desktop ring calls, reused rather than a
	## second table (this file's own header, and the milestone's own brief).
	app.arm_tool("region")
	await _frames(2)
	fan.press(centre)
	await _frames(1)
	fan.pointer(centre + Vector2(0.0, -150.0 * scale))  ## N, outer radius
	await _frames(2)
	fan.release()
	await _frames(4)
	_ok("A drag to N arms Inspect", app.armed_tool, "inspect")
	_ok("A after arming: the fan closed", fan.is_open(), false)

	## E (Measure) carries `children` (`GlobalTools._measure_ring_children()`,
	## the Measure ▸ sub-ring CM-3's own desktop ring already opens for the
	## same row) -- a drag-release onto E opens the sub-fan rather than arming
	## anything directly, exactly as `RadialRing.release()` does for the same
	## row (`if row.has("children"): _open_sub(...); return`). Verified as a
	## sub-open here, then a second selection (via `click_at`, the sticky-tap
	## path leg K already exercises) arms Measure through the child's own
	## `set_measure_mode()`, which is what actually calls `app.arm_tool`.
	fan.press(centre)
	await _frames(1)
	fan.pointer(centre + Vector2(150.0 * scale, 0.0))  ## E, outer radius
	await _frames(2)
	fan.release()
	await _frames(4)
	_ok("A2 drag to E opens its sub-fan (Measure ▸)", fan.sub_open(), true)
	_ok("A2 the sub-fan is non-empty", fan.debug_state().get("sub_count", 0) > 0, true)
	## Child index 1 ("Bearing"), not 0: `GlobalTools._measure_mode` defaults
	## to item 0's own id ("distance"), and `set_measure_mode()` early-returns
	## with no `app.arm_tool()` call when the requested mode is already the
	## live one (`if _measure_mode == id: return`) -- picking the
	## already-active mode would be a no-op indistinguishable from a broken
	## wire. Index 1 is guaranteed to differ from the static default. Same
	## angle formula `tool_fan.gd`'s `_hit_test()`/`_Overlay._draw_sub()` use
	## for `n` items on the outer arc.
	var sub_n: int = fan.debug_state().get("sub_count", 0)
	_ok("A2 the sub-fan has at least two entries (index 1 exists)", sub_n >= 2, true)
	var ang1: float = deg_to_rad(180.0 - 1.0 * 180.0 / float(sub_n - 1))
	var child1_pos: Vector2 = centre + Vector2(cos(ang1), -sin(ang1)) * 150.0 * scale
	fan.click_at(child1_pos)
	await _frames(4)
	_ok("A2 selecting the sub-fan's second child arms Measure", app.armed_tool, "measure")
	_ok("A2 after arming from the sub-fan: closed", fan.is_open(), false)
	app.arm_tool("region")
	await _frames(2)

	fan.press(centre)
	await _frames(1)
	fan.pointer(centre + Vector2(-150.0 * scale, 0.0))  ## W, outer radius
	await _frames(2)
	fan.release()
	await _frames(4)
	_ok("A2 drag to W arms Region", app.armed_tool, "region")

	# -- Z: a release with no hover goes sticky, not closed ----------------------
	fan.press(centre)
	await get_tree().create_timer(0.3).timeout  ## past FAN_HOLD_MS -- the still-hold path opens it
	fan.pointer(centre)  ## no movement -- still in the dead zone
	await _frames(2)
	fan.release()
	await _frames(2)
	_ok("Z release with no hover: still open", fan.is_open(), true)
	_ok("Z release with no hover: sticky", fan.is_sticky(), true)

	# -- K: once sticky, a tap on a slot (via click_at) arms it -------------------
	## N (Inspect), not E: E carries `children` (Measure's own sub-ring, leg
	## A2 already covers it) and a tap on it would open the sub-fan rather
	## than arm anything -- exactly `RadialRing.release()`'s own rule for a
	## slot with children, reused here rather than re-derived.
	_ok("K pre-req: armed tool is not yet Inspect", app.armed_tool != "inspect", true)
	fan.click_at(centre + Vector2(0.0, -150.0 * scale))  ## N == Inspect
	await _frames(4)
	_ok("K sticky tap on N arms Inspect", app.armed_tool, "inspect")
	_ok("K after arming from sticky: closed", fan.is_open(), false)

	# -- X: once sticky, a tap outside every slot dismisses the fan --------------
	fan.press(centre)
	await get_tree().create_timer(0.3).timeout
	await _frames(2)
	_ok("X pre-req: the fan is open", fan.is_open(), true)
	fan.release()
	await _frames(2)
	_ok("X pre-req: sticky", fan.is_sticky(), true)
	fan.click_at(centre + Vector2(400.0 * scale, 400.0 * scale))  ## far outside every slot
	await _frames(2)
	_ok("X tap outside dismisses the fan", fan.is_open(), false)
	_ok("X armed tool unchanged by the dismiss tap", app.armed_tool, "inspect")

	# -- R: the fan's own bounding rect stays inside the phone viewport ----------
	var vis: Rect2 = app.get_viewport().get_visible_rect()
	fan.press(centre)
	await get_tree().create_timer(0.3).timeout
	await _frames(2)
	var bounds: Rect2 = fan.fan_bounds_rect()
	print("R fan bounds px=%s dp=%s viewport_px=%s" %
		[bounds, Rect2(bounds.position / scale, bounds.size / scale), vis.size])
	_ok("R the fan's bounds are non-empty", bounds.size.x > 0.0 and bounds.size.y > 0.0, true)
	_ok("R the fan stays inside the viewport", vis.grow(1.0).encloses(bounds), true)
	fan.release()  ## end the still-held press (-> sticky, RadialRing's own "first release" rule)
	await _frames(2)
	fan.click_at(centre)  ## cleanup: a second tap on the dead zone, now sticky, cancels
	await _frames(2)
	_ok("cleanup: closed at end of run", fan.is_open(), false)


func _ready() -> void:
	var vp_size := Vector2i(1080, 2340)
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
		print("### CTXFAN ABORT: unknown argument '%s' ###" % a)
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
	if form != "phone":
		print("### CTXFAN ABORT: CM-6 needs the phone form -- booted as %s. Use --force-touch and a phone-sized --vp (e.g. 1080x2340) ###" % form)
		get_tree().quit(2)
		return
	if app._tool_fan == null:
		print("### CTXFAN ABORT: no ToolFan on this build ###")
		get_tree().quit(2)
		return
	await _run()
	print("### CTXFAN %s  %d/%d checks passed ###" % [
		"GREEN" if _fails == 0 else "RED", _checks - _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
