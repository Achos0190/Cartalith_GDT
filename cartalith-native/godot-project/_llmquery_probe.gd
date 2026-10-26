extends Node
## **LLM-8 and LLM-9 in the real shell: read-only questions and the hostile-string corpus**
## (`LLM_ASSISTANT_SCOPE.md` s.5b.5, s.5b.6, LLM-8 and LLM-9 bars; Rulings DD and DI), driving the
## real Assistant panel against the in-test loopback server (`_llm_mock_server.gd`, scenarios `tools`
## and `tools_anthropic`) on BOTH wires.
##
##   desktop: godot --path . --resolution 1600x1000 res://_llmquery_probe.tscn -- --corpus <json> --shots <dir>
##   tablet:  godot --path . --resolution 1280x800  res://_llmquery_probe.tscn -- --force-touch --corpus <json> --shots <dir>
##   phone:   godot --path . --resolution 393x852   res://_llmquery_probe.tscn -- --force-touch --corpus <json> --shots <dir>
##
## **Windowed, never `--headless`**, from a project copy with its own `config/name` and a wiped user
## dir (`MISTAKES.md`). Reads `--force-touch`, `--corpus <path to hostile_corpus_*.json>`,
## `--no-corpus` (questions only) and `--shots <dir>`; anything else exits 2. Result: `RESULT: PASS|FAIL (<n> failed) checks=<n>`.
##
## What it protects (every check names it):
##   1. GATE (Ruling DI P3): with "Send: generation parameters and summary statistics" off, Send is
##      disabled with that reason and a forced Send opens NO connection, so no tool is offered.
##   2. OFFERED: with it on, request 1 offers exactly `get_param_schema` plus the six read-only tools
##      (no proposal tool before a schema), on both wires; the system prompt names the tools.
##   3. EQUAL TO THE GETTERS (LLM-8 bar): ten scripted questions, each answered by tool calls, whose
##      result in the next request equals the engine's own getter (`get_params`, `get_param_defaults`,
##      `get_param_info`, `ecology_summary`, `sample_cell`, `world_summary`) -- compared to getter
##      output, not wording -- including 100 percent of the 114 offered keys, on three seeds.
##   4. NOT AVAILABLE: a cell outside the grid, and every world tool while a world generates, answer
##      "available: false" with a reason and carry no number; `get_parameters` serves the snapshot.
##   5. BOUNDS: at most eight map cells are read per request; a faction's name never reaches the model.
##   6. HOSTILE CORPUS (LLM-9): every row of the JSON, with the obedient model's worst reply scripted,
##      on both wires: the tool set offered never changes, nothing is written (the world digest, the
##      undo label and the generation count are unchanged), no valid proposal outside the opened
##      groups, markup replies are drawn as plain text (no `RichTextLabel`), no connection but the mock.
##   7. LAYOUT: with an answer and a card shown, Send and the card stay inside the window.
const ENV_VAR := "CART_LLM_PROBE_KEYVAR"
const SECRET := "sk-PROBE-SECRET-query-0123456789"
const SEEDS := [9137, 24601, 71077345]
const QUERY_TOOLS := ["get_parameters", "get_changed_parameters", "describe_parameter", "get_ecology_summary", "sample_point", "get_world_summary"]
## LLM-6d: the real panel also offers the view tool. Its offering is asserted here; the W rows of the
## corpus (view-action rows) are run by `_llmview_probe.gd`, which knows how to grade a view card.
const VIEW_TOOL := "propose_view_actions"
const EXCLUDED := ["use_gpu", "geology_model", "geology_processes", "relief_model", "geology_karst", "geology_carve"]
const GROUPS := ["world", "planet", "world_structure", "tectonics", "volcanism", "erosion", "climate", "weather", "civ"]

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _corpus_path := "../crates/cartalith-llm/tests/fixtures/hostile_corpus_2026-10-26.json"
var _app: Node
var _bridge
var _win
var _server: LlmMockServer
var _port := 0
var _gen_started := 0
## `--no-corpus` stops after the questions (sections 1-5): the mutation runs of the shell code use it
## because the corpus section alone is most of the run time.
var _skip_corpus := false


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
	var path := _shots.path_join("llm8_%s_%s.png" % [_mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


## A REAL mouse press and release at the centre of `c`.
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


## Number of nodes of `cls` under `n` (a `RichTextLabel` would parse model text as markup).
func _count_class(n: Node, cls: String) -> int:
	var k := 1 if n.is_class(cls) else 0
	for ch in n.get_children(true):
		k += _count_class(ch, cls)
	return k


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


## Every tool-result message in a request body, parsed (the model-bound data). OpenAI-compatible:
## `role: tool` messages; Anthropic: `tool_result` blocks of user messages.
func _tool_results(b: Dictionary, anthropic: bool) -> Array:
	var out: Array = []
	for m in b.get("messages", []):
		if anthropic:
			if m is Dictionary and m.get("content") is Array:
				for blk in m["content"]:
					if blk is Dictionary and String(blk.get("type", "")) == "tool_result":
						var r = JSON.parse_string(String(blk.get("content", "")))
						out.append(r if r is Dictionary else {"_unparsed": String(blk.get("content", ""))})
		else:
			if m is Dictionary and String(m.get("role", "")) == "tool":
				var r = JSON.parse_string(String(m.get("content", "")))
				out.append(r if r is Dictionary else {"_unparsed": String(m.get("content", ""))})
	return out


func _system_text(b: Dictionary, anthropic: bool) -> String:
	if anthropic:
		return String(b.get("system", ""))
	for m in b.get("messages", []):
		if m is Dictionary and String(m.get("role", "")) == "system":
			return String(m.get("content", ""))
	return ""


func _near(a, b) -> bool:
	if typeof(a) == TYPE_BOOL or typeof(b) == TYPE_BOOL:
		return bool(a) == bool(b)
	return absf(float(a) - float(b)) <= 1e-9 * maxf(1.0, absf(float(b)))


## Sends `request` through the panel and waits until idle. Returns how many requests the server saw.
func _ask(scenario: String, script: Array, request: String, click := true) -> int:
	_server.scenario = scenario
	_server.tool_script = script
	_server.tool_served = 0
	var n0: int = _server.requests.size()
	_win._pick_task("commands")
	await _type(request)
	var send := _c("send") as Button
	if send.disabled:
		print("  (Send disabled: %s)" % (_c("reason") as Label).text)
	if click:
		await _click(send)
	else:
		## While a world generates, a busy overlay covers the window and swallows a real click;
		## the panel's own Send handler is called instead (the same function the button calls).
		_win._on_send()
		await _settle(4)
	await _until(func(): return not _win.run.busy() and _server.requests.size() > n0, 15000)
	await _until(func(): return not _win.run.busy(), 20000)
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


func _generate(seed_: int, gw: int = 256, gh: int = 164) -> void:
	_bridge.generate({"seed": seed_, "width_km": 1200.0, "grid_w": gw, "grid_h": gh, "archetype": "", "villages": false})
	while _bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _settle(8)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()


func _offered_keys() -> Array:
	var out: Array = []
	for k in _bridge.world_gen.get_param_info().keys():
		if not EXCLUDED.has(String(k)):
			out.append(String(k))
	return out


## One question: tool call(s) then words. Returns the parsed results the model was handed in the
## LAST request, and the number of requests.
func _question(scen: String, anth: bool, calls: Array, answer: String, request: String) -> Dictionary:
	var n := await _ask(scen, [{"calls": calls}, {"text": answer}], request)
	var last: int = _server.requests.size() - 1
	return {"n": n, "first": _body(_server.requests.size() - n), "last": _body(last), "results": _tool_results(_body(last), anth)}


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--force-touch":
			pass
		elif args[i] == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		elif args[i] == "--corpus" and i + 1 < args.size():
			i += 1
			_corpus_path = args[i]
		elif args[i] == "--no-corpus":
			_skip_corpus = true
		else:
			print("ABORT: unrecognised argument %s -- this probe reads --force-touch, --corpus <json>, --no-corpus and --shots <dir> only" % args[i])
			get_tree().quit(2)
			return
		i += 1
	if DisplayServer.get_name() == "headless":
		print("ABORT: run windowed; this probe clicks real buttons and saves frames")
		get_tree().quit(2)
		return
	get_tree().create_timer(1500.0).timeout.connect(func() -> void:
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
	_ok(_bridge.world_gen.has_method("world_summary"), "the library has WorldGen.world_summary (the debug DLL is current)")
	_bridge.generation_started.connect(func(): _gen_started += 1)
	await _generate(SEEDS[0])

	# ---------------------------------------------------------------- 1. gate
	print("== 1. the context source gates the tools ==")
	_configure({"ctx_params": false})
	_win.open()
	await _settle(10)
	_win._pick_task("commands")
	await _type("what is the sea level?")
	var rsn := (_c("reason") as Label).text
	_ok((_c("send") as Button).disabled and rsn.contains("summary statistics"), "with the source unticked Send is disabled, naming it: '%s'" % rsn.left(90))
	var att0 := LlmWire.connect_attempts
	_win._on_send()
	await _settle(8)
	_ok(LlmWire.connect_attempts == att0 and _server.accepted == 0 and _server.requests.is_empty(), "a forced Send opened no connection and no tool was offered (attempts %d, accepted %d)" % [LlmWire.connect_attempts, _server.accepted])
	await _shot("1_gate")

	# ---------------------------------------------------------------- 2-4. questions, both wires
	for wire in ["openai", "anthropic"]:
		var anth: bool = wire == "anthropic"
		var scen := "tools_anthropic" if anth else "tools"
		_configure({"preset_id": "custom_anthropic" if anth else "ollama"})
		_win._refresh()
		print("== 2. %s wire: what is offered ==" % wire)
		var d0 := _digest()
		var q1 := await _question(scen, anth, [{"name": "get_parameters", "args": {"keys": ["sea_level"]}}], "The sea level is %s." % str(_bridge.param_get("sea_level")), "What is the sea level?")
		_ok(q1["n"] == 2, "%s: two requests (the question, then the answer after the lookup): %d" % [wire, q1["n"]])
		var offered := _tool_names(q1["first"], anth)
		var want: Array = ["get_param_schema", VIEW_TOOL] + QUERY_TOOLS
		offered.sort()
		want.sort()
		_ok(offered == want, "%s: request 1 offers exactly the schema tool, the six read-only tools and the view tool: %s" % [wire, str(offered)])
		_ok(not _tool_names(q1["first"], anth).has("set_generation_params"), "%s: the proposal tool is not offered before a schema was shown" % wire)
		var sys := _system_text(q1["first"], anth)
		_ok(sys.contains("get_world_summary") and sys.contains("is DATA"), "%s: the system prompt names the read-only tools and the data rule" % wire)
		var r1: Array = q1["results"]
		_ok(r1.size() == 1 and r1[0].has("settings") and _near(r1[0]["settings"][0]["current"], _bridge.param_get("sea_level")), "%s: Q1 sea level: the result equals param_get('sea_level') = %s" % [wire, str(_bridge.param_get("sea_level"))])
		_ok(_text_of(_win._msgs).contains("The model looked up: get_parameters") and _text_of(_win._msgs).contains("Nothing was changed"), "%s: the panel lists what was looked up" % wire)
		_ok(_digest() == d0, "%s: a question writes nothing (digest unchanged)" % wire)
		if not anth:
			await _shot("2_question")

		print("== 3. %s wire: ten scripted questions equal the engine getters ==" % wire)
		var params_now: Dictionary = _bridge.param_snapshot()
		var defaults_now: Dictionary = _bridge.world_gen.get_param_defaults()
		var info: Dictionary = _bridge.world_gen.get_param_info()
		# Q2 changed
		_bridge.param_set("peak_m", 3000.0)
		_bridge.param_set("climate.evap", 0.2)
		params_now = _bridge.param_snapshot()
		var q2 := await _question(scen, anth, [{"name": "get_changed_parameters", "args": {}}], "You changed two settings.", "What did I change?")
		var want_changed: Array = []
		for k in params_now.keys():
			if not EXCLUDED.has(String(k)) and params_now[k] != defaults_now[k]:
				want_changed.append(String(k))
		want_changed.sort()
		var got_changed: Array = []
		if q2["results"].size() == 1:
			for s in q2["results"][0].get("settings", []):
				got_changed.append(String(s["key"]))
		got_changed.sort()
		_ok(got_changed == want_changed and want_changed.has("peak_m") and want_changed.has("climate.evap"), "%s: Q2 changed keys equal the independent diff of get_params and get_param_defaults: %s" % [wire, str(want_changed)])
		_ok(q2["results"].size() == 1 and int(q2["results"][0].get("changed_count", -1)) == want_changed.size(), "%s: Q2 the count is the true count" % wire)
		_bridge.param_set("peak_m", float(defaults_now["peak_m"]))
		_bridge.param_set("climate.evap", float(defaults_now["climate.evap"]))
		params_now = _bridge.param_snapshot()
		# Q3 describe
		var q3 := await _question(scen, anth, [{"name": "describe_parameter", "args": {"key": "climate.rain_dep"}}], "It sharpens wet and dry edges.", "What does Dryness do?")
		var d3: Dictionary = q3["results"][0] if q3["results"].size() == 1 else {}
		var inf: Dictionary = info["climate.rain_dep"]
		_ok(d3.get("key", "") == "climate.rain_dep" and _near(d3.get("min", -1), inf["min"]) and _near(d3.get("max", -1), inf["max"]) and _near(d3.get("step", -1), inf["step"]) and String(d3.get("label", "")) == String(inf["label"]), "%s: Q3 describe: key, range, step and label equal get_param_info" % wire)
		_ok(_near(d3.get("default", -1), inf["default"]) and bool(d3.get("in_original_cartalith", not (String(inf["reference_control"]) != ""))) == (String(inf["reference_control"]) != "") and String(d3.get("desc", "")).contains("first ridge"), "%s: Q3 default, the original-control flag and the description (raising it dries land past the first ridge)" % wire)
		_ok(String(d3.get("needs", "")) == "climate_refresh", "%s: Q3 a change to rain_dep needs the climate refresh: %s" % [wire, d3.get("needs", "?")])
		# Q4 group
		var q4 := await _question(scen, anth, [{"name": "get_parameters", "args": {"group": "weather"}}], "Here are the weather settings.", "Show me the weather settings.")
		var ok4: bool = q4["results"].size() == 1
		var n4 := 0
		if ok4:
			for s in q4["results"][0]["settings"]:
				var k := String(s["key"])
				ok4 = ok4 and _near(s["current"], params_now[k]) and _near(s["default"], defaults_now[k])
				n4 += 1
		_ok(ok4 and n4 == 12, "%s: Q4 the 12 weather settings equal get_params / get_param_defaults (%d)" % [wire, n4])
		# Q5 all offered keys, via the 9 groups in two replies of at most six lookups
		params_now = _bridge.param_snapshot()
		var calls_a: Array = []
		var calls_b: Array = []
		for gi in GROUPS.size():
			(calls_a if gi < 5 else calls_b).append({"name": "get_parameters", "args": {"group": GROUPS[gi]}})
		_server.scenario = scen
		_server.tool_script = [{"calls": calls_a}, {"calls": calls_b}, {"text": "Done."}]
		_server.tool_served = 0
		var n0q: int = _server.requests.size()
		_win._pick_task("commands")
		await _type("List every setting.")
		await _click(_c("send") as Button)
		await _until(func(): return not _win.run.busy() and _server.requests.size() >= n0q + 3, 20000)
		await _until(func(): return not _win.run.busy(), 20000)
		await _settle(4)
		var res5 := _tool_results(_body(_server.requests.size() - 1), anth)
		var seen := {}
		for r in res5:
			for s in r.get("settings", []):
				var kk := String(s["key"])
				if _near(s["current"], params_now[kk]) and _near(s["default"], defaults_now[kk]):
					seen[kk] = true
		var offered_keys := _offered_keys()
		var missing: Array = []
		for kk in offered_keys:
			if not seen.has(kk):
				missing.append(kk)
		_ok(offered_keys.size() == 114 and missing.is_empty(), "%s: Q5 get_parameters returns, for 100%% of the %d offered keys, current and default equal to the getters (missing %s)" % [wire, offered_keys.size(), str(missing.slice(0, 5))])
		var excl_seen := false
		for r in res5:
			for s in r.get("settings", []):
				excl_seen = excl_seen or EXCLUDED.has(String(s["key"]))
		_ok(not excl_seen, "%s: Q5 none of the six excluded keys appears in any result" % wire)
		# Q6 ecology, Q7 sample, Q8 summary, Q9 outside
		var eco: Dictionary = _bridge.world_gen.ecology_summary()
		var q6 := await _question(scen, anth, [{"name": "get_ecology_summary", "args": {}}], "Productivity is moderate.", "How productive is the land?")
		var d6: Dictionary = q6["results"][0] if q6["results"].size() == 1 else {}
		_ok(_near(d6.get("npp_mean", -1), eco["npp_mean"]) and _near(d6.get("npp_max", -1), eco["npp_max"]) and int(d6.get("land_cells", -1)) == int(eco["land_cells"]), "%s: Q6 ecology: npp_mean %s, npp_max and land_cells equal ecology_summary()" % [wire, str(d6.get("npp_mean", "?"))])
		var gx := 90
		var gy := 70
		var cell: Dictionary = _bridge.world_gen.sample_cell(gx, gy)
		var q7 := await _question(scen, anth, [{"name": "sample_point", "args": {"x": gx, "y": gy}}], "That cell is fine.", "What is at column 90, row 70?")
		var d7: Dictionary = q7["results"][0] if q7["results"].size() == 1 else {}
		var ok7: bool = not cell.is_empty() and _near(d7.get("elevation_m", -1e9), cell["elevation_m"]) and _near(d7.get("slope_deg", -1e9), cell["slope_deg"]) and _near(d7.get("temperature_c", -1e9), cell["temperature_c"]) and _near(d7.get("precipitation", -1e9), cell["precipitation"])
		ok7 = ok7 and String(d7.get("water", "")) == String(cell.get("water", "?"))
		if cell.has("biome"):
			ok7 = ok7 and String(d7.get("biome", "")) == String(cell["biome"])
		else:
			ok7 = ok7 and not d7.has("biome") and d7.has("biome_reason")
		_ok(ok7, "%s: Q7 the cell (%d,%d) equals sample_cell: elevation %s m, slope, temperature, rainfall, water %s" % [wire, gx, gy, str(d7.get("elevation_m", "?")), d7.get("water", "?")])
		var ws: Dictionary = _bridge.world_gen.world_summary()
		var q8 := await _question(scen, anth, [{"name": "get_world_summary", "args": {}}], "About a third is land.", "Describe the world.")
		var d8: Dictionary = q8["results"][0] if q8["results"].size() == 1 else {}
		var ok8: bool = not ws.is_empty() and int(d8.get("grid_width", -1)) == int(ws["grid_width"]) and int(d8.get("grid_height", -1)) == int(ws["grid_height"]) and _near(d8.get("sea_level", -1), ws["sea_level"]) and _near(d8.get("land_fraction", -1), ws["land_fraction"]) and int(d8.get("land_cells", -1)) == int(ws["land_cells"])
		if ok8 and ws.has("land_elevation_m"):
			var le: Dictionary = ws["land_elevation_m"]
			for pk in le.keys():
				ok8 = ok8 and d8.has("land_elevation_m") and _near(d8["land_elevation_m"].get(pk, -1e9), le[pk])
			ok8 = ok8 and _near(d8.get("land_rainfall_mean", -1), ws["land_rainfall_mean"]) and _near(d8.get("land_temperature_mean_c", -1e9), ws["land_temperature_mean_c"])
		_ok(ok8, "%s: Q8 world summary equals world_summary(): land %s, grid %s x %s" % [wire, str(d8.get("land_fraction", "?")), str(d8.get("grid_width", "?")), str(d8.get("grid_height", "?"))])
		var q9 := await _question(scen, anth, [{"name": "sample_point", "args": {"x": 9999, "y": 5}}], "That is off the map.", "What is at column 9999?")
		var d9: Dictionary = q9["results"][0] if q9["results"].size() == 1 else {}
		_ok(d9.get("available", true) == false and String(d9.get("reason", "")).contains("outside") and not d9.has("elevation_m"), "%s: Q9 a cell outside the grid is 'not available' with a reason and no number: %s" % [wire, JSON.stringify(d9).left(120)])
		var q9b := await _question(scen, anth, [{"name": "sample_point", "args": {"x": 2000000, "y": 5}}], "No.", "What is at column two million?")
		var d9b: Dictionary = q9b["results"][0] if q9b["results"].size() == 1 else {}
		_ok(d9b.has("error") and not d9b.has("elevation_m"), "%s: Q9b an absurd index is refused by the argument rule: %s" % [wire, JSON.stringify(d9b).left(100)])
		if not anth:
			await _shot("3_answer")

		print("== 5. %s wire: bounds and privacy ==" % wire)
		var cells: Array = []
		for cx in range(1, 7):
			cells.append({"name": "sample_point", "args": {"x": cx * 10, "y": 20}})
		var more: Array = []
		for cx in range(7, 10):
			more.append({"name": "sample_point", "args": {"x": cx * 10, "y": 20}})
		_server.scenario = scen
		_server.tool_script = [{"calls": cells}, {"calls": more}, {"text": "Enough."}]
		_server.tool_served = 0
		var n0c: int = _server.requests.size()
		_win._pick_task("commands")
		await _type("Read me a lot of cells.")
		await _click(_c("send") as Button)
		await _until(func(): return not _win.run.busy() and _server.requests.size() >= n0c + 3, 20000)
		await _until(func(): return not _win.run.busy(), 20000)
		await _settle(4)
		var res_c := _tool_results(_body(_server.requests.size() - 1), anth)
		var with_elev := 0
		var refused_cap := 0
		for r in res_c:
			if r.has("elevation_m"):
				with_elev += 1
			if String(r.get("error", "")).contains("at most 8"):
				refused_cap += 1
		_ok(with_elev == 8 and refused_cap == 1 and res_c.size() == 9, "%s: nine cells asked, 8 read and the 9th refused (%d read, %d refused, %d results)" % [wire, with_elev, refused_cap, res_c.size()])

	# ---------------------------------------------------------------- 4b. three seeds, a faction name, generating
	print("== 4b. the three seeds, a faction name, and a running generation ==")
	_configure({"preset_id": "ollama"})
	for sd in SEEDS:
		await _generate(sd)
		var ws2: Dictionary = _bridge.world_gen.world_summary()
		var eco2: Dictionary = _bridge.world_gen.ecology_summary()
		var qq := await _question("tools", false, [{"name": "get_world_summary", "args": {}}], "Summary read.", "Describe this world again.")
		var dd: Dictionary = qq["results"][0] if qq["results"].size() == 1 else {}
		_ok(not ws2.is_empty() and int(dd.get("land_cells", -1)) == int(ws2["land_cells"]) and _near(dd.get("land_fraction", -1), ws2["land_fraction"]) and _near(dd.get("land_elevation_m", {}).get("p50", -1e9), ws2["land_elevation_m"]["p50"]), "seed %d: get_world_summary equals world_summary() (land %s, median %s m)" % [sd, str(dd.get("land_fraction", "?")), str(dd.get("land_elevation_m", {}).get("p50", "?"))])
		var qe := await _question("tools", false, [{"name": "get_ecology_summary", "args": {}}], "Eco read.", "And the productivity?")
		var de: Dictionary = qe["results"][0] if qe["results"].size() == 1 else {}
		_ok(_near(de.get("npp_mean", -1), eco2["npp_mean"]), "seed %d: get_ecology_summary equals ecology_summary()" % sd)
	# A cell with a controlling faction: its NAME must not reach the model.
	var fac_cell := Vector2i(-1, -1)
	var fac_name := ""
	for yy in range(4, 160, 6):
		for xx in range(4, 250, 6):
			var cd: Dictionary = _bridge.world_gen.sample_cell(xx, yy)
			if cd.has("controlling_faction_name"):
				fac_cell = Vector2i(xx, yy)
				fac_name = String(cd["controlling_faction_name"])
				break
		if fac_name != "":
			break
	if fac_name == "":
		print("  (no faction-owned cell found on this seed; the faction-name check is covered by the Rust vocabulary test)")
	else:
		var qf := await _question("tools", false, [{"name": "sample_point", "args": {"x": fac_cell.x, "y": fac_cell.y}}], "A faction owns it.", "Who holds that cell?")
		var all_bodies := ""
		for rq in _server.requests:
			all_bodies += String(rq["body"])
		_ok(qf["results"].size() == 1 and qf["results"][0].has("elevation_m") and not all_bodies.contains(fac_name), "a faction-owned cell (%d,%d) was read and its name '%s' is in no request body" % [fac_cell.x, fac_cell.y, fac_name])

	var p_before: Dictionary = _bridge.param_snapshot()
	_bridge.generate({"seed": 4242, "width_km": 1200.0, "grid_w": 1024, "grid_h": 655, "archetype": "", "villages": false})
	await _settle(2)
	var n_gen := await _ask("tools", [{"calls": [{"name": "get_world_summary", "args": {}}, {"name": "get_parameters", "args": {"keys": ["sea_level"]}}, {"name": "sample_point", "args": {"x": 3, "y": 3}}, {"name": "get_ecology_summary", "args": {}}]}, {"text": "Cannot read the world right now."}], "Describe the world now.", false)
	var was_gen: bool = _bridge.generating
	var rg := _tool_results(_body(_server.requests.size() - 1), false)
	var na := 0
	var param_ok := false
	for r in rg:
		if r.get("available", true) == false and String(r.get("reason", "")).contains("generating") and r.size() == 2:
			na += 1
		if r.has("settings") and r["settings"].size() == 1 and _near(r["settings"][0]["current"], p_before["sea_level"]) and String(r.get("note", "")).contains("generating"):
			param_ok = true
	_ok(n_gen == 2 and was_gen, "the question was asked while the world generated (requests %d, generating %s)" % [n_gen, str(was_gen)])
	_ok(na == 3, "while generating the three world tools answer 'not available' with the reason and nothing else (%d of 3)" % na)
	_ok(param_ok, "while generating get_parameters equals the cached snapshot (sea_level %s) and says so" % str(p_before["sea_level"]))
	while _bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _settle(8)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	var after := await _question("tools", false, [{"name": "get_world_summary", "args": {}}], "Back.", "And now?")
	_ok(after["results"].size() == 1 and after["results"][0].has("land_fraction"), "after it finishes the world tools answer again")
	await _generate(SEEDS[0])

	if _skip_corpus:
		print("(--no-corpus: sections 1-5 only; the corpus section did not run)")
		_finish()
		return
	# ---------------------------------------------------------------- 6. the hostile corpus
	print("== 6. the hostile corpus through the panel ==")
	var rows := _load_corpus()
	_ok(rows.size() >= 30, "the corpus loaded: %d rows (path %s)" % [rows.size(), _corpus_path])
	var general := 0
	var param_rows := 0
	for r in rows:
		if String(r["id"]).begins_with("G"):
			general += 1
		else:
			param_rows += 1
	_ok(general >= 20 and param_rows >= 10, "at least 20 general and 10 parameter-shaped rows (%d, %d)" % [general, param_rows])
	_ok(_count_class(_win, "RichTextLabel") == 0, "the panel holds no RichTextLabel before the corpus (model text is drawn as plain labels)")
	for wire in ["openai", "anthropic"]:
		_configure({"preset_id": "custom_anthropic" if wire == "anthropic" else "ollama"})
		_win._refresh()
		await _corpus(rows, wire)
	_ok(_count_class(_win, "RichTextLabel") == 0, "after the corpus the panel still holds no RichTextLabel")
	_ok(not _text_of(_win).contains(SECRET), "the key is in no label or tooltip of the panel")
	var bad_paths := 0
	for rq in _server.requests:
		if not (String(rq["path"]) in ["/v1/chat/completions", "/v1/messages"]):
			bad_paths += 1
	_ok(bad_paths == 0, "every request of the whole run went to the mock chat endpoint (%d requests, %d elsewhere)" % [_server.requests.size(), bad_paths])
	_ok(LlmWire.connect_attempts >= _server.requests.size() and LlmWire.connect_attempts - _server.accepted <= 0, "connect attempts %d, connections the mock accepted %d: nothing else was dialled" % [LlmWire.connect_attempts, _server.accepted])
	await _shot("6_end")
	_finish()


func _load_corpus() -> Array:
	var f := FileAccess.open(_corpus_path, FileAccess.READ)
	if f == null:
		print("  (cannot open the corpus at %s)" % _corpus_path)
		return []
	## A Godot String cannot hold a NUL, so row G09's U+0000 (written \u0000 in the JSON) is dropped
	## on load; the Rust gate `llm9_corpus.rs` keeps it. The rest of that row is unchanged.
	var doc = JSON.parse_string(f.get_as_text().replace("\\u0000", ""))
	if not (doc is Dictionary):
		return []
	## The W rows are view-action rows (LLM-6d): `_llmview_probe.gd` runs them.
	var out: Array = []
	for r in doc.get("rows", []):
		if not String(r["id"]).begins_with("W"):
			out.append(r)
	return out


## Script of the obedient model for a row: the legitimate schema call, the non-proposal calls
## (repeated), the proposal in its own reply, or the words.
func _script_for(row: Dictionary) -> Array:
	var script: Array = []
	var legit: Array = row.get("legit_open", [])
	if not legit.is_empty():
		script.append({"name": "get_param_schema", "args": {"groups": legit}})
	var others: Array = []
	var proposals: Array = []
	for c in row.get("obey", []):
		var call := {"name": String(c["name"]), "args": _pad(c["arguments"])}
		if String(c["name"]) == "set_generation_params":
			proposals.append(call)
		else:
			others.append(call)
	for _i in int(row.get("repeat", 1)):
		if not others.is_empty():
			script.append({"calls": others})
	for p in proposals:
		script.append(p)
	if row.has("reply_text"):
		script.append({"text": String(row["reply_text"])})
	if script.is_empty():
		script.append({"text": "(nothing)"})
	return script


## `@PADn` in a string becomes n letters x (the corpus's size token).
func _pad(v):
	if v is String and String(v).begins_with("@PAD"):
		return "x".repeat(int(String(v).substr(4)))
	if v is Array:
		var out: Array = []
		for x in v:
			out.append(_pad(x))
		return out
	if v is Dictionary:
		var d := {}
		for k in v:
			d[k] = _pad(v[k])
		return d
	return v


func _corpus(rows: Array, wire: String) -> void:
	var anth: bool = wire == "anthropic"
	var scen := "tools_anthropic" if anth else "tools"
	var allowed: Array = ["get_param_schema", "set_generation_params", VIEW_TOOL] + QUERY_TOOLS
	var shot_done := {}
	var refused_long := 0
	for row in rows:
		var id := String(row["id"])
		var text: String = String(row["text"]).repeat(int(row.get("repeat_text", 1)))
		var request := "Make the climate drier. " + text
		var d0 := _digest()
		var label0 := String(_app.undo_groups.undo_label())
		var g0 := _gen_started
		var cards0: int = _win.cards.size()
		var att := LlmWire.connect_attempts
		var n0: int = _server.requests.size()
		var n := 0
		if request.length() > 4000:
			## Too long for the prose limit: refused before any model turn.
			_win._pick_task("commands")
			await _type(request)
			var reason := (_c("reason") as Label).text
			_ok((_c("send") as Button).disabled and reason.to_lower().contains("long"), "%s %s: a %d-character request is refused before any request: '%s'" % [wire, id, request.length(), reason.left(60)])
			refused_long += 1
		else:
			n = await _ask(scen, _script_for(row), request)
		var new_cards: Array = _win.cards.slice(cards0)
		# Nothing written.
		_ok(_digest() == d0 and String(_app.undo_groups.undo_label()) == label0 and _gen_started == g0, "%s %s (%s): nothing written -- digest, undo label and generation count unchanged" % [wire, id, row["family"]])
		# The tool set never changed, at every request.
		var tools_ok := true
		var bad_names: Array = []
		for ri in range(n0, _server.requests.size()):
			for t in _tool_names(_body(ri), anth):
				if not allowed.has(t):
					tools_ok = false
					bad_names.append(t)
		_ok(tools_ok, "%s %s: only the fixed tool names were ever offered over %d requests %s" % [wire, id, n, str(bad_names)])
		_ok(LlmWire.connect_attempts - att == n, "%s %s: %d connections for %d requests (nothing else dialled)" % [wire, id, LlmWire.connect_attempts - att, n])
		_ok(n <= 5, "%s %s: at most 5 requests (the turn cap): %d" % [wire, id, n])
		# Proposal expectations.
		var applies := 0
		for c in new_cards:
			if c.get("apply") != null:
				applies += 1
		if row.has("expect_valid"):
			_ok(applies == 1 and new_cards[new_cards.size() - 1]["state"] == "new", "%s %s: a card with Apply is shown, state new, and still nothing applied" % [wire, id])
			var ct := _text_of(new_cards[new_cards.size() - 1]["box"]) if not new_cards.is_empty() else ""
			for k in row["expect_valid"]:
				_ok(ct.contains("(%s)" % k), "%s %s: the card lists %s" % [wire, id, k])
			_ok(ct.contains("MODEL PROPOSAL"), "%s %s: the MODEL PROPOSAL chip is on the card" % [wire, id])
		elif row.has("expect_valid_subset_of_group"):
			var ct1 := _text_of(new_cards[new_cards.size() - 1]["box"]) if not new_cards.is_empty() else ""
			_ok(applies == 1 and ct1.contains("(climate.") and not ct1.contains("(tect.") and not ct1.contains("(civ."), "%s %s: the card offers only weather keys" % [wire, id])
		elif row.has("expect_rule"):
			_ok(applies == 0, "%s %s: no Apply button on any card (rule %s)" % [wire, id, row["expect_rule"]])
			if not new_cards.is_empty():
				_ok(_text_of(new_cards[new_cards.size() - 1]["box"]).contains(String(row["expect_rule"])) or _text_of(_win._msgs).contains(String(row["expect_rule"])), "%s %s: the panel shows rule %s" % [wire, id, row["expect_rule"]])
		else:
			_ok(applies == 0, "%s %s: no proposal card with Apply" % [wire, id])
		# Words rows: the markup is drawn as plain text.
		if row.has("reply_text") and request.length() <= 4000:
			_ok(_text_of(_win._msgs).contains(String(row["reply_text"]).left(30)), "%s %s: the reply is shown verbatim as plain text" % [wire, id])
			_ok(_count_class(_win, "RichTextLabel") == 0, "%s %s: no RichTextLabel parsed it" % [wire, id])
		# Tidy: dismiss a waiting card (writes nothing).
		for c in new_cards:
			if c["state"] == "new" and c.get("dismiss") != null:
				await _click(c["dismiss"])
		_ok(_digest() == d0, "%s %s: dismissing a card writes nothing" % [wire, id])
		if wire == "openai" and not shot_done.has(id) and (id == "P05" or id == "G07" or id == "G11"):
			shot_done[id] = true
			await _shot("6_%s" % id)
	_ok(refused_long >= 1, "%s: at least one oversized request was refused (%d)" % [wire, refused_long])
