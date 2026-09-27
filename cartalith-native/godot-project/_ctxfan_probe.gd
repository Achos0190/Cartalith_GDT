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
##   G  Ruling BC (`LARGE_ITEM_RULINGS.md`) on `shell/tool_fan.gd`: a slot's
##      ground is drawn behind its label (nothing bleeds below the slot's own
##      circle any more), and the sub-fan's own BACK button fills in the
##      theme accent with a glow ring around it

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


static func _close_color(a: Color, b: Color, tol: float) -> bool:
	return absf(a.r - b.r) <= tol and absf(a.g - b.g) <= tol and absf(a.b - b.b) <= tol


## True if any pixel within `radius` of `pt` differs from the CORNER of that
## same box (a stand-in "ground" sample) by more than a faint threshold --
## i.e. there is text/ink drawn somewhere in that little box, not a flat
## fill. Used by leg G to assert the ABSENCE of ink just past a slot's own
## edge (where the label used to hang).
func _ink_present(img: Image, pt: Vector2, radius: int) -> bool:
	var cx := int(pt.x)
	var cy := int(pt.y)
	var corner := img.get_pixelv(Vector2i(cx - radius, cy - radius))
	for y in range(cy - radius, cy + radius + 1):
		for x in range(cx - radius, cx + radius + 1):
			if x < 0 or y < 0 or x >= img.get_width() or y >= img.get_height():
				continue
			if not _close_color(img.get_pixel(x, y), corner, 0.05):
				return true
	return false


## Leg L's own check, phone-specific: the fan has no disc behind its slots
## (`tool_fan.gd`'s own `_Overlay._draw()` -- only the 55%-alpha scrim), so
## the area just outside a slot circle is largely raw map, whose natural
## contrast (coastlines, terrain) routinely exceeds a median-luminance
## threshold on its own -- measured: 11 of 14 slots "failed" a median check
## the first time this leg ran, none of them a real overflow. A hover-based
## before/after (the "W" check above) does not generalise to the sub-fan
## EITHER: once sticky, `ToolFan.pointer()` no-ops (`if not _pressed:
## return`) -- there is no live hover preview for a sub item, only a tap that
## immediately resolves and closes, so nothing to diff against.
##
## The reliable signal that needs neither: the scrim's own blend is a KNOWN,
## fixed formula (`Color(0.031,0.035,0.039,0.55)` over whatever is beneath,
## `tool_fan.gd::_Overlay._draw()`'s own literal). From a plain "fan closed"
## capture of the raw map, `_expect_scrim()` predicts exactly what an
## unmarked point should read once the fan opens; a point that still matches
## that prediction has nothing extra drawn on it, whatever the surrounding
## terrain looks like. A point carrying label/glyph ink will not match.
const _SCRIM_RGB := Color(0.031, 0.035, 0.039)
const _SCRIM_A := 0.55

static func _expect_scrim(raw: Color) -> Color:
	return raw.lerp(_SCRIM_RGB, _SCRIM_A)


