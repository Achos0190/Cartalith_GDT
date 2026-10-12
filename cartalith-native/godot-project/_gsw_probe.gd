extends Node
## Protects: the mid-generation sweep of 2026-10-12 -- every shell entry point that can reach a `WorldGen`
## `#[func]` while the worker thread owns it (`bridge.generating`) WITHOUT a user click (a timer, a window
## notification) or by a direct call, and that the 2026-10-11 arming sweep left open. Each must neither panic
## (`Gd<T>::bind() failed`) nor drop the user's intent silently: it is refused with the shell's "Not done" line,
## held, or replayed when the run finishes. Run WINDOWED from a scratch copy, never headless, never from the live
## project. Legs, all fired during ONE real `bridge.generate` (512 x 320) after a small world exists:
##   R  resource_overlay._refresh (direct + its 0.5 s timer)      -> held text, redrawn after the run
##   T  dcc_shell._tl_tick (direct + the 0.6 s playback timer)    -> skipped; advances again after the run
##   C  civilization_workspace._tl_on_play_tick (direct)          -> skipped
##   B  WM_GO_BACK_REQUEST notification                            -> refused with a hint; works after the run
##   X  confirm_unsaved_world (the OS-close card)                  -> seed dashed with its reason; real seed after
##   U  menus._on_undo_budget rung + Clear                         -> rung deferred and replayed; Clear refused
##   D  menus._redo_last                                           -> refused with a hint
##   P  menus._paste_clipboard / _cut / _copy / _select_all        -> refused, clipboard kept, paste replays after
##   K  right_dock paint commit / discard                          -> refused; commit works after
## Panics are counted by an in-process Logger here AND by grepping stderr afterwards (the caller's job).
## usage: godot --path <scratch copy> res://_gsw_probe.tscn -- [--tag T]

var app: Node
var bridge: Node
var _vp: SubViewport
var _tag := "gsw"
var _checks := 0
var _fail := 0

class PanicLogger extends Logger:
	var hits := 0
	var lines: PackedStringArray = PackedStringArray()
	func _log_error(function: String, _file: String, _line: int, code: String, rationale: String, _editor_notify: bool, _error_type: int, bt: Array[ScriptBacktrace]) -> void:
		var s := "%s %s %s" % [function, code, rationale]
		if "bind() failed" in s or "cannot borrow" in s or "panicked" in s or "already bound" in s or "already mutably" in s:
			hits += 1
			var frame := ""
			for b in bt:
				if b != null and b.get_frame_count() > 0:
					frame = "%s:%d %s" % [b.get_frame_file(0).get_file(), b.get_frame_line(0), b.get_frame_function(0)]
					break
			if lines.size() < 60:
				lines.append("%s | %s" % [s.substr(0, 80), frame])
	func _log_message(_m: String, _e: bool) -> void:
		pass

var _pl: PanicLogger


func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])


func _check(ok: bool, what: String) -> void:
	_checks += 1
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _guard() -> bool:
	var img := Image.create_empty(2, 2, false, Image.FORMAT_RGBA8)
	img.fill(Color(0, 0, 0, 0))
	var t := ImageTexture.create_from_image(img)
	img.set_pixel(1, 1, Color(1, 0, 0, 1))
	t.update(img)
	return t.get_image().get_pixel(1, 1).a > 0.0


func _hint() -> String:
	return String((app._status_labels["hint"] as Label).text)


## Runs `c` and checks it raised no engine panic; returns the panic delta.
func _fire(name: String, c: Callable) -> int:
	var before := _pl.hits
	c.call()
	var d := _pl.hits - before
	_check(d == 0, "%s: 0 engine panics (got %d)" % [name, d])
	return d


func _texts(root: Node) -> String:
	var out := PackedStringArray()
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is Label:
			out.append(String((n as Label).text))
		elif n is Button:
			out.append(String((n as Button).text))
		if n is Control and String((n as Control).tooltip_text) != "":
			out.append(String((n as Control).tooltip_text))
		for c in n.get_children(true):
			stack.append(c)
	return " | ".join(out)


## The bridge's pending undo budget; -999 on a shell without the field (the HEAD negative control).
func _pending() -> int:
	var v = bridge.get("_pending_undo_budget_mb")
	return -999 if v == null else int(v)


## Waits for the run to finish, then lets the finish handlers settle.
func _wait_done() -> void:
	var spins := 0
	while bridge.generating and spins < 1200:
		await _frames(5)
		spins += 1
	await _frames(20)


func _civ_ws() -> Node:
	for ws in app._workspaces:
		if ws != null and ws.has_method("_tl_on_play_tick"):
			return ws
	return null


