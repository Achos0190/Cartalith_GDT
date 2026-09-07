extends Node

## Verifier probe for the 2026-09-07 batch. Asserts four things the two lanes
## claimed, each at the symbol rather than at a restated number:
##
##   A  the phone tool sheet's grab row is >= DccTheme.PHONE_TAP_MIN dp, and the
##      region that RAISES the sheet equals the region that is DRAWN -- the
##      equality form, not `last_raised - first_raised`, which measures the
##      ladder rather than the target.
##   B  `_phone_sheet_grab` is in none of the three populations the brief named
##      as arbitration suspects (HSlider / BaseButton / LineEdit-SpinBox), and
##      carries exactly one `gui_input` listener.
##   C  after `open_journey_planner()` the planner's whole control column is
##      visible IN TREE -- `is_visible_in_tree()`, not `.visible`, which is the
##      assertion every passing desktop probe missed.
##   D  every Preferences submenu row either carries its group's sole checked
##      value after the separator, or carries no readout at all; and firing
##      `about_to_popup` three times does not append three times.

var app: Node
var _vp: SubViewport
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[vfy] %s" % s)

func _check(ok: bool, what: String) -> void:
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _dp(px: float) -> float:
	return px / float(app.phone_scale())

func _sheet_h() -> float:
	return (app._phone_tool_sheet as Control).size.y

func _press(at: Vector2, down: bool) -> void:
	var e := InputEventMouseButton.new()
	e.button_index = MOUSE_BUTTON_LEFT
	e.pressed = down
	e.position = at
	e.global_position = at
	_vp.push_input(e, true)

func _motion(at: Vector2, rel: Vector2) -> void:
	var e := InputEventMouseMotion.new()
	e.position = at
	e.global_position = at
	e.relative = rel
	e.button_mask = MOUSE_BUTTON_MASK_LEFT
	_vp.push_input(e, true)

func _drag_raises(from: Vector2, dy: float) -> bool:
	var h0 := _sheet_h()
	_press(from, true)
	await _frames(1)
	var mid := from + Vector2(0.0, dy * 0.5)
	_motion(mid, Vector2(0.0, dy * 0.5))
	await _frames(2)
	var to := from + Vector2(0.0, dy)
	_motion(to, Vector2(0.0, dy * 0.5))
	await _frames(2)
	_press(to, false)
	await _frames(1)
	await get_tree().create_timer(0.40).timeout
	await _frames(2)
	var raised: bool = _sheet_h() > h0 + 2.0
	if raised:
		app.set_phone_detent("peek")
		await get_tree().create_timer(0.35).timeout
		await _frames(2)
	return raised

