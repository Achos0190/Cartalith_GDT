extends Node
## VERIFIER probe -- `app.gd::open_storage_locations()` on a handset, measured
## independently of `_dlgscale_probe.gd`.
##
##   ... _vfydlg_probe.tscn -- --force-touch --vp 1080x2400
##
## Claim under test (batch 09ff8e2): before the fix this dialog was a 680x340
## window with `content_scale_factor` 1.0000 and a `Browse…` 26 PHYSICAL px tall
## against a 115 px floor (44 dp x 2.6214); after it, every tappable is at or
## above 44 dp and inside the window's own client rect.
##
## Everything is measured in BOTH spaces on purpose: `Window.size` is physical,
## every child rect is in the content-scaled space, and conflating them is what
## made the pre-fix number look survivable.

var app: Node
var _tag := "vfydlg"
var _fail := 0

func _a(n: String, d: String) -> String:
	var g := OS.get_cmdline_user_args()
	var i := g.find(n)
	return String(g[i + 1]) if (i >= 0 and i + 1 < g.size()) else d

func _log(s: String) -> void: print("[%s] %s" % [_tag, s])
func _chk(ok: bool, w: String) -> void:
	if not ok: _fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", w])
func _frames(n: int) -> void:
	for i in n: await get_tree().process_frame

func _tappables(n: Node, out: Array) -> Array:
	if n is Button or n is LineEdit or n is OptionButton or n is CheckBox:
		var c := n as Control
		if c.visible and c.is_visible_in_tree() and c.size.x > 0.0 and c.size.y > 0.0:
			out.append(c)
	for ch in n.get_children():
		_tappables(ch, out)
	return out

func _ready() -> void:
	_tag = _a("--tag", "vfydlg")
	if DisplayServer.get_name() == "headless":
		_log("ABORT headless. exit 2"); get_tree().quit(2); return
	var p: PackedStringArray = _a("--vp", "1080x2400").split("x")
	DisplayServer.window_set_size(Vector2i(int(p[0]), int(p[1])))
	await _frames(6)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	if app.open_project_dialog != null: app.open_project_dialog.hide()
	await _frames(4)
	var phone := DccTheme.is_phone()
	var scale: float = app.phone_scale() if phone else 1.0
	_log("phone=%s scale=%.4f  vp=%s" % [phone, scale, app.get_viewport_rect().size])
	if not phone:
		_log("ABORT not in phone mode -- pass --force-touch. exit 2"); get_tree().quit(2); return

	app.open_storage_locations()
	await _frames(12)
	var d: Window = null
	for c in app.get_children():
		if c is AcceptDialog and (c as Window).visible and String((c as Window).title) == "Storage locations":
			d = c as Window
	if d == null:
		for c in app.get_children():
			if c is AcceptDialog and (c as Window).visible:
				d = c as Window
	if d == null:
		_log("ABORT could not find the dialog. exit 2"); get_tree().quit(2); return

	var client := d.get_visible_rect()
	var screen := Vector2(DisplayServer.window_get_size())
	_log("dialog: size=%s (PHYSICAL) client=%s (LAID-OUT) csf=%.4f borderless=%s wrap=%s pos=%s"
		% [d.size, client.size, d.content_scale_factor, d.borderless, d.wrap_controls, d.position])
	## The tap floor is 44 DP. In the laid-out space that is 44 units when the
	## window carries the scale, and 44*scale PHYSICAL px either way.
	var floor_phys := 44.0 * scale
	var carries := absf(d.content_scale_factor - scale) < 0.01
	_chk(carries, "the window carries the phone content scale (csf=%.4f vs phone_scale %.4f)"
		% [d.content_scale_factor, scale])
	_chk(absf(d.size.x - screen.x) < 1.0, "the window is the screen's width (%d vs %.0f physical)" % [d.size.x, screen.x])

	var worst := 1e9
	var worst_n := ""
	var off := 0
	for cn in _tappables(d, []):
		var c := cn as Control
		var h: float = c.size.y
		var phys: float = h * (d.content_scale_factor if d.content_scale_factor > 0.0 else 1.0)
		var nm: String = (c as Button).text if c is Button else c.get_class()
		if h < worst:
			worst = h
			worst_n = nm
		var r: Rect2 = c.get_global_rect()
		if r.position.x < -0.5 or r.end.x > client.size.x + 0.5 or r.position.y < -0.5 or r.end.y > client.size.y + 0.5:
			off += 1
			_log("    OFF-CLIENT %-12s rect=[%.0f..%.0f x %.0f..%.0f] vs client %s"
				% [nm, r.position.x, r.end.x, r.position.y, r.end.y, client.size])
		if nm.begins_with("Browse"):
			_log("    Browse… h=%.1f dp  = %.1f PHYSICAL px   (floor 44 dp = %.1f physical)"
				% [h, phys, floor_phys])
	_log("  worst tappable = %.1f dp  (%s);  off-client = %d" % [worst, worst_n, off])
	_chk(worst >= 43.5, "every visible tappable is at or above the 44 dp floor (worst %.1f dp, %s)" % [worst, worst_n])
	_chk(off == 0, "every visible tappable is inside the window's own client rect (%d outside)" % off)
	_log("RESULT %s fail=%d" % [_tag, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
