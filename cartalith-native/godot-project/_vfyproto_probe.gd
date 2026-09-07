extends Node
## VERIFIER probe -- drive `DccWidgets.phone_protocol_grade()` over REAL,
## SHIPPED dialog sites rather than the synthetic specimens
## `_phoneproto_probe.gd` builds. A grader that only ever sees hand-copied
## shapes has never met the code it grades.
##
##   ... _vfyproto_probe.tscn -- --force-touch --vp 1080x2400
##
## Three real sites, three different expected answers:
##   app.open_storage_locations()  fixed by 09ff8e2         -> expect ok
##   app.open_about()              fixed by 09ff8e2         -> expect ok
##   app.open_credits()        touched by NEITHER pass, and graded by
##                                 NEITHER census            -> expect fails:*
##   right_dock_ctrl._confirm_revert()  graded `none`        -> expect fails:*

var app: Node
var _tag := "vfyproto"
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

func _newest_visible_dialog(before: Array) -> Window:
	var found: Window = null
	for c in app.get_children():
		if c is Window and (c as Window).visible and not before.has(c):
			found = c as Window
	return found

func _snapshot() -> Array:
	var a: Array = []
	for c in app.get_children():
		if c is Window:
			a.append(c)
	return a

func _grade(label: String, expect_ok: bool, before: Array) -> void:
	await _frames(14)
	var d := _newest_visible_dialog(before)
	if d == null:
		_chk(false, "%s: a dialog actually opened (premise -- nothing to grade)" % label)
		return
	var g: Dictionary = DccWidgets.phone_protocol_grade(d, app)
	_log("  %-22s verdict=%-24s shaped=%s present=%s fit_ran=%s backwards=%s worst=%.0fdp('%s') off=%d n=%d"
		% [label, String(g["verdict"]), g["shaped"], g["presented"], g["fit_ran"],
			g["backwards"], g["worst"], g["worst_name"], g["offscreen"], g["tappables"]])
	_log("      size=%s csf=%.4f pos=%s borderless=%s wrap=%s"
		% [d.size, d.content_scale_factor, d.position, d.borderless, d.wrap_controls])
	if expect_ok:
		_chk(String(g["verdict"]) == "ok", "%s grades ok (a real FIXED site -- the grader must be able to PASS)" % label)
	else:
		_chk(String(g["verdict"]).begins_with("fails:"),
			"%s grades NON-CONFORMING (a real UNFIXED site -- the grader must be able to FAIL) -- got '%s'"
				% [label, String(g["verdict"])])
	d.hide()
	await _frames(4)

func _ready() -> void:
	_tag = _a("--tag", "vfyproto")
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
	if not DccTheme.is_phone():
		_log("ABORT not phone -- pass --force-touch. exit 2"); get_tree().quit(2); return
	_log("phone=true scale=%.4f vp=%s" % [app.phone_scale(), app.get_viewport_rect().size])

	var b1 := _snapshot(); app.open_storage_locations(); await _grade("storage_locations", true, b1)
	var b2 := _snapshot(); app.open_about();             await _grade("about", true, b2)
	var b3 := _snapshot(); app.open_credits();       await _grade("credits", true, b3)
	var b4 := _snapshot(); app.right_dock_ctrl._confirm_revert(1, 2); await _grade("right_dock revert", false, b4)

	_log("RESULT %s fail=%d" % [_tag, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