static func _no_bleed_past_circle(img_raw: Image, img_open: Image, centre: Vector2, radius: float) -> Dictionary:
	var steps := 40
	var radii := [radius + 2.0, radius + 4.0, radius + 6.0]
	for i in steps:
		var ang := TAU * float(i) / float(steps)
		for rr in radii:
			var p: Vector2 = centre + Vector2(cos(ang), sin(ang)) * float(rr)
			var pi := Vector2i(int(p.x), int(p.y))
			if pi.x < 0 or pi.y < 0 or pi.x >= img_raw.get_width() or pi.y >= img_raw.get_height():
				continue
			var expected := _expect_scrim(img_raw.get_pixel(pi.x, pi.y))
			var actual := img_open.get_pixel(pi.x, pi.y)
			if not _close_color(expected, actual, 0.05):
				return {"ok": false, "at": p, "expected": expected.to_html(false), "actual": actual.to_html(false)}
	return {"ok": true}


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

	# -- G: Ruling BC -- label inside the slot, BACK button in the accent -------
	app.select_domain("world")
	await _frames(4)
	app.arm_tool("inspect")
	await _frames(2)
	## Re-read fresh, not the `centre` captured at the very top of `_run()` --
	## measured the first time this leg ran: the pill's own rect had shifted
	## 132 px vertically by this point in the run (some other leg's UI change
	## moved it), which put a diagonal aim at the OLD centre 132 px off the
	## live `NW` slot -- just outside `HIT_MAX` -- and read as "nothing
	## hovered" instead of a wrong-but-close hit.
	var pc: Vector2 = fan.pill_global_rect().get_center()
	fan.press(pc)
	await get_tree().create_timer(0.3).timeout
	await _frames(2)
	## The scrim sits OVER the live map (55% alpha, `tool_fan.gd`'s own
	## `_Overlay._draw()`), so a fixed screen corner is not a reliable
	## "plain ground" reference -- it reads whatever terrain tile is under
	## it. Measured the first time this leg ran: a point comfortably past
	## `W`'s own circle read nothing like a supposedly-identical corner,
	## because the two sit over different map content, not because
	## anything bled past the slot.
	##
	## The reliable check is a BEFORE/AFTER at the exact same pixel: hover
	## some OTHER direction first (so the point past `W`'s circle is plain
	## scrimmed map, whatever colour that map happens to be), then hover
	## `W` itself and re-sample the identical pixel. If the label/glyph
	## still reached past the circle, this pixel would change; if it stayed
	## inside, it does not.
	var w_pos := pc + Vector2(-150.0 * scale, 0.0)
	var d: float = 28.0 * scale   ## `fan.SLOT_D * scale * 0.5` -- the live slot radius.
	var above_pt := w_pos + Vector2(0.0, -d * 1.4)
	fan.pointer(pc + Vector2(0.0, -150.0 * scale))  ## hover N -- anywhere but W
	await _frames(2)
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	var img_before: Image = _vp.get_texture().get_image()
	var before_px := img_before.get_pixelv(Vector2i(int(above_pt.x), int(above_pt.y)))
	fan.pointer(pc + Vector2(-150.0 * scale, 0.0))  ## now hover W (Region)
	await _frames(2)
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	var img_after: Image = _vp.get_texture().get_image()
	var after_px := img_after.get_pixelv(Vector2i(int(above_pt.x), int(above_pt.y)))
	print("G above-W before=%s after=%s w_pos=%s above_pt=%s scale=%.3f" %
		[before_px.to_html(false), after_px.to_html(false), w_pos, above_pt, scale])
	_ok("G nothing draws past the W slot's own edge (label moved inside)",
		_close_color(before_px, after_px, 0.03), true)
	fan.release()
	await _frames(4)
	app.arm_tool("inspect")
	await _frames(2)

	## Uplift (NW, `children`) opens the sub-fan; its own centre -- the pill's
	## centre -- then draws the accent-filled BACK button (mockup: 56 px,
	## `background:#e0a34a`, glow `0 0 0 4px rgba(224,163,74,.25)`).
	## `fanPos()`'s own convention (canvas: `FAX+r*cos(a), FAY-r*sin(a)`, math
	## angle, NW=135deg): BOTH offsets are negative (up and to the left) --
	## `tool_fan.gd::_hit_test()`'s own `Vector2(cos(ang2), -sin(ang2))`
	## reproduces the same sign. Checked and screenshotted in both palettes.
	for dark_g in [true, false]:
		var was_g := DccTheme.is_dark()
		if was_g != dark_g:
			DccTheme.apply_theme(dark_g)
			app.rebuild_theme(was_g)
			await _frames(8)
		var tag := "dark" if dark_g else "light"
		## Re-read the pill's centre again rather than reuse `pc` from above
		## -- the same drift this leg's header already measured once is
		## cheap to guard against a second time.
		var pc2: Vector2 = fan.pill_global_rect().get_center()
		var glow_pt := pc2 + Vector2(0.0, -(56.0 * scale * 0.5 + 6.0))
		fan.press(pc2)
		await get_tree().create_timer(0.3).timeout   ## still-hold: fan opens, no sub yet
		await _frames(2)
		await RenderingServer.frame_post_draw
		await RenderingServer.frame_post_draw
		var glow_before := _vp.get_texture().get_image().get_pixelv(Vector2i(int(glow_pt.x), int(glow_pt.y)))
		var nw_ang := deg_to_rad(135.0)
		var nw_target := pc2 + Vector2(cos(nw_ang), -sin(nw_ang)) * 150.0 * scale
		fan.pointer(nw_target)  ## NW, outer radius
		await _frames(2)
		fan.release()
		await _frames(4)
		if fan.sub_open():
			await RenderingServer.frame_post_draw
			await RenderingServer.frame_post_draw
			var img_g2: Image = _vp.get_texture().get_image()
			var back_px := img_g2.get_pixelv(Vector2i(int(pc2.x), int(pc2.y)))
			var accent := DccTheme.c("accent")
			print("G %s sub-fan centre=%s accent=%s" % [tag, back_px.to_html(false), accent.to_html(false)])
			_ok("G %s the sub-fan's own BACK button fills in the theme accent" % tag,
				_close_color(back_px, accent, 0.10), true)
			## Baseline captured moments earlier at the SAME pixel, fan open
			## but no sub yet -- not a fixed screen corner, since the scrim
			## sits over the live map and a corner reads whatever terrain
			## tile is under IT.
			var glow_px := img_g2.get_pixelv(Vector2i(int(glow_pt.x), int(glow_pt.y)))
			print("G %s glow pixel=%s before(no sub)=%s" % [tag, glow_px.to_html(false), glow_before.to_html(false)])
			_ok("G %s a glow ring is drawn around the BACK button" % tag,
				not _close_color(glow_px, glow_before, 0.02), true)
			if OS.get_environment("CTXFAN_SHOT_DIR") != "":
				img_g2.save_png(OS.get_environment("CTXFAN_SHOT_DIR").path_join("ctxfan_sub_%s.png" % tag))
		else:
			_ok("G %s Uplift opened the sub-fan (precondition for the BACK-button check)" % tag,
				fan.sub_open(), true)
		fan.click_at(pc2 + Vector2(400.0 * scale, 400.0 * scale))
		await _frames(2)
		app.arm_tool("inspect")
		await _frames(2)

	# -- L: every label sits fully inside its own circle (Ruling BC) ------------
	var pc3: Vector2 = fan.pill_global_rect().get_center()
	var d_l: float = 28.0 * scale   ## `fan.SLOT_D * scale * 0.5` -- the live main-slot radius.
	## The raw map, fan fully closed -- the "nothing drawn here at all" frame
	## `_expect_scrim()` predicts from, reused for every point below (the map
	## itself never changes across this leg).
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	var img_raw: Image = _vp.get_texture().get_image()

	fan.press(pc3)
	await get_tree().create_timer(0.3).timeout   ## still-hold, no move -- every slot un-hovered.
	await _frames(2)
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	var img_main: Image = _vp.get_texture().get_image()
	for f in fan.FAN:
		var dir_l: String = f[0]
		var ang_l := deg_to_rad(float(f[1]))
		var pos_l: Vector2 = pc3 + Vector2(cos(ang_l), -sin(ang_l)) * float(f[2]) * scale
		var res := _no_bleed_past_circle(img_raw, img_main, pos_l, d_l)
		if not res.get("ok", true):
			print("L fan main %s FAIL at=%s expected=%s actual=%s" %
				[dir_l, res["at"], res["expected"], res["actual"]])
		_ok("L fan main slot %s: its label stays inside its own circle" % dir_l, res.get("ok", false), true)
	fan.release()
	await _frames(4)
	app.arm_tool("inspect")
	await _frames(2)

	## Uplift's own sub-fan -- "Cliff / Escarpment" is the long-text case the
	## wrap/shrink path exists for, same as `_ctxring_probe.gd`'s own leg L.
	fan.press(pc3)
	await _frames(1)
	var nw_ang_l := deg_to_rad(135.0)
	fan.pointer(pc3 + Vector2(cos(nw_ang_l), -sin(nw_ang_l)) * 150.0 * scale)
	await _frames(2)
	fan.release()
	await _frames(4)
	if fan.sub_open():
		var sub_n_l: int = fan.debug_state().get("sub_count", 0)
		await RenderingServer.frame_post_draw
		await RenderingServer.frame_post_draw
		var img_sub: Image = _vp.get_texture().get_image()
		for i in sub_n_l:
			var ang2: float = deg_to_rad(180.0 - float(i) * 180.0 / float(sub_n_l - 1)) if sub_n_l > 1 \
				else deg_to_rad(90.0)
			var pos2: Vector2 = pc3 + Vector2(cos(ang2), -sin(ang2)) * fan.OUTER_R * scale
			var res2 := _no_bleed_past_circle(img_raw, img_sub, pos2, d_l)
			if not res2.get("ok", true):
				print("L fan sub #%d FAIL at=%s expected=%s actual=%s" %
					[i, res2["at"], res2["expected"], res2["actual"]])
			_ok("L fan sub slot #%d: its label stays inside its own circle" % i, res2.get("ok", false), true)
	else:
		_ok("L Uplift opened the sub-fan (precondition for the sub-slot label check)",
			fan.sub_open(), true)
	fan.click_at(pc3 + Vector2(400.0 * scale, 400.0 * scale))
	await _frames(2)
	app.arm_tool("inspect")
	await _frames(2)


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
