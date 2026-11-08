extends Node
## **AC-7 in the real shell: the Ask tab's action cards, one per KIND** (`ASSISTANT_ACTIONS_PROPOSAL.md` AC-7;
## `PC_SHELL_PROPOSAL.md` s.3.5 "Cards"; Ruling DW), against the in-test loopback server (`_llm_mock_server.gd`,
## scenario `tools`). The words are `shell/llm_action_cards.gd`; the runner is still `llm_actions.gd::run_card`.
##
##   desktop: godot --path . --resolution 1600x1000 res://_ac7_probe.tscn
##   touch:   godot --path . --resolution 1600x1000 res://_ac7_probe.tscn -- --force-touch
##
## **Windowed, never `--headless`**, with a wiped user dir (`MISTAKES.md`). Apply and Dismiss are pressed through
## the button's own `pressed`, only while the button is enabled (pixel pushes into the embedded window did not
## reach it, measured 2026-11-07 by `_ac3_probe.gd`); where the panel is DOCKED in the Ask tab a real mouse press
## is pushed at the button first, and each check line says which route landed. Result: `RESULT: PASS|FAIL
## (<n> failed) checks=<n>`. Exit 0 pass, 1 fail, 2 bad argument or watchdog.
##
## What it protects, per kind (each names the section text on the card, Apply's effect, and Undo):
##   1. SETTING: the SETTING section reads `was -> value` from the app, nothing moved before Apply, Apply writes
##      the value, ONE Edit > Undo restores it, and the card follows its group.
##   2. VIEW: a layer and the style preset: the section says the style is one Undo step and the layer is session
##      state; Apply hides the layer and changes the preset; Undo restores the preset only.
##   3. SELECT: the card NAMES the settlement (read from the engine's list, never the bare number), the year row
##      says where the cursor is now, an out-of-range number is worded as absent or refused.
##   4. EDIT: arming a tool says it draws nothing and Apply arms it; `edit.undo` alone applies Edit > Undo.
##   5. WINDOW: the card says the window is opened for the person; Apply opens it.
##   6. MIXED: sections appear in the reading order (setting, view, select, window), each under its heading, one
##      Apply covers all four, and Dismiss changes nothing.
##   7. DOCKED: on the desktop and tablet the card lives in the Ask tab (the panel is docked), not a dialog.

const ENV_VAR := "CART_LLM_PROBE_KEYVAR"
const SECRET := "sk-PROBE-SECRET-ac7-0123456789"

var _fails := 0
var _checks := 0
var _app: Node
var _bridge
var _win
var _server: LlmMockServer
var _port := 0
var _real_clicks := 0
var _emit_clicks := 0


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _until(cond: Callable, ms: int = 10000) -> bool:
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < ms:
		if cond.call():
			return true
		await get_tree().process_frame
	return bool(cond.call())


func _c(name: String) -> Control:
	return _win.controls.get(name) as Control


func _reset_llm() -> void:
	DccSettings._ensure_loaded()
	if DccSettings._cfg.has_section("llm"):
		DccSettings._cfg.erase_section("llm")
		DccSettings._save()


func _configure() -> void:
	var s := {"enabled": true, "commands": true, "prose": false, "local_allowed": true, "preset_id": "ollama",
		"base_url": "http://127.0.0.1:%d/v1" % _port, "model": "mock-model",
		"key_source": "env", "env_var": ENV_VAR, "ctx_params": true}
	for k in s:
		DccSettings.set_llm_value(k, s[k])
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % _port)


func _type(text: String) -> void:
	var inp := _c("input") as TextEdit
	inp.text = text
	inp.text_changed.emit()
	await _settle(3)


## Sends `request` through the real panel with the scripted replies; returns the requests it took.
func _ask(script: Array, request: String) -> int:
	_server.scenario = "tools"
	_server.tool_script = script
	_server.tool_served = 0
	var n0: int = _server.requests.size()
	_win._pick_task("commands")
	await _type(request)
	var send := _c("send") as Button
	for _i in 3:
		send.pressed.emit()
		await _settle(8)
		if _server.requests.size() > n0 or _win.run.busy():
			break
	await _until(func(): return not _win.run.busy() and _server.requests.size() > n0, 20000)
	await _until(func(): return not _win.run.busy(), 20000)
	await _settle(6)
	return _server.requests.size() - n0


