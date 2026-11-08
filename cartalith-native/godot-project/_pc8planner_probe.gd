extends Node
## PC-8 / PW-4 probe: the Journey planner in its own window on the desktop (`PC_SHELL_PROPOSAL.md` §3.7,
## milestone PC-8; `POPOUT_WINDOWS_SCOPE.md` PW-4; Ruling DY PQ4, Ruling DK), inside the real shell, with
## real mouse clicks, right clicks, wheel notches, middle drags and keys through `Input.parse_input_event()`.
##
##   desktop: godot --path . --resolution 1600x1000 _pc8planner_probe.tscn
##   tablet:  godot --path . --resolution 1600x1000 _pc8planner_probe.tscn -- --force-touch
##   phone:   godot --path . --resolution 393x852 _pc8planner_probe.tscn -- --force-touch --shots <dir>
##
## **Windowed, never `--headless`** (the busy overlay, the map's input and the captures need frames), with
## a wiped user dir (the runner sets `APPDATA` to an empty folder). Reads `--force-touch`, `--shots <dir>`
## (phone: two captures for the pixel-identity comparison) and `--mutant <name>` (prints a marker only);
## anything else exits 2. Runs on the tree BEFORE PC-8 too: there it prints the baseline lines only (the
## route results and the control inventory, see below) and exits 0, so the same file measures both trees.
##
## What it protects (PC-8's bar: "the planner is a window: `_show` no longer hides `app.viewport`; with it
## open the map draws and takes input; Start / Add / End from the selection each add one undo step; its Pop
## out button is present", and PW-4's: "main map stays live while the planner is native; planner results
## equal embedded results; the close path leaves no stale `timeline_row` content"). Desktop, one world
## (seed 4242, 256 x 192, as `_pc7windows_probe.gd`):
##   R. ROUTES: for three seeds (4242, 24601, 71077345) two routes each are committed through the engine's
##      own route calls between fixed settlement pairs and opened in the planner; each `_last_result` and
##      each request (`_plan_request()`) is hashed. The hashes are compared against the literals the SAME
##      legs printed on the pre-PC-8 tree (`BEFORE_ROUTES`), so a planned route is identical.
##   I. INVENTORY: every interactive control the planner draws (party form, centre, results, toolbar, day
##      band) is listed as `INV <class>|<text>` lines, before and after; the harness diffs the two logs.
##   E. ENTRY POINTS, each by a real input: Story ▸ Journey planner (a real click on the bar, then keys in
##      the menu), Shift+J over the map, the CIVIL rail's Journey planner node, a Saved journeys row, and
##      the map card's Plan journey from here / Journey to here (real right clicks, real row clicks). Each
##      opens the window, not exclusive, the map visible, the domain and the armed tool as they were (the
##      rail is the one that writes CIVIL), the tool-options bar and the timeline strip untouched.
##   M. MAP LIVE: with the window focused, a real left click on a settlement outside it selects it, a wheel
##      zooms the map, a middle drag pans it; the window stays open and the stop row names the selection.
##   S. STOPS: Start here, Add as stop and End here by real clicks; End commits a route and the planner
##      opens on it; each press is exactly one Ctrl+Z (route count, draft and the planner's route back) and
##      one Ctrl+Y; a reason disables each button when it cannot run.
##   C. CLOSE / REOPEN: the window's real Close hides it and nothing on the main screen changes; a reopen
##      by Shift+J shows the same route, request and position.
##   P. POP OUT: the real Pop out button makes it native (two OS windows), the request and the result are
##      unchanged, a pan and a wheel on the main map change neither, a generate mutes the pop-out, Dock
##      brings it back embedded with the same state.
##   B. BUSY: a generate with the window docked mutes it; a real click on a stop button does nothing.
## Tablet (`--force-touch`): the planner keeps the swap: no planner window exists, opening hides the map
## and arms Journey, as before PC-8 (§3.7: the tablet keeps it).
## Phone (`--shots`): two captures (the shell after a generate, then with the planner open from the PLAN
## tab's own opener) for the harness to compare with two captures of the pristine tree.
## Exit 0 pass, 1 fail, 2 bad argument, watchdog or no world, 3 a leg could not run.

## The pre-PC-8 tree's hashes of leg R, pasted from its run (see `PC_SHELL_PROPOSAL.md` §6, PC-8's note).
## Key "seed:pair" -> [result hash, request hash], from the baseline run on `de1568cc` (2026-11-08,
## `cartalith-native/docs/evidence/pc8-2026-11-08/base_desk.log`). An empty table FAILS the comparison (it
## never passes by having nothing to compare). The far pair of each seed is a BLOCKED journey (no
## calendar, `total_days` -1): a real answer of the planner, and compared like any other.
const BEFORE_ROUTES := {
	"4242:0": [3929851557, 2183103439],
	"4242:1": [3426043075, 2183103439],
	"24601:0": [76986086, 2183103439],
	"24601:1": [2688649912, 2183103439],
	"71077345:0": [2710342202, 2183103439],
	"71077345:1": [3777137194, 2183103439],
}

const SEEDS := [4242, 24601, 71077345]

var _fails := 0
var _checks := 0
var _app: Node
var _bridge
var _jp
var _shots_dir := ""
var _touch := false


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _move(at: Vector2, mask := 0) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 1)
	mm.button_mask = mask
	Input.parse_input_event(mm)
	await _settle(2)


