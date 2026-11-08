extends Node
## MS-1 swap probe (`MOBILE_SHELL_MS1_SPEC.md` §3.5 and bar B5; `cartalith-native/docs/evidence/ms1-2026-11-07/
## S4_INVENTORY.md` §8 and §12). Boots the real shell in a `--vp`-sized host `SubViewport` and drives class
## changes by resizing the host and emitting `root.size_changed`, as `_ms1class_probe.gd`'s live legs do (a host
## resize does not resize the root window, and the shell listens to the root).
##
##   godot --path . --resolution 1600x1000 _ms1swap_probe.tscn -- --leg quiet --force-touch --vp 1800x3200 --density 1
##
## Legs:
##   quiet   (stage S4.1) the quiescence gate. The shell must report no blocker when idle; a held dock drag, a
##           visible confirmation, a generating engine and an open New world form must each be reported by
##           `quiescent_blockers()`; a size change across the line while one is held must WAIT (no swap run,
##           the status hint naming the blocker) and run once the blocker ends; a size that returns to the
##           current class while waiting must cancel the wait (no swap run, the hint cleared). At density 1,
##           1 800 x 3 200 dp is a tablet and 880 x 1 564 dp a phone (past 900 - 16, `DccTheme.BAND_DP`).
##
## Run windowed with a wiped user dir (the runner sets `APPDATA` to an empty folder). Logic and layout only, so
## nothing here reads pixels. Arguments read by THIS file: `--leg quiet`, `--vp WxH`; accepted because the
## shell reads them: `--force-touch`, `--density D`. Anything else aborts with exit 2 (`MISTAKES.md`, "Write a
## probe's usage header").
##
## **It fails on the tree before S4.1** by construction: there `quiescent_blockers()` does not exist and the
## swap request runs at once, so the first check fails and the wait checks cannot hold (`MISTAKES.md`, "Add a
## probe in the same commit that changes the behaviour"). Expected values are literals.
##
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument, 3 watchdog.

var _vp: SubViewport
var _app: Node
var _fails := 0
var _checks := 0

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1

func _frames(n: int) -> void:
	for k in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var k := args.find(name)
	if k >= 0 and k + 1 < args.size():
		return String(args[k + 1])
	return dflt

## Resizes the host and tells the shell, then waits past the 150 ms debounce (0.4 s).
func _resize(w: int, h: int) -> void:
	_vp.size = Vector2i(w, h)
	await _frames(2)
	get_tree().root.size_changed.emit()
	await get_tree().create_timer(0.4).timeout
	await _frames(3)

func _blockers() -> Array:
	return _app.call("quiescent_blockers")

func _has_blocker(prefix: String) -> bool:
	for b in _blockers():
		if String(b).begins_with(prefix):
			return true
	return false

func _hint() -> String:
	return String(_app.status_slot_text("hint"))

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 120.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("RESULT: FAIL (watchdog)")
		get_tree().quit(3))
	wd.start()
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--force-touch":
			i += 1
		elif a in ["--leg", "--vp", "--density"]:
			i += 2
		else:
			print("ABORT: unknown argument ", a)
			get_tree().quit(2)
			return
	var leg := _arg("--leg", "")
	var parts := _arg("--vp", "").split("x")
	if leg != "quiet" or parts.size() != 2:
		print("ABORT: needs --leg quiet and --vp WxH")
		get_tree().quit(2)
		return
	var vw := int(parts[0])
	var vh := int(parts[1])
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.6).timeout
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if "phone_project_picker" in _app and _app.phone_project_picker != null:
		_app.phone_project_picker.hide()
	await _frames(6)
	await _quiet()
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)

