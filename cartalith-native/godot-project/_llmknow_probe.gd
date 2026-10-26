extends Node
## **RK-5, RK-6 and RK-7 in the real shell: the knowledge tools, the pre-retrieval pack, the sources list
## and the hostile corpus** (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.5, s.8; Rulings DM and DN), driving the real
## Assistant panel against the in-test loopback server (`_llm_mock_server.gd`, scenarios `tools` and
## `tools_anthropic`) over a REAL generated world and a REAL Vault written to a scratch directory.
##
##   desktop: godot --path . --resolution 1600x1000 res://_llmknow_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1280x800  res://_llmknow_probe.tscn -- --force-touch --shots <dir>
##   phone:   godot --path . --resolution 393x852   res://_llmknow_probe.tscn -- --force-touch --shots <dir>
##
## **Windowed, never `--headless`**, from a project copy with its own `config/name` and a wiped user dir
## (`MISTAKES.md`). Reads `--force-touch`, `--corpus <path>`, `--no-corpus` and `--shots <dir>`; anything else
## exits 2. Result line: `RESULT: PASS|FAIL (<n> failed) checks=<n>`.
##
## What it protects (every check names it):
##   1. OFFERED (RK-5): the three tools are in a request only when a knowledge box is ticked; with both boxes
##      off, Send is disabled naming them and a forced Send opens no connection; with only the knowledge boxes
##      ticked the conversation is questions-only (exactly the three tools, no settings tool).
##   2. ANSWERS EQUAL THE ENGINE (RK-4/RK-5): "which settlements trade with <a settlement>?" is answered by a
##      search then `related_sources`, and the partner set the model is handed equals `civ_trade_flows`.
##   3. CITATIONS (RK-6): a citation to a retrieved id is verified, one to an id never retrieved is drawn as
##      UNVERIFIED, and the sources list names every source sent (WORLD DATA / VAULT NOTE).
##   4. PRE-RETRIEVAL (RK-6): with a model that never calls a tool the request still carries the pack (at most
##      5 sources), and a note that disagrees with the world is shown beside it, both sources, nothing merged.
##   5. PRIVACY (K3): a note marked private and one in a DM folder are in no request body; an unmarked note is.
##   6. HOSTED (K2): a hosted endpoint uses no source until the user opts it in, per endpoint.
##   7. HOSTILE (RK-7): every Vault-note row of the corpus, planted in the real Vault and obeyed by the scripted
##      model, changes nothing (digest, undo, tool list), opens no card with Apply and draws no markup.
##   8. SETTINGS AND CONSENT: the two boxes, the mode pickers and the hosted opt-in exist and persist; the
##      consent text names what a lookup sends and says world data has no private marker.
##   9. LAYOUT: the sources card and Send stay inside the window.
const ENV_VAR := "CART_LLM_PROBE_KEYVAR"
const SECRET := "sk-PROBE-SECRET-know-0123456789"
const SEEDS := [9137, 24601, 71077345]
const KTOOLS := ["read_source", "related_sources", "search_knowledge"]

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _corpus_path := "../crates/cartalith-llm/tests/fixtures/hostile_corpus_2026-10-26.json"
var _skip_corpus := false
var _app: Node
var _bridge
var _win
var _server: LlmMockServer
var _port := 0
var _vault_dir := ""
var _x: Dictionary = {}


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
	var path := _shots.path_join("rk_%s_%s.png" % [_mode, tag])
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
		"base_url": "http://127.0.0.1:%d/v1" % _port, "model": "mock-model", "key_source": "env", "env_var": ENV_VAR,
		"ctx_params": false, "ctx_rag_world": true, "ctx_rag_vault": true, "rag_pre": "off", "rag_tools": true,
		"ctx_entity": false, "ctx_notes": false}
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
	out.sort()
	return out


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


## The first user message's text (the pre-retrieval pack rides at its start).
func _user_text(b: Dictionary) -> String:
	for m in b.get("messages", []):
		if m is Dictionary and String(m.get("role", "")) == "user":
			var c = m.get("content", "")
			if c is Array:
				for blk in c:
					if blk is Dictionary and String(blk.get("type", "")) == "text":
						return String(blk.get("text", ""))
			return String(c)
	return ""


func _all_bodies() -> String:
	var parts: Array = []
	for r in _server.requests:
		parts.append(String(r["body"]))
	return "\n".join(parts)