func _button(at: Vector2, idx: MouseButton, down: bool) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = idx
	ev.pressed = down
	ev.position = at
	ev.global_position = at
	if down:
		ev.button_mask = {MOUSE_BUTTON_LEFT: MOUSE_BUTTON_MASK_LEFT, MOUSE_BUTTON_RIGHT: MOUSE_BUTTON_MASK_RIGHT,
			MOUSE_BUTTON_MIDDLE: MOUSE_BUTTON_MASK_MIDDLE}.get(idx, 0)
	Input.parse_input_event(ev)
	await _settle(2)


func _click_at(at: Vector2, idx := MOUSE_BUTTON_LEFT) -> void:
	await _move(at)
	await _button(at, idx, true)
	await _button(at, idx, false)
	await _settle(4)


func _wheel(at: Vector2) -> void:
	await _move(at)
	await _button(at, MOUSE_BUTTON_WHEEL_UP, true)
	await _button(at, MOUSE_BUTTON_WHEEL_UP, false)
	await _settle(4)


func _middle_drag(from: Vector2, by: Vector2) -> void:
	await _move(from)
	await _button(from, MOUSE_BUTTON_MIDDLE, true)
	for i in 4:
		await _move(from + by * float(i + 1) / 4.0, MOUSE_BUTTON_MASK_MIDDLE)
	await _button(from + by, MOUSE_BUTTON_MIDDLE, false)
	await _settle(4)


func _key(code: Key, shift := false, ctrl := false) -> void:
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = code
		ev.physical_keycode = code
		ev.pressed = down
		ev.shift_pressed = shift
		ev.ctrl_pressed = ctrl
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(4)


## Root-viewport position of a control drawn inside embedded window `w`.
func _in_win(w: Window, c: Control) -> Vector2:
	return Vector2(w.position) + c.get_global_rect().get_center()


## A real click on control `c` of embedded window `w`, scrolled into view first.
func _press_in(w: Window, c: Control, what: String) -> bool:
	if c == null or not c.is_visible_in_tree():
		_ok(false, "%s is drawn to be pressed" % what)
		return false
	var sc: Node = c.get_parent()
	while sc != null and not (sc is ScrollContainer) and sc != w:
		sc = sc.get_parent()
	if sc is ScrollContainer:
		(sc as ScrollContainer).ensure_control_visible(c)
		await _settle(4)
	await _click_at(_in_win(w, c))
	await _settle(4)
	return true


func _screen_of(i: int) -> Vector2:
	var s: Dictionary = _bridge.settlements()[i]
	var ov = _app.viewport.overlay
	return (ov as Control).get_global_transform() * ov._cell_to_screen(Vector2(float(s.get("x", 0)), float(s.get("y", 0))), ov._displayed_rect())


## Named settlements whose marker the overlay's own hit test gives to them, inside the map, clear of
## window `w`'s frame and of the chrome over the map (`_pc7windows_probe.gd`'s rule).
func _clickable(w: Window) -> Array:
	var ov = _app.viewport.overlay
	var map_r: Rect2 = (ov as Control).get_global_rect().grow(-40.0)
	var win_r := Rect2(Vector2(w.position) - Vector2(0, 32), Vector2(w.size) + Vector2(0, 32)).grow(40.0) \
		if (w != null and w.visible and w.is_embedded()) else Rect2()
	var rect: Rect2 = ov._displayed_rect()
	var out: Array = []
	var all: Array = _bridge.settlements()
	for i in all.size():
		if String((all[i] as Dictionary).get("name", "")).strip_edges() == "":
			continue
		var p := _screen_of(i)
		if not map_r.has_point(p) or win_r.has_point(p) or _covered(p):
			continue
		if p.y > float(get_viewport().get_visible_rect().size.y) - 140.0:
			continue
		var local: Vector2 = (ov as Control).get_global_transform().affine_inverse() * p
		var pk: Dictionary = ov._pick_mark(local, ov._interior_rect(rect), rect)
		if int(pk.get("s", -1)) == i and int(pk.get("l", -1)) == -1:
			out.append(i)
	return out


