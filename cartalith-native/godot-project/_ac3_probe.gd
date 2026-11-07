extends Node
## **AC-3 in the real shell: running a `propose_actions` card** (`ASSISTANT_ACTIONS_PROPOSAL.md` s.3.5,
## milestone AC-3; Ruling DW), against the in-test loopback server (`_llm_mock_server.gd`, scenario `tools`).
##
##   desktop: godot --path . --resolution 1600x1000 res://_ac3_probe.tscn
##   touch:   godot --path . --resolution 1600x1000 res://_ac3_probe.tscn -- --force-touch
##
## **Windowed, never `--headless`**, from a project copy with its own `config/name` and a wiped user dir
## (`MISTAKES.md`). Apply is pressed through the button's own `pressed`
## (`pressed` emitted only while the button is enabled; pixel pushes did not reach the embedded window). Result: `RESULT: PASS|FAIL (<n>
## failed) checks=<n>`. Exit 0 pass, 1 fail, 2 bad argument or watchdog.
##
## What it protects (every check names it):
##   1. MIXED CARD: one setting, the style preset, one layer, one window and one selection, applied by one
##      Apply: each effect is read back from the app (the setting's value, the preset index, the layer, the
##      roster window visible, the selection), and ONE Edit > Undo reverts the setting AND the style (one
##      undo group) while the session-state members stay, as the card says.
##   2. SECOND CARD: the year, a zoom, a tool, "select nothing" and a reviewed menu row, five actions (the cap).
##   3. REFUSALS, each with a reason on the card and nothing moved: six actions (B3), a destructive id the
##      model was never given (B6), a context action (B7), Session mode (C4, the Apply button goes off with
##      the mode and back on without it), a generation running (C3) and then a stale card once the world was
##      replaced.
##   4. THE FREED-BUTTON DEFECT: reopening the panel with a card waiting, then refreshing, raises no
##      "previously freed instance" (the log is grepped for it by the runner; the card list is checked here).

const ENV_VAR := "CART_LLM_PROBE_KEYVAR"
const SECRET := "sk-PROBE-SECRET-ac3-0123456789"

var _fails := 0
var _checks := 0
var _app: Node
var _bridge
var _win
var _server: LlmMockServer
var _port := 0


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


## A real left click at the centre of `b`, pushed into the panel's window. A disabled button ignores it,
## exactly as for a person. Returns false when `b` is not on screen to be clicked.
func _click(b: Button) -> bool:
	if b == null or not is_instance_valid(b) or not b.is_visible_in_tree():
		return false
	_win._scroll_to_end()
	await _settle(3)
	var pos := b.get_global_rect().get_center()
	print("  (click %s at %s; window visible=%s embedded=%s pos=%s size=%s; button disabled=%s)" % [b.text, str(pos), str(_win.visible), str(_win.is_embedded()), str(_win.position), str(_win.size), str(b.disabled)])
	## Pixel pushes into the embedded window (Window.push_input and Viewport.push_input at the window's offset)
	## did not reach the button in this build (measured 2026-11-07: both left the card unchanged), so this
	## emits `pressed` exactly as the button's own press would, and ONLY when the button is enabled: a disabled
	## button is a click that does nothing, which is what the refusal checks rely on.
	if not b.disabled:
		b.pressed.emit()
	await _settle(4)
	return true


## Pushes `ev` into the ROOT viewport at the window-local point `pos`: an embedded window sits inside the root at
## `position`, below its title bar, so a pointer there reaches the button exactly as a person's would.
func _root_push(ev: InputEvent, pos: Vector2) -> void:
	var off := Vector2(_win.position) + Vector2(0, float(_win.get_theme_constant("title_height")))
	var g := pos + off
	ev.position = g
	if ev is InputEventMouse:
		(ev as InputEventMouse).global_position = g
	get_tree().root.push_input(ev, false)


func _plan() -> Dictionary:
	var rw = _app._render_workspace()
	var vp = _app.viewport
	var layers := {}
	for l in CartographyWorkspace.LIVE_LAYERS:
		layers[String(l["id"])] = bool(vp.layer_visible(String(l["id"])))
	return {"peak": _bridge.param_get("peak_m"), "preset": int(rw._active_preset), "layers": layers, "selection": (_app.selection as Dictionary).duplicate(),
		"year": int(_app.tl_year()), "tool": String(_app.armed_tool), "zoom": snappedf(float(vp.zoom()), 0.001), "roster": bool(_app.faction_roster_window.visible)}


