extends Node
## First-run coach tips must not sit on working UI (owner report 2026-11-15: "Drag this handle to expand tool options." over the pin
## palette's first tile row in `phone_phone_sheet_half.png`/`phone_change_mode.png`; "MAP . domain . STORY switch tasks here" over the
## CARTO sheet header and the pin strip in `final_phone_strip.png`). Cause: `_show_phone_toast()` places a tip once, above its anchor,
## and the sheet then moved under it; and a tip started while the sheet already showed content landed on it. The fix is in
## `dcc_shell.gd`: `_coach_blocked()` defers a tip (not marked seen) while the sheet is above `peek` or the tool strip is visible,
## `_dismiss_coach_tips()` removes a live tip when either happens, and a tap anywhere dismisses it without being consumed.
##
## PHONE  : `-- --vp 1080x2340 --force-touch --density 2.625 --out <dir> --tag <name>`  (fresh scratch APPDATA, so the tips show)
## DESKTOP: `-- --vp 1700x1000 --out <dir> --tag <name>`   (no tip may ever appear; the three marks still exist)
## Frames `<tag>_<form>_<name>.png`. Exit 0 held, 1 failed, 2 could not run. Windowed only.
var app: Node
var _vp: SubViewport
var _fails := 0
var _out := ""
var _tag := "real"
var _form := "desktop"
var _last := Vector2.ZERO
const TEXTS := ["MAP · domain · STORY switch tasks here", "Drag this handle to expand tool options.", "A one-finger drag only moves the map"]


func _p(s: String) -> void:
	print(s)

func _ok(name: String, cond: bool, detail: String = "") -> void:
	if cond:
		_p("PASS " + name)
	else:
		_fails += 1
		_p("FAIL %s  %s" % [name, detail])

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png(_out.path_join("%s_%s_%s" % [_tag, _form, name]))
	_p("shot %s_%s_%s" % [_tag, _form, name])

func _click(at: Vector2) -> void:
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.device = InputEvent.DEVICE_ID_EMULATION
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		ev.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
		_vp.push_input(ev)
		await _frames(3)

## Every visible Label on screen that carries one of the three tip texts (the tip as drawn, not the shell's bookkeeping).
func _tip_labels() -> Array:
	var out: Array = []
	for l in app.find_children("*", "Label", true, false):
		var lab := l as Label
		if lab.is_visible_in_tree():
			for t in TEXTS:
				if lab.text.begins_with(t):
					out.append(lab)
	return out

func _tip_rects() -> Array:
	var out: Array = []
	for l in _tip_labels():
		var w: Control = l.get_parent() as Control
		out.append(w.get_global_rect())
	return out

func _wait_tip(sec: float) -> bool:
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < sec * 1000.0:
		if not _tip_rects().is_empty():
			await _frames(14)   # past the 0.18 s fade-in, so a frame shows it opaque
			return not _tip_rects().is_empty()
		await _frames(5)
	return false

func _hits(r: Rect2, ctrls: Array) -> Array:
	var bad: Array = []
	for c in ctrls:
		var cc := c as Control
		if cc.is_visible_in_tree() and cc.get_global_rect().size.x > 1.0 and r.intersects(cc.get_global_rect()):
			bad.append(String(cc.name))
	return bad

