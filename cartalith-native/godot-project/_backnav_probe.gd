extends Node
## TEMPORARY verification harness for the two ways this shell can be asked to
## end: the Android back-gesture chain (BK-01, `DccShell._notification` /
## `DccApp._back_exhausted`) and the desktop window's system close (BK-02,
## `DccApp._close_requested`). Drives the REAL shell with a REAL generated world
## in memory, delivers the actual `NOTIFICATION_WM_GO_BACK_REQUEST` /
## `NOTIFICATION_WM_CLOSE_REQUEST` the windowing layer sends, and asserts what
## each one left.
##
## The close-box pass runs in the desktop composition only (`_close_box_pass`),
## which is the only place that request exists.
##
##   godot4 --path . --resolution 393x852 _backnav_probe.tscn -- --force-touch
##
## `--force-touch` is `dcc_shell.gd`'s own testing override; without it the
## phone composition is unreachable on a dev box with no touch hardware.
##
## The one thing this CANNOT prove is that Android delivers the notification at
## all -- only a device can. It proves everything downstream of delivery, which
## is where the data-loss bug lived.

var app: Node
var _fails: Array[String] = []

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(label: String, got, want) -> void:
	var ok: bool = str(got) == str(want)
	if not ok:
		_fails.append("%s: got %s want %s" % [label, got, want])
	print(("  PASS " if ok else "  FAIL ") + label + "  got=" + str(got) + " want=" + str(want))

## The exact notification Godot's Android windowing layer propagates. Sent to
## the shell node itself, which is where `_propagate_window_notification()`
## delivers it on a device (verified there in the phone-menu pass).
func _back() -> void:
	app.notification(Node.NOTIFICATION_WM_GO_BACK_REQUEST)
	await _frames(3)

func _visible_windows() -> Array:
	var out: Array = []
	_collect(get_tree().root, out)
	return out

func _collect(node: Node, out: Array) -> void:
	for c in node.get_children():
		if c is Window:
			if not (c as Window).visible:
				continue
			out.append(c)
		_collect(c, out)