func _action_card() -> Dictionary:
	for i in range(_win.cards.size() - 1, -1, -1):
		if String(_win.cards[i].get("kind", "params")) == "actions":
			return _win.cards[i]
	return {}


## Presses `b`. Docked (the Ask tab, a plain control in the root viewport) a real left press and release is
## pushed at the button's centre first; when the panel is a dialog, or the push did not land while the button is
## enabled, `pressed` is emitted (only for an ENABLED button: a disabled one ignores a click, as for a person).
## Returns false when `b` is not on screen to be clicked. Counts which route was used.
func _click(b) -> bool:
	if b == null or not is_instance_valid(b) or not (b as Button).is_visible_in_tree():
		return false
	var btn := b as Button
	_win._scroll_to_end()
	await _settle(3)
	if btn.disabled:
		return true
	var landed := false
	if _win.is_docked():
		var pos := btn.get_global_rect().get_center()
		var down := InputEventMouseButton.new()
		down.button_index = MOUSE_BUTTON_LEFT
		down.pressed = true
		down.position = pos
		down.global_position = pos
		get_tree().root.push_input(down, false)
		var up := InputEventMouseButton.new()
		up.button_index = MOUSE_BUTTON_LEFT
		up.pressed = false
		up.position = pos
		up.global_position = pos
		get_tree().root.push_input(up, false)
		await _settle(4)
		landed = not is_instance_valid(btn) or btn.disabled
	if landed:
		_real_clicks += 1
	else:
		_emit_clicks += 1
		btn.pressed.emit()
		await _settle(4)
	return true


func _plan() -> Dictionary:
	var rw = _app._render_workspace()
	var vp = _app.viewport
	var layers := {}
	for l in CartographyWorkspace.LIVE_LAYERS:
		layers[String(l["id"])] = bool(vp.layer_visible(String(l["id"])))
	return {"peak": _bridge.param_get("peak_m"), "preset": int(rw._active_preset), "layers": layers, "selection": (_app.selection as Dictionary).duplicate(),
		"year": int(_app.tl_year()), "tool": String(_app.armed_tool), "roster": bool(_app.faction_roster_window.visible)}


func _finish() -> void:
	print("clicks: %d by a real mouse press, %d by the button's own pressed signal" % [_real_clicks, _emit_clicks])
	OS.unset_environment(ENV_VAR)
	LlmWire.shutdown_all()
	if _server != null:
		_server.stop()
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


## Every Label's text under `n`, joined with newlines: what a person reads on the card.
func _texts(n: Node) -> String:
	var out := ""
	if n is Label:
		out += (n as Label).text + "\n"
	for ch in n.get_children():
		out += _texts(ch)
	return out


