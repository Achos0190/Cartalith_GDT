extends Node
## **AC-4 and AC-5 in the real shell: the preloaded action pack, pre-retrieval, Undo and the master switch**
## (`ASSISTANT_ACTIONS_PROPOSAL.md` s.3.4, milestones AC-4 and AC-5; Ruling DW), against the in-test loopback
## server (`_llm_mock_server.gd`, scenario `tools`).
##
##   desktop: godot --path . --resolution 1600x1000 res://_ac4_probe.tscn
##   touch:   godot --path . --resolution 1600x1000 res://_ac4_probe.tscn -- --force-touch
##
## **Windowed, never `--headless`**, from a project copy with its own `config/name` and a wiped user dir
## (`MISTAKES.md`). Result: `RESULT: PASS|FAIL (<n> failed) checks=<n>`. Exit 0 pass, 1 fail, 2 bad argument
## or watchdog.
##
## What it protects (every check names it):
##   1. THE COMMANDS BUTTON PRELOADS ITS PACK: the request's first user message starts with one
##      `<context kind="actions" origin="pack">` block naming the eight pack ids, the system prompt names the
##      block, and a model that proposes a pack id WITHOUT any list or search call gets a valid card in ONE
##      request (the ids count as returned). A guessed destructive id is still refused (B6).
##   2. PRE-RETRIEVAL IS OFF BY DEFAULT: the setting reads false, the block's origin is `pack`, and no
##      "Looked up" line is shown. Switched on, the block's origin is `search+pack`, its first action is the
##      one the words name, the user's words are NOT inside the block, and the panel says how many it found.
##   3. THE OTHER BUTTONS PRELOAD NOTHING: a Story request carries no actions block and no tool.
##   4. UNDO (AC-5): `facts` reports the undo step; the pack names `edit.undo` with the step's label while a
##      step exists; a card that is Undo alone applies through `DccApp.undo_last` and the setting it reverts
##      comes back; Undo beside another action is refused C8 on the card with Apply off.
##   5. THE GUARD DROPS THE PACK FIRST: a request that fits without the block and not with it is sent
##      without the block, and the panel says so; a request that does not fit even without it is refused.
##   6. OFFLINE: with the master switch off, Send opens no connection and sends nothing.

