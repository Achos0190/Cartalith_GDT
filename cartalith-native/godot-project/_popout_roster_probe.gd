extends Node
## Probe for PW-2 of `POPOUT_WINDOWS_SCOPE.md` (Ruling DK): the faction roster as a native, transient,
## non-exclusive window reached by the global setting AND by its own "Pop out / Dock" button, against
## the real shell, the real `PopoutHost`, `PopoutKeyForwarder` and `BusyOverlay`.
##
##   desktop: godot --path . --resolution 1400x900 _popout_roster_probe.tscn -- --form desktop [--shots <dir>]
##   tablet:  godot --path . --resolution 1280x800 _popout_roster_probe.tscn -- --form tablet --force-touch
##   phone:   godot --path . --resolution 393x852  _popout_roster_probe.tscn -- --form phone  --force-touch
##
## **Windowed, never `--headless`** (it opens real native windows and reads their textures). Arguments:
## `--form desktop|tablet|phone` (required; the probe checks the shell booted into that form), `--shots
## <dir>`, `--force-touch` (the shell's own flag) and `--child <dir>` (internal: the bar-d child
## process). Anything else aborts, exit 2. **Run from a scratch worktree with its own `config/name` and a
## wiped user dir**; section 0 fails if it was not clean. One monitor is assumed: only the single-monitor
## halves of the bars are claimed, the second-monitor halves are listed as owner-run in the result.
##
## What it protects (the PW-2 bars):
##   1. TABLES of literals: `LIVE_KEYS`, `effective_wish`, `forward_action`, `scale_for_screen`,
##      `scaled_size` (a term dropped from any of them changes some row).
##   2. THE BUTTON: present on the desktop only, named `PopoutButton`, text "Pop out" docked and "Dock" native,
##      same laid-out width as its invisible balance, Close not moved by it (the pristine-identity pixel
##      probe `_popout_roster_id_probe.gd` carries the framebuffer half).
##   3. bar (g): state across a move, with the setting OFF (manual pop-out) and ON (pop out at open): the
##      selected faction, the tab, both scroll positions, a half-typed field's text; the button and the
##      setting agree; flipping the Preferences row clears manual choices.
##   4. bar (b): Ctrl+Z, Ctrl+Y, Ctrl+Shift+Z and Ctrl+S delivered to the native window's own viewport do
##      to the app exactly what the same event does in the main window; a focused LineEdit keeps its own
##      Ctrl+Z. (The OS-keystroke leg is owner-run: PW-0 could not inject one and a stray keystroke into
##      whatever has the OS focus is not a test.)
##   5. bar (e): the colour picker's popup and the Transfer chooser open at their button, within 2 px, docked and
##      native.
##   6. per-screen scale: a simulated second monitor (1.0 -> 1.5) resizes the window with the factor, the
##      content fits (visible rect 880 x 620) and Close is inside it.
##   7. bar (a): during a generate every control in the native window ignores a click (0 hits against 1
##      without a run), the card mirror is present and changes pixels, a move DURING the run keeps it muted,
##      and the state returns to normal after.
##   8. bar (c): a scripted ten-step faction edit leaves the same civ-state hash docked and native on three
##      seeds, and a world generated with the roster native hashes equal to one generated docked.
##   9. bar (d): a child process that opens the native roster and quits its main tree exits (no orphan).
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument/watchdog.

const KEY := "faction_roster_window"
const SEEDS := [483920, 24601, 71077345]
const GRID_SMALL := Vector2i(160, 120)

var _fails := 0
var _checks := 0
var _form := ""
var _shots := ""
var _child := ""
## `--only a,b`: run just those sections (tables, button, keys, popups, scale, busy, setting, hashes, orphan); the
## mutation runs use it so a mutant of one section is not judged by a 15-minute full run. Empty = all.
var _only := ""
var _app: Node
var _bridge
var _w: FactionRosterWindow


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(n: int = 4) -> void:
	for _i in n:
		await get_tree().process_frame


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks) form=%s" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks, _form])
	get_tree().quit(0 if _fails == 0 else 1)