func _covered(p: Vector2) -> bool:
	if not (_app.viewport as Control).get_global_rect().grow(-30.0).has_point(p):
		return true
	for n in _app._region_nodes.values():
		if n is Control and (n as Control).is_visible_in_tree() and (n as Control).get_global_rect().grow(20.0).has_point(p):
			return true
	if _app.timeline_row != null and (_app.timeline_row as Control).is_visible_in_tree() \
			and (_app.timeline_row as Control).get_global_rect().grow(20.0).has_point(p):
		return true
	return false


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var k := 0
	while k < args.size():
		var a := args[k]
		if a == "--force-touch":
			_touch = true
		elif a == "--shots" and k + 1 < args.size():
			k += 1
			_shots_dir = args[k]
		elif a == "--mutant" and k + 1 < args.size():
			k += 1
			print("MUTANT %s" % args[k])
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
		k += 1
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(2)
		return
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_bridge = _app.bridge
	_jp = _app.journey_planner_view
	var mode := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	var built := _app.has_method("planner_in_window")
	print("== mode %s, PC-8 %s ==" % [mode, "built" if built else "NOT built (baseline run)"])
	if mode == "phone":
		await _leg_phone()
	elif mode == "tablet":
		await _generate(SEEDS[0])
		await _leg_tablet(built)
	else:
		await _leg_routes()
		await _generate(SEEDS[0])
		await _leg_inventory()
		if built:
			await _leg_desktop()
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _generate(seed: int) -> void:
	_bridge.generate({"seed": seed, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await _settle(2)
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _settle(10)
	await _settle(30)
	if not bool(_bridge.has_world) or _bridge.settlements().size() < 4:
		print("[FATAL] seed %d: no world with four settlements" % seed)
		get_tree().quit(2)


## Commits a route between settlements `a` and `b` through the engine's own three route calls.
func _commit_pair(a: int, b: int) -> int:
	var all: Array = _bridge.settlements()
	if not _bridge.route_begin("mixed"):
		return -1
	for i in [a, b]:
		_bridge.route_append_stop(float((all[i] as Dictionary)["x"]), float((all[i] as Dictionary)["y"]))
	var idx: int = _bridge.route_commit()
	if idx >= 0:
		_app.viewport.overlay.set_manual_routes(_app.viewport.manual_routes())
	return idx


## Settlement 0's nearest and farthest other settlements: two fixed pairs per seed.
func _pairs() -> Array:
	var all: Array = _bridge.settlements()
	var s0 := Vector2(float(all[0]["x"]), float(all[0]["y"]))
	var near := -1
	var far := -1
	var dn := INF
	var df := -1.0
	for i in range(1, all.size()):
		var d := Vector2(float(all[i]["x"]), float(all[i]["y"])).distance_to(s0)
		if d < dn:
			dn = d
			near = i
		if d > df:
			df = d
			far = i
	return [[0, near], [0, far]]


func _leg_routes() -> void:
	print("== R: route results ==")
	var now := {}
	for seed in SEEDS:
		await _generate(seed)
		var pairs := _pairs()
		for p in pairs.size():
			var idx := _commit_pair(int(pairs[p][0]), int(pairs[p][1]))
			if idx < 0:
				_ok(false, "R seed %d pair %d: the route commits" % [seed, p])
				continue
			_app.open_journey_planner_with_route(idx)
			await _settle(8)
			var res_h := hash(var_to_str(_jp._last_result))
			var req_h := hash(var_to_str(_jp._plan_request()))
			var ok := bool(_jp._last_result.get("ok", false))
			var days := float((_jp._last_result.get("plan", {}) as Dictionary).get("total_days", -1.0))
			print("ROUTE %d:%d %d %d ok=%s days=%.3f" % [seed, p, res_h, req_h, str(ok), days])
			_ok(ok, "R seed %d pair %d: the planner computed (ok; %.2f days, -1 = blocked)" % [seed, p, days])
			now["%d:%d" % [seed, p]] = [res_h, req_h]
			_close_planner()
			await _settle(4)
	if _app.has_method("planner_in_window"):
		_ok(not BEFORE_ROUTES.is_empty(), "R: the pre-PC-8 hashes are pasted (an empty table compares nothing)")
		for key in now:
			_ok(BEFORE_ROUTES.has(key) and BEFORE_ROUTES[key] == now[key],
				"R %s: result and request equal the pre-PC-8 tree's (%s vs %s)" % [key, str(now[key]), str(BEFORE_ROUTES.get(key, "missing"))])


## Closes the planner in whichever form this tree has, through its own close path.
func _close_planner() -> void:
	if _jp.has_method("in_window") and _jp.in_window():
		_jp.window().hide()
	else:
		_app.arm_tool("inspect")


## Every interactive control under `root`, as `INV <class>|<text>` lines (sorted by the caller).
func _inv(root: Node, out: Array, surface: String) -> void:
	if root == null:
		return
	for n in root.find_children("*", "Control", true, false):
		var c := n as Control
		if not c.is_visible_in_tree():
			continue
		var line := ""
		if c is OptionButton:
			line = "Option|%s/%d" % [(c as OptionButton).text, (c as OptionButton).item_count]
		elif c is BaseButton:
			line = "%s|%s" % [c.get_class(), (c as Button).text if c is Button else ""]
		elif c is SpinBox:
			line = "SpinBox|%s" % str((c as SpinBox).value)
		elif c is Range and not (c is ScrollBar):
			line = "%s|%s" % [c.get_class(), str((c as Range).value)]
		elif c is LineEdit and not (c.get_parent() is SpinBox):
			line = "LineEdit|%s" % (c as LineEdit).placeholder_text
		if line != "":
			out.append("%s %s" % [surface, line])


func _leg_inventory() -> void:
	print("== I: inventory ==")
	var pairs := _pairs()
	var idx := _commit_pair(int(pairs[0][0]), int(pairs[0][1]))
	_app.open_journey_planner_with_route(idx)
	await _settle(10)
	var lines: Array = []
	var win: Window = _jp.window() if _jp.has_method("window") else null
	_inv(_jp._left_panel, lines, "form")
	_inv(_jp._center_panel, lines, "centre")
	if win != null:
		_inv(win.toolbar, lines, "toolbar")
		_inv(win.results_body, lines, "results")
		_inv(win.timeline_slot, lines, "band")
	else:
		## The swap form: the bar row and the strip are the shell's; the results are the right dock's
		## appended section, drawn by `build_results()` (the same builder the window calls).
		_inv(_app.tool_options_row, lines, "toolbar")
		var tmp := VBoxContainer.new()
		_app.add_child(tmp)
		_jp.build_results(tmp)
		await _settle(2)
		_inv(tmp, lines, "results")
		tmp.queue_free()
		_inv(_app.timeline_row, lines, "band")
	lines.sort()
	for l in lines:
		print("INV " + String(l))
	print("INV-COUNT %d" % lines.size())
	_ok(lines.size() > 30, "I: the inventory is not empty (%d controls)" % lines.size())
	_close_planner()
	await _settle(4)


func _leg_tablet(built: bool) -> void:
	print("== tablet ==")
	if built:
		_ok(not _app.planner_in_window(), "tablet: planner_in_window() is false")
		_ok(not _jp.in_window() and _jp.window() == null, "tablet: no planner window is built")
	_ok(_app.find_child("PlannerWindow", true, false) == null, "tablet: no PlannerWindow node anywhere")
	_app.open_journey_planner()
	await _settle(8)
	_ok(not _app.viewport.visible, "tablet: opening the planner hides the map (the shipped swap)")
	_ok(String(_app.armed_tool) == "journey", "tablet: Journey is armed (the shipped swap)")
	_ok(_jp._center_panel.is_visible_in_tree(), "tablet: the planner's centre panel shows where the map was")
	_app.arm_tool("inspect")
	await _settle(6)
	_ok(_app.viewport.visible, "tablet: releasing Journey gives the map back")


func _leg_phone() -> void:
	if _shots_dir == "":
		print("[FATAL] the phone leg needs --shots <dir>")
		get_tree().quit(3)
		return
	await _generate(SEEDS[0])
	await _settle(30)
	await _shot("phone_shell")
	_app.open_journey_planner()
	await _settle(30)
	await _shot("phone_planner")
	_ok(true, "phone: two captures written to %s" % _shots_dir)


func _shot(tag: String) -> void:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	var path := _shots_dir.path_join(tag + ".png")
	img.save_png(path)
	print("SHOT %s" % path)


# ---------------------------------------------------------------- desktop, the window form ----

func _win() -> AcceptDialog:
	return _jp.window()


## The checks every entry point shares: the window is open and non-blocking, and the main screen kept
## everything (the map visible, the domain and the armed tool as they were, the bar and the strip untouched).
func _assert_open(how: String, domain_before: String, tool_before: String, expect_domain := "") -> void:
	var w := _win()
	_ok(w != null and w.visible, "E %s: the planner window is open" % how)
	if w == null:
		return
	_ok(not w.exclusive, "E %s: it is not exclusive" % how)
	_ok(_app.viewport.visible and _app.viewport.is_visible_in_tree(), "E %s: the map is still visible" % how)
	var want_dom := expect_domain if expect_domain != "" else domain_before
	_ok(_app.active_domain() == want_dom, "E %s: the domain is %s (was %s, now %s)" % [how, want_dom, domain_before, _app.active_domain()])
	_ok(String(_app.armed_tool) == tool_before, "E %s: the armed tool is still %s (now %s)" % [how, tool_before, _app.armed_tool])
	var bar_text := ""
	for c in _app.tool_options_row.find_children("*", "Label", true, false):
		bar_text += (c as Label).text + " "
	_ok(not bar_text.contains("JOURNEY PLANNER"), "E %s: the tool-options bar carries no planner row (Ruling DZ)" % how)
	var strip_text := ""
	for c in _app.timeline_row.find_children("*", "Label", true, false):
		strip_text += (c as Label).text + " "
	_ok(not strip_text.contains("day 1") and not strip_text.contains("no committed route selected"),
		"E %s: the timeline strip holds none of the planner's band" % how)


func _leg_desktop() -> void:
	print("== desktop ==")
	_ok(_app.planner_in_window() and _jp.in_window() and _win() != null, "the desktop builds the planner window")
	var w := _win()
	if w == null:
		get_tree().quit(3)
		return
	_ok(PopoutHost.key_for(w) == "planner_window" and PopoutHost.LIVE_KEYS.has("planner_window"), "PW-4: the window's key is live")
	_ok(w.find_child(PopoutHost.BUTTON_NAME, true, false) != null, "PW-4: the window carries its Pop out button")
	_ok(w.find_child("ContextKeyRelay", true, false) != null, "the window carries PC-7's chord relay")
	var menus_tip := ""
	var story: PopupMenu = null
	for ch in _app.menu_bar_row.get_children():
		if ch is MenuButton and (ch as MenuButton).text == "Story":
			story = (ch as MenuButton).get_popup()
	if story != null:
		var i := story.get_item_index(DccMenus.ID_STORY_JOURNEY_PLANNER)
		menus_tip = story.get_item_tooltip(i) if i >= 0 else ""
	_ok(menus_tip.begins_with("Opens the Journey planner in its own window"), "Story ▸ Journey planner's tooltip names the window")

	_app.select_domain("world")
	_app.arm_tool("inspect")
	await _settle(6)
	## E1. Story ▸ Journey planner: a real click on the bar, then Down to the row and Enter.
	await _open_from_story(story)
	await _assert_open("Story ▸ Journey planner", "world", "inspect")
	## Move the window to the right edge at its floor width, as a user drags it aside, so the map's left
	## part is clear. Fully on screen: a reopen keeps an on-screen window where it was (`popup()` clamps
	## one that hangs off the viewport back inside it, which is the engine's rule, not this window's).
	var vs := get_viewport().get_visible_rect().size
	w.size = Vector2i(w.min_size.x, 640)
	w.position = Vector2i(int(vs.x) - w.size.x, 70)
	await _settle(6)
	await _leg_layout(w)
	## The floor is wider than the map at 1600 px, so the user drags it to hang past the screen's right
	## edge (an embedded window may), as `_pc7windows_probe.gd` does, leaving the map's left part clear.
	var mr := (_app.viewport as Control).get_global_rect()
	w.position = Vector2i(int(mr.position.x + mr.size.x * 0.45), 70)
	await _settle(6)
	await _leg_map(w)
	await _leg_stops(w)
	if _shots_dir != "":
		await _settle(10)
		await _shot("desktop_window")
	await _leg_close_reopen(w)
	await _leg_saved(w)
	await _leg_card(w)
	await _leg_rail(w)
	await _leg_popout(w)
	await _leg_busy(w)


## Opens Story's menu with a real click and picks Journey planner with real keys (Down to its row, Enter).
func _open_from_story(story: PopupMenu) -> void:
	var mb: MenuButton = null
	for ch in _app.menu_bar_row.get_children():
		if ch is MenuButton and (ch as MenuButton).text == "Story":
			mb = ch
	if mb == null or story == null:
		_ok(false, "E: the Story menu exists")
		return
	await _click_at(mb.get_global_rect().get_center())
	await _settle(4)
	var target := story.get_item_index(DccMenus.ID_STORY_JOURNEY_PLANNER)
	var guard := 0
	while story.visible and story.get_focused_item() != target and guard < 60:
		await _key(KEY_DOWN)
		guard += 1
	_ok(story.get_focused_item() == target, "E: real Down keys reach Story ▸ Journey planner (%d presses)" % guard)
	await _key(KEY_ENTER)
	await _settle(8)


## L. Every column is drawn inside the window at its floor size: the results column (the right dock's
## old section) is on screen, at its width, with content; the toolbar's last control is reachable by its
## own scroller. Found by looking at the capture: before the row scrollers, the toolbar's width pushed the
## results column past the window's right edge.
func _leg_layout(w: AcceptDialog) -> void:
	print("== L: layout at the floor size %s ==" % str(w.size))
	await _settle(6)
	for n in ["PlannerLeftScroll", "PlannerCenter", "PlannerResultsScroll"]:
		var c := w.find_child(n, true, false) as Control
		var r := c.get_global_rect() if c != null else Rect2()
		_ok(c != null and r.position.x >= 0.0 and r.end.x <= float(w.size.x) + 0.5 and r.size.x >= 300.0,
			"L: %s is inside the window (x %.0f..%.0f of %d)" % [n, r.position.x, r.end.x, w.size.x])
	_ok(w.results_body.get_child_count() > 0, "L: the results column has content (%d groups)" % w.results_body.get_child_count())
	_ok(w.title == "Journey planner · " + _jp.readout_text() and _jp.readout_text().contains(" d · "), "L: the title carries the dock header's readout ('%s')" % w.title)
	var export_btn: Button = null
	for b in w.toolbar.find_children("*", "Button", true, false):
		if (b as Button).text == "export table":
			export_btn = b
	_ok(export_btn != null, "L: the toolbar holds export table")
	if export_btn != null:
		var sc := w.find_child("PlannerToolbarScroll", true, false) as ScrollContainer
		sc.ensure_control_visible(export_btn)
		await _settle(4)
		var er := export_btn.get_global_rect()
		_ok(er.position.x >= 0.0 and er.end.x <= float(w.size.x) + 0.5, "L: its last control scrolls into the window (x %.0f..%.0f)" % [er.position.x, er.end.x])
		sc.scroll_horizontal = 0
		await _settle(4)


func _leg_map(w: AcceptDialog) -> void:
	print("== M: the map stays live ==")
	var cands := _clickable(w)
	_ok(cands.size() >= 3, "M: three clickable settlements outside the window (%d)" % cands.size())
	if cands.size() < 3:
		return
	await _focus_inside(w)
	_ok(w.has_focus(), "M: a click inside focuses the window")
	await _click_at(_screen_of(int(cands[0])))
	_ok(_app.selection == {"kind": "settlement", "id": int(cands[0])},
		"M: a real map click outside the window selects that place (%s)" % str(_app.selection))
	_ok(w.visible, "M: the window stays open")
	var subj := w.find_child("StopsSubject", true, false) as Label
	var nm := String((_bridge.settlements()[int(cands[0])] as Dictionary).get("name", ""))
	_ok(subj != null and subj.text == nm, "M: the stop row names the selection ('%s' vs '%s')" % [subj.text if subj else "?", nm])
	await _focus_inside(w)
	var cam = _app.viewport._camera
	var z0: Vector2 = cam.scale
	await _wheel(_screen_of(int(cands[1])))
	_ok(not is_equal_approx(cam.scale.x, z0.x), "M: a wheel notch over the map zooms it with the window focused (%.4f -> %.4f)" % [z0.x, cam.scale.x])
	await _focus_inside(w)
	var p0: Vector2 = cam.position
	await _middle_drag(_screen_of(int(cands[1])) + Vector2(0, -20), Vector2(60, 30))
	_ok(cam.position.distance_to(p0) > 20.0, "M: a middle drag pans the map with the window open (%.1f px)" % cam.position.distance_to(p0))
	_app.viewport.reset_view()
	await _settle(8)


## Focuses `w` by a real click on an empty spot of its own content (its top-left, inside the toolbar's
## margin, where no control is), as `_pc7windows_probe.gd` does.
func _focus_inside(w: Window) -> void:
	await _click_at(Vector2(w.position) + Vector2(4, 3))
	await _settle(2)


func _btn(w: Window, n: String) -> Button:
	return w.find_child(n, true, false) as Button


func _draft() -> int:
	return (_jp._stops_draft as Array).size()


func _select_by_click(w: Window, i: int) -> void:
	await _click_at(_screen_of(i))
	await _settle(4)


func _leg_stops(w: AcceptDialog) -> void:
	print("== S: stops from the selection ==")
	_app.select("", -1)
	await _settle(4)
	_ok(_btn(w, "StopStart").disabled and _btn(w, "StopStart").tooltip_text.begins_with("Select a settlement"),
		"S: with no selection Start here is disabled, with its reason ('%s')" % _btn(w, "StopStart").tooltip_text)
	var cands := _clickable(w)
	if cands.size() < 3:
		_ok(false, "S: three clickable settlements")
		return
	var a := int(cands[0])
	var b := int(cands[1])
	var c := int(cands[2])
	var routes0: int = _bridge.route_count()
	var route_before: int = _jp._route_index
	await _select_by_click(w, a)
	_ok(not _btn(w, "StopStart").disabled, "S: a selected settlement enables Start here")
	_ok(_btn(w, "StopAdd").disabled and _btn(w, "StopAdd").tooltip_text.begins_with("Press Start here"),
		"S: Add as stop is disabled before a start, with its reason")
	await _press_in(w, _btn(w, "StopStart"), "Start here")
	_ok(_draft() == 1 and int(_jp._stops_draft[0]["index"]) == a, "S: Start here makes a one-stop draft at the selection")
	_ok(_app.undo_target_label().begins_with("Journey: start at"), "S: Edit ▸ Undo names the step ('%s')" % _app.undo_target_label())
	await _select_by_click(w, b)
	await _press_in(w, _btn(w, "StopAdd"), "Add as stop")
	_ok(_draft() == 2 and int(_jp._stops_draft[1]["index"]) == b, "S: Add as stop appends the selection")
	_ok(_btn(w, "StopAdd").disabled, "S: the stop just added cannot be added twice in a row")
	await _select_by_click(w, c)
	await _press_in(w, _btn(w, "StopEnd"), "End here")
	var end_idx := routes0
	_ok(_bridge.route_count() == routes0 + 1, "S: End here commits one route (%d -> %d)" % [routes0, _bridge.route_count()])
	_ok(int(_jp._route_index) == end_idx and bool(_jp._last_result.get("ok", false)),
		"S: the planner opens on it and computes (route %d, ok %s)" % [_jp._route_index, str(_jp._last_result.get("ok", false))])
	var stops: Array = (_jp._last_result.get("plan", {}) as Dictionary).get("stops", [])
	_ok(stops.size() >= 2, "S: the plan has its stops (%d)" % stops.size())
	_ok(_draft() == 0, "S: End here empties the draft")
	## One Ctrl+Z per press, real keys with the focus on the map.
	await _click_at(_screen_of(c))
	await _key(KEY_Z, false, true)
	_ok(_bridge.route_count() == routes0 and _draft() == 2 and int(_jp._route_index) == route_before,
		"S: one Ctrl+Z undoes End here: the route is gone, the draft and the planner's route are back (%d, %d, %d)" % [_bridge.route_count(), _draft(), _jp._route_index])
	await _key(KEY_Z, false, true)
	_ok(_draft() == 1, "S: a second Ctrl+Z undoes Add as stop (%d)" % _draft())
	await _key(KEY_Z, false, true)
	_ok(_draft() == 0, "S: a third Ctrl+Z undoes Start here (%d)" % _draft())
	await _key(KEY_Y, false, true)
	_ok(_draft() == 1, "S: Ctrl+Y redoes Start here (%d)" % _draft())
	await _key(KEY_Y, false, true)
	_ok(_draft() == 2, "S: Ctrl+Y redoes Add as stop (%d)" % _draft())
	await _key(KEY_Y, false, true)
	_ok(_bridge.route_count() == routes0 + 1 and _draft() == 0 and int(_jp._route_index) == routes0,
		"S: Ctrl+Y redoes End here: the route is committed again and opened (%d, %d)" % [_bridge.route_count(), _jp._route_index])
	## The Route tool shares the engine's one route draft: End here refuses while it is armed.
	await _select_by_click(w, a)
	await _press_in(w, _btn(w, "StopStart"), "Start here (for the refusal)")
	await _select_by_click(w, b)
	var rc: int = _bridge.route_count()
	_app.arm_tool("route")
	await _settle(4)
	await _press_in(w, _btn(w, "StopEnd"), "End here with Route armed")
	_ok(_bridge.route_count() == rc and _draft() == 1, "S: End here refuses while the Route tool is armed (routes %d -> %d)" % [rc, _bridge.route_count()])
	_app.arm_tool("inspect")
	await _settle(4)
	await _click_at(_screen_of(b))
	await _key(KEY_Z, false, true)
	_ok(_draft() == 0, "S: and Ctrl+Z takes that start back (%d)" % _draft())


func _leg_close_reopen(w: AcceptDialog) -> void:
	print("== C: close and reopen ==")
	var req_h := hash(var_to_str(_jp._plan_request()))
	var route: int = _jp._route_index
	## On screen for this leg (a reopen clamps a window that hangs off the viewport back inside it, the
	## engine's `popup()` rule), then put back where the other legs had it.
	var hang := w.position
	w.position = Vector2i(get_viewport().get_visible_rect().size.x - w.size.x - 20, 60)
	await _settle(4)
	var pos := w.position
	var strip_before: int = _app.timeline_row.get_child_count()
	var close: Button = w.get_ok_button()
	await _press_in(w, close, "Close")
	_ok(not w.visible, "C: the window's real Close hides it")
	_ok(not _jp._active, "C: the planner is marked closed")
	_ok(_app.viewport.visible and _app.timeline_row.get_child_count() == strip_before,
		"C: closing touched nothing on the main screen (map visible, strip %d -> %d children)" % [strip_before, _app.timeline_row.get_child_count()])
	await _move(_screen_of(int(_clickable(null)[0])))
	await _key(KEY_J, true)
	await _settle(8)
	_ok(w.visible, "C: Shift+J over the map reopens it")
	_ok(int(_jp._route_index) == route and hash(var_to_str(_jp._plan_request())) == req_h,
		"C: the reopened planner shows the same route and request")
	_ok(w.position == pos, "C: it reopens where the user left it (%s vs %s)" % [str(w.position), str(pos)])
	w.position = hang
	await _settle(4)


func _leg_saved(w: AcceptDialog) -> void:
	print("== E: Saved journeys ==")
	_jp._commit_saved_journey("pc8 probe journey")
	await _settle(4)
	var rows: Array = _bridge.journey_list()
	_ok(rows.size() >= 1, "E saved: a journey is saved (%d)" % rows.size())
	w.hide()
	await _settle(4)
	_app.open_saved_journeys()
	await _settle(10)
	var sj: AcceptDialog = _app.saved_journeys_window
	sj.position = Vector2i(40, 80)
	await _settle(4)
	var row_btn: Button = null
	for b in sj.find_children("*", "Button", true, false):
		if (b as Button).text == "pc8 probe journey":
			row_btn = b
	_ok(row_btn != null, "E saved: the Saved journeys window lists it")
	if row_btn == null:
		return
	_jp._active_journey = -1
	await _press_in(sj, row_btn, "the saved journey's row")
	_ok(w.visible, "E saved: a real click on its row opens the planner window")
	_ok(int(_jp._active_journey) >= 0, "E saved: on that journey (%d)" % _jp._active_journey)
	_ok(sj.visible, "E saved: Saved journeys stays open beside it (PC-8: the planner no longer replaces the map)")
	sj.hide()
	await _settle(4)


func _card():
	var b = _app.get("context_broker")
	return b.card if b != null else null


## A real right click on settlement `i`, then a real click on the card row `id`. False when the row is
## not drawn enabled.
func _card_verb(i: int, id: String) -> bool:
	await _click_at(_screen_of(i), MOUSE_BUTTON_RIGHT)
	await _settle(8)
	var c = _card()
	if c == null or not c.visible:
		_ok(false, "E card: a right click opens the card")
		return false
	for r in c.drawn_rows():
		var d: Dictionary = r
		if String(d.get("id", "")) == id and String(d.get("kind", "")) == "action":
			var n: Control = d["node"]
			var at: Vector2 = n.get_global_rect().get_center()
			if c is Window:
				at += Vector2((c as Window).position)
			await _click_at(at)
			await _settle(8)
			return true
	_ok(false, "E card: row %s is drawn enabled (%s)" % [id, str(c.drawn_rows().map(func(r): return r.get("id", "")))])
	c.hide()
	return false


func _leg_card(w: AcceptDialog) -> void:
	print("== E: the card's verbs ==")
	w.hide()
	await _settle(4)
	_app.select_domain("civilization")
	_app.arm_tool("inspect")
	await _settle(6)
	var cands := _clickable(null)
	if cands.size() < 2:
		_ok(false, "E card: two clickable settlements")
		return
	var routes0: int = _bridge.route_count()
	if await _card_verb(int(cands[0]), "civ.journey_from"):
		_ok(String(_app.armed_tool) == "route", "E card: Plan journey from here arms Route")
		if await _card_verb(int(cands[1]), "civ.journey_to"):
			_ok(_bridge.route_count() == routes0 + 1, "E card: Journey to here commits the route")
			await _assert_open("card ▸ Journey to here", "civilization", "inspect")
			_ok(int(_jp._route_index) == routes0, "E card: the planner opens on that route (%d)" % _jp._route_index)
	await _settle(4)


func _leg_rail(w: AcceptDialog) -> void:
	print("== E: the CIVIL rail node ==")
	w.hide()
	_app.select_domain("world")
	await _settle(6)
	var node: Control = _app._rail_node_rows.get("civilization/planner")
	if node != null and not node.is_visible_in_tree():
		## The rail's expansion opens on its own gesture; that gesture is not this milestone's subject,
		## so the probe opens it directly and says so.
		print("  info the rail expansion was closed; opened by set_rail_expanded(true) as setup")
		_app.set_rail_expanded(true)
		await _settle(8)
	_ok(node != null and node.is_visible_in_tree(), "E rail: the CIVIL rail's Journey planner node is drawn")
	if node == null or not node.is_visible_in_tree():
		return
	await _click_at(node.get_global_rect().get_center())
	await _settle(8)
	await _assert_open("the CIVIL rail node", "world", "inspect", "civilization")


func _leg_popout(w: AcceptDialog) -> void:
	print("== P: pop out ==")
	if not w.visible:
		_app.open_journey_planner()
		await _settle(8)
	var res_h := hash(var_to_str(_jp._last_result))
	var req_h := hash(var_to_str(_jp._plan_request()))
	var pop := w.find_child(PopoutHost.BUTTON_NAME, true, false) as Button
	_ok(pop != null and pop.text == "Pop out", "P: the button reads Pop out")
	var mem0 := Performance.get_monitor(Performance.RENDER_TEXTURE_MEM_USED)
	await _press_in(w, pop, "Pop out")
	await _settle(12)
	_ok(not w.is_embedded() and w.visible, "P: a real click pops it out (native, visible)")
	_ok(DisplayServer.get_window_list().size() == 2, "P: two OS windows (%d)" % DisplayServer.get_window_list().size())
	_ok(_app.viewport.visible, "P: the main map is still visible")
	var mem1 := Performance.get_monitor(Performance.RENDER_TEXTURE_MEM_USED)
	print("  info texture memory docked %.1f MB, popped out %.1f MB (delta %+.1f MB)" % [mem0 / 1048576.0, mem1 / 1048576.0, (mem1 - mem0) / 1048576.0])
	var cam = _app.viewport._camera
	var p0: Vector2 = cam.position
	var cands := _clickable(w)
	if cands.size() >= 1:
		await _middle_drag(_screen_of(int(cands[0])), Vector2(70, 40))
		await _wheel(_screen_of(int(cands[0])))
	_ok(cam.position.distance_to(p0) > 20.0, "P: the main map pans while the planner is native (%.1f px)" % cam.position.distance_to(p0))
	_ok(hash(var_to_str(_jp._last_result)) == res_h and hash(var_to_str(_jp._plan_request())) == req_h,
		"P: a pan and a wheel on the main map change neither the planner's request nor its result")
	if cands.size() >= 1:
		await _click_at(_screen_of(int(_clickable(w)[0])))
		var subj := w.find_child("StopsSubject", true, false) as Label
		_ok(_app.selection.get("kind", "") == "settlement" and subj != null and subj.text != "no settlement selected",
			"P: a map click selects, and the native window's stop row follows ('%s')" % (subj.text if subj else "?"))
	## A generate with it native: muted while the engine works.
	_bridge.generate({"seed": 24601, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	await _settle(3)
	if bool(_bridge.generating):
		_ok(w.gui_disable_input, "P: a generate mutes the native planner window")
	else:
		_ok(false, "P: the generate was still running when checked (it finished too fast to check)")
	while bool(_bridge.generating):
		await _settle(10)
	await _settle(30)
	_ok(not w.gui_disable_input, "P: input returns after the generate")
	pop = w.find_child(PopoutHost.BUTTON_NAME, true, false) as Button
	_ok(pop != null and pop.text == "Dock", "P: the button reads Dock while native")
	## Dock: a real click inside the native window, pushed into its own viewport (PW-2 finding 4).
	if pop != null:
		var at := pop.get_global_rect().get_center()
		for down in [true, false]:
			var ev := InputEventMouseButton.new()
			ev.button_index = MOUSE_BUTTON_LEFT
			ev.pressed = down
			ev.position = at
			ev.global_position = at
			w.push_input(ev)
			await _settle(3)
		await _settle(12)
	_ok(w.is_embedded() and w.visible, "P: Dock brings it back embedded")
	_ok(DisplayServer.get_window_list().size() == 1, "P: one OS window again")


func _leg_busy(w: AcceptDialog) -> void:
	print("== B: busy ==")
	if not w.visible:
		_app.open_journey_planner()
		await _settle(8)
	_bridge.generate({"seed": 71077345, "width_km": 2400.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.45})
	await _settle(3)
	if bool(_bridge.generating):
		_ok(w.gui_disable_input, "B: a generate mutes the docked planner window")
		var draft := _draft()
		var b := _btn(w, "StopStart")
		if b != null and b.is_visible_in_tree():
			await _click_at(_in_win(w, b))
		_ok(_draft() == draft and bool(_bridge.generating), "B: a real click on Start here during the run does nothing")
	else:
		_ok(false, "B: the generate was still running when checked")
	while bool(_bridge.generating):
		await _settle(10)
	await _settle(30)
	_ok(not w.gui_disable_input and w.visible, "B: after the run the window takes input again and is still open")
	_ok(_draft() == 0 and _jp._stops_hist.is_empty(), "B: a new world dropped the stop draft and its undo steps")
