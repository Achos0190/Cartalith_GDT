extends Node
## Assistant-on-tablet probe (owner report 2026-11-09: the assistant on the Pad is buggy; the Pad's logcat
## shows "Attempting to make child window exclusive, but the parent window already has another exclusive
## child" and "freed or unreferenced while a signal is being emitted").
##
##   tablet: godot --path . _asstfix_probe.tscn -- --force-touch --vp 3392x2400 --density 2.225 --ui-scale 2 [--live]
##   small : godot --path . _asstfix_probe.tscn -- --force-touch --vp 1600x1000
##   phone : godot --path . _asstfix_probe.tscn -- --force-touch --vp 1080x2340 --density 2.8125 [--live]
##   `--addr URL` (default http://192.168.1.238:1234/v1), `--shots DIR`.
##
## Windowed, never headless, from a project copy with its own `config/name` and a wiped user dir. No
## network without `--live`; `--live` makes real requests to the running LM Studio with the model as
## loaded (qwen3-14b), thinking off. It never loads, unloads, switches models or stops the server. The key
## is never printed.
##
## What it protects: a second dialog opened while an exclusive dialog is showing (the consent card from the
## settings window or from the phone's assistant sheet) must open on top and work, with no engine
## "already has another exclusive child" error (the caller greps the log), and the assistant's Send,
## proposal, Apply and Undo run to the end.
var _fails := 0
var _checks := 0
var _live := false
var _addr := "http://192.168.1.238:1234/v1"
var _shots := ""
var _app: Node
var _bridge
var _win
var _view: Viewport
var _leg_done := false


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(n: int = 4) -> void:
	for _i in n:
		await get_tree().process_frame


func _shot(tag: String) -> void:
	if _shots == "":
		return
	await _settle(3)
	await RenderingServer.frame_post_draw
	var img := _view.get_texture().get_image()
	if img != null:
		var p := _shots.path_join("af_%s.png" % tag)
		img.save_png(p)
		print("SHOT %s" % p)


func _push(ev: InputEvent) -> void:
	if _view is SubViewport:
		(_view as SubViewport).push_input(ev, true)
	else:
		Input.parse_input_event(ev)


func _click_at(at: Vector2) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 2)
	_push(mm)
	await _settle(2)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		_push(ev)
		await _settle(2)
	await _settle(4)


func _type(text: String) -> void:
	for ch in text:
		var ev := InputEventKey.new()
		ev.unicode = ch.unicode_at(0)
		ev.keycode = KEY_NONE
		ev.pressed = true
		_push(ev)
		var up := ev.duplicate()
		up.pressed = false
		_push(up)
		await get_tree().process_frame
	await _settle(4)

## The control's centre in the probe viewport's coordinates. A control inside an EMBEDDED dialog lives in
## that window's own canvas (its content scale applied, then its position): mapping it by the control's
## global rect alone clicks the wrong pixel.
func _centre_in_view(c: Control) -> Vector2:
	var local := c.get_global_transform_with_canvas() * (c.size * 0.5)
	var win := c.get_window()
	if win != null and win != _app.get_tree().root and win.is_embedded():
		return Vector2(win.position) + win.get_final_transform() * local
	return local


func _press(c: Control, what: String) -> bool:
	if c == null or not is_instance_valid(c):
		_ok(false, "%s exists to be pressed" % what)
		return false
	var p := c.get_parent()
	while p != null:
		if p is ScrollContainer and (p as Control).is_visible_in_tree():
			(p as ScrollContainer).ensure_control_visible(c)
		p = p.get_parent()
	await _settle(4)
	var at := _centre_in_view(c)
	var inside := Rect2(Vector2.ZERO, _view.get_visible_rect().size).has_point(at) and c.is_visible_in_tree()
	_ok(inside, "%s is drawn on screen (centre %s)" % [what, str(at)])
	if not inside:
		return false
	await _click_at(at)
	return true


func _c(name: String) -> Control:
	return _win.controls.get(name) as Control