func _collect_menu_buttons(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children():
		_collect_menu_buttons(c, out)

func _collect_popups(n: Node, out: Array) -> void:
	if n is PopupMenu:
		out.append(n)
	for c in n.get_children():
		_collect_popups(c, out)

func _ready() -> void:
	var parts: PackedStringArray = _arg("--vp", "1080x2340").split("x")
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.8).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)

	if not app.is_phone():
		_log("ABORT not a phone shell -- pass --force-touch")
		get_tree().quit(2)
		return

	# ---- B: what the grab node actually is ---------------------------------
	var g: Control = app._phone_sheet_grab
	_log("B  arbiter elimination")
	_check(g != null, "grab node exists")
	_check(g.get_class() == "Control", "class is bare Control, got %s" % g.get_class())
	_check(g.get_script() == null, "script is null")
	_check(g.mouse_filter == Control.MOUSE_FILTER_STOP, "filter STOP")
	_check(not (g is Range), "not a Range/HSlider -- PgSlider cannot reach it")
	_check(not (g is BaseButton), "not a BaseButton -- touch_release_button cannot")
	_check(not (g is LineEdit) and not (g is SpinBox), "not LineEdit/SpinBox -- PgField cannot")
	_check(g.gui_input.get_connections().size() == 1,
		"exactly 1 gui_input listener, got %d" % g.gui_input.get_connections().size())

	# ---- A: drawn size, and hit-region == drawn-region ---------------------
	app.set_phone_detent("peek")
	await get_tree().create_timer(0.4).timeout
	await _frames(3)
	var r: Rect2 = g.get_global_rect()
	var h_dp := _dp(r.size.y)
	_log("A  grab row %.2f px = %.2f dp (floor %d dp, scale %.4f)"
		% [r.size.y, h_dp, DccTheme.PHONE_TAP_MIN, app.phone_scale()])
	_check(h_dp >= float(DccTheme.PHONE_TAP_MIN) - 0.25,
		"grab row >= %d dp floor" % DccTheme.PHONE_TAP_MIN)
	## Two-sided, and pinned to figures this file does not own: the canvas
	## authored 20 dp, the floor is 44 dp, so a correct row is EXACTLY the
	## floor. `_pscale(20)` fails the lower bound, `_ptap(60)` fails this one.
	_check(h_dp <= float(DccTheme.PHONE_TAP_MIN) + 0.25,
		"grab row is exactly the floor, not padded past it (%.2f dp)" % h_dp)
	var cx := r.position.x + r.size.x * 0.5
	var inside := [r.position.y + 2.0, r.position.y + r.size.y * 0.25,
		r.position.y + r.size.y * 0.5, r.position.y + r.size.y * 0.75,
		r.position.y + r.size.y - 3.0]
	var outside := [r.position.y - 6.0, r.position.y - 20.0,
		r.position.y + r.size.y + 6.0, r.position.y + r.size.y + 20.0]
	var in_ok := 0
	for y in inside:
		if await _drag_raises(Vector2(cx, y), -520.0):
			in_ok += 1
		else:
			_log("     inside rung y=%.1f did NOT raise" % y)
	var out_bad := 0
	for y in outside:
		if await _drag_raises(Vector2(cx, y), -520.0):
			out_bad += 1
			_log("     outside rung y=%.1f DID raise" % y)
	_check(in_ok == inside.size(), "all %d rungs inside the drawn rect raise (%d)"
		% [inside.size(), in_ok])
	_check(out_bad == 0, "no rung outside the drawn rect raises (%d did)" % out_bad)

	# ---- C: the planner's controls are visible IN TREE ---------------------
	_log("C  planner reachability")
	app.open_journey_planner()
	await get_tree().create_timer(0.9).timeout
	await _frames(6)
	var jp: Node = app.journey_planner_view
	var lp: Control = jp._left_panel
	_check(lp.visible, "_left_panel.visible -- what every old probe asserted")
	_check(lp.is_visible_in_tree(), "_left_panel.is_visible_in_tree() -- THE assertion")
	_check(app.left_dock.visible, "left_dock.visible")
	_check(lp.get_global_rect().size.x > 100.0 and lp.get_global_rect().size.y > 100.0,
		"_left_panel laid out %s" % str(lp.get_global_rect().size))
	app.open_journey_planner()
	await get_tree().create_timer(0.7).timeout
	await _frames(6)
	var civ: Control = app._workspace_panels.get("civilization")
	_check(lp.is_visible_in_tree(), "re-open: planner still visible in tree")
	_check(civ == null or not civ.is_visible_in_tree(),
		"re-open: civilization panel not put back over it")
	app.arm_tool("pan")
	await get_tree().create_timer(0.7).timeout
	await _frames(6)
	_check(not app.left_dock.visible, "leaving the planner closes the sheet it opened")

	# ---- D: preferences readouts -------------------------------------------
	_log("D  preferences readouts")
	var pref: PopupMenu = null
	var btns: Array = []
	_collect_menu_buttons(app, btns)
	for mb in btns:
		if String((mb as MenuButton).text) == "Preferences":
			pref = (mb as MenuButton).get_popup()
			break
	if pref == null:
		var found: Array = []
		_collect_popups(app, found)
		for p in found:
			if String(p.name).to_lower() == "preferences":
				pref = p
				break
	if pref == null:
		_check(false, "found a Preferences PopupMenu")
	else:
		pref.about_to_popup.emit()
		await _frames(2)
		var first := {}
		var groups := 0
		var stamped := 0
		var bad := 0
		for i in pref.item_count:
			var sn := pref.get_item_submenu(i)
			if sn == "":
				continue
			var sub := pref.get_node_or_null(NodePath(sn)) as PopupMenu
			if sub == null:
				continue
			groups += 1
			var txt := pref.get_item_text(i)
			first[sn] = txt
			var n := 0
			var only := ""
			for j in sub.item_count:
				if sub.is_item_separator(j) or not sub.is_item_checked(j):
					continue
				n += 1
				only = sub.get_item_text(j)
			if n == 1:
				if txt.ends_with("   " + only) and not txt.ends_with("    " + only):
					stamped += 1
				else:
					bad += 1
					_log("     '%s' one check '%s' but row reads '%s'" % [sn, only, txt])
			elif txt.find("   ") >= 0:
				bad += 1
				_log("     '%s' has %d checks yet reads '%s'" % [sn, n, txt])
		_log("     %d value groups, %d carry a readout" % [groups, stamped])
		_check(bad == 0, "no group fabricates or drops a value (%d bad)" % bad)
		_check(stamped > 0, "at least one group carries a readout")
		pref.about_to_popup.emit()
		await _frames(2)
		pref.about_to_popup.emit()
		await _frames(2)
		var drift := 0
		for i in pref.item_count:
			var sn2 := pref.get_item_submenu(i)
			if sn2 == "" or not first.has(sn2):
				continue
			if pref.get_item_text(i) != first[sn2]:
				drift += 1
				_log("     DRIFT '%s': '%s' -> '%s'" % [sn2, first[sn2], pref.get_item_text(i)])
		_check(drift == 0, "three about_to_popup fires leave every row identical")
		## `n == 1`, not `n >= 1`: no live group carries two checks, so a
		## tree-only probe cannot tell the two apart. Fabricate one.
		var m = app.menus
		var fx := PopupMenu.new()
		fx.add_radio_check_item("Alpha", 0)
		fx.add_radio_check_item("Beta", 1)
		add_child(fx)
		_check(String(m._sole_checked_text(fx)) == "", "zero checks -> no value")
		fx.set_item_checked(0, true)
		_check(String(m._sole_checked_text(fx)) == "Alpha", "one check -> that value")
		fx.set_item_checked(1, true)
		_check(String(m._sole_checked_text(fx)) == "",
			"TWO checks -> no value (kills n >= 1), got '%s'" % m._sole_checked_text(fx))
		fx.queue_free()

	_log("RESULT %s (%d failures)" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(0 if _fail == 0 else 1)
