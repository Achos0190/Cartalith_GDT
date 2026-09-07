extends Node
## VERIFIER probe, 2026-09-07 -- independent re-derivation of batch `09ff8e2`'s
## footer claims. Written from scratch: its own visible-rect math, its own
## app-viewport mapping, its own assertions. It deliberately does NOT import
## `_panemin_probe.gd`'s helpers -- running a lane's own probe is not
## independent verification.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _vfyjob1_probe.tscn -- --vp 1152x648
##
## Flags: --vp WxH, --route ID (default export_world), --minx N (override the
## window's declared min_size.x BEFORE it pops -- the counterfactual), --tag.
##
## WINDOWED ONLY. An embedded `Window` clamps against the real screen and the
## dummy driver's screen is not this machine's; there is also nothing to draw.

var app: Node
var _tag := "vfy1"

func _a(name: String, d: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	return String(args[i + 1]) if (i >= 0 and i + 1 < args.size()) else d

func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## My own version, derived independently: a control is seen only where its own
## rect survives intersection with every ancestor Control AND the owning
## Window's client rect (`get_visible_rect()`, the engine's answer in the space
## the children are laid out in -- `Window.size` is physical and lies whenever
## `content_scale_factor != 1`).
func _seen(c: Control) -> Rect2:
	var r := c.get_global_rect()
	var n: Node = c.get_parent()
	while n != null:
		if n is Control:
			r = r.intersection((n as Control).get_global_rect())
		elif n is Window:
			r = r.intersection((n as Window).get_visible_rect())
			break
		if r.size.x <= 0.0 or r.size.y <= 0.0:
			return Rect2(0, 0, 0, 0)
		n = n.get_parent()
	return r

func _ratio(c: Control) -> float:
	var o := c.get_global_rect()
	if o.size.x <= 0.0 or o.size.y <= 0.0:
		return 0.0
	var s := _seen(c)
	return (s.size.x * s.size.y) / (o.size.x * o.size.y)

func _buttons(n: Node, out: Array) -> Array:
	if n is Button:
		var t := String((n as Button).text)
		if t.begins_with("Browse") or t.begins_with("Choose"):
			out.append(n)
	for ch in n.get_children():
		_buttons(ch, out)
	return out

func _ready() -> void:
	_tag = _a("--tag", "vfy1")
	if DisplayServer.get_name() == "headless":
		_log("ABORT headless -- run windowed. exit 2")
		get_tree().quit(2)
		return
	var p: PackedStringArray = _a("--vp", "1152x648").split("x")
	DisplayServer.window_set_size(Vector2i(int(p[0]), int(p[1])))
	await _frames(6)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	var vp: Vector2 = app.get_viewport_rect().size
	var w: Node = app.data_manager_window
	var win := w as Window
	_log("viewport=%s  declared min_size=%s  phone=%s tablet=%s"
		% [vp, win.min_size, DccTheme.is_phone(), DccTheme.is_tablet()])

	## The counterfactual lever: the batch's claim is that raising the DECLARED
	## minimum is not a fix, because `_popup_full()` pops at
	## `maxi(viewport.x, min_size.x)`. Set it before the window pops.
	var minx := int(_a("--minx", "0"))
	if minx > 0:
		win.min_size = Vector2i(minx, win.min_size.y)
		_log("COUNTERFACTUAL min_size.x forced to %d" % minx)

	## The phone entry screen is a separate full-screen Control.
	if app.get("phone_project_picker") != null:
		app.phone_project_picker.hide()
	await _frames(4)
	app.open_data_manager()
	await _frames(8)

	## `--all`: every route, footer/body/contents minima against the room the
	## pane actually has. `PANE_PAD_X` twice, plus `W_RAIL` only where the rail
	## sits BESIDE the pane (it stacks above on a handset).
	if "--all" in OS.get_cmdline_user_args():
		var chrome: float = float(w.PANE_PAD_X) * 2.0
		if not DccTheme.is_phone():
			chrome += float(w.W_RAIL)
		var room: float = win.get_visible_rect().size.x - chrome
		_log("ALL ROUTES  client_w=%.0f chrome=%.0f room=%.0f phone=%s footer_class=%s"
			% [win.get_visible_rect().size.x, chrome, room, DccTheme.is_phone(),
				w._pane_footer.get_class()])
		var worst_f := 0.0
		var worst_r := ""
		var over := 0
		for rr in w.ROUTES:
			var rid := String(rr["id"])
			w.open_route(rid)
			await _frames(8)
			var fo: Control = w._pane_footer
			var bo: Control = w._pane_body
			var fm: float = fo.get_combined_minimum_size().x
			var sum := 0.0
			for ch in fo.get_children():
				if ch is Control:
					sum += (ch as Control).get_combined_minimum_size().x
			if fm > worst_f:
				worst_f = fm
				worst_r = rid
			if fm > room:
				over += 1
			_log("  %-18s footer_min=%7.1f (child sum %7.1f)  body_min=%7.1f  contents=%7.1f  %s"
				% [rid, fm, sum, bo.get_combined_minimum_size().x,
					w.get_contents_minimum_size().x, "OVER ROOM" if fm > room else ""])
		_log("WORST footer_min = %.1f @%s ; routes over room = %d ; room = %.0f"
			% [worst_f, worst_r, over, room])
		get_tree().quit(0)
		return

	var route := _a("--route", "export_world")
	w.open_route(route)
	await _frames(10)

	var cmin: Vector2 = w.get_contents_minimum_size()
	var client := win.get_visible_rect()
	var body: Control = w._pane_body
	var foot: Control = w._pane_footer
	## The scroll is the body's ancestor two levels up (body_pad -> scroll).
	var scroll: Node = body.get_parent().get_parent()
	_log("route=%s  win.size=%s win.position=%s  client=%s" % [route, win.size, win.position, client.size])
	_log("  contents_min.x = %.1f   (declared min_size.x = %d, client_w = %.0f)"
		% [cmin.x, win.min_size.x, client.size.x])
	_log("  pane_body min.x   = %.1f   [%s]" % [body.get_combined_minimum_size().x, body.get_class()])
	_log("  scroll   min.x    = %.1f   [%s  hmode=%d]"
		% [(scroll as Control).get_combined_minimum_size().x, scroll.get_class(),
			(scroll as ScrollContainer).horizontal_scroll_mode if scroll is ScrollContainer else -1])
	_log("  pane_footer min.x = %.1f   [%s]  children=%d"
		% [foot.get_combined_minimum_size().x, foot.get_class(), foot.get_child_count()])
	var terms := PackedStringArray()
	var total := 0.0
	for ch in foot.get_children():
		if ch is Control:
			var m: float = (ch as Control).get_combined_minimum_size().x
			total += m
			terms.append("%s:%.0f" % [(ch as Button).text if ch is Button else ch.get_class(), m])
	_log("  footer child mins: %s   sum=%.0f" % [" + ".join(terms), total])
	## DRAWN width of each footer child. The floor's whole purpose is that an
	## ellipsised Label reports a minimum of 1 and then GETS 1 -- so the
	## property is not the claim; the painted box is.
	for ch in foot.get_children():
		if ch is Control:
			var c := ch as Control
			var nm: String = (c as Button).text if c is Button else ("NOTE:" + String((c as Label).text).substr(0, 28) if c is Label else c.get_class())
			_log("    drawn %-34s x=[%.0f..%.0f] w=%.0f  min=%.0f  overrun=%s"
				% [nm, c.get_global_rect().position.x, c.get_global_rect().end.x,
					c.size.x, c.get_combined_minimum_size().x,
					str((c as Label).text_overrun_behavior) if c is Label else "-"])

	## Is the footer a SIBLING of the scroll, or inside it? The batch's claimed
	## cause. Answered by walking up, not by reading the source.
	var anc: Node = foot
	var chain := PackedStringArray()
	while anc != null and not (anc is Window):
		chain.append(anc.get_class())
		anc = anc.get_parent()
	_log("  footer ancestry: %s" % " < ".join(chain))
	var inside_scroll := false
	var q: Node = foot
	while q != null:
		if q == scroll:
			inside_scroll = true
			break
		q = q.get_parent()
	_log("  footer is inside the ScrollContainer: %s" % inside_scroll)

	for b in _buttons(w, []):
		var btn := b as Button
		var own := btn.get_global_rect()
		var seen := _seen(btn)
		## Map into the APP's viewport space. An embedded sub-window's
		## `position` is in its parent viewport's coordinates, and on desktop
		## `content_scale_factor` is 1, so this is a straight add.
		var app_x0 := win.position.x + own.position.x
		var app_x1 := win.position.x + own.end.x
		_log("  PICKER %-10s own=[%.0f..%.0f] seen=[%.0f..%.0f] shown=%.3f | app-space=[%.0f..%.0f] vs viewport %.0f  past=%+.0f"
			% [btn.text, own.position.x, own.end.x, seen.position.x, seen.end.x,
				_ratio(btn), app_x0, app_x1, vp.x, app_x1 - vp.x])
	_log("DONE")
	get_tree().quit(0)