func _nwin() -> int:
	return DisplayServer.get_window_list().size()


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a in ["--form", "--shots", "--child", "--only"] and i + 1 < args.size():
			match a:
				"--form": _form = String(args[i + 1])
				"--shots": _shots = String(args[i + 1])
				"--child": _child = String(args[i + 1])
				"--only": _only = String(args[i + 1])
			i += 2
		elif a == "--force-touch":
			i += 1
		else:
			print("ABORT: unknown argument %s" % a)
			get_tree().quit(2)
			return
	if not _form in ["desktop", "tablet", "phone"]:
		print("ABORT: --form desktop|tablet|phone is required")
		get_tree().quit(2)
		return
	get_tree().create_timer(1500.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	if _child != "":
		await _child_main()
		return
	print("== 0. clean user dir ==")
	_ok(not FileAccess.file_exists(ProjectSettings.globalize_path(DccSettings.CONFIG_PATH)), "no cartalith_settings.cfg at start (the user dir was wiped)")
	await _boot()
	if _form != "desktop":
		await _forms_other()
		_finish()
		return
	if _want("tables"):
		_tables()
	await _gen(SEEDS[0], GRID_SMALL)
	if _want("button"):
		await _button_and_state()
	if _want("keys"):
		await _keys()
	if _want("popups"):
		await _popups()
	if _want("scale"):
		await _scale()
	if _want("busy"):
		await _busy()
	if _want("setting"):
		await _setting_on()
	if _want("hashes"):
		await _hashes()
	if _want("orphan"):
		await _orphan()
	_finish()


## Whether section `name` runs (`--only` empty means all of them).
func _want(name: String) -> bool:
	return _only == "" or (name in _only.split(","))


func _boot() -> void:
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	var got := "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	_ok(got == _form, "the shell booted into the %s form (got %s)" % [_form, got])
	_bridge = _app.bridge
	_w = _app.faction_roster_window


func _gen(seed: int, grid: Vector2i, poll: Callable = Callable()) -> void:
	_bridge.generate({"seed": seed, "width_km": 1200.0, "grid_w": grid.x, "grid_h": grid.y, "archetype": "", "villages": true})
	var polls := 0
	while _bridge.generating:
		await get_tree().create_timer(0.1).timeout
		polls += 1
		if poll.is_valid():
			await poll.call(polls)
	await _settle(10)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()


# -- 1 ------------------------------------------------------------------------------------

func _tables() -> void:
	print("== 1. tables ==")
	_ok(PopoutHost.LIVE_KEYS == PackedStringArray(["faction_roster_window"]), "LIVE_KEYS is exactly the roster (literal)")
	_ok(PopoutHost.key_for(_w) == KEY, "the roster's window key is '%s'" % KEY)
	var live := PackedStringArray([KEY])
	for r in [[true, true, KEY, true], [false, true, KEY, false], [true, false, KEY, false], [true, true, "other", false]]:
		_ok(PopoutHost.wants_native(r[0], r[1], r[2], live) == r[3], "wants_native(%s, %s, %s) == %s" % [r[0], r[1], r[2], r[3]])
	# effective_wish(setting, has_choice, choice)
	for r in [[false, false, false, false], [true, false, false, true], [false, true, true, true], [true, true, false, false], [false, true, false, false], [true, true, true, true]]:
		_ok(PopoutHost.effective_wish(r[0], r[1], r[2]) == r[3], "effective_wish(setting=%s, has_choice=%s, choice=%s) == %s" % [r[0], r[1], r[2], r[3]])
	# forward_action(keycode, cmd_or_ctrl, shift, alt, typing)
	var fa := [
		[KEY_Z, true, false, false, false, "undo"], [KEY_Z, true, true, false, false, "redo"],
		[KEY_Y, true, false, false, false, "redo"], [KEY_Y, true, true, false, false, ""],
		[KEY_S, true, false, false, false, "save"], [KEY_S, true, true, false, false, ""],
		[KEY_Z, false, false, false, false, ""], [KEY_Z, true, false, true, false, ""],
		[KEY_Z, true, false, false, true, ""], [KEY_Y, true, false, false, true, ""], [KEY_S, true, false, false, true, ""],
		[KEY_X, true, false, false, false, ""], [KEY_Q, true, false, false, false, ""], [KEY_Y, false, false, false, false, ""],
		[KEY_S, false, false, false, false, ""], [KEY_Z, true, true, true, false, ""],
	]
	for r in fa:
		_ok(PopoutHost.forward_action(r[0], r[1], r[2], r[3], r[4]) == r[5], "forward_action(%s, ctrl=%s, shift=%s, alt=%s, typing=%s) == '%s'" % [OS.get_keycode_string(r[0]), r[1], r[2], r[3], r[4], r[5]])
	# scale_for_screen(main_factor, main_screen_scale, win_screen_scale)
	for r in [[1.0, 1.0, 1.0, 1.0], [1.0, 1.0, 1.5, 1.5], [1.0, 2.0, 1.0, 0.5], [1.25, 1.0, 2.0, 2.5], [1.0, 0.0, 1.0, -1.0], [1.0, 1.0, 0.0, -1.0], [0.0, 1.0, 1.0, -1.0], [1.0, -1.0, 1.0, -1.0], [1.0, 1.0, -1.0, -1.0]]:
		_ok(is_equal_approx(PopoutHost.scale_for_screen(r[0], r[1], r[2]), r[3]), "scale_for_screen(%s, %s, %s) == %s" % [r[0], r[1], r[2], r[3]])
	_ok(PopoutHost.scaled_size(Vector2i(880, 620), 1.0, 1.5) == Vector2i(1320, 930), "scaled_size 880x620 from 1 to 1.5 is 1320x930")
	_ok(PopoutHost.scaled_size(Vector2i(880, 620), 1.5, 1.0) == Vector2i(587, 413), "scaled_size 880x620 from 1.5 to 1 is 587x413 (rounded)")
	_ok(PopoutHost.scaled_size(Vector2i(880, 620), 0.0, 1.5) == Vector2i(880, 620), "a zero source factor leaves the size alone")
	_ok(PopoutHost.native_windows(get_tree()).is_empty(), "no native window at boot")
	var fresh := AcceptDialog.new()
	_ok(fresh.transient, "a fresh AcceptDialog is already transient (so PopoutHost.prepare() setting transient = true is redundant for dialogs and a mutant that clears it is equivalent)")
	fresh.free()


func _forms_other() -> void:
	print("== 2'. %s: no pop-out surface at all ==" % _form)
	_ok(not PopoutHost.supported(), "PopoutHost.supported() is false on the %s" % _form)
	_ok(_w.find_child(PopoutHost.BUTTON_NAME, true, false) == null, "the roster has no PopoutButton on the %s" % _form)
	_ok(PopoutHost.add_toggle_button(_w) == null, "add_toggle_button() adds nothing")
	DccSettings.set_popout_enabled(true)
	_w.open()
	await _settle(8)
	_ok(_w.visible and _w.is_embedded() and not _w.force_native and _nwin() == 1, "with the setting ON the roster still opens embedded (window count %d)" % _nwin())
	_w.hide()
	DccSettings.set_popout_enabled(false)


# -- 3 ------------------------------------------------------------------------------------

func _btn() -> Button:
	return _w.find_child(PopoutHost.BUTTON_NAME, true, false) as Button


func _inspector_scroll() -> ScrollContainer:
	return _w._inspector_body.get_parent() as ScrollContainer


func _list_scroll() -> ScrollContainer:
	return _w._list_body.get_parent() as ScrollContainer


func _press(b: Button) -> void:
	b.pressed.emit()


func _snap_state() -> Dictionary:
	return {"sel": _w._selected, "tab": _w._tab, "iv": _inspector_scroll().scroll_vertical, "lv": _list_scroll().scroll_vertical}


func _button_and_state() -> void:
	print("== 2. the button ==")
	DccSettings.set_popout_enabled(false)
	_w.open(3, "economy")
	await _settle(10)
	_ok(_w.visible and _w.is_embedded() and not _w.force_native and _nwin() == 1, "setting off: the roster opens embedded, window count 1")
	var b := _btn()
	_ok(b != null and b.text == "Pop out" and b.visible, "a 'Pop out' button exists beside Close")
	var bal := _w.find_child(PopoutHost.BUTTON_NAME + "Balance", true, false) as Control
	_ok(bal != null and is_equal_approx(bal.custom_minimum_size.x, b.size.x), "the invisible balance is as wide as the button's laid-out width (%s vs %s)" % [str(bal.custom_minimum_size.x if bal else -1), str(b.size.x)])
	_ok(b.size.x >= 70 and b.size.x <= 100, "the button is %d px wide (a literal band, not the constant)" % b.size.x)
	var ok := _w.get_ok_button()
	var mid := ok.get_global_rect().get_center().x
	_ok(absf(mid - 431.0) <= 1.0, "Close is where the pristine tree puts it, centre x 431 (got %.1f)" % mid)

	print("== 3. state across a move, setting OFF (manual pop-out) ==")
	# a state worth keeping: faction 3, Economy tab, both scrollers scrolled, half-typed text
	var isc := _inspector_scroll()
	isc.scroll_vertical = 70
	_list_scroll().scroll_vertical = 0
	await _settle(3)
	var before := _snap_state()
	_ok(before["sel"] == 3 and before["tab"] == "economy" and before["iv"] == 70, "set up: faction 3, tab economy, inspector scroll 70 (%s)" % str(before))
	var field: LineEdit = _first("LineEdit") as LineEdit
	var typed := ""
	if field != null:
		typed = field.text + "~pw2"
		field.text = typed
	await PopoutHost.toggle(_w)
	await _settle(6)
	_ok(not _w.is_embedded() and _w.visible and _nwin() == 2, "the button popped it out: native, window count %d" % _nwin())
	_ok(_w.transient and not _w.exclusive, "transient and non-exclusive")
	_ok(_btn().text == "Dock", "the button now reads 'Dock'")
	var after := _snap_state()
	_ok(after == before, "selected faction, tab and both scroll positions survived the move: %s" % str(after))
	if field != null:
		_ok(is_instance_valid(field) and field.text == typed, "a half-typed field kept its text")
	_ok(PopoutHost.native_windows(get_tree()) == [_w], "native_windows() lists exactly the roster")
	if _shots != "":
		_w.get_texture().get_image().save_png(_shots.path_join("pw2_roster_native.png"))
	await PopoutHost.toggle(_w)
	await _settle(6)
	_ok(_w.is_embedded() and _w.visible and _nwin() == 1, "the button docked it back: embedded, window count %d" % _nwin())
	_ok(_btn().text == "Pop out", "the button reads 'Pop out' again")
	_ok(_snap_state() == before, "state survived the move back: %s" % str(_snap_state()))
	if _shots != "":
		get_viewport().get_texture().get_image().save_png(_shots.path_join("pw2_roster_docked.png"))
	_ok(PopoutHost.native_windows(get_tree()).is_empty(), "docked: native_windows() is empty")
	# the pressed-button path itself, not just toggle()
	_press(_btn())
	await _settle(14)
	_ok(not _w.is_embedded() and _nwin() == 2, "pressing the real button pops it out (count %d)" % _nwin())
	_press(_btn())
	await _settle(14)
	_ok(_w.is_embedded() and _nwin() == 1, "pressing it again docks it (count %d)" % _nwin())
	_w.hide()
	await _settle(3)


# -- 4 ------------------------------------------------------------------------------------

func _key(code: int, ctrl: bool, shift: bool) -> InputEventKey:
	var e := InputEventKey.new()
	e.keycode = code
	e.physical_keycode = code
	e.ctrl_pressed = ctrl
	e.shift_pressed = shift
	e.pressed = true
	return e


func _hist() -> Array:
	return [_app.look_history.can_undo(), _app.look_history.can_redo(), _app.look_history.cursor]


func _reset_history() -> void:
	var lh = _app.look_history
	while lh.can_undo():
		_app.undo_last()
	lh.entries.clear()
	lh.cursor = 0
	for n in 3:
		lh.commit("pw2 step %d" % n, {"zz": n + 1}, _bridge, {"zz": n})


## Sends the same key sequence into `target` (the main viewport or the native window) and returns the
## look-history state after each key, so the two paths are compared step by step.
func _sequence(target: Viewport) -> Array:
	_reset_history()
	var out: Array = [_hist()]
	for k in [[KEY_Z, true, false], [KEY_Z, true, false], [KEY_Y, true, false], [KEY_Z, true, true], [KEY_Z, true, false]]:
		target.push_input(_key(k[0], k[1], k[2]), true)
		await _settle(2)
		out.append(_hist())
	return out


func _keys() -> void:
	print("== 4. keys typed in the native window ==")
	_w.open(1, "identity")
	await _settle(8)
	await PopoutHost.toggle(_w)
	await _settle(8)
	_ok(not _w.is_embedded() and _w.get_node_or_null(PopoutHost.FORWARDER_NAME) != null, "native, with its PopoutKeyForwarder child")
	var main_seq: Array = await _sequence(get_tree().root)
	var pop_seq: Array = await _sequence(_w)
	print("  main window: %s" % str(main_seq))
	print("  pop-out:     %s" % str(pop_seq))
	_ok(main_seq.size() == 6 and main_seq[0] == [true, false, 3], "control: the main window starts at 3 undoable steps")
	_ok(main_seq[1] != main_seq[0] and main_seq[3] != main_seq[2], "control: the main window's sequence moves the history (undo, then redo)")
	_ok(pop_seq == main_seq, "the pop-out's history after each of 5 keys equals the main window's")
	# a focused text field keeps its own Ctrl+Z
	_reset_history()
	var tf := _first("LineEdit") as LineEdit
	if tf != null:
		tf.grab_focus()
		await _settle(2)
		var h0 := _hist()
		_w.push_input(_key(KEY_Z, true, false), true)
		await _settle(2)
		_ok(_hist() == h0, "Ctrl+Z with a LineEdit focused is not forwarded (history %s unchanged)" % str(h0))
		tf.release_focus()
		## The same guard, reached directly: a real Ctrl+Z is consumed by the LineEdit itself and never arrives as
		## "unhandled", so the push above cannot show the forwarder's own typing check. Calling the forwarder with
		## the field still focused does, and a call after the focus is released is the positive control.
		var fw := _w.get_node(PopoutHost.FORWARDER_NAME)
		tf.grab_focus()
		await _settle(2)
		var h1 := _hist()
		fw._unhandled_key_input(_key(KEY_Z, true, false))
		await _settle(2)
		_ok(_hist() == h1, "the forwarder itself refuses Ctrl+Z while a LineEdit has focus (history %s unchanged)" % str(h1))
		tf.release_focus()
		await _settle(2)
		fw._unhandled_key_input(_key(KEY_Z, true, false))
		await _settle(2)
		_ok(_hist() != h1, "control: the same call with nothing focused undoes a step (%s -> %s)" % [str(h1), str(_hist())])
	# Ctrl+S saves exactly as in the main window
	var p1 := ProjectSettings.globalize_path("user://pw2_save_main.zip")
	var p2 := ProjectSettings.globalize_path("user://pw2_save_pop.zip")
	for p in [p1, p2]:
		if FileAccess.file_exists(p):
			DirAccess.remove_absolute(p)
	_app.current_project_path = p1
	get_tree().root.push_input(_key(KEY_S, true, false), true)
	await _wait_save(p1)
	_app.current_project_path = p2
	_w.push_input(_key(KEY_S, true, false), true)
	await _wait_save(p2)
	_ok(FileAccess.file_exists(p1) and FileAccess.get_file_as_bytes(p1).size() > 1000, "control: Ctrl+S in the main window wrote a project (%d bytes)" % FileAccess.get_file_as_bytes(p1).size())
	_ok(FileAccess.file_exists(p2) and FileAccess.get_file_as_bytes(p2).size() > 1000, "Ctrl+S in the pop-out wrote a project (%d bytes)" % FileAccess.get_file_as_bytes(p2).size())
	_app.current_project_path = ""
	# docked: the forwarder stays out of the way
	await PopoutHost.toggle(_w)
	await _settle(6)
	_ok(_w.is_embedded(), "docked again")
	_w.hide()
	await _settle(3)


func _wait_save(path: String) -> void:
	for _i in 60:
		await get_tree().create_timer(0.25).timeout
		if FileAccess.file_exists(path) and not _bridge.generating and not _app.busy_overlay.is_blocking():
			break
	await _settle(4)


# -- 5 ------------------------------------------------------------------------------------

## The button's rectangle in the coordinate frame its popups are positioned in. A popup is embedded in
## the nearest viewport that embeds subwindows: for a DOCKED roster that is the main window, whose frame
## is the roster's rectangle plus the roster's own position; for a NATIVE roster it is the roster itself
## (a native window embeds its own popups), whose frame is its client area. Measured, not assumed:
## `_diag()` prints the facts on every run.
func _frame_rect(c: Control) -> Rect2:
	var r := c.get_global_rect()
	var w := c.get_window()
	if w.is_embedded():
		r.position += Vector2(w.position)
	return r


## The shadow margins `DccWidgets.style_popup` puts around a menu: `shadow_size` 34 and `shadow_offset`
## (0, 14), so the visible panel starts 34 px right of and 20 px below the popup window's own position.
const SHADOW_L := 34.0
const SHADOW_T := 20.0


## A real left click on `c` delivered to its window, so a button's own `pressed()` override runs (a bare
## `pressed.emit()` does not open a ColorPickerButton's popup).
func _click(c: Control) -> void:
	## An embedded window does not take a local-coordinate event pushed into its own viewport (measured:
	## a docked tab click did nothing), so a docked control is clicked through the MAIN window at the
	## control's position in the main window's frame; a native one is clicked in its own viewport.
	var w := c.get_window()
	var target: Viewport = get_tree().root if w.is_embedded() else w
	var p: Vector2 = _frame_rect(c).get_center() if w.is_embedded() else c.get_global_rect().get_center()
	for pressed in [true, false]:
		var e := InputEventMouseButton.new()
		e.button_index = MOUSE_BUTTON_LEFT
		e.pressed = pressed
		e.position = p
		e.global_position = p
		target.push_input(e, true)
		await _settle(2)


func _popups() -> void:
	print("== 5. popups open at their button, docked and native ==")
	var colour_rel := {}
	for mode in ["docked", "native"]:
		_w.open(1, "identity")
		await _settle(8)
		if mode == "native":
			await PopoutHost.toggle(_w)
			await _settle(8)
		_ok((mode == "native") == (not _w.is_embedded()), "%s: the roster is %s" % [mode, mode])
		var tabb: Button = _w._tab_buttons["relations"]
		await _click(tabb)
		print("  %s: real click on the Relations tab -> tab=%s (window embedded=%s)" % [mode, _w._tab, str(_w.is_embedded())])
		_w._select_tab("identity")
		await _settle(6)
		var picker := _first("ColorPickerButton") as ColorPickerButton
		_ok(picker != null, "%s: a colour picker button exists" % mode)
		if picker != null:
			await _click(picker)
			await _settle(6)
			print("  %s colour after click 1: visible=%s pressed=%s pos=%s rect=%s" % [mode, str(picker.get_popup().visible), str(picker.button_pressed), str(picker.get_popup().position), str(picker.get_global_rect())])
			await _settle(8)
			var pop := picker.get_popup()
			_ok(pop.visible, "%s: the colour popup is visible after a real click" % mode)
			var br := _frame_rect(picker)
			var rel := Vector2(pop.position) - br.position
			colour_rel[mode] = rel
			print("  %s colour: %s" % [mode, _diag(pop, picker)])
			print("  %s colour: popup minus button in its frame = %s" % [mode, str(rel)])
			var pr := Rect2(Vector2(pop.position), Vector2(pop.size))
			## The engine's own rule for this popup, derived from the two measured placements (docked
			## 661,276 and native 401,0 of a 298 x 624 popup): centred on the button horizontally,
			## directly below it, else directly above it, else bottom-aligned to the frame and never
			## above 0; all clamped inside the frame the popup is embedded in (the main window docked, the
			## roster window native). The expected position is computed from that rule and the frame size.
			var frame := get_tree().root.get_visible_rect().size if mode == "docked" else _w.get_visible_rect().size
			var pw := float(pop.size.x)
			var ph := float(pop.size.y)
			var ex := clampf(br.get_center().x - pw * 0.5, 0.0, maxf(0.0, frame.x - pw))
			var below := br.end.y
			var ey := below if below + ph <= frame.y else (br.position.y - ph if br.position.y - ph >= 0.0 else maxf(0.0, frame.y - ph))
			_ok(absf(float(pop.position.x) - ex) <= 2.0 and absf(float(pop.position.y) - ey) <= 2.0,
				"%s: the colour popup opens where the engine's rule puts it for its button within 2 px (popup %s, expected (%.0f, %.0f), frame %s)" % [mode, str(pop.position), ex, ey, str(frame)])
			colour_rel[mode] = Vector2(pop.position)
			if _shots != "" and mode == "native":
				_w.get_texture().get_image().save_png(_shots.path_join("pw2_roster_native_colour_popup.png"))
			pop.hide()
			await _settle(3)
		_w._select_tab("settlements")
		await _settle(8)
		var t := _w.find_child("Transfer_0", true, false) as Button
		_ok(t != null, "%s: a Transfer button exists on the Settlements tab" % mode)
		if t != null:
			var n0 := _nwin()
			t.pressed.emit()
			await _settle(8)
			var pm := _w.get_node_or_null("TransferMenu") as PopupMenu
			_ok(pm != null and pm.visible, "%s: the Transfer menu is visible" % mode)
			if pm != null:
				var br2 := _frame_rect(t)
				var want := Vector2(br2.position.x - SHADOW_L, br2.end.y + 4.0 - SHADOW_T)
				print("  %s transfer: windows %d->%d  %s" % [mode, n0, _nwin(), _diag(pm, t)])
				_ok(absf(float(pm.position.x) - want.x) <= 2.0 and absf(float(pm.position.y) - want.y) <= 2.0,
					"%s: the Transfer menu's panel opens under its button within 2 px (popup %s, expected %s)" % [mode, str(pm.position), str(want)])
				if _shots != "" and mode == "native":
					_w.get_texture().get_image().save_png(_shots.path_join("pw2_roster_native_transfer_menu.png"))
				pm.hide()
				await _settle(3)
		if mode == "native":
			await PopoutHost.toggle(_w)
			await _settle(6)
		_w.hide()
		await _settle(3)


# -- 6 ------------------------------------------------------------------------------------

func _scale() -> void:
	print("== 6. per-screen scale (a simulated second monitor of scale 1.5) ==")
	_w.open(1, "identity")
	await _settle(8)
	await PopoutHost.toggle(_w)
	await _settle(8)
	var s0 := _w.size
	_ok(_w.content_scale_factor == 1.0, "on this one monitor the native window keeps factor 1.0")
	PopoutHost.fit_scale(_w, true, 1.0, 1.0)
	await _settle(4)
	_ok(_w.content_scale_factor == 1.0 and _w.size == s0, "equal screen scales change nothing (size %s)" % str(_w.size))
	PopoutHost.fit_scale(_w, true, 1.0, 1.5)
	await _settle(8)
	_ok(is_equal_approx(_w.content_scale_factor, 1.5), "scale 1.5 on the window's screen gives factor 1.5 (%s)" % str(_w.content_scale_factor))
	_ok(_w.size == PopoutHost.scaled_size(s0, 1.0, 1.5), "the window grew with the factor: %s from %s" % [str(_w.size), str(s0)])
	var vr := _w.get_visible_rect().size
	_ok(absf(vr.x - float(s0.x)) <= 1.5 and absf(vr.y - float(s0.y)) <= 1.5, "the logical visible rect is the authored %s (got %s), so nothing is cropped" % [str(s0), str(vr)])
	var ok := _w.get_ok_button().get_global_rect()
	_ok(Rect2(Vector2.ZERO, vr).encloses(ok), "Close lies inside the visible rect (%s in %s)" % [str(ok), str(vr)])
	_ok(Rect2(Vector2.ZERO, vr).encloses(_btn().get_global_rect()), "the Pop out / Dock button lies inside it too")
	if _shots != "":
		_w.get_texture().get_image().save_png(_shots.path_join("pw2_roster_native_scale_1_5_simulated.png"))
	PopoutHost.fit_scale(_w, true, 1.0, 1.0)
	await _settle(6)
	_ok(is_equal_approx(_w.content_scale_factor, 1.0) and _w.size == s0, "back to equal scales: factor 1.0, size %s" % str(_w.size))
	_ok(PopoutHost.scale_for_screen(1.0, 1.0, 0.0) < 0.0, "a screen that reports no scale answers -1, not a plausible 1.0")
	await PopoutHost.toggle(_w)
	await _settle(6)
	_w.hide()
	await _settle(3)


# -- 7 ------------------------------------------------------------------------------------

## Click the Economy tab button in `_w` with a pushed event; returns the number of `pressed` hits.
func _click_count() -> int:
	var tb: Button = _w._tab_buttons["economy"]
	var hits := [0]
	var cb := func() -> void: hits[0] += 1
	tb.pressed.connect(cb)
	var c := tb.get_global_rect().get_center()
	for pressed in [true, false]:
		var e := InputEventMouseButton.new()
		e.button_index = MOUSE_BUTTON_LEFT
		e.pressed = pressed
		e.position = c
		e.global_position = c
		_w.push_input(e, true)
		await _settle(1)
	tb.pressed.disconnect(cb)
	return hits[0]


func _pixels_differ(a: Image, b: Image) -> int:
	if a.get_size() != b.get_size():
		return -1
	a.convert(Image.FORMAT_RGBA8)
	b.convert(Image.FORMAT_RGBA8)
	var n := 0
	for y in a.get_height():
		for x in a.get_width():
			if a.get_pixel(x, y) != b.get_pixel(x, y):
				n += 1
	return n


func _busy() -> void:
	print("== 7. the busy overlay and a native window ==")
	_w.open(1, "identity")
	await _settle(8)
	await PopoutHost.toggle(_w)
	await _settle(8)
	_ok(not _w.is_embedded(), "native")
	_ok(await _click_count() == 1, "control: with no run, a click lands in the native window (1 hit)")
	var st := {"checked": false, "muted": false, "blocked": false, "mirror": 0, "hits": -1, "diff": -1, "moved_muted": false, "moved_native": false}
	var poll := func(n: int) -> void:
		if st["checked"] or n < 3:
			return
		st["checked"] = true
		st["blocked"] = _app.busy_overlay.is_blocking() and _bridge.generating
		st["muted"] = _w.gui_disable_input
		st["mirror"] = _app.busy_overlay.mirror_count()
		st["hits"] = await _click_count()
		var with_card := _w.get_texture().get_image()
		var mirror := _w.get_node_or_null("BusyOverlayMirror") as CanvasItem
		if mirror != null:
			mirror.visible = false
			await _settle(3)
			var without := _w.get_texture().get_image()
			st["diff"] = _pixels_differ(with_card, without)
			mirror.visible = true
			await _settle(2)
		if _shots != "":
			with_card.save_png(_shots.path_join("pw2_roster_native_during_generate.png"))
		# a move DURING the run keeps it muted (the state survives the toggle)
		await PopoutHost.toggle(_w)
		await _settle(4)
		st["moved_muted"] = _w.is_embedded() and _w.gui_disable_input
		if _shots != "":
			get_viewport().get_texture().get_image().save_png(_shots.path_join("pw2_roster_docked_during_generate.png"))
		await PopoutHost.toggle(_w)
		await _settle(4)
		st["moved_native"] = (not _w.is_embedded()) and _w.gui_disable_input
	await _gen(24601, Vector2i(512, 384), poll)
	_ok(st["checked"], "the in-run checks ran (the generate lasted long enough)")
	_ok(st["blocked"], "the overlay was blocking and the engine generating at the check")
	_ok(st["muted"], "the native window's gui_disable_input was true during the run")
	_ok(int(st["mirror"]) >= 1, "the overlay carries a card mirror into it (mirror_count %d)" % int(st["mirror"]))
	_ok(st["hits"] == 0, "a click during the run reached 0 controls in the native window (control above: 1)")
	_ok(int(st["diff"]) > 500, "the card mirror changes %d pixels of the native window's own texture (flip-the-flag)" % int(st["diff"]))
	_ok(st["moved_muted"], "popped back in DURING the run, the window is still muted")
	_ok(st["moved_native"], "popped out again DURING the run, it is still muted")
	_ok(not _w.gui_disable_input, "after the run the window takes input again")
	_ok(_app.busy_overlay.mirror_count() == 0, "and no mirror is left (%d)" % _app.busy_overlay.mirror_count())
	await _settle(6)
	_ok(await _click_count() == 1, "after the run a click lands again (1 hit)")
	# bar (a) still holds after a toggle: dock, re-pop, run again
	await PopoutHost.toggle(_w)
	await _settle(6)
	await PopoutHost.toggle(_w)
	await _settle(8)
	var st2 := {"hits": -1, "checked": false}
	var poll2 := func(n: int) -> void:
		if st2["checked"] or n < 3:
			return
		st2["checked"] = true
		st2["hits"] = await _click_count()
	await _gen(71077345, Vector2i(512, 384), poll2)
	_ok(st2["checked"] and st2["hits"] == 0, "after a dock and re-pop the native window is muted again during a run (hits %d)" % int(st2["hits"]))
	await PopoutHost.toggle(_w)
	await _settle(6)
	_w.hide()
	await _settle(3)


# -- 8 ------------------------------------------------------------------------------------

func _setting_on() -> void:
	print("== 3b. state across a move, setting ON (pops out at open) ==")
	var idx := CommandIndex.new()
	idx.build(_app, _bridge)
	var row := {}
	for r in idx.all():
		if String(r["title"]) == "Open panels in their own window":
			row = r
	_ok(not row.is_empty(), "the Preferences row exists")
	var popup: PopupMenu = row["_popup"]
	var id := int(row["_id"])
	DccSettings.set_popout_enabled(false)
	popup.id_pressed.emit(id)
	_ok(DccSettings.popout_enabled(), "the Preferences row turned the setting ON")
	await _gen(SEEDS[0], GRID_SMALL)
	_w.open(2, "military")
	await _settle(12)
	_ok(_w.visible and not _w.is_embedded() and _nwin() == 2, "setting on: open() pops the roster out (count %d)" % _nwin())
	_ok(_btn().text == "Dock", "and the button reads 'Dock'")
	_ok(_w._selected == 2 and _w._tab == "military", "it opened on faction 2, tab military")
	_inspector_scroll().scroll_vertical = 60
	await PopoutHost.toggle(_w)
	await _settle(8)
	_ok(_w.is_embedded() and _w._selected == 2 and _w._tab == "military", "Dock with the setting ON: embedded, same faction and tab")
	_w.hide()
	await _settle(3)
	_w.open(2, "military")
	await _settle(10)
	_ok(_w.is_embedded(), "reopened: the manual Dock choice holds for the session even with the setting on")
	_w.hide()
	await _settle(3)
	popup.id_pressed.emit(id)
	_ok(not DccSettings.popout_enabled(), "the row turned the setting OFF")
	popup.id_pressed.emit(id)
	_ok(DccSettings.popout_enabled(), "and ON again")
	_ok(not _w.has_meta(PopoutHost.META_CHOICE), "flipping the setting cleared the window's manual choice")
	_w.open(2, "military")
	await _settle(12)
	_ok(not _w.is_embedded() and _nwin() == 2, "after the flip the window follows the setting again (native)")
	_w.hide()
	await _settle(4)
	_ok(_nwin() == 1, "hiding it leaves no OS window (count %d)" % _nwin())
	popup.id_pressed.emit(id)
	_ok(not DccSettings.popout_enabled(), "the setting is OFF for the rest of the run")
	_w.open(2, "military")
	await _settle(10)
	_ok(_w.is_embedded() and _nwin() == 1, "setting off, no choice: embedded again, count 1")
	_w.hide()
	await _settle(3)


# -- 9 ------------------------------------------------------------------------------------

func _civ_hash() -> String:
	var b := JSON.stringify([_bridge.get_factions(), _bridge.settlements().size(), _bridge.civ_faction_currency(1), _bridge.civ_trade_tariff(1, 2)])
	return b.sha256_text().substr(0, 16)


func _world_hash() -> String:
	## A fingerprint of the generated terrain and civilisation: `sample_cell` over an 11 x 9 lattice of the
	## grid plus the faction list. `export_heightmap_png` was tried first and wrote nothing here. The
	## length is part of the answer so a silently empty fingerprint cannot compare equal to itself.
	var parts: Array = []
	for gy in range(5, GRID_SMALL.y, 13):
		for gx in range(5, GRID_SMALL.x, 15):
			parts.append(_bridge.sample_cell(gx, gy))
	parts.append(_bridge.get_factions())
	var s := JSON.stringify(parts)
	return s.sha256_text().substr(0, 16) + ":" + str(s.length())


func _ten_steps() -> void:
	_w._add_faction()                           # 1
	await _settle(2)
	_w._select_tab("economy")                   # 2
	await _settle(2)
	_w._set_currency("name", "Quid", null)      # 3
	_w._set_currency("symbol", "Q", null)       # 4
	_w._set_currency("rate", "2.5", null)       # 5
	_w._set_tariff(1, "12", null)               # 6
	_w._select_tab("identity")                  # 7
	await _settle(2)
	_w._set_field("name", "Pwtwoland")          # 8
	_w._select_tab("relations")                 # 9
	await _settle(2)
	_w._selected = 1                            # 10
	_w._rebuild()
	await _settle(2)


func _hashes() -> void:
	print("== 8. civ state and generated world, docked versus native ==")
	DccSettings.set_popout_enabled(false)
	for seed in SEEDS:
		var res := {}
		for mode in ["docked", "native"]:
			await _gen(seed, GRID_SMALL)
			_w.open(2, "identity")
			await _settle(8)
			if mode == "native":
				await PopoutHost.toggle(_w)
				await _settle(8)
			_ok((mode == "native") == (not _w.is_embedded()), "seed %d %s: roster is %s" % [seed, mode, mode])
			var pre := _civ_hash()
			await _ten_steps()
			_ok(pre != _civ_hash(), "control: the ten steps changed the civ state (%s -> %s), so equal hashes compare something" % [pre, _civ_hash()])
			res[mode] = [_civ_hash(), _world_hash()]
			if mode == "native":
				await PopoutHost.toggle(_w)
				await _settle(4)
			_w.hide()
			await _settle(3)
		print("  seed %d: docked %s native %s" % [seed, str(res["docked"]), str(res["native"])])
		_ok(res["docked"][0] != "" and res["docked"][0] == res["native"][0], "seed %d: the civ-state hash after ten edits is equal docked and native (%s)" % [seed, res["docked"][0]])
		_ok(res["docked"][1] == res["native"][1], "seed %d: the world hash is equal (%s)" % [seed, res["docked"][1]])
	# a world GENERATED while the roster is native hashes equal to one generated docked
	var gen := {}
	for mode in ["docked", "native"]:
		_w.open(1, "identity")
		await _settle(8)
		if mode == "native":
			await PopoutHost.toggle(_w)
			await _settle(8)
		await _gen(SEEDS[0], GRID_SMALL)
		gen[mode] = _world_hash()
		if mode == "native":
			await PopoutHost.toggle(_w)
			await _settle(4)
		_w.hide()
		await _settle(3)
	_ok(gen["docked"] == gen["native"], "a world generated with the roster native equals one generated docked (%s)" % gen["docked"])
	_ok(gen["docked"].split(":")[1].to_int() > 3000, "control: the fingerprint covered real cells (%s)" % gen["docked"])


# -- 10 -----------------------------------------------------------------------------------

func _orphan() -> void:
	print("== 9. closing the main window closes the pop-out ==")
	var dir := ProjectSettings.globalize_path("user://pw2_child")
	DirAccess.make_dir_recursive_absolute(dir)
	var marker := dir.path_join("child.txt")
	if FileAccess.file_exists(marker):
		DirAccess.remove_absolute(marker)
	var pid := OS.create_process(OS.get_executable_path(), ["--path", ProjectSettings.globalize_path("res://"), "--resolution", "900x640", "res://_popout_roster_probe.tscn", "--", "--form", "desktop", "--child", dir])
	_ok(pid > 0, "a child process started (pid %d)" % pid)
	var t := 0.0
	while OS.is_process_running(pid) and t < 120.0:
		await get_tree().create_timer(0.5).timeout
		t += 0.5
	_ok(not OS.is_process_running(pid), "the child process has exited (%.1f s): no orphan window or process" % t)
	var txt := FileAccess.get_file_as_string(marker) if FileAccess.file_exists(marker) else ""
	_ok(txt.begins_with("native windows=2"), "control: the child had really reached 2 OS windows before it quit ('%s')" % txt.strip_edges())


func _child_main() -> void:
	await _boot()
	await _gen(SEEDS[1], GRID_SMALL)
	_w.open(1, "identity")
	await _settle(8)
	await PopoutHost.toggle(_w)
	await _settle(10)
	var f := FileAccess.open(_child.path_join("child.txt"), FileAccess.WRITE)
	f.store_string("native windows=%d embedded=%s\n" % [_nwin(), str(_w.is_embedded())])
	f.close()
	await get_tree().create_timer(0.5).timeout
	get_tree().quit(0)


## First descendant of the roster of class `cls` (find_child has no class filter), or null.
func _first(cls: String) -> Node:
	var l := _w.find_children("*", cls, true, false)
	return l[0] if not l.is_empty() else null


## One diagnostic line about where a popup really is, so a failing popup bar is explainable from the log.
func _diag(p: Window, c: Control) -> String:
	return "popup.position=%s popup.size=%s embedded=%s force_native=%s | control window=%s pos=%s embedded=%s | root.position=%s | control.global=%s screen_position=%s" % [
		str(p.position), str(p.size), str(p.is_embedded()), str(p.force_native), str(c.get_window().name), str(c.get_window().position),
		str(c.get_window().is_embedded()), str(get_tree().root.position), str(c.get_global_rect().position), str(c.get_screen_position())]