## Every visible embedded window under the probe viewport whose text contains `needle`.
func _windows_with(needle: String) -> Array:
	var out: Array = []
	for w in _vp.get_embedded_subwindows():
		if w is Window and (w as Window).visible and needle in _texts(w):
			out.append(w)
	return out


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var ti := args.find("--tag")
	if ti >= 0 and ti + 1 < args.size():
		_tag = String(args[ti + 1])
	if not _guard():
		_log("ABORT run WINDOWED")
		get_tree().quit(2)
		return
	_pl = PanicLogger.new()
	OS.add_logger(_pl)
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)
	bridge = app.bridge
	_log("user dir %s" % OS.get_user_data_dir())
	_log("DLL mtime (unix) %d" % FileAccess.get_modified_time(ProjectSettings.globalize_path("res://").path_join("../target/debug/cartalith_godot.dll")))

	# -- a world first, so every path has something to read --------------------------------------
	bridge.generate({"seed": 4242, "width_km": 800.0, "grid_w": 192, "grid_h": 124, "sea_level": 0.5, "villages": true})
	await bridge.generation_finished
	await _frames(12)
	_check(bridge.has_world, "setup: a world exists")
	app.arm_tool("inspect")
	await _frames(4)
	var menus = app.menus
	var civ := _civ_ws()
	_check(civ != null, "setup: civilisation workspace found")
	app.resource_overlay.visible = true
	await _frames(2)
	var tl_ok: bool = app.tl_available()
	if tl_ok and not app.tl_playing:
		app.tl_toggle_play()
	_log("timeline available=%s playing=%s" % [tl_ok, app.tl_playing])
	menus._clipboard = {"labels": [{"x": 20.0, "y": 20.0, "text": "GSW"}]}
	var hits_idle := _pl.hits
	_check(hits_idle == 0, "setup: 0 panics before the run (got %d)" % hits_idle)

	# -- the run ----------------------------------------------------------------------------------
	bridge.generate({"seed": 777, "width_km": 1600.0, "grid_w": 512, "grid_h": 320, "sea_level": 0.5, "villages": true})
	await _frames(2)
	_check(bridge.generating, "run: bridge.generating is true when the legs fire")
	var t0 := Time.get_ticks_msec()

	_fire("R resource_overlay._refresh direct", func(): app.resource_overlay._refresh())
	_check(String(app.resource_overlay._label.text).begins_with("Generating"), "R: overlay text says it is held ('%s')" % String(app.resource_overlay._label.text).substr(0, 40))
	_fire("T dcc_shell._tl_tick direct x3", func():
		app._tl_tick()
		app._tl_tick()
		app._tl_tick())
	_fire("C civ._tl_on_play_tick direct x2", func():
		civ._tl_on_play_tick()
		civ._tl_on_play_tick())
	var wins_before := _vp.get_embedded_subwindows().size()
	_fire("B WM_GO_BACK_REQUEST", func(): app.notification(NOTIFICATION_WM_GO_BACK_REQUEST))
	_check("Not done: Back" in _hint(), "B: refused with the hint ('%s')" % _hint().substr(0, 50))
	_check(_vp.get_embedded_subwindows().size() == wins_before, "B: no exit card opened mid-run")
	var holder: Array = [null]
	_fire("X confirm_unsaved_world (OS close card)", func():
		holder[0] = app.confirm_unsaved_world("Exit Cartalith", "Exit the app?", "Discard and exit", "Save and exit", func(): pass))
	var card_mid: AcceptDialog = holder[0]
	_check(card_mid != null and "the world is generating; its seed is readable" in _texts(card_mid), "X: seed dashed with its reason")
	if card_mid != null:
		card_mid.hide()
	_fire("U undo budget rung 128 MB", func(): menus._on_undo_budget(1))
	_check("applied when the run finishes" in _hint(), "U: rung says it is deferred ('%s')" % _hint().substr(0, 50))
	_check(_pending() == 128, "U: 128 MB pending (got %d)" % _pending())
	_fire("U undo Clear", func(): menus._on_undo_budget(DccMenus.ID_PREF_UNDO_CLEAR))
	_check("Not done: Clear undo history" in _hint(), "U: Clear refused with the hint")
	_fire("D menus._redo_last", func(): menus._redo_last())
	_check("Not done: Redo" in _hint(), "D: refused with the hint")
	_fire("P menus._paste_clipboard", func(): menus._paste_clipboard())
	_check("Not done: Paste" in _hint(), "P: paste refused with the hint")
	_check(not menus._clipboard.is_empty(), "P: clipboard kept for after the run")
	_fire("P menus._cut_selection", func(): menus._cut_selection())
	_check("Not done: Cut" in _hint(), "P: cut refused")
	_fire("P menus._copy_selection", func(): menus._copy_selection())
	_check("Not done: Copy" in _hint(), "P: copy refused")
	_fire("P menus._select_all", func(): menus._select_all())
	_check("Not done: Select all" in _hint(), "P: select all refused")
	_fire("K right_dock paint commit", func(): app.right_dock_ctrl._on_paint_commit_from_dock())
	_check("Not done: Commit paint" in _hint(), "K: commit refused")
	_fire("K right_dock paint discard", func(): app.right_dock_ctrl._on_paint_discard_from_dock())
	_check("Not done: Discard paint draft" in _hint(), "K: discard refused")

	_log("direct legs fired in %d ms" % (Time.get_ticks_msec() - t0))
	await _wait_done()

	# -- the timer legs: a second, larger run (1024 x 640) long enough for the 0.5 s overlay timer and
	# the 0.6 s playback timer to fire while it holds the engine; each timeout is counted only while
	# `generating`, so a pass cannot come from timers that fired after the run.
	if not app.tl_playing and app.tl_available():
		app.tl_toggle_play()
	var fired := {"overlay": 0, "playback": 0}
	app.resource_overlay._timer.timeout.connect(func(): if bridge.generating: fired["overlay"] += 1)
	if app._tl_timer != null:
		app._tl_timer.timeout.connect(func(): if bridge.generating: fired["playback"] += 1)
	bridge.generate({"seed": 778, "width_km": 1600.0, "grid_w": 1024, "grid_h": 640, "sea_level": 0.5, "villages": true})
	await _frames(2)
	var before_timers := _pl.hits
	var t1 := Time.get_ticks_msec()
	while bridge.generating and Time.get_ticks_msec() - t1 < 1600:
		await _frames(1)
	var held_text := String(app.resource_overlay._label.text)
	var still := bool(bridge.generating)
	_check(still, "timers: still generating after %d ms" % (Time.get_ticks_msec() - t1))
	_check(int(fired["overlay"]) >= 2 and int(fired["playback"]) >= 2, "timers: overlay fired %d, playback fired %d times under the run" % [int(fired["overlay"]), int(fired["playback"])])
	_check(_pl.hits - before_timers == 0, "R/T timers: 0 engine panics while ticking under the run (got %d)" % (_pl.hits - before_timers))
	_check(held_text.begins_with("Generating"), "R: timer kept the held text")
	await _wait_done()
	_check(not bridge.generating, "after: run finished")
	if app.tl_playing:
		app.tl_toggle_play()
	_check(not String(app.resource_overlay._label.text).begins_with("Generating"), "R after: overlay redrawn ('%s')" % String(app.resource_overlay._label.text).substr(0, 30))
	_check(_pending() == -1, "U after: nothing pending")
	var us: Dictionary = bridge.undo_stats()
	_check(int(us.get("budget_bytes", -1)) == 128 * 1048576, "U after: engine budget is 128 MB (budget_bytes %d)" % int(us.get("budget_bytes", -1)))
	var y0: int = app.tl_year()
	app._tl_tick()
	var y1: int = app.tl_year()
	_check(y1 == mini(y0 + int(app.tl_speed), DccShell.TL_YEAR_MAX), "T after: a tick advances the year (%d -> %d)" % [y0, y1])
	menus._paste_clipboard()
	var pass_text := String((app._status_labels["pass"] as Label).text)
	_check(pass_text.begins_with("pasted 1"), "P after: the kept paste lands ('%s')" % pass_text.substr(0, 40))
	menus._redo_last()
	_check(not ("Not done" in _hint()), "D after: redo reaches the engine ('%s')" % _hint().substr(0, 40))
	app.right_dock_ctrl._on_paint_commit_from_dock()
	_check(_hint().begins_with("painted"), "K after: commit runs ('%s')" % _hint().substr(0, 30))
	var card_after: AcceptDialog = app.confirm_unsaved_world("Exit Cartalith", "Exit the app?", "Discard and exit", "Save and exit", func(): pass)
	_check(card_after != null and str(bridge.world_gen.get_seed()) in _texts(card_after), "X after: the card shows the real seed")
	if card_after != null:
		card_after.hide()
	await _frames(4)
	app.notification(NOTIFICATION_WM_GO_BACK_REQUEST)
	await _frames(4)
	var exit_cards := _windows_with("Exit the app?")
	_check(exit_cards.size() >= 1, "B after: back reaches the exit card (%d open)" % exit_cards.size())
	for w in exit_cards:
		(w as Window).hide()
	await _frames(4)

	_log("panic frames (first %d):" % _pl.lines.size())
	for l in _pl.lines:
		_log("    " + l)
	_log("RESULT %s checks=%d fail=%d engine-panic-hits(total)=%d" % [_tag, _checks, _fail, _pl.hits])
	get_tree().quit(1 if _fail > 0 else 0)
