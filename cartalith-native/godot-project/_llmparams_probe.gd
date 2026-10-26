extends Node
## **LLM-6a/6b in the real shell: setting proposals, the card, Apply and its one undo step**
## (`LLM_ASSISTANT_SCOPE.md` s.5b, LLM-6 bar; Ruling DI), against the in-test loopback server
## (`_llm_mock_server.gd`, scenarios `tools` and `tools_anthropic`) on BOTH wires.
##
##   desktop: godot --path . --resolution 1600x1000 res://_llmparams_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1280x800  res://_llmparams_probe.tscn -- --force-touch --shots <dir>
##   phone:   godot --path . --resolution 393x852   res://_llmparams_probe.tscn -- --force-touch --shots <dir>
##
## **Windowed, never `--headless`**, from a project copy with its own `config/name` and a wiped
## user dir (`MISTAKES.md`). Reads `--force-touch` and `--shots <dir>`; anything else exits 2.
##
## What it protects (every check names it):
##   1. GATE: with Commands on but "Send: generation parameters" off, Send is disabled with that
##      reason and a forced Send opens NO connection (Ruling DI P3).
##   2. TWO STEPS, per wire: the first request offers `get_param_schema` and the six read-only
##      query tools of LLM-8 (`_llmquery_probe.gd` owns those) and NOT the proposal tool; the second
##      adds `set_generation_params`, its key enum is exactly the 34 keys of weather + world + tectonics, and it
##      replays the call and the tool result in the wire's own shape; the Ollama preset asks for
##      `reasoning_effort: none`, the Anthropic wire sends no such field.
##   3. CARD: the worked example (s.5b.3) arrives as a card with the MODEL PROPOSAL chip, the five
##      rows with old -> new values and units, the cost note, Apply and Dismiss; and **nothing was
##      written** before the click (the world digest is unchanged, `generation_started` 0).
##   4. APPLY: a real click on Apply writes the five values (each read back with `param_get`),
##      makes ONE undo group, starts no generation; ONE `app.undo_last()` returns the WHOLE world
##      digest to the pre-Apply value, on three seeds.
##   5. HOSTILE, per wire: an unknown key, an out-of-range number, a fractional int, a duplicate,
##      an excluded key and 25 changes each show a refusal reason naming the rule, are retried
##      ONCE (3 requests), offer no Apply, and write nothing (digest unchanged).
##   6. WORDS: a model that answers in text gives "No setting change was proposed" and no card.
##   7. V13: while a world generates, an open card's Apply is disabled with the reason.
##   9. REOPEN: reopening the panel twice with cards present clears the cards array (no freed
##      buttons kept), notes a dropped waiting card, and a card made after the reopen obeys V13.
##   8. DISMISS and TOUCH: Dismiss closes a card writing nothing; on tablet and phone the card's
##      buttons meet the touch floor.
## Result: `RESULT: PASS|FAIL (<n> failed) checks=<n>`. Exit 0 pass, 1 fail, 2 bad argument/watchdog.

const ENV_VAR := "CART_LLM_PROBE_KEYVAR"
const SECRET := "sk-PROBE-SECRET-params-0123456789"
const SEEDS := [9137, 24601, 71077345]
## The six read-only tools (LLM-8) the Commands flow offers beside `get_param_schema`, in the order
## `cartalith_llm::query::QUERY_TOOLS` lists them.
const QUERY_TOOLS := ["get_parameters", "get_changed_parameters", "describe_parameter", "get_ecology_summary", "sample_point", "get_world_summary"]

## The worked example's step-4 arguments (scope s.5b.3), as the model would send them.
const WORKED := {"summary": "Less ocean moisture and a lower relief ceiling.", "changes": [
	{"key": "climate.ocean", "value": 0.7, "reason": "Cut the moisture budget by 30 percent."},
	{"key": "climate.evap", "value": 0.09, "reason": "Less pickup over the sea."},
	{"key": "climate.rain_dep", "value": 0.5, "reason": "Rain out sooner so interiors dry."},
	{"key": "peak_m", "value": 2800, "reason": "Lower the vertical scale by 30 percent."},
	{"key": "tect.alpha", "value": 0.65, "reason": "Weight the tectonic signal less."}]}