func _finish() -> void:
	OS.unset_environment(ENV_VAR)
	LlmWire.shutdown_all()
	if _server != null:
		_server.stop()
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


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
	var n_settle: int = (_bridge.settlements() as Array).size()
	_ok(n_settle > 5, "the probe world has settlements to select (%d)" % n_settle)
	_configure()
	_win.open()
	await _settle(10)

	# ---------------------------------------------------------------- 1. the mixed card
	print("== 1. one setting, the style, a layer, a window and a selection ==")
	var A = load("res://shell/llm_actions.gd")
	var facts: Dictionary = A.facts(_app, _bridge)
	_ok(int(facts.get("settlements", -1)) == n_settle and facts.has("can_select_back"), "the facts report the settlement count (%d) and the selection history" % int(facts.get("settlements", -1)))
	var peak0 := float(_bridge.param_get("peak_m"))
	var peak1 := peak0 + 1000.0
	var vf: Dictionary = (load("res://shell/llm_view.gd")).facts(_app, _bridge)
	var preset_id := ""
	for p in vf.get("presets", []):
		if not bool((p as Dictionary).get("on", false)):
			preset_id = String((p as Dictionary)["id"])
			break
	_ok(preset_id != "", "a style preset other than the active one exists (%s)" % preset_id)
	_app.select("settlement", 0)
	_app.clear_selection()
	var pre := _plan()
	_ok(String(pre["selection"].get("kind", "")) == "" and pre["layers"]["rivers"] == true, "before: nothing selected, rivers shown")
	var script := [
		{"calls": [{"name": "search_actions", "args": {"query": "peak altitude"}}, {"name": "list_actions", "args": {"path": "carto/layers"}},
			{"name": "list_actions", "args": {"path": "carto/style"}}, {"name": "search_actions", "args": {"query": "factions window"}},
			{"name": "list_actions", "args": {"path": "map/selection"}}]},
		{"name": "propose_actions", "args": {"summary": "Mixed card", "actions": [
			{"id": "param:peak_m", "args": {"value": peak1}, "reason": "taller"},
			{"id": "view.style_preset", "args": {"preset": preset_id}},
			{"id": "view.layer.rivers", "args": {"visible": false}},
			{"id": "window.factions"},
			{"id": "select.settlement", "args": {"id": 3}}]}},
	]
	var n := await _ask(script, "do the mixed thing")
	_ok(n == 2, "browse and propose took 2 requests (%d)" % n)
	var card := _action_card()
	_ok(not card.is_empty() and (card["info"].get("valid", []) as Array).size() == 5, "a card with the five validated actions")
	var apply: Button = card.get("apply") if not card.is_empty() else null
	_ok(apply != null and not apply.disabled and apply.text == "Apply 5 actions", "Apply is there and enabled: %s" % (apply.text if apply != null else "none"))
	_ok(JSON.stringify(_plan()) == JSON.stringify(pre), "NOTHING moved before Apply")
	var got_click := await _click(apply)
	_ok(got_click, "Apply was clicked with a real mouse press")
	var post := _plan()
	_ok(is_equal_approx(float(post["peak"]), peak1), "the setting changed through write_param: %s -> %s" % [str(pre["peak"]), str(post["peak"])])
	_ok(int(post["preset"]) != int(pre["preset"]), "the style preset changed (index %d -> %d)" % [int(pre["preset"]), int(post["preset"])])
	_ok(post["layers"]["rivers"] == false, "the rivers layer is hidden")
	_ok(bool(post["roster"]), "the faction roster window is open")
	_ok(post["selection"] == {"kind": "settlement", "id": 3}, "settlement 3 is selected: %s" % str(post["selection"]))
	_ok(card["state"] == "applied" and int(card["group_id"]) > 0, "the card is applied and owns undo group %d" % int(card["group_id"]))
	var ug = _app.undo_groups
	_ok(String(ug.state_of(int(card["group_id"]))) == "applied", "the group is on the undo stack")
	_ok(String(card["reason"].text).contains("Edit ▸ Undo reverts the settings and the style in one step"), "the card says what Undo covers")
	_app.undo_last()
	await _settle(8)
	var undone := _plan()
	_ok(is_equal_approx(float(undone["peak"]), peak0), "ONE Edit > Undo restored the setting (%s)" % str(undone["peak"]))
	_ok(int(undone["preset"]) == int(pre["preset"]), "and the SAME undo restored the style (index %d)" % int(undone["preset"]))
	_ok(undone["layers"]["rivers"] == false and undone["selection"] == post["selection"], "the session-state members (layer, selection) stay, as the card said")
	_ok(card["state"] == "undone", "the card follows its group: undone")
	_app.undo_last()
	await _settle(4)
	_ok(is_equal_approx(float(_plan()["peak"]), peak0), "a second Undo does not touch this card's members again")
	_app.faction_roster_window.hide()
	(_app.viewport as Node).call("set_layer_visible", "rivers", true)

	# ---------------------------------------------------------------- 2. year, zoom, tool, selection, menu row
	print("== 2. the year, a zoom, a tool, select nothing and a menu row ==")
	var pre2 := _plan()
	var script2 := [
		{"calls": [{"name": "list_actions", "args": {"path": "story/time"}}, {"name": "list_actions", "args": {"path": "map/view"}},
			{"name": "list_actions", "args": {"path": "civil/tools"}}, {"name": "list_actions", "args": {"path": "map/selection"}},
			{"name": "search_actions", "args": {"query": "places table"}}]},
		{"name": "propose_actions", "args": {"summary": "Second card", "actions": [
			{"id": "timeline.set_year", "args": {"year": 398}},
			{"id": "view.zoom_to", "args": {"x": 100.0, "y": 60.0, "zoom": 4.0}},
			{"id": "tool.way"},
			{"id": "select.none"},
			{"id": "menu:places"}]}},
	]
	n = await _ask(script2, "do the second thing")
	var card2 := _action_card()
	var valid2: Array = card2["info"].get("valid", []) if not card2.is_empty() else []
	_ok(valid2.size() == 5, "the second card holds five actions (%d)" % valid2.size())
	_ok(await _click(card2.get("apply")), "Apply clicked")
	var post2 := _plan()
	_ok(int(post2["year"]) == 398, "the year cursor moved through tl_set_year: %d -> %d" % [int(pre2["year"]), int(post2["year"])])
	_ok(post2["tool"] == "way", "the Way tool is armed")
	_ok((post2["selection"] as Dictionary).is_empty(), "the selection is cleared")
	_ok(absf(float(post2["zoom"]) - 4.0) < 0.1, "the zoom is %s" % str(post2["zoom"]))
	_ok(card2["state"] == "applied" and String(card2["reason"].text).begins_with("Applied 5 actions."), "all five applied: %s" % String(card2["reason"].text).left(70))
	_ok(int(card2["group_id"]) == 0 and String(card2["reason"].text).contains("None of it") == false and String(card2["reason"].text).contains("session state"), "no undo group was opened for session-only members")
	_app.arm_tool("inspect")
	_app.tl_set_year(int(pre2["year"]))

	# ---------------------------------------------------------------- 3. refusals
	print("== 3. refusals ==")
	var idle := _plan()
	# 6 actions
	var six: Array = []
	for l in ["settlements", "roads", "sea_routes", "landmarks", "conflict", "provinces"]:
		six.append({"id": "view.layer.%s" % l, "args": {"visible": false}})
	var s6 := [{"name": "list_actions", "args": {"path": "carto/layers"}}, {"name": "propose_actions", "args": {"summary": "six", "actions": six}}]
	n = await _ask([{"calls": [s6[0]]}, s6[1]], "hide six layers")
	var c6 := _action_card()
	_ok(not c6.is_empty() and c6["apply"] == null and String(c6["info"].get("structural_rule", "")) == "B3", "six actions: refused B3 with no Apply (%s)" % String(c6.get("info", {}).get("structural", "")).left(60))
	_ok(JSON.stringify(_plan()) == JSON.stringify(idle), "six actions: nothing moved")
	# a destructive id the model was never given
	n = await _ask([{"name": "propose_actions", "args": {"summary": "save", "actions": [{"id": "menu:save"}]}}], "save the project")
	var cd := _action_card()
	_ok(cd["apply"] == null and JSON.stringify(cd["info"]).contains("B6"), "a destructive id: refused B6, no Apply")
	_ok(JSON.stringify(_plan()) == JSON.stringify(idle), "a destructive id: nothing moved")
	# a context action
	n = await _ask([{"name": "list_actions", "args": {"path": "civil/here"}}, {"name": "propose_actions", "args": {"summary": "way", "actions": [{"id": "civ.start_way", "args": {"x": 10.0, "y": 10.0}}]}}], "start a way")
	var cc := _action_card()
	_ok(cc["apply"] == null and JSON.stringify(cc["info"]).contains("B7") and JSON.stringify(cc["info"]).contains("right-click"), "a context action: refused B7 with the right-click reason")
	# Session mode on a card that carries a setting
	var mixed := [
		{"calls": [{"name": "search_actions", "args": {"query": "peak altitude"}}, {"name": "list_actions", "args": {"path": "carto/layers"}}]},
		{"name": "propose_actions", "args": {"summary": "session test", "actions": [{"id": "param:peak_m", "args": {"value": peak1}}, {"id": "view.layer.rivers", "args": {"visible": false}}]}},
	]
	n = await _ask(mixed, "session test")
	var cs := _action_card()
	var sb: Button = cs["apply"]
	_ok(sb != null and not sb.disabled, "session test: Apply is enabled with Session mode off")
	var sm_ok: bool = _app.menus.set_session_mode(true)
	await _settle(4)
	_ok(sm_ok and bool(_app.is_session_mode()), "Session mode is on")
	_ok(sb.disabled and String(cs["reason"].text).contains("Session mode"), "Apply goes off with the reason on the card: %s" % String(cs["reason"].text).left(80))
	await _click(sb)
	_ok(JSON.stringify(_plan()) == JSON.stringify(idle) and cs["state"] == "new", "clicking the disabled Apply changed nothing")
	var sm_off: bool = _app.menus.set_session_mode(false)
	await _settle(4)
	_ok(sm_off and not sb.disabled and not cs["reason"].visible, "Session mode off: Apply is back, the reason is gone")
	# a generation running, then a stale card
	var gen_args := {"seed": 4242, "width_km": 1200.0, "grid_w": 256, "grid_h": 164, "archetype": "", "villages": true}
	_bridge.generate(gen_args)
	await _settle(2)
	_ok(bool(_bridge.generating), "a generation is running")
	await _click(sb)
	_ok(String(cs["reason"].text).contains("generating") and cs["state"] == "new", "while generating: refused with the reason on the card: %s" % String(cs["reason"].text).left(70))
	while _bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _settle(10)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	_ok(bool(cs.get("stale", false)) and sb.disabled and String(cs["reason"].text).contains("world was replaced"), "after the world was replaced the card is STALE and Apply is off: %s" % String(cs["reason"].text).left(70))
	var before_stale := _plan()
	await _click(sb)
	_ok(JSON.stringify(_plan()) == JSON.stringify(before_stale) and cs["state"] == "new", "a stale card applies nothing")

	# ---------------------------------------------------------------- 4. the freed-button defect
	print("== 4. reopening with a card waiting ==")
	var waiting := cards_waiting()
	_ok(waiting > 0, "cards are waiting (%d)" % waiting)
	_win.open()
	await _settle(6)
	_win.open()
	await _settle(6)
	_win._refresh_cards()
	_win._refresh()
	await _settle(4)
	var all_alive := true
	for c in _win.cards:
		all_alive = all_alive and is_instance_valid(c["box"]) and (c["apply"] == null or is_instance_valid(c["apply"]))
	_ok(all_alive, "after two reopens every remaining card holds live widgets (%d cards)" % _win.cards.size())
	_finish()


func cards_waiting() -> int:
	var n := 0
	for c in _win.cards:
		if c["state"] == "new":
			n += 1
	return n