## The index of the first line of `t` that contains `needle`, or -1 (reading order on the card).
func _line_of(t: String, needle: String) -> int:
	var lines := t.split("\n")
	for i in lines.size():
		if lines[i].contains(needle):
			return i
	return -1


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		if a != "--force-touch":
			print("ABORT: unrecognised argument %s -- this probe reads --force-touch only" % a)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("ABORT: run windowed; this probe drives the real panel")
		get_tree().quit(2)
		return
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	OS.set_environment(ENV_VAR, SECRET)
	_server = LlmMockServer.new()
	add_child(_server)
	_ok(_server.start(), "the mock server listens")
	_port = _server.port
	_reset_llm()
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_bridge = _app.bridge
	_win = _app.assistant_window
	_ok(_bridge.llm_api and _win != null, "the library has LlmBridge; the shell has the assistant panel")
	if not _bridge.llm_api or _win == null:
		_finish()
		return
	print("touch form=%s phone=%s tablet=%s" % ["--force-touch" in OS.get_cmdline_user_args(), DccTheme.is_phone(), DccTheme.is_tablet()])
	_bridge.generate({"seed": 9137, "width_km": 1200.0, "grid_w": 256, "grid_h": 164, "archetype": "", "villages": true})
	while _bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _settle(8)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	var all: Array = _bridge.settlements()
	_ok(all.size() > 5, "the probe world has settlements to select (%d)" % all.size())
	_configure()
	_app.open_assistant_panel()
	await _settle(10)
	if not DccTheme.is_phone():
		_ok(_win.is_docked() and not _win.visible, "7. the panel is docked in the Ask tab, not shown as a dialog")
	var peak0 := float(_bridge.param_get("peak_m"))
	var peak1 := peak0 + 1000.0

	# ---------------------------------------------------------------- 1. SETTING
	print("== 1. a setting card ==")
	var pre := _plan()
	var n := await _ask([{"calls": [{"name": "search_actions", "args": {"query": "peak altitude"}}]},
		{"name": "propose_actions", "args": {"summary": "Taller peaks", "actions": [{"id": "param:peak_m", "args": {"value": peak1}, "reason": "taller"}]}}], "make the mountains taller")
	var c1 := _action_card()
	_ok(not c1.is_empty() and (c1["info"].get("valid", []) as Array).size() == 1, "a card with one validated setting")
	var t1 := _texts(c1["box"]) if not c1.is_empty() else ""
	_ok(t1.contains("MODEL PROPOSAL · SETTING"), "the chip names the kind: SETTING")
	_ok(t1.contains("→ %s" % str(peak1)), "the row reads 'was → %s' from the app's own value: %s" % [str(peak1), t1.left(150).replace("\n", " | ")])
	_ok(t1.contains("Edit ▸ Undo reverts these in one step") and t1.contains("press Generate"), "the section says what Undo covers and that the world is not rebuilt")
	_ok(JSON.stringify(_plan()) == JSON.stringify(pre), "NOTHING moved before Apply")
	_ok(await _click(c1.get("apply")), "Apply clicked")
	_ok(is_equal_approx(float(_bridge.param_get("peak_m")), peak1) and c1["state"] == "applied" and int(c1["group_id"]) > 0, "the setting is written and the card owns an undo group")
	_app.undo_last()
	await _settle(6)
	_ok(is_equal_approx(float(_bridge.param_get("peak_m")), peak0) and c1["state"] == "undone", "ONE Edit > Undo restores the setting and the card follows: %s" % str(c1["state"]))

	# ---------------------------------------------------------------- 2. VIEW
	print("== 2. a view card ==")
	var vf: Dictionary = (load("res://shell/llm_view.gd")).facts(_app, _bridge)
	var preset_id := ""
	for p in vf.get("presets", []):
		if not bool((p as Dictionary).get("on", false)):
			preset_id = String((p as Dictionary)["id"])
			break
	_ok(preset_id != "", "a style preset other than the active one exists (%s)" % preset_id)
	var pre2 := _plan()
	n = await _ask([{"calls": [{"name": "list_actions", "args": {"path": "carto/layers"}}, {"name": "list_actions", "args": {"path": "carto/style"}}]},
		{"name": "propose_actions", "args": {"summary": "Old look", "actions": [{"id": "view.layer.rivers", "args": {"visible": false}}, {"id": "view.style_preset", "args": {"preset": preset_id}}]}}], "hide rivers, old look")
	var c2 := _action_card()
	var t2 := _texts(c2["box"]) if not c2.is_empty() else ""
	_ok(t2.contains("MODEL PROPOSAL · VIEW") and t2.contains("Hide the") and t2.contains("Style: "), "the VIEW card words the layer and the style: %s" % t2.left(160).replace("\n", " | "))
	_ok(t2.contains("The style is one Undo step; layers and the camera are session state"), "the section says the style is one Undo step and the layer is session state")
	_ok(await _click(c2.get("apply")), "Apply clicked")
	var post2 := _plan()
	_ok(post2["layers"]["rivers"] == false and int(post2["preset"]) != int(pre2["preset"]), "the layer is hidden and the preset changed")
	_app.undo_last()
	await _settle(8)
	var undone2 := _plan()
	_ok(int(undone2["preset"]) == int(pre2["preset"]) and undone2["layers"]["rivers"] == false, "Undo restored the preset only; the layer stays hidden, as the card said")
	(_app.viewport as Node).call("set_layer_visible", "rivers", true)

	# ---------------------------------------------------------------- 3. SELECT
	print("== 3. a selection card ==")
	var sname := String((all[3] as Dictionary).get("name", ""))
	_ok(sname != "", "settlement 3 has a name (%s)" % sname)
	_app.clear_selection()
	## A cursor away from 0, so "it is at N now" cannot pass with a literal 0 (a mutant did).
	_app.tl_set_year(int(_app.tl_range_min()) + 7)
	var year0 := int(_app.tl_year())
	_ok(year0 != 0 and year0 != 398, "the year cursor sits at %d before the card" % year0)
	var pre3 := _plan()
	n = await _ask([{"calls": [{"name": "list_actions", "args": {"path": "map/selection"}}, {"name": "list_actions", "args": {"path": "story/time"}}]},
		{"name": "propose_actions", "args": {"summary": "Go there", "actions": [{"id": "select.settlement", "args": {"id": 3}}, {"id": "timeline.set_year", "args": {"year": 398}}]}}], "show me that place in 398")
	var c3 := _action_card()
	var t3 := _texts(c3["box"]) if not c3.is_empty() else ""
	_ok(t3.contains("MODEL PROPOSAL · SELECT"), "the chip names the kind: SELECT")
	_ok(t3.contains("Select %s (settlement 3)" % sname), "the card NAMES the settlement, not only its number: %s" % sname)
	_ok(t3.contains("Move the year cursor to 398 (it is at %d now)" % year0), "the year row says where the cursor is now (%d)" % year0)
	_ok(t3.contains("Session state:") and JSON.stringify(_plan()) == JSON.stringify(pre3), "session-state wording, and nothing moved before Apply")
	_ok(await _click(c3.get("apply")), "Apply clicked")
	var post3 := _plan()
	_ok(post3["selection"] == {"kind": "settlement", "id": 3} and int(post3["year"]) == 398, "settlement 3 is selected and the year is 398")
	_ok(int(c3["group_id"]) == 0, "no undo group was opened for session state")
	_app.tl_set_year(year0)
	_app.clear_selection()
	# a number outside the list is refused (the catalogue's bound), never worded as a name
	var far := all.size() + 40
	n = await _ask([{"calls": [{"name": "list_actions", "args": {"path": "map/selection"}}]},
		{"name": "propose_actions", "args": {"summary": "Nowhere", "actions": [{"id": "select.settlement", "args": {"id": far}}]}}], "select a far place")
	var c3b := _action_card()
	var t3b := _texts(c3b["box"]) if not c3b.is_empty() else ""
	_ok(not c3b.is_empty() and c3b["apply"] == null and not t3b.contains("Select  ("), "an out-of-range settlement is refused with no Apply: %s" % t3b.left(140).replace("\n", " | "))
	_ok((_app.selection as Dictionary).is_empty(), "nothing was selected")

	# ---------------------------------------------------------------- 4. EDIT
	print("== 4. an edit card (arm a tool) and Undo ==")
	n = await _ask([{"calls": [{"name": "list_actions", "args": {"path": "civil/tools"}}]},
		{"name": "propose_actions", "args": {"summary": "Draw a road", "actions": [{"id": "tool.way"}]}}], "let me draw a road")
	var c4 := _action_card()
	var t4 := _texts(c4["box"]) if not c4.is_empty() else ""
	_ok(t4.contains("MODEL PROPOSAL · EDIT") and t4.contains("Arm the way tool: you then draw on the map yourself, this card draws nothing"), "the EDIT card says it arms a tool and draws nothing")
	_ok(t4.contains("Arming a tool changes nothing in the world until you use it."), "the section says the world is unchanged until the tool is used")
	_ok(await _click(c4.get("apply")), "Apply clicked")
	_ok(String(_app.armed_tool) == "way", "the Way tool is armed")
	_app.arm_tool("inspect")
	# an applied setting, then a card that is Undo alone
	n = await _ask([{"calls": [{"name": "search_actions", "args": {"query": "peak altitude"}}]},
		{"name": "propose_actions", "args": {"summary": "Taller again", "actions": [{"id": "param:peak_m", "args": {"value": peak1}}]}}], "taller again")
	await _click(_action_card().get("apply"))
	_ok(is_equal_approx(float(_bridge.param_get("peak_m")), peak1), "a setting was applied so there is a step to undo")
	n = await _ask([{"calls": [{"name": "search_actions", "args": {"query": "undo"}}]}, {"name": "propose_actions", "args": {"summary": "Undo", "actions": [{"id": "edit.undo"}]}}], "undo that")
	var c4u := _action_card()
	var t4u := _texts(c4u["box"]) if not c4u.is_empty() else ""
	_ok(t4u.contains("Undo the last step ("), "the EDIT card names the step Undo would revert: %s" % t4u.left(140).replace("\n", " | "))
	_ok(await _click(c4u.get("apply")), "Apply clicked")
	await _settle(6)
	_ok(is_equal_approx(float(_bridge.param_get("peak_m")), peak0), "the card's Undo reverted the setting through Edit > Undo")

	# ---------------------------------------------------------------- 5. WINDOW
	print("== 5. a window card ==")
	_app.faction_roster_window.hide()
	n = await _ask([{"calls": [{"name": "search_actions", "args": {"query": "factions window"}}]},
		{"name": "propose_actions", "args": {"summary": "Open factions", "actions": [{"id": "window.factions"}]}}], "open the factions window")
	var c5 := _action_card()
	var t5 := _texts(c5["box"]) if not c5.is_empty() else ""
	_ok(t5.contains("MODEL PROPOSAL · WINDOW") and t5.contains("it opens for you to use, the assistant does not work inside it"), "the WINDOW card says it opens for the person")
	_ok(not _app.faction_roster_window.visible, "the window is not open before Apply")
	_ok(await _click(c5.get("apply")), "Apply clicked")
	_ok(bool(_app.faction_roster_window.visible), "the faction roster is open")
	_app.faction_roster_window.hide()

	# ---------------------------------------------------------------- 6. MIXED, reading order, Dismiss
	print("== 6. a mixed card: sections in reading order; Dismiss ==")
	var pre6 := _plan()
	n = await _ask([{"calls": [{"name": "search_actions", "args": {"query": "peak altitude"}}, {"name": "list_actions", "args": {"path": "carto/layers"}},
			{"name": "list_actions", "args": {"path": "map/selection"}}, {"name": "search_actions", "args": {"query": "factions window"}}]},
		{"name": "propose_actions", "args": {"summary": "Mixed", "actions": [{"id": "window.factions"}, {"id": "select.settlement", "args": {"id": 2}},
			{"id": "view.layer.roads", "args": {"visible": false}}, {"id": "param:peak_m", "args": {"value": peak1}}]}}], "mixed")
	var c6 := _action_card()
	var t6 := _texts(c6["box"]) if not c6.is_empty() else ""
	var iset := _line_of(t6, "SETTING")
	var iview := _line_of(t6, "VIEW")
	var isel := _line_of(t6, "SELECT")
	var iwin := _line_of(t6, "WINDOW")
	_ok(t6.contains("MODEL PROPOSAL · ACTIONS"), "a mixed card's chip says ACTIONS")
	_ok(iset >= 0 and iset < iview and iview < isel and isel < iwin, "sections run setting, view, select, window (lines %d %d %d %d)" % [iset, iview, isel, iwin])
	_ok(not c6.is_empty() and String(c6["apply"].text) == "Apply 4 actions", "one Apply covers all four")
	_ok(await _click(c6.get("dismiss")), "Dismiss clicked")
	_ok(c6["state"] == "dismissed" and JSON.stringify(_plan()) == JSON.stringify(pre6), "Dismiss changed nothing")
	_finish()
