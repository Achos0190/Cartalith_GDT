extends Node
## Ruling EA item 11 (2026-11-01, *"Yes, same monitor"*): a dialog a pop-out spawns opens on the asking
## window's monitor -- inside that window where it fits, as its own window on that screen where it would
## not (`OUTSTANDING_WORK.md`, "Dialogs a pop-out spawns open on the asking window's monitor";
## `POPOUT_WINDOWS_SCOPE.md` s.11 finding 5 and 6). Against the real shell, the real call sites, real mouse events
## and the real busy overlay.
##
##   godot --path . --resolution 1600x1000 _popdlg_probe.tscn -- --form desktop [--shots <dir>]
##   godot --headless --path . _popdlg_probe.tscn -- --pure          (the literal tables only, no window)
##
## **Windowed**, desktop only (a tablet and a phone never pop out; `_pw5_probe.gd` proves their zero-native
## state and the unchanged `present()` fallback). Run from a scratch tree with a wiped user dir. Exit 0/1, 2 abort.
##
## **One monitor is attached.** "Opens on the asking window's monitor" is therefore proved the way one monitor
## can: the dialog's parent IS the pop-out, an embedded dialog is inside the pop-out's rectangle, and a dialog that
## became its own native window reports the SAME `current_screen` as its asker and sits inside that screen's usable
## area. The choice of screen for a different monitor is exercised only on SIMULATED screen rectangles (`centred_in`
## and `dialog_fits`, pure). Dragging a pop-out to monitor 2 and opening a dialog there is OWNER-RUN.
##
## What it protects (each check names it):
##   pure     `dialog_fits()` and `centred_in()` by literal rows.
##   docked   with every window docked, each site's dialog is a child of the APP and embedded: unchanged.
##   native   with the window popped out, the SAME call sites put the dialog under the pop-out, embedded in it, and
##            no extra OS window opens: roster Remove, place Delete, vault preview and Compare, the assistant's
##            consent card, the planner's Save journey, the vault's folder chooser.
##   own      a dialog too big for its pop-out opens as its own transient native window on the asker's screen, inside
##            its usable area, muted and mirrored by the busy overlay during a generate, real-click confirmable, and
##            gone (window count back) after it.
##   reshape  `PopoutHost.reshape()` on an own-window dialog moves/resizes it inside the usable area.

const KEY_ROSTER := "faction_roster_window"
const VAULT_DIR := "user://popdlg_probe_vault"

var _fails := 0
var _checks := 0
var _shots := ""
var _app: Node
var _bridge


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(n: int = 4) -> void:
	for _i in n:
		await get_tree().process_frame


func _until(cond: Callable, ms: int = 8000) -> bool:
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < ms:
		if cond.call():
			return true
		await get_tree().process_frame
	return bool(cond.call())