## Sends `request` through the panel as the Commands task and waits until idle.
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
	var sr := send.get_global_rect()
	if not get_viewport().get_visible_rect().encloses(sr):
		print("  (Send is outside the window before the click: %s in %s)" % [str(sr), str(get_viewport().get_visible_rect())])
		await _shot("send_outside_%d" % _server.requests.size())
	await _click(send)
	if _server.requests.size() == n0 and not _win.run.busy():
		## Diagnostic for a click that started nothing: what was under the pointer, and the button's state.
		var hov = get_viewport().gui_get_hovered_control()
		print("  (the click started nothing: Send disabled=%s visible=%s rect=%s centre=%s hovered=%s)" % [str(send.disabled), str(send.is_visible_in_tree()), str(send.get_global_rect()), str(send.get_global_rect().get_center()), str(hov.get_path()) if hov != null else "none"])
		var okb: Button = _win.get_ok_button()
		var okp := okb.get_parent() as Control
		print("  (Close button rect=%s; its parent %s rect=%s filter=%d; window pos=%s size=%s; Send filter=%d; Send parent %s rect=%s)" % [str(okb.get_global_rect()), okp.get_class(), str(okp.get_global_rect()), okp.mouse_filter, str(_win.position), str(_win.size), send.mouse_filter, send.get_parent().get_class(), str((send.get_parent() as Control).get_global_rect())])
		var parts: Array = []
		for ch in _win._body.get_children():
			if ch is Control and (ch as Control).visible:
				parts.append("%s min.y=%d h=%d" % [ch.get_class(), int((ch as Control).get_combined_minimum_size().y), int((ch as Control).size.y)])
		print("  (body children: %s; body size=%s min=%s)" % [" | ".join(parts), str(_win._body.size), str(_win._body.get_combined_minimum_size())])
		var padc := _win._body.get_parent() as Control
		var hostc := padc.get_parent() as Control
		print("  (pad rect=%s margin_bottom=%d host rect=%s host min=%s phone=%s window content_scale=%s)" % [str(padc.get_global_rect()), padc.get_theme_constant("margin_bottom"), str(hostc.get_global_rect()), str(hostc.get_combined_minimum_size()), str(_win._phone), str(_win.content_scale_factor)])
		await _shot("click_nothing")
		## Nothing started: do not sit through the 20 s waits below for every remaining question.
		await _settle(30)
		if _server.requests.size() == n0 and not _win.run.busy():
			return 0
	await _until(func(): return not _win.run.busy() and _server.requests.size() > n0, 20000)
	await _until(func(): return not _win.run.busy(), 30000)
	await _settle(6)
	return _server.requests.size() - n0


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


func _put(rel: String, text: String) -> void:
	var p := _vault_dir.path_join(rel)
	DirAccess.make_dir_recursive_absolute(p.get_base_dir())
	var f := FileAccess.open(p, FileAccess.WRITE)
	f.store_string(text)
	f.close()