const SCHEMA3 := {"name": "get_param_schema", "args": {"groups": ["weather", "world", "tectonics"]}}

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _app: Node
var _bridge
var _win
var _server: LlmMockServer
var _port := 0
var _gen_started := 0


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


func _shot(tag: String) -> void:
	if _shots == "":
		return
	await _settle(4)
	var path := _shots.path_join("llm6_%s_%s.png" % [_mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


## A REAL mouse press and release at the centre of `c` (the `_llmpanel_probe` helper).
func _click(c: Control) -> void:
	var local := c.get_global_rect().get_center()
	var vp := c.get_viewport()
	var at: Vector2 = vp.get_final_transform() * local
	if vp is Window and vp != get_tree().root:
		at += Vector2((vp as Window).position)
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	Input.parse_input_event(mm)
	await _settle(2)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(3)


func _all_text(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
	if n is Control:
		out.append((n as Control).tooltip_text)
	for ch in n.get_children(true):
		_all_text(ch, out)


func _text_of(n: Node) -> String:
	var parts: Array = []
	_all_text(n, parts)
	return "\n".join(parts)


func _reset_llm() -> void:
	DccSettings._ensure_loaded()
	if DccSettings._cfg.has_section("llm"):
		DccSettings._cfg.erase_section("llm")
		DccSettings._save()


func _configure(over: Dictionary = {}) -> void:
	var s := {"enabled": true, "commands": true, "prose": false, "local_allowed": true, "preset_id": "ollama",
		"base_url": "http://127.0.0.1:%d/v1" % _port, "model": "mock-model",
		"key_source": "env", "env_var": ENV_VAR, "ctx_params": true}
	s.merge(over, true)
	for k in s:
		DccSettings.set_llm_value(k, s[k])
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % _port)


func _type(text: String) -> void:
	var inp := _c("input") as TextEdit
	inp.text = text
	inp.text_changed.emit()
	await _settle(3)


func _digest() -> Dictionary:
	return (_bridge.edit_state_digest() as Dictionary).duplicate()


func _body(i: int) -> Dictionary:
	if i < 0 or i >= _server.requests.size():
		return {}
	var b = JSON.parse_string(String(_server.requests[i]["body"]))
	return b if b is Dictionary else {}


func _tool_names(b: Dictionary, anthropic: bool) -> Array:
	var out: Array = []
	for t in b.get("tools", []):
		out.append(String(t["name"]) if anthropic else String(t["function"]["name"]))
	return out


func _key_enum(b: Dictionary, anthropic: bool) -> Array:
	for t in b.get("tools", []):
		var name := String(t["name"]) if anthropic else String(t["function"]["name"])
		if name == "set_generation_params":
			var sch: Dictionary = t["input_schema"] if anthropic else t["function"]["parameters"]
			return sch["properties"]["changes"]["items"]["properties"]["key"]["enum"]
	return []


## Sends `request` through the panel's Send button with `script` on `scenario`, and waits until
## the run is idle. Returns the number of requests the server saw for it.
func _ask(scenario: String, script: Array, request: String) -> int:
	_server.scenario = scenario
	_server.tool_script = script
	_server.tool_served = 0
	var n0: int = _server.requests.size()
	_win._pick_task("commands")
	await _type(request)
	var send := _c("send") as Button
	if send.disabled:
		print("  (Send disabled: %s)" % (_c("reason") as Label).text)
	await _click(send)
	await _until(func(): return not _win.run.busy() and _server.requests.size() > n0, 15000)
	await _until(func(): return not _win.run.busy(), 15000)
	await _settle(6)
	return _server.requests.size() - n0


func _last_card() -> Dictionary:
	return _win.cards[_win.cards.size() - 1] if not _win.cards.is_empty() else {}


func _finish() -> void:
	OS.unset_environment(ENV_VAR)
	LlmWire.shutdown_all()
	if _server != null:
		_server.stop()
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _generate(seed_: int) -> void:
	_bridge.generate({"seed": seed_, "width_km": 1200.0, "grid_w": 256, "grid_h": 164, "archetype": "", "villages": false})
	while _bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _settle(8)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--force-touch":
			pass
		elif args[i] == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		else:
			print("ABORT: unrecognised argument %s -- this probe reads --force-touch and --shots <dir> only" % args[i])
			get_tree().quit(2)
			return
		i += 1
	if DisplayServer.get_name() == "headless":
		print("ABORT: run windowed; this probe clicks real buttons and saves frames")
		get_tree().quit(2)
		return
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
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
	_mode = "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s, viewport %s ==" % [_mode, get_viewport().get_visible_rect().size])
	_bridge = _app.bridge
	_win = _app.assistant_window
	_ok(_bridge.llm_api and _win != null and _app.get("undo_groups") != null, "the library has LlmBridge; the shell has the panel and the grouped undo")
	if not _bridge.llm_api or _win == null:
		_finish()
		return
	_bridge.generation_started.connect(func(): _gen_started += 1)
	await _generate(SEEDS[0])

	# ---------------------------------------------------------------- 1. gate
	print("== 1. the parameter context source gates the flow ==")
	_configure({"ctx_params": false})
	_win.open()
	await _settle(10)
	_win._pick_task("commands")
	await _type("drier climate, lower mountains")
	_ok(not (_c("task_commands") as Button).disabled, "the Commands chip is enabled")
	_ok((_c("send") as Button).disabled and (_c("reason") as Label).text.contains("Send: generation parameters"), "with the source unticked Send is disabled, naming it: '%s'" % (_c("reason") as Label).text.left(80))
	var att0 := LlmWire.connect_attempts
	_win._on_send()
	await _settle(8)
	_ok(LlmWire.connect_attempts == att0 and _server.accepted == 0, "a forced Send opened no connection (attempts %d, accepted %d)" % [LlmWire.connect_attempts, _server.accepted])
	_ok(_text_of(_win._msgs).contains("Send: generation parameters"), "...and says why in the panel")
	await _shot("1_gate")

	for wire in ["openai", "anthropic"]:
		var anth: bool = wire == "anthropic"
		var scen := "tools_anthropic" if anth else "tools"
		_configure({"preset_id": "custom_anthropic" if anth else "ollama"})
		_win._refresh()
		print("== 2-3. %s wire: two steps and the card ==" % wire)
		var d_before := _digest()
		var g0 := _gen_started
		var nreq := await _ask(scen, [SCHEMA3, {"name": "set_generation_params", "args": WORKED}], "Make the climate drier and the mountains lower.")
		_ok(nreq == 2, "%s: exactly two requests (schema, proposal): %d" % [wire, nreq])
		var r0: int = _server.requests.size() - nreq
		var b1 := _body(r0)
		var b2 := _body(r0 + 1)
		_ok(_tool_names(b1, anth) == ["get_param_schema"] + QUERY_TOOLS, "%s: request 1 offers get_param_schema and the six read-only tools, not the proposal tool: %s" % [wire, str(_tool_names(b1, anth))])
		_ok(_tool_names(b2, anth) == ["get_param_schema", "set_generation_params"] + QUERY_TOOLS, "%s: request 2 adds the proposal tool to the same set" % wire)
		_ok(_key_enum(b2, anth).size() == 34, "%s: the key enum is the 34 keys of weather + world + tectonics: %d" % [wire, _key_enum(b2, anth).size()])
		_ok(not _key_enum(b2, anth).has("use_gpu") and not _key_enum(b2, anth).has("geology_model"), "%s: no excluded key is offered" % wire)
		var replay := String(_server.requests[r0 + 1]["body"]) if r0 + 1 < _server.requests.size() else ""
		_ok(replay.contains("climate.rain_dep") and replay.contains("first ridge"), "%s: request 2 replays the schema result, desc included" % wire)
		if anth:
			_ok(replay.contains("\"tool_result\"") and replay.contains("\"tool_use\"") and not replay.contains("reasoning_effort"), "anthropic: tool_use / tool_result blocks, no thinking field")
			_ok(r0 < _server.requests.size() and String(_server.requests[r0]["headers"].get("x-api-key", "")) == SECRET, "anthropic: the key goes in x-api-key")
		else:
			_ok(replay.contains("\"tool_call_id\"") and replay.contains("\"role\":\"tool\""), "openai: an assistant tool_calls turn and a role:tool result")
			_ok(b1.get("reasoning_effort", "") == "none", "openai/ollama preset: reasoning_effort none is sent (thinking off)")
		var card := _last_card()
		_ok(not card.is_empty() and card["state"] == "new", "%s: a proposal card is shown" % wire)
		if card.is_empty() or card.get("apply") == null:
			## Nothing below can run without the card; stop rather than error (a mutant run
			## must end with a RESULT line, not hang to the watchdog).
			_ok(false, "%s: the card with an Apply button is missing; stopping" % wire)
			_finish()
			return
		var ct := _text_of(card["box"]) if not card.is_empty() else ""
		_ok(ct.contains("MODEL PROPOSAL"), "%s: the card carries the MODEL PROPOSAL chip" % wire)
		for row in [["climate.ocean", "1.00", "0.70"], ["climate.evap", "0.12", "0.09"], ["climate.rain_dep", "0.35", "0.50"], ["peak_m", "4000", "2800"], ["tect.alpha", "0.85", "0.65"]]:
			_ok(ct.contains("(%s)" % row[0]) and ct.contains(row[1]) and ct.contains(row[2]), "%s: card row %s shows %s -> %s" % [wire, row[0], row[1], row[2]])
		_ok(ct.contains("4000 m") and ct.contains("2800 m") and ct.contains("-30%"), "%s: units and the percentage are shown" % wire)
		_ok(ct.contains("full Generate") and ct.contains("Recompute") and ct.contains("does not regenerate"), "%s: the cost note says what each change needs" % wire)
		_ok(card.get("apply") != null and (card["apply"] as Button).text == "Apply 5 changes" and not (card["apply"] as Button).disabled, "%s: 'Apply 5 changes' is enabled" % wire)
		## The card must not widen the panel: a horizontally-disabled scroller folds a child's
		## minimum width into every ancestor (`MISTAKES.md`), which pushed Send off a phone sheet.
		var srect: Rect2 = (_c("send") as Button).get_global_rect()
		var crect: Rect2 = (card["box"] as Control).get_global_rect()
		var wsz := Vector2(_win.size)
		_ok(srect.end.x <= wsz.x + 0.5 and crect.end.x <= wsz.x + 0.5, "%s: with the card shown, Send %s and the card %s stay inside the window width %.0f" % [wire, srect, crect, wsz.x])
		_ok(_digest() == d_before, "%s: NOTHING was written before the click (world digest unchanged)" % wire)
		_ok(_gen_started == g0, "%s: no generation started" % wire)
		if not anth:
			await _shot("3_card_%s" % wire)
		if _mode != "desktop":
			for k in ["apply", "dismiss"]:
				var bsz: Vector2 = (card[k] as Button).get_global_rect().size
				_ok(bsz.y >= 44.0 - 0.5 and bsz.x >= 44.0 - 0.5, "%s: the card's %s meets the 44 px touch floor: %s" % [wire, k, bsz])

		print("== 4. %s: Apply is one undo step ==" % wire)
		await _click(card["apply"])
		await _settle(6)
		_ok(card["state"] == "applied", "%s: the click applied the card: %s" % [wire, (card["reason"] as Label).text])
		var want := {"climate.ocean": 0.7, "climate.evap": 0.09, "climate.rain_dep": 0.5, "peak_m": 2800.0, "tect.alpha": 0.65}
		var landed := true
		for k in want:
			landed = landed and absf(float(_bridge.param_get(k)) - float(want[k])) < 1e-9
		_ok(landed, "%s: all five values landed exactly (never clamped)" % wire)
		_ok(_digest() != d_before, "%s: positive control: the digest moved" % wire)
		_ok(_gen_started == g0, "%s: Apply started no generation (generation_started %d)" % [wire, _gen_started - g0])
		_ok(String(_app.undo_groups.undo_label()).begins_with("Assistant: "), "%s: Edit > Undo names the group: '%s'" % [wire, _app.undo_groups.undo_label()])
		if not anth:
			await _shot("4_applied")
		_app.undo_last()
		await _settle(6)
		_ok(_digest() == d_before, "%s: ONE undo returns the whole world digest to before the Apply" % wire)
		_ok(absf(float(_bridge.param_get("peak_m")) - 4000.0) < 1e-9, "%s: peak_m is 4000 again" % wire)
		_ok(_gen_started == g0, "%s: the undo started no generation" % wire)

		print("== 5. %s: hostile proposals ==" % wire)
		var many: Array = []
		for j in 25:
			many.append({"key": "peak_m", "value": 1000 + j})
		var hostile := [
			["unknown key", {"changes": [{"key": "climate.aridity", "value": 0.8}]}, "V5"],
			["out of range", {"changes": [{"key": "climate.rain_k", "value": 2.7}]}, "V10"],
			["fractional int", {"changes": [{"key": "tect.plates", "value": 3.5}]}, "V8"],
			["duplicate", {"changes": [{"key": "peak_m", "value": 3000}, {"key": "peak_m", "value": 2500}]}, "V7"],
			["excluded key", {"changes": [{"key": "use_gpu", "value": false}]}, "V6"],
			["25 changes", {"changes": many}, "V3"],
		]
		for h in hostile:
			var dh := _digest()
			var cards0: int = _win.cards.size()
			var bad := {"name": "set_generation_params", "args": h[1]}
			var n := await _ask(scen, [SCHEMA3, bad, bad], "Make it %s." % h[0])
			_ok(n == 3, "%s %s: schema, proposal and ONE retry = 3 requests (%d)" % [wire, h[0], n])
			var msgs := _text_of(_win._msgs)
			_ok(msgs.contains("could not be used") and msgs.contains(String(h[2])), "%s %s: the panel shows the refusal with rule %s" % [wire, h[0], h[2]])
			var hc := _last_card()
			_ok(not hc.is_empty() and _win.cards.size() == cards0 + 1 and hc.get("apply") == null and _text_of(hc["box"]).contains("Nothing could be applied") and _text_of(hc["box"]).contains(String(h[2])), "%s %s: the final card lists the refusal (%s) and has NO Apply" % [wire, h[0], h[2]])
			_ok(_digest() == dh, "%s %s: nothing was written" % [wire, h[0]])
			if h[0] == "unknown key" and not anth:
				await _shot("5_hostile_unknown")
			if h[0] == "25 changes" and not anth:
				await _shot("5_hostile_25")
		_ok(_gen_started == g0, "%s: no generation over the hostile cases" % wire)

		print("== 6. %s: an answer in words ==" % wire)
		var cards1: int = _win.cards.size()
		var nn := await _ask(scen, [{"text": "I cannot delete a named ocean; these settings change the whole world."}], "delete the Pacific")
		_ok(nn == 1 and _win.cards.size() == cards1 and _text_of(_win._msgs).contains("No setting change was proposed"), "%s: words, no card (requests %d)" % [wire, nn])

	print("== 4b. Apply then one undo on two more seeds ==")
	_configure({"preset_id": "ollama"})
	for s in [SEEDS[1], SEEDS[2]]:
		await _generate(s)
		var d0 := _digest()
		var g1 := _gen_started
		await _ask("tools", [SCHEMA3, {"name": "set_generation_params", "args": WORKED}], "drier, lower")
		var card2 := _last_card()
		_ok(card2.get("apply") != null, "seed %d: a card with Apply" % s)
		if card2.get("apply") == null:
			continue
		await _click(card2["apply"])
		await _settle(4)
		_ok(card2["state"] == "applied" and _digest() != d0, "seed %d: applied, digest moved" % s)
		_app.undo_last()
		await _settle(4)
		_ok(_digest() == d0, "seed %d: ONE undo restores the digest" % s)
		_ok(_gen_started == g1, "seed %d: no generation by Apply or Undo" % s)

	print("== 7. V13: Apply disabled while a world generates ==")
	await _ask("tools", [SCHEMA3, {"name": "set_generation_params", "args": WORKED}], "drier")
	var c7 := _last_card()
	if c7.is_empty() or c7.get("apply") == null:
		_ok(false, "7: no card with Apply to test V13 on; stopping")
		_finish()
		return
	_bridge.generate({"seed": 4242, "width_km": 1200.0, "grid_w": 256, "grid_h": 164, "archetype": "", "villages": false})
	await _settle(2)
	_win._refresh()
	await _settle(2)
	var was_gen: bool = _bridge.generating
	_ok(was_gen and (c7["apply"] as Button).disabled and (c7["reason"] as Label).text.contains("generating"), "while generating Apply is disabled with the reason: '%s'" % (c7["reason"] as Label).text)
	while _bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _settle(8)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	_win._refresh()
	await _settle(2)
	_ok(not (c7["apply"] as Button).disabled, "after it finishes Apply is enabled again")

	print("== 8. Dismiss ==")
	var d8 := _digest()
	await _click(c7["dismiss"])
	_ok(c7["state"] == "dismissed" and (c7["apply"] as Button).disabled and _digest() == d8, "Dismiss closes the card and writes nothing")
	_ok(not _text_of(_win).contains(SECRET), "the key is in no label or tooltip of the panel")
	await _shot("8_dismissed")

	print("== 9. Reopening the panel with cards present (REPORT.md Defects 2) ==")
	## Two cards (one new, one dismissed) exist; reopen TWICE. Before the fix `cards` kept the
	## freed buttons and `_refresh_cards` raised "previously freed instance" on the next refresh;
	## the runner greps the log for SCRIPT ERROR, and the checks below assert the array is
	## cleared, a note says a waiting card was dropped, and a card made AFTER the reopen is
	## fully live: Apply is disabled with the reason while a world generates.
	await _ask("tools", [SCHEMA3, {"name": "set_generation_params", "args": WORKED}], "drier again")
	var c9 := _last_card()
	_ok(not c9.is_empty() and c9["state"] == "new" and _win.cards.size() >= 2, "9: a waiting card and an older one exist before the reopen (%d cards)" % _win.cards.size())
	for n in 2:
		_win.open()
		await _settle(4)
		_win._refresh()
		await _settle(2)
		_ok(_win.controls.has("cleared_note") == (n == 0), "9: reopen %d: the cleared-proposal note shows once, on the reopen that dropped a waiting card" % (n + 1))
		_ok(_win.cards.is_empty(), "9: reopen %d clears the cards array (was holding freed buttons)" % (n + 1))
		_ok(not is_instance_valid(c9["box"]) or not c9["box"].is_inside_tree(), "9: reopen %d: the old card's widgets are gone from the panel" % (n + 1))
	await _ask("tools", [SCHEMA3, {"name": "set_generation_params", "args": WORKED}], "drier after reopen")
	var c9b := _last_card()
	_ok(not c9b.is_empty() and c9b.get("apply") != null and not (c9b["apply"] as Button).disabled, "9: a card made after the reopen has an enabled Apply")
	if not c9b.is_empty() and c9b.get("apply") != null:
		_bridge.generate({"seed": 4343, "width_km": 1200.0, "grid_w": 256, "grid_h": 164, "archetype": "", "villages": false})
		await _settle(2)
		_win._refresh()
		await _settle(2)
		_ok(_bridge.generating and (c9b["apply"] as Button).disabled and (c9b["reason"] as Label).text.contains("generating"), "9: V13 holds for a card made after the reopen: '%s'" % (c9b["reason"] as Label).text)
		while _bridge.generating:
			await get_tree().create_timer(0.2).timeout
		await _settle(8)
		if _app.open_project_dialog:
			_app.open_project_dialog.hide()
	_finish()