const ENV_VAR := "CART_LLM_PROBE_KEYVAR"
const SECRET := "sk-PROBE-SECRET-ac4-0123456789"

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
	var s := {"enabled": true, "commands": true, "prose": true, "local_allowed": true, "preset_id": "ollama",
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


## Sends `request` through the real panel in `task` with the scripted replies; returns the requests it took.
func _ask_in(task: String, script: Array, request: String) -> int:
	_server.scenario = "tools"
	_server.tool_script = script
	_server.tool_served = 0
	var n0: int = _server.requests.size()
	_win._pick_task(task)
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


func _ask(script: Array, request: String) -> int:
	return await _ask_in("commands", script, request)


func _body(i: int) -> Dictionary:
	var v = JSON.parse_string(String(_server.requests[i]["body"]))
	return v if typeof(v) == TYPE_DICTIONARY else {}


func _sys_text(b: Dictionary) -> String:
	for m in b.get("messages", []):
		if String((m as Dictionary).get("role", "")) == "system":
			return String((m as Dictionary).get("content", ""))
	return ""


## The first USER message of a request (the one that carries the pack), whatever came after it.
func _user_text(b: Dictionary) -> String:
	for m in b.get("messages", []):
		if String((m as Dictionary).get("role", "")) == "user":
			return String((m as Dictionary).get("content", ""))
	return ""


func _tool_names(b: Dictionary) -> Array:
	var out: Array = []
	for t in b.get("tools", []):
		out.append(String(((t as Dictionary).get("function", {}) as Dictionary).get("name", "")))
	return out


## The ids on the lines of the `actions` block, in order (`- <id> | ...`).
func _block_ids(user: String) -> Array:
	var out: Array = []
	var end := user.find("</context>")
	var blk := user.substr(0, end) if end >= 0 else user
	for line in blk.split("\n"):
		if line.begins_with("- "):
			out.append(line.substr(2).split(" | ")[0])
	return out


func _text_of(n: Node) -> String:
	var parts: Array = []
	_all_text(n, parts)
	return "\n".join(parts)


func _all_text(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
	for ch in n.get_children(true):
		_all_text(ch, out)


func _action_card() -> Dictionary:
	for i in range(_win.cards.size() - 1, -1, -1):
		if String(_win.cards[i].get("kind", "params")) == "actions":
			return _win.cards[i]
	return {}


func _click(b: Button) -> bool:
	if b == null or not is_instance_valid(b) or not b.is_visible_in_tree():
		return false
	_win._scroll_to_end()
	await _settle(3)
	if not b.disabled:
		b.pressed.emit()
	await _settle(6)
	return true


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
	_configure()
	_win.open()
	await _settle(10)

	# ---------------------------------------------------------------- 1. the Commands button's pack
	print("== 1. the Commands button preloads its pack ==")
	var pack_ids := ["view.layer.territory", "view.style_preset", "timeline.set_year", "select.settlement", "window.factions", "view.zoom_to", "tool.way"]
	_ok(DccSettings.llm_value("actions_pre") == false, "pre-retrieval of actions is OFF by default")
	var A = load("res://shell/llm_actions.gd")
	var facts: Dictionary = A.facts(_app, _bridge)
	_ok(facts.has("undo") and bool((facts["undo"] as Dictionary).get("available", true)) == (String(_app.undo_target_label()) != "" or bool(_app.can_undo_any())), "the facts carry the undo step Edit > Undo would take (%s)" % str(facts.get("undo", "absent")))
	var i0: int = _server.requests.size()
	var n := await _ask([{"name": "propose_actions", "args": {"summary": "to 398", "actions": [{"id": "timeline.set_year", "args": {"year": 398}}]}}], "go to the year 398")
	_ok(n == 1, "ONE request: the model proposed straight from the pack, with no list or search call (%d)" % n)
	var b0 := _body(i0)
	var u0 := _user_text(b0)
	_ok(u0.begins_with("<context kind=\"actions\" origin=\"pack\">"), "the user message starts with the pack's fence: %s" % u0.left(46))
	var ids0 := _block_ids(u0)
	var all_there := true
	for id in pack_ids:
		all_there = all_there and ids0.has(id)
	_ok(all_there and ids0.size() <= 8, "the block names the pack ids (%s)" % ", ".join(PackedStringArray(ids0)))
	_ok(u0.count("<context") == 1 and u0.count("</context") == 1 and u0.ends_with("go to the year 398"), "one fenced block, then the user's words")
	_ok(_sys_text(b0).contains("The <context kind=\"actions\"> block in the user's message lists actions the app already found"), "the system prompt names the block")
	_ok(_tool_names(b0).has("list_actions") and _tool_names(b0).has("propose_actions"), "the four catalogue tools are still offered beside the pack")
	var card := _action_card()
	_ok(not card.is_empty() and (card["info"].get("valid", []) as Array).size() == 1 and card["apply"] != null, "the pack id was accepted: a valid card with Apply")
	var shown := _text_of(_win._msgs)
	_ok(not shown.contains("Looked up"), "no 'Looked up' line with pre-retrieval off")
	_ok(await _click(card.get("apply")) and int(_app.tl_year()) == 398, "Apply moved the year to 398 through the same click path as AC-3")
	n = await _ask([{"name": "propose_actions", "args": {"summary": "save", "actions": [{"id": "menu:save"}]}}], "save the project")
	var cd := _action_card()
	_ok(cd["apply"] == null and JSON.stringify(cd["info"]).contains("B6"), "a destructive id the pack never named is still refused (B6)")

	# ---------------------------------------------------------------- 2. pre-retrieval
	print("== 2. pre-retrieval ==")
	var antique := "make the map look like an old antique map"
	i0 = _server.requests.size()
	n = await _ask([{"text": "ok"}], antique)
	var u1 := _user_text(_body(i0))
	_ok(u1.begins_with("<context kind=\"actions\" origin=\"pack\">"), "off: the block's origin is the pack alone")
	DccSettings.set_llm_value("actions_pre", true)
	i0 = _server.requests.size()
	n = await _ask([{"text": "ok"}], antique)
	var u2 := _user_text(_body(i0))
	var ids2 := _block_ids(u2)
	_ok(u2.begins_with("<context kind=\"actions\" origin=\"search+pack\">"), "on: the origin is search+pack: %s" % u2.left(52))
	_ok(ids2.size() > 0 and ids2[0] == "view.style_preset" and ids2.size() <= 8, "the first action is the one the words name (%s), at most eight" % ", ".join(PackedStringArray(ids2)))
	var blk_end := u2.find("</context>")
	_ok(blk_end > 0 and not u2.substr(0, blk_end).contains("antique map") and u2.ends_with(antique), "the user's words are NOT inside the block; they follow it")
	_ok(_text_of(_win._msgs).contains("Looked up"), "the panel says how many matching actions it looked up")
	DccSettings.set_llm_value("actions_pre", false)

	# ---------------------------------------------------------------- 3. the other buttons
	print("== 3. the other buttons preload nothing ==")
	## A Story or Note send needs a place or a note chosen, so this reads the request arguments the panel
	## builds for each task: only the Commands task asks for a pack or sends the actions facts.
	for task in ["story", "note", "prose"]:
		_win._pick_task(task)
		var targs: Dictionary = _win._args("hello")
		_ok(not targs.has("action_pack") and not targs.has("actions"), "%s: the request arguments carry no action pack and no actions facts" % task)


	# ---------------------------------------------------------------- 4. Undo
	print("== 4. Undo ==")
	var peak0 := float(_bridge.param_get("peak_m"))
	var peak1 := peak0 + 1000.0
	var script := [
		{"calls": [{"name": "search_actions", "args": {"query": "peak altitude"}}]},
		{"name": "propose_actions", "args": {"summary": "taller", "actions": [{"id": "param:peak_m", "args": {"value": peak1}}]}},
	]
	n = await _ask(script, "make the mountains taller")
	var cp := _action_card()
	_ok(await _click(cp.get("apply")) and is_equal_approx(float(_bridge.param_get("peak_m")), peak1), "a setting card applied (peak %s -> %s)" % [str(peak0), str(_bridge.param_get("peak_m"))])
	_ok(String(_app.undo_target_label()) != "", "an undo step now exists: %s" % _app.undo_target_label())
	var rivers0: bool = _app.viewport.layer_visible("rivers")
	n = await _ask([{"calls": [{"name": "search_actions", "args": {"query": "undo"}}, {"name": "search_actions", "args": {"query": "rivers layer"}}]},
		{"name": "propose_actions", "args": {"summary": "both", "actions": [{"id": "view.layer.rivers", "args": {"visible": false}}, {"id": "edit.undo"}]}}], "hide the rivers and undo")
	var cb := _action_card()
	_ok(not cb.is_empty() and cb["apply"] != null and cb["apply"].disabled and String(cb["reason"].text).contains("Undo takes a card to itself"), "Undo beside another action: refused C8 on the card, Apply off: %s" % String(cb["reason"].text).left(70))
	await _click(cb.get("apply"))
	_ok(_app.viewport.layer_visible("rivers") == rivers0 and cb["state"] == "new", "clicking the disabled Apply changed nothing")

	i0 = _server.requests.size()
	n = await _ask([{"name": "propose_actions", "args": {"summary": "undo", "actions": [{"id": "edit.undo"}]}}], "undo that")
	var u4 := _user_text(_body(i0))
	_ok(n == 1 and _block_ids(u4).has("edit.undo") and u4.contains("would undo: "), "the pack names edit.undo with the step's label, and the model proposed it from the pack (%d request)" % n)
	var cu := _action_card()
	_ok(not cu.is_empty() and (cu["info"].get("valid", []) as Array).size() == 1 and cu["apply"] != null, "a valid card holding Undo alone")
	_ok(await _click(cu.get("apply")), "Apply clicked")
	_ok(is_equal_approx(float(_bridge.param_get("peak_m")), peak0), "Undo ran through DccApp.undo_last: the setting is back (%s)" % str(_bridge.param_get("peak_m")))
	_ok(cu["state"] == "applied", "the card is applied")
	# ---------------------------------------------------------------- 5. the guard drops the pack first
	print("== 5. the request-size guard ==")
	# The guard's own refusal states the request's size ("about N tokens plus 1024"), so a first send into a
	# 3 000-token window reads the plain request's size N; the block adds about 520 more. A window of N + 1024
	# + 60 then fits the request without the block and not with it.
	DccSettings.set_llm_value("context_tokens", 3000)
	n = await _ask([{"text": "ok"}], "hide the rivers again")
	var rx := RegEx.create_from_string("about (\\d+) tokens plus (\\d+)")
	var m := rx.search(_text_of(_win._msgs))
	_ok(n == 0 and m != null and _text_of(_win._msgs).contains("3000-token window"), "a request that does not fit even without the block is refused with the numbers, not sent or cut (%d requests)" % n)
	if m != null:
		var window := int(m.get_string(1)) + int(m.get_string(2)) + 60
		DccSettings.set_llm_value("context_tokens", window)
		i0 = _server.requests.size()
		n = await _ask([{"text": "ok"}], "hide the rivers")
		_ok(n == 1, "window %d: over it with the block (+~520), under it without: the request went (%d)" % [window, n])
		if n >= 1:
			var ub := _user_text(_body(i0))
			_ok(not ub.contains("kind=\"actions\"") and ub.begins_with("hide the rivers"), "and it went WITHOUT the block")
			_ok(_text_of(_win._msgs).contains("preloaded action list was left out"), "and the panel says so")
	DccSettings.set_llm_value("context_tokens", 0)

	# ---------------------------------------------------------------- 6. offline
	print("== 6. the master switch ==")
	DccSettings.set_llm_value("enabled", false)
	await _settle(4)
	var acc0: int = _server.accepted
	var req0: int = _server.requests.size()
	_win._pick_task("commands")
	await _type("go to the year 120")
	var send := _c("send") as Button
	send.pressed.emit()
	await _settle(20)
	_ok(_server.accepted == acc0 and _server.requests.size() == req0 and not _win.run.busy(), "with the Assistant off, Send opens no connection and sends nothing")
	DccSettings.set_llm_value("enabled", true)
	_finish()