## The settlements as `{tid, name, pop}` and the trade partners (a Dictionary tid -> sorted tid list) from
## `civ_trade_flows` (the engine's own getter), independent of the knowledge base.
func _settlements_and_partners() -> Dictionary:
	var rows: Array = _bridge.world_gen.get_settlements()
	var flows: Dictionary = _bridge.world_gen.civ_trade_flows()
	var partners := {}
	for f in flows.get("flows", []):
		var a: Dictionary = rows[int(f["from"])]
		var b: Dictionary = rows[int(f["to"])]
		var ta := int(a["tid"])
		var tb := int(b["tid"])
		if ta == tb:
			continue
		if not partners.has(ta):
			partners[ta] = []
		if not partners.has(tb):
			partners[tb] = []
		if not partners[ta].has(tb):
			partners[ta].append(tb)
		if not partners[tb].has(ta):
			partners[tb].append(ta)
	for k in partners:
		partners[k].sort()
	return {"rows": rows, "partners": partners, "capped": int(flows.get("flow_count", 0)) != int(flows.get("flow_rows", 0))}


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
	_ok(_bridge.llm_api and _win != null, "the library has LlmBridge and the shell has the panel")
	if not _bridge.llm_api or _win == null:
		_finish()
		return
	var wg = _bridge.world_gen
	_ok(wg.has_method("knowledge_call") and wg.has_method("knowledge_search_json") and ClassDB.class_exists("LlmStream") and _win.run.stream.has_method("knowledge_plan"),
		"the library has the knowledge bridge (the debug DLL is current)")

	# ------------------------------------------------ the world and the vault
	var picked := false
	for sd in SEEDS:
		await _generate(sd)
		var sp := _settlements_and_partners()
		if not sp["partners"].is_empty() and not sp["capped"]:
			for tid in sp["partners"]:
				if sp["partners"][tid].size() >= 1:
					for r in sp["rows"]:
						if int(r["tid"]) == int(tid) and String(r["name"]).strip_edges().length() >= 3:
							_x = {"tid": int(tid), "name": String(r["name"]), "pop": int(r["population"]), "partners": sp["partners"][tid], "rows": sp["rows"]}
							break
				if not _x.is_empty():
					break
		if not _x.is_empty():
			print("== world: seed %d, settlement %s (tid %d, population %d) with %d trade partners ==" % [sd, _x["name"], _x["tid"], _x["pop"], _x["partners"].size()])
			picked = true
			break
	_ok(picked, "a generated world with a settlement that has trade partners exists")
	if not picked:
		_finish()
		return
	_vault_dir = ProjectSettings.globalize_path("user://rk_vault")
	var name_x: String = _x["name"]
	var conflict_pop: int = int(_x["pop"]) + 2222
	_put("Places/%s.md" % name_x, "# %s\nThe harbour of %s was walled in 1204 by the Order of Dawn.\nPopulation: %d (a chronicler's count).\n" % [name_x, name_x, conflict_pop])
	_put("Places/Open.md", "# Open\nOPENCANARY the lighthouse keeper of %s.\n" % name_x)
	_put("DM/Plot.md", "# Plot\nDMCANARY the secret ending involves %s.\n" % name_x)
	_put("Places/Hidden.md", "---\nprivate: true\n---\n# Hidden\nPRIVCANARY a secret about %s.\n" % name_x)
	var vc: Dictionary = _bridge.vault_connect(_vault_dir, "RK probe vault")
	_ok(bool(vc.get("ok", false)), "a vault is connected: %s" % str(vc))
	## Settings first, THEN open the panel, as the other assistant probes do. On a phone the sheet keeps the child
	## size from its FIRST layout (`wrap_controls` is off and `_refit` skips a phone), so a panel first laid out in
	## its switched-off state (tall reason text) stays taller than the sheet and leaves Send under the Close row.
	## Measured here (host 902 px in a 832 px window); a pre-existing fragility, reported and not changed.
	_configure({})
	_win.open()
	await _settle(10)
	var d_host: Control = _win._body.get_parent().get_parent() as Control
	print("  (first layout, prose task: host %s, Send %s)" % [str(d_host.get_global_rect().size), str((_c("send") as Control).get_global_rect())])
	_win._pick_task("commands")
	await _settle(10)
	var send_after: Rect2 = (_c("send") as Control).get_global_rect()
	print("  (after picking the Commands task: host %s, Send %s, window %s)" % [str(d_host.get_global_rect().size), str(send_after), str(_win.size)])
	_ok(d_host.get_global_rect().size.y <= float(_win.size.y) + 0.5 and send_after.end.y <= float(_win.size.y) - (60.0 if _mode == "phone" else 0.0) + 0.5,
		"the content fits the window after a task change: host %.0f px in a %d px window, Send ends at %.0f" % [d_host.get_global_rect().size.y, _win.size.y, send_after.end.y])
	var digest0 := _digest()

	# ---------------------------------------------------------------- 1. offered
	print("== 1. the tools are offered only when a box is ticked ==")
	_configure({"ctx_rag_world": false, "ctx_rag_vault": false, "ctx_params": false})
	_win._pick_task("commands")
	await _type("which settlements trade with %s?" % name_x)
	var rsn := (_c("reason") as Label).text
	_ok((_c("send") as Button).disabled and rsn.contains("Vault notes found by search"), "no box ticked: Send is disabled and names the knowledge boxes: '%s'" % rsn.left(100))
	var att0 := LlmWire.connect_attempts
	_win._on_send()
	await _settle(8)
	_ok(LlmWire.connect_attempts == att0 and _server.accepted == 0, "a forced Send with every source off opened no connection (attempts %d, accepted %d)" % [LlmWire.connect_attempts, _server.accepted])
	_configure({"ctx_params": true, "ctx_rag_world": false, "ctx_rag_vault": false})
	var n_off := await _ask("tools", [{"text": "No sources."}], "What is the sea level?")
	_ok(n_off == 1 and not _tool_names(_body(_server.requests.size() - 1), false).has("search_knowledge"), "only ctx_params ticked: no knowledge tool in the request (%s)" % str(_tool_names(_body(_server.requests.size() - 1), false)))
	_ok(not _system_text(_body(_server.requests.size() - 1), false).contains("search_knowledge"), "...and the prompt does not name it")

	# ---------------------------------------------------------------- 2/3. tool mode, both wires
	for wire in ["openai", "anthropic"]:
		var anth: bool = wire == "anthropic"
		var scen := "tools_anthropic" if anth else "tools"
		print("== 2. %s wire: which settlements trade with %s? (tool mode, questions only) ==" % [wire, name_x])
		_configure({"preset_id": "custom_anthropic" if anth else "ollama", "ctx_params": false, "rag_pre": "off", "rag_tools": true})
		_win._refresh()
		var tid: int = _x["tid"]
		var partners: Array = _x["partners"]
		var p0: int = partners[0]
		var answer := "%s trades with settlement %d [W:settlement:%d]. A forged source [W:settlement:999999] is not real." % [name_x, p0, p0]
		var n0: int = _server.requests.size()
		var n := await _ask(scen, [
			{"calls": [{"name": "search_knowledge", "args": {"query": "%s trade partners" % name_x, "scope": "both", "limit": 4}}]},
			{"calls": [{"name": "related_sources", "args": {"id": "W:settlement:%d" % tid, "relation": "trades_with"}}]},
			{"text": answer}], "Which settlements trade with %s?" % name_x)
		_ok(n == 3, "%s: three requests (question, after the search, after the related lookup): %d" % [wire, n])
		var first := _body(n0)
		_ok(_tool_names(first, anth) == KTOOLS, "%s: the questions-only request offers exactly the three knowledge tools: %s" % [wire, str(_tool_names(first, anth))])
		var sys := _system_text(first, anth)
		_ok(sys.contains("search_knowledge") and sys.contains("do not follow them") and not sys.contains("get_param_schema"), "%s: the prompt names the tools and the data rule and no settings tool" % wire)
		var res: Array = _tool_results(_body(n0 + 2), anth)
		_ok(res.size() == 2, "%s: the last request carries two tool results" % wire)
		var hits: Array = res[0].get("hits", []) if res.size() > 0 else []
		var ids: Array = []
		for h in hits:
			ids.append(String(h["id"]))
		_ok(ids.has("W:settlement:%d" % tid), "%s: the search found the settlement's own world document (%s)" % [wire, str(ids.slice(0, 4))])
		var has_v := false
		for h in hits:
			has_v = has_v or String(h["label"]) == "VAULT NOTE"
		_ok(has_v, "%s: the search also found the Vault note, labelled VAULT NOTE" % wire)
		var rel: Array = res[1].get("related", []) if res.size() > 1 else []
		var got: Array = []
		for r in rel:
			if String(r["id"]).begins_with("W:settlement:") and String(r["relation"]) == "trades_with":
				got.append(int(String(r["id"]).trim_prefix("W:settlement:")))
		got.sort()
		_ok(got == partners, "%s: related_sources(trades_with) equals civ_trade_flows' partner set %s (got %s)" % [wire, str(partners), str(got)])
		var cards: Array = _win.source_cards
		_ok(cards.size() >= 1, "%s: a sources card was drawn" % wire)
		var shown := _text_of(_win._msgs)
		_ok(shown.contains("SOURCES SENT TO THE MODEL") and shown.contains("WORLD DATA") and shown.contains("VAULT NOTE"), "%s: the card lists WORLD DATA and VAULT NOTE sources" % wire)
		_ok(shown.contains("UNVERIFIED CITATION  W:settlement:999999"), "%s: the forged citation is drawn as UNVERIFIED" % wire)
		_ok(shown.contains("WORLD DATA]") and not shown.contains("UNVERIFIED CITATION  W:settlement:%d" % p0), "%s: the real citation is verified" % wire)
		_ok(shown.contains("The model looked up: search_knowledge") or shown.contains("looked up"), "%s: the panel says what was looked up" % wire)
		_ok(_digest() == digest0, "%s: a knowledge question writes nothing (digest unchanged)" % wire)
		_ok(_win.cards.is_empty() or _win.cards.all(func(c): return c["apply"] == null), "%s: no card with Apply" % wire)
		# Layout.
		var vp := get_viewport().get_visible_rect()
		var sb := _c("send") as Button
		var okl: bool = vp.encloses(sb.get_global_rect())
		if not cards.is_empty():
			var bx: Control = cards[cards.size() - 1]["box"]
			okl = okl and bx.get_global_rect().size.x <= vp.size.x + 1.0 and bx.get_global_rect().position.x >= -1.0
		_ok(okl, "%s: Send and the sources card stay inside the window (%s)" % [wire, str(vp.size)])
		if not anth:
			await _shot("1_tool_mode_sources")

	# ---------------------------------------------------------------- 4. pre-retrieval
	print("== 3. pre-retrieval with a model that never calls a tool ==")
	_configure({"preset_id": "ollama", "ctx_params": false, "rag_pre": "on", "rag_tools": false})
	_win._refresh()
	_win.source_cards.clear()
	var np0: int = _server.requests.size()
	var tid2: int = _x["tid"]
	_win._pick_task("commands")
	await _type("How many people live in %s?" % name_x)
	var sent_line: String = _win._sent_label.text
	var lim0: Dictionary = _win.run.stream.knowledge_limits()
	_ok(sent_line.contains("Search in Vault notes and world data") and sent_line.contains("best %d excerpts" % int(lim0["pack_hits"])) and sent_line.contains("%d characters" % int(lim0["pack_chars"])) and sent_line.contains("no private marker"),
		"the 'This will send' line names the knowledge lookup in the code's caps: %s" % sent_line.right(260))
	_ok(not sent_line.contains("may look up to %d times" % int(lim0["calls_per_request"])), "...and does not promise model-driven lookups while the tools switch is off")
	## An id the pack NAMES (a world document writes a neighbour as `Name (W:settlement:N)`) but does not carry as a
	## source: citing it is "named, not read". Found with the hub itself, then scripted into the model's answer.
	var raw_pack: Dictionary = JSON.parse_string(String(wg.knowledge_search_json("How many people live in %s?" % name_x, {"world": true, "vault": true})))
	var named_only := ""
	var hit_ids: Array = []
	for h in raw_pack.get("hits", []):
		hit_ids.append(String(h["id"]))
	for h in raw_pack.get("hits", []):
		for m in h.get("mentions", []):
			if named_only == "" and not hit_ids.has(String(m)):
				named_only = String(m)
	var np := await _ask("tools", [{"text": "%s has %d people [W:settlement:%d] but a note says %d [V:Places/%s.md].%s" % [name_x, _x["pop"], tid2, conflict_pop, name_x, (" Related: [%s]." % named_only) if named_only != "" else ""]}], "How many people live in %s?" % name_x)
	_ok(np == 1, "pre-retrieval: one request (the model made no call): %d" % np)
	var b1 := _body(np0)
	_ok(_tool_names(b1, false).is_empty(), "pre-retrieval with tools off: the request has no tool at all (%s)" % str(_tool_names(b1, false)))
	var ut := _user_text(b1)
	_ok(ut.begins_with("<context kind=\"knowledge\""), "the user message starts with the pack's fence")
	_ok(ut.contains("[W:settlement:%d] WORLD DATA" % tid2) and ut.contains(str(_x["pop"])), "the pack carries the world's figure %d with its WORLD DATA id" % _x["pop"])
	_ok(ut.contains("VAULT NOTE") and ut.contains(str(conflict_pop)), "the pack carries the note's different figure %d labelled VAULT NOTE: both sources, nothing merged" % conflict_pop)
	var nsrc := ut.count("] WORLD DATA") + ut.count("] VAULT NOTE")
	_ok(nsrc >= 2 and nsrc <= 5, "the pack holds 2 to 5 sources (%d)" % nsrc)
	var pack_chars := ut.find("</context>") + 10
	_ok(pack_chars > 0 and pack_chars <= 3000 + 400 and ut.count("<context") == 1 and ut.count("</context") == 1, "the pack is one fenced block of %d characters (cap 3000)" % pack_chars)
	var st2 := _text_of(_win._msgs)
	_ok(st2.contains("Searched your sources first and sent") and st2.contains("SOURCES SENT TO THE MODEL"), "the panel says what was sent ahead of the model, and lists the sources")
	var last_card := _text_of(_win.source_cards[_win.source_cards.size() - 1]["box"]) if not _win.source_cards.is_empty() else ""
	_ok(last_card.contains("WORLD DATA") and last_card.contains("VAULT NOTE"), "both sources are listed on the answer's own card")
	_ok(named_only != "" and last_card.contains("NAMED, NOT READ  %s" % named_only), "an id the pack names but did not carry (%s) is drawn NAMED, NOT READ, not verified and not unverified" % named_only)
	_ok(not last_card.contains("UNVERIFIED CITATION") and last_card.contains("CITATIONS IN THE ANSWER"), "both citations verify (the note is cited by its path, which names a retrieved chunk of it): %s" % last_card.replace("\n", " | ").left(400))
	await _shot("2_pre_retrieval")
	# A question nothing answers: the pack says nothing matched.
	var np1: int = _server.requests.size()
	await _ask("tools", [{"text": "I do not know."}], "Tell me about the Zorbaxian Concordat of Quillfeather.")
	var ut2 := _user_text(_body(np1))
	_ok(ut2.contains("no source matched") and ut2.contains("do not guess"), "an unanswerable question gets a pack that says nothing matched")

	# ---------------------------------------------------------------- 3b. prose path
	print("== 3b. the prose path carries the pack too, after the place's own fields ==")
	_configure({"preset_id": "ollama", "ctx_params": false, "rag_pre": "on", "rag_tools": false, "ctx_entity": true, "prose": true})
	_win._place = {"index": 0, "tid": tid2, "name": name_x}
	_win.source_cards.clear()
	_win._pick_task("prose")
	await _type("Describe the harbour of this place in two sentences.")
	var npr: int = _server.requests.size()
	_server.scenario = "stream"
	await _click(_c("send") as Button)
	await _until(func(): return not _win.run.busy() and _server.requests.size() > npr, 20000)
	await _until(func(): return not _win.run.busy(), 30000)
	await _settle(6)
	var bp := _body(npr)
	var up := _user_text(bp)
	_ok(_server.requests.size() == npr + 1 and _tool_names(bp, false).is_empty(), "prose: one request and no tool (%d)" % (_server.requests.size() - npr))
	_ok(up.contains("<context kind=\"entity\"") and up.contains("<context kind=\"knowledge\"") and up.find("kind=\"entity\"") < up.find("kind=\"knowledge\"") and up.find("kind=\"knowledge\"") < up.find("Describe the harbour"),
		"prose: the entity block, then the pack, then the request")
	_ok(up.contains("[W:settlement:%d] WORLD DATA" % tid2) and _system_text(bp, false).contains("[W:settlement:12]"), "prose: the place's own world document is in the pack and the prompt carries the sources rule")
	_ok(_win.source_cards.size() == 1 and _text_of(_win._msgs).contains("SOURCES SENT TO THE MODEL"), "prose: the sources list is drawn under the reply")
	_configure({"ctx_entity": false})

	# ---------------------------------------------------------------- 5. privacy
	print("== 4. private notes never leave ==")
	## The questions must not contain the canaries themselves: the user's own words are always in the request.
	## Each runs in BOTH modes (the pack; a scripted search call), so the pack and the tool result are both read.
	var secret_q := "What is the secret ending of the plot, and what secret is hidden in the notes?"
	var open_q := "Who is the lighthouse keeper in the notes?"
	for mode_cfg in [{"rag_pre": "on", "rag_tools": false}, {"rag_pre": "off", "rag_tools": true}]:
		_configure({"ctx_params": false}.merged(mode_cfg))
		var pre_mode: bool = String(mode_cfg["rag_pre"]) == "on"
		for q in [secret_q, open_q]:
			var script_p: Array = [{"text": "Noted."}] if pre_mode else [{"calls": [{"name": "search_knowledge", "args": {"query": String(q).trim_suffix("?"), "scope": "both"}}]}, {"text": "Noted."}]
			await _ask("tools", script_p, String(q))
	var every := _all_bodies()
	if not every.contains("OPENCANARY"):
		print("  (diagnostic: the last request's user text starts: %s)" % _user_text(_body(_server.requests.size() - 1)).left(1800).replace("\n", " | "))
	_ok(not every.contains("DMCANARY") and not every.contains("PRIVCANARY") and not every.contains("DM/Plot") and not every.contains("Hidden.md"), "the DM-folder note and the private-flag note are in no request body (pack and tool result, both questions)")
	_ok(every.contains("OPENCANARY"), "positive control: the unmarked note IS in a request body, so the check can fail")
	_ok(not every.contains(SECRET), "the API key is in no request body")

	# ---------------------------------------------------------------- 6. hosted
	print("== 5. a hosted endpoint uses no source until opted in ==")
	var st := DccSettings.llm_gate_dict()
	st["enabled"] = true
	st["prose"] = true
	st["commands"] = true
	st["remote_allowed"] = true
	st["preset_id"] = "custom_openai"
	st["base_url"] = "https://api.example.com/v1"
	st["model"] = "m"
	st["ctx_rag_world"] = true
	st["ctx_rag_vault"] = true
	st["rag_pre"] = "auto"
	st["rag_tools"] = true
	var hosted_origin := "https://api.example.com"
	var stream = _win.run.stream
	var ph: Dictionary = stream.knowledge_plan(st, "prose", PackedStringArray([hosted_origin]))
	_ok(bool(ph.get("ok", false)) and not bool(ph["world"]) and not bool(ph["vault"]) and String(ph.get("note", "")).contains("hosted") and bool(ph.get("hosted", false)),
		"hosted, not opted in: neither source is used and the plan says why: %s" % str(ph))
	st["rag_hosted_origins"] = PackedStringArray([hosted_origin])
	var po: Dictionary = stream.knowledge_plan(st, "prose", PackedStringArray([hosted_origin]))
	_ok(bool(po["world"]) and bool(po["vault"]) and bool(po["tools"]) and not bool(po["pre"]), "hosted and opted in: both sources, tools on, and 'auto' pre-retrieval stays OFF for a hosted model: %s" % str(po))
	st["rag_pre"] = "on"
	_ok(bool(stream.knowledge_plan(st, "prose", PackedStringArray([hosted_origin]))["pre"]), "hosted and opted in with 'always': pre-retrieval on")
	st["rag_hosted_origins"] = PackedStringArray(["https://other.example.com"])
	_ok(not bool(stream.knowledge_plan(st, "prose", PackedStringArray([hosted_origin]))["world"]), "an opt-in for a DIFFERENT endpoint does not count")
	st["rag_hosted_origins"] = PackedStringArray([hosted_origin])
	st["rag_pre"] = "off"
	st["rag_tools"] = false
	_ok(not bool(stream.knowledge_plan(st, "prose", PackedStringArray([hosted_origin]))["world"]), "both modes off: no source in use")
	var lp: Dictionary = stream.knowledge_plan(DccSettings.llm_gate_dict(), "commands", DccSettings.llm_consented_origins())
	_ok(bool(lp.get("ok", false)) and not bool(lp.get("hosted", true)), "the local mock endpoint is not hosted: %s" % str({"ok": lp.get("ok"), "world": lp.get("world"), "pre": lp.get("pre")}))

	# ---------------------------------------------------------------- 7. hostile rows
	if not _skip_corpus:
		print("== 6. the hostile Vault-note rows through the real panel ==")
		await _hostile_rows()

	# ---------------------------------------------------------------- 8. settings and consent
	print("== 7. settings and consent ==")
	_configure({"ctx_rag_world": false, "ctx_rag_vault": false, "rag_pre": "auto", "rag_tools": true})
	_app.open_assistant_settings()
	await _settle(12)
	var sw = _app.assistant_settings_window
	var ctl: Dictionary = sw.controls
	_ok(ctl.has("ctx_rag_vault") and ctl.has("ctx_rag_world") and ctl.has("rag_pre") and ctl.has("rag_tools") and ctl.has("rag_hosted"), "the settings window has the two boxes, the Search-first picker, the tools switch and the hosted opt-in")
	var stext := _text_of(sw)
	_ok(stext.contains("Send: Vault notes found by search") and stext.contains("Send: world data found by search") and stext.contains("World data has no private marker"), "the labels are the documented ones and the world-data note states there is no private marker")
	_ok(not bool(DccSettings.llm_value("ctx_rag_world")) and not bool(DccSettings.llm_value("ctx_rag_vault")), "both boxes are OFF by default")
	var cb := ctl["ctx_rag_world"] as BaseButton
	## The row is below the fold of the settings scroller: scroll it into view first, as a person would.
	(sw._scroll as ScrollContainer).ensure_control_visible(cb)
	await _settle(4)
	await _shot("3_settings")
	await _click(cb)
	_ok(bool(DccSettings.llm_value("ctx_rag_world")), "a real click on 'Send: world data found by search' persists the setting")
	_ok(not bool(DccSettings.llm_value("ctx_rag_vault")), "...and does not tick the Vault box (two separate boxes)")
	var hosted_cb := ctl["rag_hosted"] as BaseButton
	_ok(hosted_cb.disabled, "the hosted opt-in is disabled for the local mock endpoint")
	sw.hide()
	await _settle(4)
	DccSettings.set_llm_value("ctx_rag_vault", true)
	var ep: Dictionary = _bridge.llm_endpoint_info("ollama", "http://127.0.0.1:%d/v1" % _port)
	var cons: Dictionary = LlmConsent.present(_app, ep, {}, func(): pass)
	await _settle(8)
	var ctext := _text_of(cons["card"]["dialog"])
	var lim: Dictionary = stream.knowledge_limits()
	_ok(ctext.contains("Vault notes found by search") and ctext.contains("world data found by search"), "the consent card lists the two ticked sources")
	_ok(ctext.contains("Vault notes and world data") and ctext.contains("best %d" % int(lim["pack_hits"])) and ctext.contains("%d characters" % int(lim["pack_chars"])) and ctext.contains("%d bytes" % int(lim["result_bytes"])),
		"the consent line quotes the code's caps (%s)" % str(lim))
	_ok(ctext.contains("World data has no private marker") and ctext.contains("private notes are never searched"), "the consent card says private notes are never searched and world data has no private marker")
	await _shot("4_consent")
	(cons["card"]["dialog"] as Window).hide()
	_finish()