func _ready() -> void:
	var vp_size := Vector2i(1700, 1000)
	var touch := false
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--vp":
			var wh := String(args[i + 1]).split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
		elif a == "--out":
			_out = String(args[i + 1])
			i += 2
		elif a == "--tag":
			_tag = String(args[i + 1])
			i += 2
		elif a == "--density":
			i += 2
		elif a == "--force-touch":
			touch = true
			i += 1
		elif a.begins_with("--density="):
			i += 1
		else:
			_p("ABORT unknown argument %s" % a)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		_p("ABORT run windowed")
		get_tree().quit(2)
		return
	get_tree().create_timer(300.0).timeout.connect(func():
		_p("WATCHDOG")
		get_tree().quit(3))
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	_vp.notification(Node.NOTIFICATION_VP_MOUSE_ENTER)
	_form = "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() and DccTheme.is_touch() else "desktop")
	_p("form: %s, viewport %s" % [_form, str(vp_size)])
	var bridge = app.bridge
	bridge.generate({"seed": 1200, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	_ok("a world generated", bridge.has_world)
	await bridge.landmark_run()
	await _frames(20)
	for k in ["tavern", "harbour", "castle"]:
		DccSettings.pin_recent_push(k)
	_p("detent=%s blocked=%s" % [app.phone_detent(), str(app._coach_blocked())])
	_p("pre-hide: seen=%s wait=%s live=%d" % [str(app._coach_mark_ids().map(func(m): return app._coach_mark_seen(m))), str(app._coach_marks_wait_for_picker), int(app._coach_live_count())])
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()   # the phone's coach marks start when the entry screen closes
	if _form != "phone":
		await get_tree().create_timer(5.0).timeout
		_ok("%s: no coach tip is ever drawn" % _form, _tip_rects().is_empty() and int(app._coach_live_count()) == 0)
		_ok("the three marks still exist", app._coach_mark_ids() == ["bottombar_tabs", "sheet_handle", "pan_drag_hint"], str(app._coach_mark_ids()))
		await _shot("none.png")
		get_tree().quit(1 if _fails > 0 else 0)
		return
	await _run_phone()
	_p("RESULT %s fails=%d" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(1 if _fails > 0 else 0)

func _restart_marks() -> void:
	app._coach_seen_session.clear()
	var cfg := ConfigFile.new()
	cfg.load(DccSettings.CONFIG_PATH)
	if cfg.has_section("coach_marks"):
		cfg.erase_section("coach_marks")
		cfg.save(DccSettings.CONFIG_PATH)
	app._show_next_coach_mark(0)

func _run_phone() -> void:
	var vis := Rect2(Vector2.ZERO, Vector2(_vp.size))
	app.select_domain("cartography")
	await _frames(8)
	## Leg 1: at peek with nothing armed, the first tip shows, is on screen, and a tap through it reaches the sheet header.
	_ok("L1: a tip shows at peek with nothing armed", await _wait_tip(8.0))
	var rs := _tip_rects()
	_ok("L1: once-only: the first mark is recorded seen the moment it shows", app._coach_mark_seen("bottombar_tabs"))
	if not rs.is_empty():
		_ok("L1: the tip is on screen %s" % str(rs[0]), vis.encloses(rs[0]))
		_ok("L1: the tip clears the sheet and the strip", not (rs[0] as Rect2).intersects((app._phone_tool_sheet as Control).get_global_rect()) and not (rs[0] as Rect2).intersects((app._tool_strip as Control).get_global_rect()), "%s vs sheet %s strip %s" % [str(rs[0]), str((app._phone_tool_sheet as Control).get_global_rect()), str((app._tool_strip as Control).get_global_rect())])
		await _shot("tip_peek.png")
		var wrap: Control = _tip_labels()[0].get_parent() as Control
		var blocking: Array = []
		for n in [wrap] + wrap.find_children("*", "Control", true, false):
			if (n as Control).mouse_filter != Control.MOUSE_FILTER_IGNORE:
				blocking.append(String(n.name))
		_ok("L1: no part of the tip takes a tap (mouse_filter IGNORE all through)", blocking.is_empty(), str(blocking))
		## A tap at the tip's centre: it must reach the map beneath (the map's overlay sees the press) and dismiss the tip.
		var got := [false]
		var ov: Control = app.viewport.overlay
		ov.gui_input.connect(func(e: InputEvent): if e is InputEventMouseButton and e.pressed: got[0] = true)
		await _click((rs[0] as Rect2).get_center())
		_ok("L1: the tap reached the map beneath the tip", got[0])
		_ok("L1: and it dismissed the tip", _tip_rects().is_empty() and int(app._coach_live_count()) == 0)
		app.set_phone_detent("peek")
		await _frames(30)
	## Leg 2: a LIVE tip is removed when the sheet opens or a tool is armed.
	_ok("L2: another tip shows", await _wait_tip(8.0))
	app.set_phone_detent("half")
	await _frames(3)
	_ok("L2: opening the sheet removes the live tip", _tip_rects().is_empty() and int(app._coach_live_count()) == 0)
	app.set_phone_detent("peek")
	await _frames(20)
	## Leg 2b: nothing armed (Inspect) but the sheet at half, content showing: a pending tip must wait, not land on it.
	_restart_marks()
	app.set_phone_detent("half")
	await _frames(20)
	var t2 := Time.get_ticks_msec()
	var seen2 := false
	while Time.get_ticks_msec() - t2 < 3500:
		seen2 = seen2 or not _tip_rects().is_empty()
		await _frames(10)
	_ok("L2b: no tip appears while the sheet shows content at half (nothing armed)", not seen2)
	app.set_phone_detent("peek")
	await _frames(10)
	_ok("L2b: the deferred tip arrives once the sheet is back at peek", await _wait_tip(5.0))
	app._dismiss_coach_tips()
	## Leg 3 needs a PENDING mark again, or "no tip" is vacuous.
	_restart_marks()
	## Leg 3: Pins armed, sheet at half: no tip appears for 6 s (>= 9 retries), none touches a tile or the strip.
	app.call_arming_from_map(func() -> void: app.arm_tool("poi"))
	await _frames(24)
	app.set_phone_detent("half")
	await _frames(30)
	var seen_tip := false
	var overlap: Array = []
	var off: Array = []
	var pal := app.find_child("PinPalette", true, false)
	var strip := app.find_child("ToolStrip", true, false)
	_ok("L3: palette and strip exist", pal != null and strip != null)
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < 7000:
		for r in _tip_rects():
			seen_tip = true
			if not vis.encloses(r):
				off.append(r)
			overlap += _hits(r, pal.find_children("Pin_*", "Button", true, false))
			overlap += _hits(r, strip.find_children("*", "Control", true, false))
		await _frames(10)
	await _shot("half_armed.png")
	_ok("L3: no tip is visible with the sheet at half and Pins armed", not seen_tip)
	_ok("L3: no tip overlaps a palette tile or a strip control", overlap.is_empty(), str(overlap.slice(0, 5)))
	_ok("L3: every tip drawn is on screen", off.is_empty(), str(off))
	var tile: Button = null
	for b in pal.find_children("Pin_*", "Button", true, false):
		var bb := b as Button
		if bb.is_visible_in_tree() and (app._phone_tool_sheet as Control).get_global_rect().encloses(bb.get_global_rect()):
			tile = bb
			break
	_ok("L3: a palette tile is on screen", tile != null)
	if tile != null:
		var hit := [false]
		tile.pressed.connect(func(): hit[0] = true)
		await _click(tile.get_global_rect().get_center())
		_ok("L3: a tap on the tile reaches it", hit[0])
	## Leg 4: sheet back at peek, tool still armed (the strip is up): still no tip, and the pin strip is clear.
	app.set_phone_detent("peek")
	await _frames(30)
	t0 = Time.get_ticks_msec()
	seen_tip = false
	while Time.get_ticks_msec() - t0 < 5000:
		if not _tip_rects().is_empty():
			seen_tip = true
			overlap += _hits(_tip_rects()[0], strip.find_children("*", "Control", true, false))
		await _frames(10)
	await _shot("peek_armed.png")
	_ok("L4: no tip over the armed strip at peek", not seen_tip, str(_tip_rects()))
	## Leg 5: disarmed at peek, the deferred tips do come (the deferral is not a loss), and are on screen.
	app.arm_tool("inspect")
	await _frames(20)
	_ok("L5: a deferred tip shows once nothing covers the way", await _wait_tip(10.0))
	var rs5 := _tip_rects()
	_ok("L5: it is on screen", not rs5.is_empty() and vis.encloses(rs5[0]))
	await _shot("deferred_tip.png")