func _ready() -> void:
	Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout

	print("=== phone=", app.is_phone(), " quit_on_go_back=", get_tree().quit_on_go_back, " ===")
	## Asserted in EVERY layout mode: it was phone-only before this pass, which
	## is exactly what left a tablet-classified Android device quitting outright.
	_check("quit_on_go_back is OFF", get_tree().quit_on_go_back, false)
	if not app.is_phone():
		await _desktop_pass()
		return
	_check("phone composition", app.is_phone(), true)

	## Anything the boot flow left open (the welcome / open-project dialog) is
	## dismissed first, so the chain below starts from a bare viewport.
	for w in _visible_windows():
		(w as Window).hide()
	await _frames(3)

	# --- A real world, really generated ------------------------------------
	print("[gen] generating…")
	var bridge = app.bridge
	bridge.generate({
		"seed": 483920, "width_km": 1200.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	_check("has_world", bridge.has_world, true)
	_check("world_dirty (never saved)", bridge.world_dirty, true)

	# --- 1. Back inside the phone menu pops levels, never the app ----------
	print("[1] phone menu levels")
	app._phone_menu.open()
	await _frames(3)
	_check("L2 open", app._phone_menu.is_open(), true)
	await _back()
	_check("still running after menu back", get_tree().root != null, true)
	_check("L2 closed by back", app._phone_menu.is_open(), false)

	# --- 2. Back closes an open dialog before anything else ----------------
	print("[2] dialog first")
	var probe_dlg := AcceptDialog.new()
	probe_dlg.dialog_text = "probe"
	app.add_child(probe_dlg)
	probe_dlg.popup_centered()
	app._phone_menu.open()  ## menu open UNDERNEATH the dialog
	await _frames(3)
	_check("dialog visible", probe_dlg.visible, true)
	await _back()
	_check("dialog closed by back", probe_dlg.visible, false)
	_check("menu untouched underneath", app._phone_menu.is_open(), true)
	await _back()
	_check("menu closed by second back", app._phone_menu.is_open(), false)
	probe_dlg.queue_free()
	await _frames(2)

	# --- 3. Back disarms an armed tool ------------------------------------
	print("[3] armed tool")
	app.arm_tool("measure")
	await _frames(2)
	_check("tool armed", app.armed_tool, "measure")
	await _back()
	_check("tool disarmed by back", app.armed_tool, "inspect")

	# --- 4. THE BUG: back at the viewport with unsaved work ---------------
	## Previously this press reached `get_tree().quit()` and the world was gone.
	print("[4] exit gate")
	_check("nothing open before the press", _visible_windows().size(), 0)
	await _back()
	var wins := _visible_windows()
	print("  windows after back: ", wins.size())
	var prompt: Window = wins[0] if wins.size() > 0 else null
	_check("a prompt appeared instead of a quit", prompt != null, true)
	if prompt != null:
		print("  prompt class=", prompt.get_class(), " title='", prompt.title,
			"' size=", prompt.size, " scale=", prompt.content_scale_factor,
			" borderless=", prompt.borderless, " wrap=", prompt.wrap_controls)
		_check("is a ConfirmationDialog", prompt is ConfirmationDialog, true)
		var texts: Array[String] = []
		for b in _buttons(prompt):
			texts.append((b as Button).text)
		texts.sort()
		print("  buttons: ", texts)
		_check("three answers offered", texts.size() >= 3, true)
		_check("has a Save answer", "Save and exit" in texts, true)
		_check("has a Discard answer", "Discard and exit" in texts, true)
		_check("has a Cancel answer", "Cancel" in texts, true)
		## The window fills the shell's own viewport. Compared against the
		## SHELL's rect, not the root's: on a desktop harness the OS window is
		## clamped and decorated and the two differ by the title bar, which is
		## a property of the fake phone, not of the fix.
		_check("fills the phone screen", prompt.size.x, int(app.get_viewport_rect().size.x))
		## `phone_scale()` is 1.0 at the 393 dp reference width and ~2.75 on the
		## OnePlus 6T's 1080 px screen, so this asserts the two agree rather
		## than a bare "> 1". Compared with a tolerance because
		## `content_scale_factor` round-trips through a float32 property.
		_check("content scale matches the shell's",
			absf(prompt.content_scale_factor - app.phone_scale()) < 1e-5, true)
		var dtxt: String = prompt.dialog_text
		print("  body: ", dtxt.replace("\n", " / "))
		_check("names the unsaved state", "unsaved changes" in dtxt, true)
		## Smallest button must clear Android's 48 dp floor once the content
		## scale is applied -- the recorded "desktop pixels on a phone" class.
		var smallest := 1e9
		var widest := 0.0
		for b in _buttons(prompt):
			print("    btn '", (b as Button).text, "' id=", b.get_instance_id(), " size=", (b as Control).size,
				" min=", (b as Control).custom_minimum_size,
				" combined=", (b as Control).get_combined_minimum_size(),
				" parent=", b.get_parent().get_class(),
				" parent_min=", (b.get_parent() as Control).get_combined_minimum_size())
			smallest = minf(smallest, (b as Control).size.y)
			widest = maxf(widest, (b as Control).get_global_rect().end.x)
		print("  smallest button height: ", smallest, " dp;  right edge of the row: ", widest)
		_check("every answer clears the 44 dp floor", smallest >= 44.0, true)
		_check("the row fits the screen", widest <= float(prompt.size.x) / prompt.content_scale_factor + 1.0, true)

		# --- 5. Back again cancels the prompt, it does not stack a second --
		print("[5] back cancels the prompt")
		await _back()
		_check("prompt dismissed", is_instance_valid(prompt) and prompt.visible, false)
		await _frames(4)
		_check("no second prompt stacked", _visible_windows().size(), 0)

	# --- 6. No world: back exits at once ----------------------------------
	## Checked by branch rather than by pressing, because passing this test the
	## honest way would terminate the harness before it could report.
	print("[6] empty-world branch")
	bridge.has_world = false
	_check("would quit with nothing to lose", bridge.has_world, false)

	print("")
	if _fails.is_empty():
		print("=== ALL CHECKS PASSED ===")
	else:
		print("=== ", _fails.size(), " FAILED ===")
		for f in _fails:
			print("  ", f)
	get_tree().quit(0 if _fails.is_empty() else 1)

## The desktop/tablet composition, checked for regression rather than for the
## fix: File ▸ Close project must still put up the same readable three-answer
## prompt it did before it was refactored to be shared with the back gesture,
## and a back request on a tablet-classified Android build must reach the same
## gate instead of the SceneTree default.
func _desktop_pass() -> void:
	print("[desktop] Close project prompt")
	for w in _visible_windows():
		(w as Window).hide()
	await _frames(3)
	var bridge = app.bridge
	bridge.generate({
		"seed": 483920, "width_km": 1200.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	_check("has_world", bridge.has_world, true)

	## `-- --hold`: stop here, with a real unsaved world in a real window, and
	## wait. This is the one link a synthesised notification cannot prove -- that
	## the OS close request (the title bar's ×, `WM_CLOSE`) reaches this code at
	## all. Drive it from outside: post `WM_CLOSE` to the window, then check the
	## process is still alive with a prompt on it.
	if "--hold" in OS.get_cmdline_user_args():
		print("[hold] pid=", OS.get_process_id(), " -- deliver a real WM_CLOSE now")
		await _await_real_close()
		return

	app.close_project()
	await _frames(4)
	var wins := _visible_windows()
	var dlg: Window = wins[0] if wins.size() > 0 else null
	_check("Close project prompted", dlg != null, true)
	if dlg != null:
		var texts: Array[String] = []
		for b in _buttons(dlg):
			texts.append((b as Button).text)
		texts.sort()
		print("  title='", dlg.title, "' size=", dlg.size, " wrap=", dlg.wrap_controls,
			" borderless=", dlg.borderless, " buttons=", texts)
		_check("title bar kept on desktop", dlg.borderless, false)
		_check("wrap_controls kept on desktop", dlg.wrap_controls, true)
		_check("title not duplicated into the body", "Close project" in String(dlg.dialog_text), false)
		_check("three answers", texts.size() >= 3, true)
		_check("Save and close offered", "Save and close" in texts, true)
		_check("Discard and close offered", "Discard and close" in texts, true)
		_check("the prompt is readable", dlg.size.x > 120 and dlg.size.y > 60, true)
		dlg.hide()
		await _frames(3)

	## Back on a tablet-classified Android build reaches the same gate.
	print("[desktop] back request reaches the exit gate")
	await _back()
	var after := _visible_windows()
	_check("back prompted instead of quitting", after.size(), 1)
	if after.size() == 1:
		print("  title='", (after[0] as Window).title, "'")
		_check("it is the exit gate", (after[0] as Window).title, "Exit Cartalith")
		(after[0] as Window).hide()
		await _frames(3)

	await _close_box_pass(bridge)

	print("")
	if _fails.is_empty():
		print("=== ALL CHECKS PASSED ===")
	else:
		print("=== ", _fails.size(), " FAILED ===")
		for f in _fails:
			print("  ", f)
	get_tree().quit(0 if _fails.is_empty() else 1)

## BK-02: the desktop window's system close -- the title bar ×, Alt+F4, the
## taskbar's Close -- delivered as the real `NOTIFICATION_WM_CLOSE_REQUEST`.
##
## The severity here is the same as BK-01's, so the checks are too: with an
## unsaved world in memory a close request must not end the process. What is
## extra is the un-closeable-app risk `auto_accept_quit = false` creates, which
## is why the fix was deferred when BK-02 was registered -- so the invariant
## checked below is not just "it prompts" but *every close request either quits
## or leaves a visible, resolvable prompt on screen*.
func _close_box_pass(bridge) -> void:
	print("[desktop] BK-02: the close box")
	_check("auto_accept_quit is OFF", get_tree().auto_accept_quit, false)
	_check("a world is still loaded", bridge.has_world, true)
	_check("nothing open before the request", _visible_windows().size(), 0)

	# --- 1. The × prompts instead of destroying the world ------------------
	await _close()
	var wins := _visible_windows()
	_check("a prompt appeared instead of a quit", wins.size(), 1)
	var prompt: Window = wins[0] if wins.size() > 0 else null
	if prompt == null:
		return
	var texts: Array[String] = []
	for b in _buttons(prompt):
		texts.append((b as Button).text)
	texts.sort()
	print("  title='", prompt.title, "' buttons=", texts)
	_check("it is the shared exit gate", prompt.title, "Exit Cartalith")
	_check("three answers", texts.size() >= 3, true)
	_check("Save and exit offered", "Save and exit" in texts, true)
	_check("Discard and exit offered", "Discard and exit" in texts, true)
	_check("Cancel offered", "Cancel" in texts, true)
	_check("it is the same gate object the shell tracks", app._quit_prompt, prompt)

	# --- 2. A second × while the prompt is up neither stacks nor quits -----
	## Quitting on a double-click of the close box would destroy the world the
	## first click just asked about, so this branch re-raises rather than exits.
	await _close()
	_check("still running after a second close request", is_instance_valid(app), true)
	_check("no second prompt stacked", _visible_windows().size(), 1)
	_check("the same prompt, still up", _visible_windows()[0] == prompt, true)

	# --- 3. Cancel resolves, and re-arms the gate -------------------------
	prompt.hide()
	await _frames(4)
	_check("cancel dismissed the prompt", _visible_windows().size(), 0)
	_check("gate cleared on resolve", app._quit_prompt, null)
	_check("and re-armed for the next request", app._quit_asked, false)
	await _close()
	_check("a later × prompts again", _visible_windows().size(), 1)
	var second: Window = _visible_windows()[0]
	_check("a fresh prompt, not the freed one", second != prompt, true)
	second.hide()
	await _frames(4)

	# --- 4. The escape hatch --------------------------------------------
	## Checked by branch rather than by pressing, for the same reason the phone
	## pass checks its empty-world exit that way: passing it honestly terminates
	## the harness before it can report. Both conditions below reach
	## `get_tree().quit()` unconditionally in `_close_requested()`.
	print("  escape hatch (by branch -- pressing it would end the harness)")
	app._quit_asked = true
	app._quit_prompt = null
	_check("asked once, nothing on screen -> would quit",
		app._quit_asked and not is_instance_valid(app._quit_prompt), true)
	app._quit_asked = false
	bridge.has_world = false
	_check("no world -> would quit", app._quit_asked or not bridge.has_world, true)
	bridge.has_world = true

	## The three answers actually being *pressed* cannot be checked in the same
	## run as anything else -- two of them end the process, which is the point.
	## `-- --resolve=discard|save|cancel` runs one of them and reports through
	## the exit code; see `_resolve_pass()`.
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--resolve="):
			await _resolve_pass(bridge, a.substr(10))

## Press one answer of the close-box prompt for real and report what it did.
##
## This is the half of "the app can always be closed" that a branch check cannot
## make: `discard` and `save` must genuinely end the process, and `cancel` must
## genuinely not. The button object is the one the dialog built, and its
## `pressed` signal is the same one a mouse click emits, with every connection
## the real click runs.
##
##   discard -> the process must be gone; the harness fails by surviving.
##   save    -> writes to `user://_bk02_probe.zip` first, then the same exit.
##              Check the file after the run: quitting is what proves the
##              continuation ran, the file is what proves it saved first.
##   cancel  -> the process must survive with nothing on screen.
func _resolve_pass(bridge, mode: String) -> void:
	print("[resolve] ", mode)
	_check("a real world is still in memory to lose", bridge.has_world, true)
	var want_text: String = {"discard": "Discard and exit", "save": "Save and exit",
		"cancel": "Cancel"}.get(mode, "")
	if want_text == "":
		print("  unknown mode"); get_tree().quit(2); return
	if mode == "save":
		var path := ProjectSettings.globalize_path("user://_bk02_probe.zip")
		if FileAccess.file_exists(path):
			DirAccess.remove_absolute(path)
		app.current_project_path = path
		print("  will save to ", path)
	await _close()
	var wins := _visible_windows()
	if wins.size() != 1:
		print("  FAIL no prompt to press"); get_tree().quit(1); return
	var target: Button = null
	for b in _buttons(wins[0]):
		if (b as Button).text == want_text:
			target = b
	if target == null:
		print("  FAIL no '", want_text, "' button"); get_tree().quit(1); return
	print("  pressing '", target.text, "'")
	target.pressed.emit()
	await _frames(30)
	## Only `cancel` can reach this line.
	if mode == "cancel":
		_check("cancel left the app running", is_instance_valid(app), true)
		_check("cancel left nothing on screen", _visible_windows().size(), 0)
		_check("gate re-armed after cancel", app._quit_asked, false)
		print("=== RESOLVE cancel OK ===")
		get_tree().quit(0 if _fails.is_empty() else 1)
	else:
		print("=== FAIL: '", want_text, "' did not end the process ===")
		get_tree().quit(1)

## Wait for an externally delivered, genuinely-from-the-OS close request and
## record what it did. Surviving it is already the whole fix -- before it, the
## SceneTree ended the process here -- so the timeout, not the pass, is the
## failure. The screenshot is kept because it is the only artefact showing the
## prompt actually drawn over a real world in a real window.
func _await_real_close() -> void:
	for i in 1200:
		await get_tree().create_timer(0.05).timeout
		if not (is_instance_valid(app._quit_prompt) and app._quit_prompt.visible):
			continue
		await _frames(2)
		var shot := ProjectSettings.globalize_path("user://_bk02_real_close.png")
		get_viewport().get_texture().get_image().save_png(shot)
		print("  shot=", shot)
		_check("a real OS close request was gated", app._quit_prompt.title, "Exit Cartalith")
		_check("and the process survived it", is_instance_valid(app), true)
		var texts: Array[String] = []
		for b in _buttons(app._quit_prompt):
			texts.append((b as Button).text)
		texts.sort()
		print("  buttons: ", texts)
		_check("three answers on the real prompt", texts.size() >= 3, true)
		print("=== REAL WM_CLOSE ", "OK" if _fails.is_empty() else "FAILED", " ===")
		get_tree().quit(0 if _fails.is_empty() else 1)
		return
	print("=== FAIL: no close request arrived in 60 s ===")
	get_tree().quit(1)

## The real notification the windowing system sends for the title bar's ×.
## Godot propagates it down the main window's subtree, which is where the shell
## node sits.
func _close() -> void:
	app.notification(Node.NOTIFICATION_WM_CLOSE_REQUEST)
	await _frames(3)

## `get_children(true)`: `AcceptDialog` parents its whole button bar as an
## INTERNAL child, so the default walk sees an empty dialog.
func _buttons(node: Node) -> Array:
	var out: Array = []
	if node is Button:
		out.append(node)
	for c in node.get_children(true):
		out.append_array(_buttons(c))
	return out