func _nwin() -> int:
	return DisplayServer.get_window_list().size()


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--form" and i + 1 < args.size():
			if String(args[i + 1]) != "desktop":
				print("ABORT: desktop only")
				get_tree().quit(2)
				return
			i += 2
		elif a == "--shots" and i + 1 < args.size():
			_shots = String(args[i + 1])
			i += 2
		elif a == "--pure":
			# the pure tables only (no window, runs headless): the mutation runs on `dialog_fits` / `centred_in` use it
			_pure()
			print("RESULT: %s (%d failed, %d checks) pure" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
			get_tree().quit(0 if _fails == 0 else 1)
			return
		else:
			print("ABORT: unknown argument %s" % a)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("ABORT: run windowed")
		get_tree().quit(2)
		return
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	print("== 0. clean user dir; screens: %d ==" % DisplayServer.get_screen_count())
	_ok(not FileAccess.file_exists(ProjectSettings.globalize_path(DccSettings.CONFIG_PATH)), "no cartalith_settings.cfg at start")
	_pure()
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_ok(not _app.is_phone() and not DccTheme.is_tablet(), "the shell booted into the desktop form")
	_bridge = _app.bridge
	_bridge.generate({"seed": 483920, "width_km": 1200.0, "grid_w": 160, "grid_h": 120, "archetype": "", "villages": true})
	while _bridge.generating:
		await get_tree().create_timer(0.1).timeout
	await _settle(10)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	_connect_vault()
	await _sites("docked")
	await _sites("native")
	await _own_window()
	await _reshape()
	await _hint()
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


# -- pure -----------------------------------------------------------------------------------------

func _pure() -> void:
	print("== 1. pure: dialog_fits and centred_in by literal rows ==")
	# need, host, expected   (DIALOG_MARGIN is 16 px a side)
	for r in [[Vector2i(380, 200), Vector2i(880, 620), true], [Vector2i(848, 200), Vector2i(880, 620), true],
			[Vector2i(849, 200), Vector2i(880, 620), false], [Vector2i(380, 588), Vector2i(880, 620), true],
			[Vector2i(380, 589), Vector2i(880, 620), false], [Vector2i(380, 200), Vector2i(0, 620), false],
			[Vector2i(380, 200), Vector2i(880, 0), false], [Vector2i(1200, 200), Vector2i(880, 620), false]]:
		_ok(PopoutHost.dialog_fits(r[0], r[1]) == bool(r[2]), "dialog_fits(%s in %s) == %s" % [r[0], r[1], r[2]])
	_ok(PopoutHost.DIALOG_MARGIN == 16, "the margin is the labelled 16")
	# SIMULATED second monitor, to the right of a 1920 one: centred inside ITS usable area
	var u2 := Rect2i(1920, 0, 2560, 1400)
	var c := PopoutHost.centred_in(u2, Vector2i(600, 400))
	_ok(c == Rect2i(2900, 516, 600, 400), "centred on a SIMULATED monitor 2: %s" % str(c))
	c = PopoutHost.centred_in(Rect2i(-1920, 0, 1920, 1040), Vector2i(600, 400))
	_ok(c == Rect2i(-1260, 336, 600, 400), "centred on a SIMULATED negative-coordinate monitor: %s" % str(c))
	c = PopoutHost.centred_in(Rect2i(1920, 0, 800, 600), Vector2i(5000, 5000))
	_ok(c.size == Vector2i(800, 568) and Rect2i(1920, 0, 800, 600).encloses(c), "an oversize dialog is shrunk to the screen: %s" % str(c))


# -- vault ------------------------------------------------------------------------------------------

func _connect_vault() -> void:
	var dir := ProjectSettings.globalize_path(VAULT_DIR)
	DirAccess.make_dir_recursive_absolute(dir)
	for e in [["Veldmark.md", "# Veldmark\nMy own notes about the port.\n"], ["Other.md", "# Other\nA second note.\n"]]:
		var f := FileAccess.open(dir.path_join(e[0]), FileAccess.WRITE)
		f.store_string(e[1])
		f.close()
	var info: Dictionary = _bridge.vault_connect(dir, "popdlg probe")
	_ok(bool(info.get("ok", false)), "a probe Vault is connected")


# -- the sites ----------------------------------------------------------------------------------------

## Every `Window` node in the tree right now.
func _all_windows() -> Array:
	return get_tree().root.find_children("*", "Window", true, false)


## Runs `trigger`, waits, and returns the dialog it made: the one new visible Window that is not a listed pop-out.
func _spawn(trigger: Callable) -> Window:
	var before := _all_windows()
	trigger.call()
	await _settle(8)
	var found: Window = null
	for w in _all_windows():
		if not before.has(w) and (w as Window).visible:
			found = w
	return found


## The sites, one row each: name, the window that asks (must be open), a trigger, and how to close the dialog.
func _site_rows() -> Array:
	return [
		{"name": "roster Remove", "win": _app.faction_roster_window,
			"open": func() -> void:
				_app.faction_roster_window.open(1, "identity")
				while _bridge.civ_faction_count() < 3:
					_bridge.civ_add_faction(),
			"trigger": func() -> void: _app.faction_roster_window._confirm_remove()},
		{"name": "place editor Delete", "win": _app.place_editor_window,
			"open": func() -> void: _app.place_editor_window.open_for(0),
			"trigger": func() -> void: _app.place_editor_window.confirm_delete(0)},
		{"name": "vault Compare", "win": _app.vault_window,
			"open": func() -> void: _app.vault_window.open_browse(),
			"trigger": func() -> void: _app.vault_window._compare_dialog("L1", "Veldmark.md", [{"op": "eq", "a": "x", "b": "x"}], false)},
		{"name": "vault write preview", "win": _app.vault_window,
			"open": func() -> void: _app.vault_window.open_browse(),
			"trigger": func() -> void: _app.vault_window._preview_dialog("Write preview", "a\nb\n", "A probe note", func() -> void: pass)},
		{"name": "vault folder chooser", "win": _app.vault_window,
			"open": func() -> void: _app.vault_window.open_browse(),
			"trigger": func() -> void: _app.vault_window._browse_vault()},
		{"name": "planner Save journey", "win": _app.journey_planner_view.window(),
			"open": func() -> void: _app.open_journey_planner(),
			"trigger": func() -> void: _app.journey_planner_view._save_journey()},
		{"name": "assistant consent", "win": _app.assistant_window,
			"open": func() -> void:
				# a configured endpoint that has not been consented to: Send then shows the consent card
				# (nothing is sent; no server is contacted before the card is answered)
				DccSettings.set_llm_value("prose", true)
				DccSettings.set_llm_value("local_allowed", true)
				DccSettings.set_llm_value("preset_id", "ollama")
				DccSettings.set_llm_value("model", "probe-model")
				DccSettings.set_llm_value("enabled", true)
				DccSettings.set_llm_value("base_url", "http://127.0.0.1:1234/v1")
				DccSettings.llm_withdraw_consent()
				_app.assistant_window.open_window(),
			"trigger": func() -> void:
				_app.assistant_window._pick_task("story")
				_app.assistant_window._input.text = "hello"
				_app.assistant_window.last_consent = {}
				_app.assistant_window._on_send()},
	]


func _close_dialogs(dlg: Window) -> void:
	if dlg != null and is_instance_valid(dlg):
		dlg.hide()
		dlg.queue_free()
	await _settle(4)


func _sites(mode: String) -> void:
	print("== %s: the same call sites with the windows %s ==" % ["2" if mode == "docked" else "3", mode.to_upper()])
	DccSettings.set_popout_enabled(mode == "native")
	var seen_names := PackedStringArray()
	for row in _site_rows():
		var w: Window = row["win"]
		(row["open"] as Callable).call()
		await _settle(8)
		if mode == "native":
			await _until(func() -> bool: return not w.is_embedded() and w.visible)
		if mode == "docked":
			_ok(PopoutHost.dialog_host(_app, w) == _app, "%s: dialog_host() is the app while the asker is docked" % row["name"])
		_ok(w.visible and (w.is_embedded() == (mode == "docked")), "%s: the asking window is %s" % [row["name"], "docked (embedded)" if mode == "docked" else "a native pop-out"])
		var n0 := _nwin()
		var dlg: Window = await _spawn(row["trigger"])
		if dlg == null:
			var aw = _app.assistant_window
			print("  diag: %s win visible=%s embedded=%s gate=%s ep=%s" % [row["name"], w.visible, w.is_embedded(), str(aw._gate()) if row["name"] == "assistant consent" else "-", str(_bridge.llm_endpoint_info(String(DccSettings.llm_value("preset_id")), String(DccSettings.llm_value("base_url")))) if row["name"] == "assistant consent" else "-"])
			_ok(false, "%s: a dialog opened" % row["name"])
			w.hide()
			continue
		seen_names.append(String(row["name"]))
		if mode == "docked":
			## 2026-11-11 (assistant-on-tablet fix): a dialog built by the shared helpers (`DccWidgets.stack_host`) and
			## opened FROM an exclusive embedded window is that window's child, because the engine allows one exclusive
			## child per parent and refused the consent card over the assistant window. Hand-built dialogs elsewhere
			## stay the app's, unchanged. So the parent is the app or the asker, and the assistant consent must be the asker.
			var pn: Node = dlg.get_parent()
			_ok(pn == _app or pn == w, "%s: docked, the dialog is a child of the app or of its asker: parent %s" % [row["name"], pn.name])
			if String(row["name"]) == "assistant consent" and w.exclusive:
				_ok(pn == w, "assistant consent: opened over the exclusive assistant window it is that window's child (the engine allows one exclusive child per parent)")
			_ok(dlg.is_embedded() and _nwin() == n0, "%s: docked, embedded, no extra OS window (%d)" % [row["name"], _nwin()])
		else:
			_ok(dlg.get_parent() == w, "%s: popped out, the dialog is a child of the POP-OUT, not the app (parent %s)" % [row["name"], dlg.get_parent().name])
			if dlg.is_embedded():
				_ok(_nwin() == n0, "%s: embedded in the pop-out, no extra OS window (%d)" % [row["name"], _nwin()])
				var r := Rect2i(dlg.position, dlg.size)
				var host := Rect2i(Vector2i.ZERO, w.size)
				_ok(host.encloses(r) or r.intersection(host).get_area() * 2 > r.get_area(), "%s: its rectangle %s lies in the pop-out %s" % [row["name"], str(r), str(host)])
			else:
				_ok(dlg.current_screen == w.current_screen, "%s: too big for the pop-out, so its own window, on the pop-out's screen %d (is %d)" % [row["name"], w.current_screen, dlg.current_screen])
				_ok(DisplayServer.screen_get_usable_rect(w.current_screen).encloses(Rect2i(dlg.position, dlg.size)), "%s: inside that screen's usable area" % row["name"])
			if _shots != "":
				var img := w.get_texture().get_image()
				img.save_png(_shots.path_join("dlg_%s_%s.png" % [mode, String(row["name"]).replace(" ", "_")]))
		await _close_dialogs(dlg)
		w.hide()
		await _settle(4)
	_ok(seen_names.size() == _site_rows().size(), "every site produced a dialog (%d of %d): %s" % [seen_names.size(), _site_rows().size(), ", ".join(seen_names)])
	_ok(_nwin() == 1, "no OS window left behind (%d)" % _nwin())
	DccSettings.set_popout_enabled(false)


# -- own window ------------------------------------------------------------------------------------------

func _click(win: Window, pos: Vector2) -> void:
	for pressed in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = pressed
		ev.position = pos
		ev.global_position = pos
		win.push_input(ev)
		await get_tree().process_frame


func _own_window() -> void:
	print("== 4. a dialog too big for its pop-out is its own native window on the asker's screen ==")
	DccSettings.set_popout_enabled(true)
	var w: Window = _app.faction_roster_window
	w.open(1, "identity")
	await _until(func() -> bool: return not w.is_embedded() and w.visible)
	await _settle(6)
	_ok(not w.is_embedded() and _nwin() == 2, "the roster is a native pop-out (count %d)" % _nwin())
	var res := {"confirmed": false, "cancelled": false}
	var wide := PopoutHost.dialog_host(_app, w)
	_ok(wide == w, "dialog_host(app, roster) is the pop-out itself")
	var dlg := DccWidgets.confirm(wide, "Own window", "This confirm is wider than the pop-out.", "Do it",
		func() -> void: res["confirmed"] = true, func() -> void: res["cancelled"] = true, 1500)
	await _settle(10)
	_ok(not dlg.is_embedded() and dlg.force_native, "the 1500 px confirm in an %d px pop-out is a native window of its own" % w.size.x)
	_ok(_nwin() == 3, "OS window count is 3 (main, pop-out, dialog): %d" % _nwin())
	_ok(dlg.size.x == 1500 and dlg.min_size.x == 1500, "it kept the 1500 px it asked for, not the pop-out's 880 (size %s, min %s): the request comes from the caller" % [str(dlg.size), str(dlg.min_size)])
	_ok(dlg.get_parent() == w and dlg.transient, "it is a child of the pop-out and transient to it")
	_ok(dlg.current_screen == w.current_screen, "same screen as the asker (%d)" % dlg.current_screen)
	var usable := DisplayServer.screen_get_usable_rect(dlg.current_screen)
	_ok(usable.encloses(Rect2i(dlg.position, dlg.size)), "inside that screen's usable area: %s in %s" % [str(Rect2i(dlg.position, dlg.size)), str(usable)])
	print("  diag: dlg pos=%s size=%s min=%s content_min=%s win_id=%d os_size=%s os_pos=%s popout pos=%s size=%s" % [dlg.position, dlg.size, dlg.min_size, dlg.get_contents_minimum_size(), dlg.get_window_id(), DisplayServer.window_get_size(dlg.get_window_id()), DisplayServer.window_get_position(dlg.get_window_id()), w.position, w.size])
	_ok(dlg.is_in_group(PopoutHost.GROUP), "it joined the pop-out group, so the busy overlay can reach it")
	_ok(PopoutHost.native_windows(get_tree()).has(dlg), "native_windows() lists it")
	if _shots != "":
		dlg.get_texture().get_image().save_png(_shots.path_join("dlg_own_window.png"))
	# the busy overlay: a generate mutes it
	_bridge.generate({"seed": 24601, "width_km": 1200.0, "grid_w": 160, "grid_h": 120, "archetype": "", "villages": true})
	await _settle(4)
	var muted := dlg.gui_disable_input
	var mirror: int = _app.busy_overlay.mirror_count()
	while _bridge.generating:
		await get_tree().create_timer(0.1).timeout
	await _settle(8)
	_ok(muted, "during a generate the own-window dialog is muted (gui_disable_input)")
	_ok(mirror >= 1, "and the overlay mirrors its card into it (mirror_count %d)" % mirror)
	_ok(not dlg.gui_disable_input, "after the generate its input is back")
	print("  (mirror_count during the run: %d)" % mirror)
	# a real click on the OK button, in the dialog's own viewport
	var ok := dlg.get_ok_button()
	await _click(dlg, ok.get_global_rect().get_center())
	await _settle(6)
	_ok(bool(res["confirmed"]) and not bool(res["cancelled"]), "a real click on 'Do it' ran the confirm and nothing else")
	await _until(func() -> bool: return _nwin() == 2)
	_ok(_nwin() == 2 and not is_instance_valid(dlg) or dlg.is_queued_for_deletion(), "the dialog's window is gone (count %d)" % _nwin())
	w.hide()
	await _settle(4)
	DccSettings.set_popout_enabled(false)


# -- reshape -----------------------------------------------------------------------------------------------

func _reshape() -> void:
	print("== 5. reshape on an own-window dialog ==")
	DccSettings.set_popout_enabled(true)
	var w: Window = _app.faction_roster_window
	w.open(1, "identity")
	await _until(func() -> bool: return not w.is_embedded() and w.visible)
	await _settle(6)
	var dlg := DccWidgets.confirm(PopoutHost.dialog_host(_app, w), "Reshape", "Wide.", "OK", func() -> void: pass, Callable(), 1500)
	await _settle(8)
	_ok(dlg.force_native and not dlg.is_embedded(), "an own-window dialog to reshape")
	dlg.min_size = Vector2i(700, 0)   # as the place editor does before it reshapes
	PopoutHost.reshape(dlg, Vector2i(700, 400))
	await _settle(8)
	var usable := DisplayServer.screen_get_usable_rect(dlg.current_screen)
	_ok(dlg.size == Vector2i(700, 400) and usable.encloses(Rect2i(dlg.position, dlg.size)), "resized to 700x400 inside the usable area: %s" % str(Rect2i(dlg.position, dlg.size)))
	await _close_dialogs(dlg)
	# an EMBEDDED dialog goes through popup_centered(want)
	var small := DccWidgets.confirm(PopoutHost.dialog_host(_app, w), "Embedded", "Small.", "OK", func() -> void: pass)
	await _settle(8)
	_ok(small.is_embedded(), "a small confirm in the pop-out is embedded")
	PopoutHost.reshape(small, Vector2i(500, 300))
	await _settle(6)
	_ok(small.is_embedded() and small.size == Vector2i(500, 300), "reshape on it is popup_centered(want): %s" % str(small.size))
	await _close_dialogs(small)
	w.hide()
	await _settle(4)
	DccSettings.set_popout_enabled(false)


# -- the caller's size request decides ---------------------------------------------------------------------

## `present()`'s `need_hint`: a small dialog that ASKS for more room than its pop-out has is its own window; the
## same dialog without the request is embedded. An embedded window is clamped to its embedder on entry, so what it
## reads back cannot say what it wanted (`present()`'s header); only the caller's request can.
func _hint() -> void:
	print("== 6. the caller's request, not the clamped read-back, decides ==")
	DccSettings.set_popout_enabled(true)
	var w: Window = _app.faction_roster_window
	w.open(1, "identity")
	await _until(func() -> bool: return not w.is_embedded() and w.visible)
	await _settle(6)
	var a := AcceptDialog.new()
	a.title = "plain"
	a.size = Vector2i(400, 300)
	w.add_child(a)
	var own_a := PopoutHost.present(a, w)
	await _settle(8)
	_ok(not own_a and a.is_embedded(), "400x300 with no request fits the 880x620 pop-out and is embedded")
	await _close_dialogs(a)
	var b := AcceptDialog.new()
	b.title = "asks for more"
	b.size = Vector2i(400, 300)
	w.add_child(b)
	var own_b := PopoutHost.present(b, w, Vector2i(400, 700))
	await _settle(10)
	_ok(own_b and b.force_native and not b.is_embedded() and _nwin() == 3, "the same dialog asking for 700 px of height in a 620 px pop-out is its own window (count %d)" % _nwin())
	_ok(b.size.y >= 700 or b.min_size.y >= 0, "and it is not clamped to the pop-out")
	await _close_dialogs(b)
	w.hide()
	await _settle(4)
	DccSettings.set_popout_enabled(false)