## Every visible Window below the root, with its exclusive flag and transient parent, for the log.
func _windows(tag: String) -> Array:
	var out: Array = []
	var names := PackedStringArray()
	for w in _app.get_tree().root.find_children("*", "Window", true, false):
		var win := w as Window
		if win.visible:
			out.append(win)
			names.append("%s[%s](excl=%s,emb=%s,rect=%s)" % [win.get_class(), win.title, win.exclusive, win.is_embedded(), str(Rect2i(win.position, win.size))])
	print("  [%s] visible windows: %s" % [tag, ", ".join(names)])
	return out


func _consent_dialog() -> Window:
	for w in _app.get_tree().root.find_children("*", "AcceptDialog", true, false):
		if (w as Window).visible and String((w as Window).title) == "Before the first request":
			return w as Window
	return null


func _buttons_named(root: Node, text: String) -> Array:
	var out: Array = []
	for b in root.find_children("*", "Button", true, false):
		if (b as Button).text == text and (b as Button).is_visible_in_tree():
			out.append(b)
	return out


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("REFUSED: headless")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	var vp := ""
	var k := 0
	while k < args.size():
		var a := args[k]
		if a == "--live":
			_live = true
		elif a == "--addr" and k + 1 < args.size():
			k += 1
			_addr = args[k]
		elif a == "--shots" and k + 1 < args.size():
			k += 1
			_shots = args[k]
		elif a == "--vp" and k + 1 < args.size():
			k += 1
			vp = args[k]
		elif a == "--force-touch":
			pass
		elif a == "--density" or a == "--ui-scale":
			k += 1
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
		k += 1
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	_app = load("res://shell/app.tscn").instantiate()
	_view = get_viewport()
	if vp != "":
		var parts := vp.split("x")
		var sv := SubViewport.new()
		sv.size = Vector2i(int(parts[0]), int(parts[1]))
		sv.gui_embed_subwindows = true
		sv.render_target_update_mode = SubViewport.UPDATE_ALWAYS
		add_child(sv)
		Input.set_emulate_touch_from_mouse(true)
		sv.add_child(_app)
		_view = sv
	else:
		add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	var mode := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s, viewport %s ==" % [mode, str(_view.get_visible_rect().size)])
	_bridge = _app.bridge
	_win = _app.assistant_window
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 128, "grid_h": 96,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await _settle(2)
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _settle(10)
	await _settle(30)
	if not bool(_bridge.has_world):
		print("[FATAL] no world")
		get_tree().quit(2)
		return
	await _leg(mode)
	_ok(_leg_done, "the leg ran to its end")
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _leg(mode: String) -> void:
	DccSettings.set_llm_value("enabled", true)
	DccSettings.set_llm_value("prose", true)
	DccSettings.set_llm_value("commands", true)
	DccSettings.set_llm_value("ctx_params", true)
	DccSettings.set_llm_value("local_allowed", true)
	DccSettings.set_llm_value("preset_id", "lm_studio")
	DccSettings.set_llm_value("base_url", _addr)
	DccSettings.set_llm_value("model", "qwen3-14b")
	DccSettings.set_llm_value("thinking", "none")
	_win._refresh()
	print("tabs: %s" % str(_app.context_tabs()))
	print("== A. entry points ==")
	_app.open_context_tab("ask")
	await _settle(8)
	_windows("after ask")
	print("  docked=%s visible=%s" % [_win.is_docked(), _win.visible])
	await _shot("a_ask_%s" % mode)
	_win.hide()
	_app.open_assistant_panel()
	await _settle(8)
	_windows("after open_assistant_panel")
	print("== B. settings then consent ==")
	_app.open_assistant_settings()
	await _settle(8)
	_windows("settings open")
	await _shot("b_settings_%s" % mode)
	var sw = _app.assistant_settings_window
	var ep: Dictionary = _bridge.llm_endpoint_info("lm_studio", _addr)
	print("endpoint ok=%s origin=%s" % [ep.get("ok"), ep.get("origin")])
	sw._ask_consent(ep, func(): print("  (agree callback)"), func(): print("  (decline callback)"))
	await _settle(12)
	_windows("consent from settings")
	_ok(_consent_dialog() != null, "the consent card is showing over the settings window")
	## Showing is not enough: a refused second exclusive window can still report visible. The engine's rule is
	## one exclusive child per parent, so the card must be the settings window's own child.
	_ok(_consent_dialog() != null and sw.is_ancestor_of(_consent_dialog()), "the consent card is a child of the settings window it opened over (so the engine accepts it as exclusive)")
	await _shot("b_consent_%s" % mode)
	var no := _buttons_named(_consent_dialog(), "Not now")
	_ok(no.size() == 1, "the consent card has its Not now button")
	if no.size() == 1:
		await _press(no[0], "Not now")
	_ok(_consent_dialog() == null, "Not now closes the consent card")
	_windows("after decline")
	_ok(sw.visible, "the settings window is still showing after the card closed")

	print("== C. Refresh in settings opens consent by a real click ==")
	var rb := _buttons_named(sw, "Refresh")
	_ok(rb.size() >= 1, "settings has a Refresh button")
	if rb.size() >= 1:
		await _press(rb[0], "Refresh")
		await _settle(8)
	_windows("after Refresh")
	_ok(_consent_dialog() != null, "Refresh with no consent opens the consent card on top of settings")
	if _consent_dialog() != null:
		if _live:
			var al := _buttons_named(_consent_dialog(), "Allow")
			if al.size() == 1:
				await _press(al[0], "Allow")
			await _wait_models(sw)
		else:
			var no2 := _buttons_named(_consent_dialog(), "Not now")
			if no2.size() == 1:
				await _press(no2[0], "Not now")
	sw.hide()
	await _settle(6)

	print("== D. Send from the assistant: consent, then a Commands request ==")
	_app.open_context_tab("ask")
	await _settle(8)
	_ok(_c("input") != null, "the assistant's input exists")
	await _press(_c("task_commands"), "the Commands chip")
	(_c("input") as TextEdit).text = ""
	(_c("input") as Control).grab_focus()
	await _type("lower the sea level a bit")
	_win._refresh()
	await _settle(4)
	await _press(_c("send"), "Send")
	await _settle(8)
	_windows("after Send")
	var cd := _consent_dialog()
	if DccSettings.llm_consented_origins().has(String(ep.get("origin", ""))):
		_ok(cd == null, "consent already recorded: Send goes straight out")
	else:
		_ok(cd != null, "Send with no consent opens the consent card")
		if cd != null:
			await _shot("d_consent_%s" % mode)
			if _live:
				var al2 := _buttons_named(cd, "Allow")
				if al2.size() == 1:
					await _press(al2[0], "Allow")
			else:
				var no3 := _buttons_named(cd, "Not now")
				if no3.size() == 1:
					await _press(no3[0], "Not now")
	if _live:
		await _wait_run()
		await _settle(10)
		await _shot("d_reply_%s" % mode)
		var ap := _find_button_prefix(_win.get("_msgs"), "Apply")
		_ok(ap != null, "a proposal card with an Apply button appeared")
		if ap != null:
			await _press(ap, "Apply")
			await _settle(12)
			_windows("after Apply")
			_ok(_app.can_undo_any(), "after Apply the app has an undo step (one Apply is one undo step)")
			_app.undo_last()
			await _settle(10)
			_ok(_has_label(_win.get("_msgs"), "Undone with Edit"), "Edit > Undo marks the card as undone")
	_leg_done = true


func _has_label(root: Node, needle: String) -> bool:
	if root == null:
		return false
	for l in root.find_children("*", "Label", true, false):
		if (l as Label).text.contains(needle):
			return true
	return false


func _find_button_prefix(root: Node, prefix: String) -> Button:
	if root == null:
		return null
	for b in root.find_children("*", "Button", true, false):
		if (b as Button).text.begins_with(prefix) and (b as Button).is_visible_in_tree():
			return b as Button
	return null


func _wait_models(sw) -> void:
	var t0 := Time.get_ticks_msec()
	while String(sw.get("_ml_state")) == "loading" or Time.get_ticks_msec() - t0 < 1500:
		if Time.get_ticks_msec() - t0 > 30000:
			break
		await get_tree().create_timer(0.25).timeout
	print("  models state=%s code=%s" % [String(sw.get("_ml_state")), String(sw.get("_ml_code"))])


func _wait_run() -> void:
	var t0 := Time.get_ticks_msec()
	await get_tree().create_timer(1.0).timeout
	while _win.run.busy() and Time.get_ticks_msec() - t0 < 300000:
		await get_tree().create_timer(0.25).timeout
