extends Node
## VERIFIER probe, 2026-09-07 -- written to be run against BOTH the pre-fix
## (HEAD) tree and the working tree by the SAME code, so it references no
## symbol the fix introduces (`FOOT_NOTE_MIN_W`, the note's expand flag, the
## removed spacer). It is deliberately not `_panemin_probe.gd`: a lane's own
## harness cannot be the evidence that the lane's fix worked.
##
## Per route it prints the window's contents minimum, the two terms that feed
## it (the disabled-axis scroller's branch and the footer's branch), and every
## picker button's DRAWN rect against its VISIBLE rect -- the visible rect
## being the own rect intersected with every ancestor Control AND with the
## owning Window's client frame, since `get_global_rect()` is unclipped.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _vfy_panemin_probe.tscn -- --vp 1152x648
##
## Flags this file actually reads, grepped from the body below:
##   `--vp WxH`       OS window size in px. Default 1152x648.
##   `--tag NAME`     line prefix. Default `VFY`.
##   `--force-touch`  NOT consumed here; read by `dcc_shell.gd` out of
##                    `OS.get_cmdline_user_args()`. Accepted so the arg check
##                    below does not abort on it, and asserted against the
##                    density actually reached.
## Any other `--flag` aborts with exit 2 rather than being ignored.
##
## **Windowed on purpose.** An embedded `Window` clamps against the real
## screen; the dummy driver's screen is not this machine's.

var app: Node
var _tag := "VFY"
var _fail := 0
var _want_touch := false

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("%s %s" % [_tag, s])