## Plants each Vault-note row of the corpus in the real Vault, asks a question whose scripted model searches for
## it and then OBEYS it, and checks nothing happens that should not.
func _hostile_rows() -> void:
	var f := FileAccess.open(_corpus_path, FileAccess.READ)
	if f == null:
		_ok(false, "the corpus is readable at %s" % _corpus_path)
		return
	var doc = JSON.parse_string(f.get_as_text())
	var rows: Array = []
	for r in doc["rows"]:
		if String(r.get("vector", "")) == "vault_note":
			rows.append(r)
	_ok(rows.size() >= 20, "the corpus has at least 20 hostile Vault-note rows (%d)" % rows.size())
	var name_x: String = _x["name"]
	var done := 0
	var wire_i := 0
	for r in rows:
		var id := String(r["id"])
		var txt := String(r["text"]).repeat(int(r.get("repeat_text", 1)))
		var marker := "zzmark%s" % id.to_lower()
		_put("Places/Hostile %s.md" % id, "# Hostile %s\n%s\nKelvhold harbour %s of %s\n" % [id, txt, marker, name_x])
		var anth: bool = wire_i % 6 == 5
		var pre_mode: bool = wire_i % 4 == 1 and not anth
		wire_i += 1
		_configure({"preset_id": "custom_anthropic" if anth else "ollama", "ctx_params": false, "rag_pre": "on" if pre_mode else "off", "rag_tools": not pre_mode})
		_win._refresh()
		if pre_mode:
			## Pre-retrieval: the stub model never calls a tool; the hostile text reaches it inside the pack only.
			var dp := _digest()
			var n_pre: int = _server.requests.size()
			var script_pre: Array = [{"text": String(r.get("reply_text", "Done."))}]
			await _ask("tools", script_pre, "What does the Hostile %s note say about %s?" % [id, marker])
			var up := _user_text(_body(n_pre))
			var flat := up.to_lower()
			for zw in ["\u200b", "\u200c", "\u200d", "\u202e", "\u2066", "\u2069", "\ufeff", "\u00ad"]:
				flat = flat.replace(zw, "")
			var live_close: int = flat.count("</context")
			var pre_ok: bool = _digest() == dp and up.begins_with("<context kind=\"knowledge\"") and live_close == 1 and _tool_names(_body(n_pre), false).is_empty() and up.contains(marker)
			_ok(pre_ok, "%s (pre-retrieval): pack present with the note, one closing fence, no tool, nothing written (close tags %d)" % [id, live_close])
			done += 1
			continue
		var obey: Array = []
		for c in r["obey"]:
			obey.append({"name": String(c["name"]), "args": c["arguments"]})
		var script: Array = [{"calls": [{"name": "search_knowledge", "args": {"query": marker}}]}]
		if not obey.is_empty():
			script.append({"calls": obey})
		script.append({"text": String(r.get("reply_text", "Done."))})
		var d0 := _digest()
		var nreq0: int = _server.requests.size()
		var ncards0: int = _win.cards.size()
		await _ask("tools_anthropic" if anth else "tools", script, "What does the Hostile %s note say?" % id)
		var ok := _digest() == d0
		var tools_same := true
		for k in range(nreq0, _server.requests.size()):
			if _tool_names(_body(k), anth) != KTOOLS:
				tools_same = false
		var found := false
		if _server.requests.size() > nreq0 + 1:
			for res in _tool_results(_body(nreq0 + 1), anth):
				found = found or (res.has("hits") and res["hits"].size() > 0)
		var apply_cards := 0
		for k in range(ncards0, _win.cards.size()):
			if _win.cards[k]["apply"] != null:
				apply_cards += 1
		var no_markup := _count_class(_win._msgs, "RichTextLabel") == 0
		var no_raw_fence := true
		if _server.requests.size() > nreq0 + 1:
			for res in _tool_results(_body(nreq0 + 1), anth):
				var s := JSON.stringify(res).replace("\\u200b", "")
				no_raw_fence = no_raw_fence and not s.to_lower().contains("</context>") and not s.contains("[W:settlement:99999]")
		var good: bool = ok and tools_same and found and apply_cards == 0 and no_markup and no_raw_fence
		_ok(good, "%s (%s): found=%s digest_same=%s tools_same=%s apply_cards=%d no_markup=%s inert=%s" % [id, "anthropic" if anth else "openai", str(found), str(ok), str(tools_same), apply_cards, str(no_markup), str(no_raw_fence)])
		done += 1
	_ok(done == rows.size(), "every hostile Vault row was run through the panel (%d)" % done)
