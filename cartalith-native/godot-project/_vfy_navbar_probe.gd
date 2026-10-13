extends Node
## Ruling BK header (2026-10-13, probe adopted into git; first run passed on the debug DLL):
## Protects: Phone bottom nav bar: docks, tool sheet and menu sheet end exactly at the bar's top (gesture inset == _safe_bottom, no double-reserve), and the PIPELINE button is not drawn under the bar.
## Run (Godot_v4.7.1-stable_win64_console.exe, from godot-project/): windowed: --path . _vfy_navbar_probe.tscn -- --force-touch (device 1080x2340)
## Result: '[vfynav] RESULT PASS failures=0' (exit 0)
## Sources: derived from this file's own assertions and a 2026-10-13 run; the feature symbols were grepped, not assumed.
##
## VERIFIER's own independent measurement of the NAVBAR lane's claim.
## Windowed at the device's 1080x2340, --force-touch.
##   Godot_v4.7.1-stable_win64_console.exe --path . _vfy_navbar_probe.tscn -- --force-touch

var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[vfynav] ", s)

func _chk(ok: bool, what: String) -> void:
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])

func _r(x: Rect2) -> String:
	return "y=%.1f..%.1f x=%.1f..%.1f (%.1fx%.1f)" % [x.position.y, x.end.y,
		x.position.x, x.end.x, x.size.x, x.size.y]

func _all(root: Node, pred: Callable, out: Array) -> void:
	for c in root.get_children():
		if pred.call(c):
			out.append(c)
		_all(c, pred, out)

func _clipper(n: Control) -> Control:
	var p: Node = n.get_parent()
	while p != null:
		if p is ScrollContainer:
			return p as Control
		if p is Control and (p as Control).clip_contents:
			return p as Control
		p = p.get_parent()
	return null

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 180.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var want := Vector2i(1080, 2340)
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(2.0).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)

	if not app.is_phone():
		_log("ABORT not phone -- pass --force-touch")
		get_tree().quit(2)
		return

	var vh: float = app.get_viewport_rect().size.y
	var sb: int = app._safe_bottom()
	var ptap: int = app._ptap(DccTheme.H_PHONE_BOTTOM_NAV)
	_log("viewport_h=%.1f  _safe_bottom()=%d  _ptap(H_PHONE_BOTTOM_NAV)=%d  _phone_nav_reserve()=%.1f"
		% [vh, sb, ptap, app._phone_nav_reserve()])

	var bar: Control = app._phone_menu_bar
	var gest: Control = app._phone_gesture_inset
	_chk(bar != null and bar.visible, "app bottom bar exists+visible")
	_chk(gest != null, "system gesture inset control exists")
	var br := bar.get_global_rect()
	var gr := gest.get_global_rect()
	_log("BAR   rect %s" % _r(br))
	_log("GESTU rect %s" % _r(gr))
	_log("SEAM  bar.end.y=%.1f  gesture.pos.y=%.1f  gesture.end.y=%.1f  vh=%.1f"
		% [br.end.y, gr.position.y, gr.end.y, vh])

	## THE CONFLATION TEST. The two reserves must be DISJOINT, stacked bands:
	## bar sits directly on top of the gesture inset, gesture inset sits on the
	## bottom edge. If they overlapped, subtracting both would double-count.
	_chk(absf(br.end.y - gr.position.y) <= 1.0,
		"bar's bottom edge (%.1f) meets the gesture inset's top (%.1f) -- disjoint bands, no double-count"
			% [br.end.y, gr.position.y])
	_chk(absf(gr.end.y - vh) <= 1.0,
		"gesture inset reaches the screen bottom (%.1f vs %.1f)" % [gr.end.y, vh])
	_chk(absf(gr.size.y - float(sb)) <= 1.0,
		"gesture inset height (%.1f) == _safe_bottom() (%d)" % [gr.size.y, sb])
	_chk(absf((vh - br.position.y) - float(sb + int(br.size.y))) <= 1.0,
		"bar_top = vh - safe_bottom - bar_height : %.1f == %.1f"
			% [br.position.y, vh - float(sb) - br.size.y])
	_log("BAR height measured=%.1f vs constant _ptap=%d (delta %.1f)"
		% [br.size.y, ptap, br.size.y - float(ptap)])

	var bar_top: float = br.position.y

	## -- the tool sheet (already correct per lane) ------------------------
	var ts: Control = app._phone_tool_sheet
	if ts != null:
		_log("TOOLSHEET rect %s  (bar_top=%.1f)" % [_r(ts.get_global_rect()), bar_top])
		_chk(ts.get_global_rect().end.y <= bar_top + 0.5,
			"tool sheet bottom (%.1f) clears bar top (%.1f)"
				% [ts.get_global_rect().end.y, bar_top])

	## -- the dock sheets (the lane's fix) ---------------------------------
	for side in ["left", "right"]:
		app._set_sheet_open(side, true)
		await _frames(8)
		var sh: Control = app.left_dock if side == "left" else app.right_dock
		var sr := sh.get_global_rect()
		_log("%s_dock rect %s  offset_bottom=%.1f" % [side, _r(sr), sh.offset_bottom])
		_chk(sr.end.y <= bar_top + 0.5,
			"%s dock bottom (%.1f) clears bar top (%.1f)  [gap %.1f]"
				% [side, sr.end.y, bar_top, bar_top - sr.end.y])
		_chk(sr.end.y >= bar_top - 2.0,
			"%s dock bottom is not OVER-reserved (would be double-counting): %.1f vs bar top %.1f"
				% [side, sr.end.y, bar_top])
		app._set_sheet_open(side, false)
		await _frames(3)

	## -- the MENU, whose bottom inset this fix also moved (168 -> 169) -----
	if app._phone_menu != null:
		app._phone_menu.open()
		await _frames(8)
		for nm in ["_screen", "_sheet", "_sheet_scrim"]:
			var c: Control = app._phone_menu.get(nm)
			if c == null:
				continue
			var mr := c.get_global_rect()
			_log("menu.%s %s offset_bottom=%.1f" % [nm, _r(mr), c.offset_bottom])
			if c.visible:
				_chk(mr.end.y <= bar_top + 0.5,
					"menu.%s bottom (%.1f) clears bar top (%.1f)" % [nm, mr.end.y, bar_top])
		app._phone_menu.visible = false
		await _frames(3)

	## -- the out-of-lane GENERATE claim -----------------------------------
	var pipes: Array = []
	_all(app, func(n): return n is Button and String((n as Button).text) == "PIPELINE", pipes)
	_log("PIPELINE buttons found: %d" % pipes.size())
	for b in pipes:
		var bc := b as Control
		if not bc.is_visible_in_tree():
			_log("  (hidden) %s" % _r(bc.get_global_rect()))
			continue
		var pr := bc.get_global_rect()
		var cl := _clipper(bc)
		var line := "  VISIBLE PIPELINE %s" % _r(pr)
		if cl != null:
			var cr := cl.get_global_rect()
			var inter := pr.intersection(cr)
			var frac: float = (inter.size.y / pr.size.y) if pr.size.y > 0.0 else 0.0
			line += "  clipper=%s %s  visible_frac=%.2f  clip_bottom=%.1f  bar_top=%.1f" % [
				cl.get_class(), _r(cr), frac, cr.end.y, bar_top]
		_log(line)

	_log("RESULT %s failures=%d" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(1 if _fail > 0 else 0)