func _quiet() -> void:
	_ok(DccTheme.is_tablet(), "boots 1800 x 3200 dp as a tablet")
	var has := _app.has_method("quiescent_blockers") and _app.has_method("pending_class_swap")
	_ok(has, "the shell has quiescent_blockers() and pending_class_swap() (S4.1)")
	if not has:
		return
	_ok(_blockers().is_empty(), "idle: no blocker (got %s)" % str(_blockers()))

	print("-- a held dock drag makes a crossing wait, and the swap runs when it ends")
	_app._dragging_dock = "left"
	_ok(_has_blocker("a dock is being resized"), "a dock drag is reported (got %s)" % str(_blockers()))
	var runs0: int = _app.class_swap_runs
	await _resize(880, 1564)
	_ok(not _app.pending_class_swap().is_empty(), "880 x 1564 under a dock drag: a swap is pending")
	_ok(_app.class_swap_runs == runs0, "and has not run (runs %d, want %d)" % [_app.class_swap_runs, runs0])
	_ok(_hint().begins_with("Layout change waiting: a dock is being resized"),
		"the status hint names the blocker (hint '%s')" % _hint())
	_ok(DccTheme.is_tablet(), "the composition is still the tablet while waiting")
	_app._dragging_dock = ""
	await get_tree().create_timer(0.8).timeout
	await _frames(2)
	_ok(_app.class_swap_runs == runs0 + 1, "the drag ended: the swap ran once (runs %d, want %d)" % [
		_app.class_swap_runs, runs0 + 1])
	_ok(_app.pending_class_swap().is_empty(), "nothing is pending after it ran")
	_ok(not _hint().begins_with("Layout change waiting"), "the waiting hint is cleared (hint '%s')" % _hint())
	await _resize(1800, 3200)

	print("-- a visible confirmation makes a crossing wait; returning to the class cancels the wait")
	var runs1: int = _app.class_swap_runs
	var dlg: ConfirmationDialog = DccWidgets.confirm(_app, "Probe question", "Probe text", "OK", func(): pass)
	await _frames(3)
	_ok(_has_blocker("a confirmation is open (Probe question)"), "a confirmation is reported (got %s)" % str(_blockers()))
	await _resize(880, 1564)
	_ok(not _app.pending_class_swap().is_empty() and _app.class_swap_runs == runs1,
		"880 x 1564 under a confirmation: pending, not run (runs %d)" % _app.class_swap_runs)
	_ok(_hint().begins_with("Layout change waiting: a confirmation is open"), "hint names it ('%s')" % _hint())
	## The settled resize must cancel the wait by itself: the poll (a second path to the same end) is stopped
	## first, so this check cannot be passed by the poll firing inside the window (found by mutation).
	_app._swap_wait_timer.stop()
	await _resize(1800, 3200)
	_ok(_app.pending_class_swap().is_empty(), "back at 1800 x 3200: the wait is cancelled")
	_ok(_app.class_swap_runs == runs1, "and no swap ran (runs %d, want %d)" % [_app.class_swap_runs, runs1])
	_ok(not _hint().begins_with("Layout change waiting"), "the waiting hint is cleared (hint '%s')" % _hint())
	_ok(is_instance_valid(dlg) and dlg.visible, "the confirmation was not touched: still open")
	print("-- the poll alone cancels a wait when the size has come back without a settled resize")
	await _resize(880, 1564)
	_ok(not _app.pending_class_swap().is_empty(), "880 x 1564 under the confirmation again: pending")
	## Back to the tablet size with no `size_changed`, so only the poll sees it.
	_vp.size = Vector2i(1800, 3200)
	await get_tree().create_timer(0.8).timeout
	await _frames(2)
	_ok(_app.pending_class_swap().is_empty() and _app.class_swap_runs == runs1,
		"the poll found the size back in the class: cancelled, no swap (runs %d, want %d)" % [_app.class_swap_runs, runs1])
	dlg.hide()
	dlg.queue_free()
	await _frames(3)

	print("-- other sources, read without a resize")
	_app.bridge.generating = true
	_ok(_has_blocker("the world is"), "a generating engine is reported (got %s)" % str(_blockers()))
	_app.bridge.generating = false
	_app.new_world_dialog.popup_centered()
	await _frames(3)
	_ok(_has_blocker("the New world form is open"), "an open New world form is reported (got %s)" % str(_blockers()))
	_app.new_world_dialog.hide()
	await _frames(3)
	_ok(_app.vault_window.quiescent_blockers().is_empty(), "the vault with no note open reports nothing")
	_ok(_blockers().is_empty(), "idle again: no blocker (got %s)" % str(_blockers()))