func _ok(label: String, cond: bool, detail: String = "") -> void:
	_log("%s %s%s" % ["ok  " if cond else "FAIL", label,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _abort(why: String) -> void:
	_log("ABORT %s" % why)
	_log("RESULT fail=abort")
	get_tree().quit(2)

## Own rect clipped by every ancestor Control and then by the owning Window.
func _visible_rect(c: Control) -> Rect2:
	var r := c.get_global_rect()
	var n: Node = c.get_parent()
	while n != null:
		if n is Control:
			r = r.intersection((n as Control).get_global_rect())
		elif n is Window:
			r = r.intersection(Rect2(Vector2.ZERO, Vector2((n as Window).size)))
			break
		if r.size.x <= 0.0 or r.size.y <= 0.0:
			return Rect2(r.position, Vector2.ZERO)
		n = n.get_parent()
	if r.size.x <= 0.0 or r.size.y <= 0.0:
		return Rect2(r.position, Vector2.ZERO)
	return r

func _shown(c: Control) -> float:
	var own := c.get_global_rect()
	if own.size.x <= 0.0 or own.size.y <= 0.0:
		return 0.0
	var v := _visible_rect(c)
	return (v.size.x * v.size.y) / (own.size.x * own.size.y)

## Every Button under `root` whose text looks like a file/folder picker.
## `DccWidgets.action()` UPPER-CASES its label, so this matches case-insensitively
## and on BOTH verbs -- `export_maps` says "Choose...", not "Browse...".
func _pickers(root: Node, out: Array) -> Array:
	if root is Button:
		var t := String((root as Button).text).to_lower()
		if t.begins_with("browse") or t.begins_with("choose"):
			out.append(root)
	for ch in root.get_children():
		_pickers(ch, out)
	return out

## The disabled-axis ScrollContainer that holds the body.
func _body_scroll(dm: Node) -> ScrollContainer:
	var n: Node = dm._pane_body
	while n != null:
		if n is ScrollContainer:
			return n as ScrollContainer
		n = n.get_parent()
	return null

func _reject_unknown_args() -> bool:
	var known := ["--vp", "--tag", "--force-touch"]
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a.begins_with("--"):
			if not known.has(a):
				_log("ABORT unknown flag %s" % a)
				return false
			if a == "--vp" or a == "--tag":
				i += 1
		i += 1
	return true

func _ready() -> void:
	if not _reject_unknown_args():
		_log("RESULT fail=abort")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	var vp := Vector2i(1152, 648)
	for i in args.size():
		var a := String(args[i])
		if a == "--vp" and i + 1 < args.size():
			var wh := String(args[i + 1]).split("x")
			vp = Vector2i(int(wh[0]), int(wh[1]))
		elif a == "--tag" and i + 1 < args.size():
			_tag = String(args[i + 1])
		elif a == "--force-touch":
			_want_touch = true
	DisplayServer.window_set_size(vp)
	await _frames(6)

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(3)

	## PREMISE CHECK. A probe that cannot answer must not report a pass.
	_log("screen=%s window=%s viewport=%s" % [DisplayServer.screen_get_size(),
		DisplayServer.window_get_size(), app.get_viewport_rect().size])
	_log("density touch=%s phone=%s tablet=%s laptop=%s csf=%.4f" % [
		DccTheme.is_touch(), DccTheme.is_phone(), DccTheme.is_tablet(),
		DccTheme.is_laptop(), get_window().content_scale_factor])
	if _want_touch and not DccTheme.is_touch():
		_abort("--force-touch passed but DccTheme.is_touch() is false")
		return
	if not _want_touch and DccTheme.is_touch():
		_abort("touch density reached without --force-touch")
		return

	var dm = app.data_manager_window
	if dm == null:
		_abort("app.data_manager_window is null")
		return

	_log("dm.min_size=%s  W_RAIL=%d PANE_PAD_X=%d  routes=%d"
		% [dm.min_size, dm.W_RAIL, dm.PANE_PAD_X, dm.ROUTES.size()])

	var worst := 0.0
	var worst_route := ""
	for r in dm.ROUTES:
		var id := String(r["id"])
		dm.open_route(id)
		await _frames(8)
		var cmin: Vector2 = dm.get_contents_minimum_size()
		var scr := _body_scroll(dm)
		var foot: Control = dm._pane_footer
		var body: Control = dm._pane_body
		var fmin := foot.get_combined_minimum_size().x
		var bmin := body.get_combined_minimum_size().x
		var smin := -1.0
		if scr != null:
			smin = scr.get_combined_minimum_size().x
		if cmin.x > worst:
			worst = cmin.x
			worst_route = id
		_log("route %-18s contents_min=%7.1f  win.size=%s  scroll_min=%6.1f  body_min=%6.1f  footer_min=%6.1f  foot_kids=%d"
			% [id, cmin.x, dm.size, smin, bmin, fmin, foot.get_child_count()])
		var pk := []
		_pickers(dm, pk)
		for b in pk:
			var own: Rect2 = (b as Control).get_global_rect()
			var vis := _visible_rect(b as Control)
			var sh := _shown(b as Control)
			_log("     picker '%s' own=[%.0f..%.0f] vis=[%.0f..%.0f] shown=%.3f"
				% [String((b as Button).text), own.position.x,
					own.position.x + own.size.x, vis.position.x,
					vis.position.x + vis.size.x, sh])
			_ok("%s: picker '%s' fully drawn" % [id, String((b as Button).text)],
				sh > 0.999,
				"shown=%.3f own right=%.0f win w=%d" % [sh,
					own.position.x + own.size.x, dm.size.x])
		for ch in foot.get_children():
			if ch is Label:
				var l := ch as Label
				var lv := _visible_rect(l)
				_log("     note min=%.0f own=%.0fx%.0f vis=%.0fx%.0f shown=%.3f expand=%d overrun=%d clip=%d len=%d"
					% [l.get_combined_minimum_size().x, l.get_global_rect().size.x,
						l.get_global_rect().size.y, lv.size.x, lv.size.y, _shown(l),
						l.size_flags_horizontal, l.text_overrun_behavior,
						int(l.clip_text), l.text.length()])
				break
		var promised := float(dm.min_size.x)
		if dm.min_size.x <= 0:
			promised = float(dm.size.x)
		_ok("%s: contents_min.x <= promised width" % id, cmin.x <= promised + 0.5,
			"contents_min=%.0f promised=%.0f" % [cmin.x, promised])

	_log("WORST contents_min=%.1f at route %s" % [worst, worst_route])
	_log("RESULT fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
