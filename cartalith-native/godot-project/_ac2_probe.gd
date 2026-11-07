extends Node
## **AC-1 and AC-2 in the real shell: the action catalogue's live facts and the four catalogue tools**
## (`ASSISTANT_ACTIONS_PROPOSAL.md` s.3.1-s.3.2, milestones AC-1/AC-2; Ruling DW), against the in-test
## loopback server (`_llm_mock_server.gd`, scenario `tools`).
##
##   desktop: godot --path . --resolution 1600x1000 res://_ac2_probe.tscn -- [--dump <file>]
##
## **Windowed, never `--headless`**, from a project copy with its own `config/name` and a wiped user dir
## (`MISTAKES.md`). `--dump <file>` writes the live facts JSON, which is how the Rust fixture
## `crates/cartalith-llm/tests/fixtures/ac1_shell_facts_2026-10-31.json` was captured (so the Rust tests
## run over REAL `CommandIndex` rows, not a typed list). Anything else exits 2.
##
## What it protects (every check names it):
##   1. FACTS: `llm_actions.gd::facts` reports only `kind == "menu"` rows the index marks pinnable, each
##      named by a `DccMenus` constant, sorted, unique, none running (the index is only READ); two reads
##      give byte-identical text (the AC-1 "identical across builds" bar on the shell half); the curated
##      windows are tagged so Rust lists them once.
##   2. OFFER: a Commands request offers `list_actions`, `search_actions`, `describe_action` and
##      `propose_actions` beside the existing tools, and the system prompt names them and carries the
##      capability map.
##   3. BROWSE AND PROPOSE: a scripted list + search, then describe, then propose, produces a card
##      for two actions, and NOTHING moves (layers, camera, windows, undo stack).
##   4. NOT RETURNED: proposing an id the model was never given (`menu:save`) is refused B6 with the
##      rule on the card, retried once, and changes nothing.
## Result: `RESULT: PASS|FAIL (<n> failed) checks=<n>`. Exit 0 pass, 1 fail, 2 bad argument/watchdog.

const ENV_VAR := "CART_LLM_PROBE_KEYVAR"
const SECRET := "sk-PROBE-SECRET-ac2-0123456789"
const TOOLS := ["list_actions", "search_actions", "describe_action", "propose_actions"]

var _fails := 0
var _checks := 0
var _dump := ""
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


func _body(i: int) -> Dictionary:
	if i < 0 or i >= _server.requests.size():
		return {}
	var b = JSON.parse_string(String((_server.requests[i] as Dictionary).get("body", "")))
	return b if b is Dictionary else {}


func _tool_names(b: Dictionary) -> Array:
	var out: Array = []
	for t in b.get("tools", []):
		out.append(String(t["function"]["name"]))
	return out


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


func _snap() -> Dictionary:
	var vp = _app.viewport
	var layers := {}
	for l in CartographyWorkspace.LIVE_LAYERS:
		layers[String(l["id"])] = bool(vp.layer_visible(String(l["id"])))
	return {"layers": layers, "zoom": snappedf(float(vp.zoom()), 0.001), "digest": (_bridge.edit_state_digest() as Dictionary).duplicate()}


func _finish() -> void:
	OS.unset_environment(ENV_VAR)
	LlmWire.shutdown_all()
	if _server != null:
		_server.stop()
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--dump" and i + 1 < args.size():
			i += 1
			_dump = args[i]
		else:
			print("ABORT: unrecognised argument %s -- this probe reads --dump <file> only" % args[i])
			get_tree().quit(2)
			return
		i += 1
	if DisplayServer.get_name() == "headless":
		print("ABORT: run windowed; this probe drives the real panel")
		get_tree().quit(2)
		return
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
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
	_bridge.generate({"seed": 9137, "width_km": 1200.0, "grid_w": 256, "grid_h": 164, "archetype": "", "villages": false})
	while _bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _settle(8)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()

	# ---------------------------------------------------------------- 1. the live facts
	print("== 1. the shell facts ==")
	var A = load("res://shell/llm_actions.gd")
	var before := _snap()
	var f: Dictionary = A.facts(_app, _bridge)
	var rows: Array = f.get("menu_rows", [])
	_ok(rows.size() > 20, "the facts carry the runnable index rows (%d)" % rows.size())
	var names := {}
	var sorted_ok := true
	var prev := ""
	var all_named := true
	for r in rows:
		var rd: Dictionary = r
		var n := String(rd.get("name", ""))
		all_named = all_named and n != "" and n == n.to_lower() and not n.begins_with("id_")
		sorted_ok = sorted_ok and n >= prev
		prev = n
		names[n] = true
	_ok(all_named, "every row is named by a lower-case DccMenus constant, without the ID_ prefix")
	_ok(names.size() == rows.size(), "the names are unique (%d rows, %d names)" % [rows.size(), names.size()])
	_ok(sorted_ok, "the rows are sorted by name")
	_ok(names.has("deselect") and not names.has("lod_export_atlas"), "two constants share the Deselect row's id; the one that matches its title names it")
	var idx := CommandIndex.new()
	idx.build(_app, _bridge)
	var pinnable := 0
	for r in idx.all():
		if String((r as Dictionary).get("kind", "")) == "menu" and bool((r as Dictionary).get("pinnable", false)):
			pinnable += 1
	_ok(rows.size() <= pinnable and rows.size() >= pinnable - 6, "the facts are the pinnable menu rows that have a named constant (%d of %d pinnable)" % [rows.size(), pinnable])
	_ok(JSON.stringify(A.facts(_app, _bridge)) == JSON.stringify(f), "two reads give byte-identical facts")
	_ok(bool(f.get("has_world", false)) and f.has("timeline") and int(f["timeline"]["min"]) < int(f["timeline"]["max"]), "the world flag and the year track are reported")
	var windows := 0
	for r in rows:
		if (r as Dictionary).has("window"):
			windows += 1
	_ok(windows >= 5, "the curated windows are tagged so Rust lists them once (%d)" % windows)
	_ok(JSON.stringify(_snap()) == JSON.stringify(before), "reading the facts moved nothing")
	if _dump != "":
		var fh := FileAccess.open(_dump, FileAccess.WRITE)
		fh.store_string(JSON.stringify(f, "  ", true) + "\n")
		fh.close()
		print("  (facts written to %s)" % _dump)

	# ---------------------------------------------------------------- 2-4. the tools through the real panel
	_configure()
	_win.open()
	await _settle(10)
	var idle_digest := _snap()
	print("== 2/3. browse, describe, propose ==")
	var script := [
		{"calls": [{"name": "list_actions", "args": {"path": "carto/layers"}}, {"name": "search_actions", "args": {"query": "factions window"}}]},
		{"name": "describe_action", "args": {"ids": ["view.layer.rivers", "window.factions"]}},
		{"name": "propose_actions", "args": {"summary": "Hide rivers, open the roster", "actions": [
			{"id": "view.layer.rivers", "args": {"visible": false}, "reason": "declutter"}, {"id": "window.factions"}]}},
	]
	var n := await _ask(script, "hide the rivers and open the faction roster")
	_ok(n == 3, "browse (2 calls in one reply), describe and propose took 3 requests (%d)" % n)
	var b0 := _body(_server.requests.size() - n)
	var names0 := _tool_names(b0)
	var all_there := true
	for t in TOOLS:
		all_there = all_there and names0.has(t)
	_ok(all_there and names0.has("get_param_schema") and names0.has("propose_view_actions"), "the first request offers the four catalogue tools beside the existing ones: %s" % str(names0))
	var sys := String((b0.get("messages", [{}])[0] as Dictionary).get("content", ""))
	_ok(sys.contains("list_actions") and sys.contains("What exists (category"), "the system prompt names the tools and carries the capability map")
	var b1 := _body(_server.requests.size() - n + 1)
	var tool_results := ""
	for m in b1.get("messages", []):
		if String((m as Dictionary).get("role", "")) == "tool":
			tool_results += String((m as Dictionary).get("content", ""))
	_ok(tool_results.contains("view.layer.rivers") and tool_results.contains("window.factions"), "the second request carries the list and search results with the ids")
	var card := _action_card()
	_ok(not card.is_empty(), "a catalogue card was drawn")
	if not card.is_empty():
		var info: Dictionary = card["info"]
		var valid: Array = info.get("valid", [])
		_ok(valid.size() == 2 and String((valid[0] as Dictionary)["id"]) == "view.layer.rivers", "the card holds the two validated actions (%d)" % valid.size())
		_ok(card["apply"] != null and not (card["apply"] as Button).disabled, "the card has an Apply button (AC-3 runs it; _ac3_probe presses it); here nothing is pressed")
		_ok(bool((valid[0] as Dictionary)["args"].get("visible", true)) == false, "the layer argument is the typed false")
	_ok(JSON.stringify(_snap()) == JSON.stringify(idle_digest), "NOTHING moved: layers, camera and world are as before the proposal")

	print("== 4. an id the model was never given ==")
	var bad := [{"name": "propose_actions", "args": {"summary": "save", "actions": [{"id": "menu:save"}]}}]
	var m := await _ask(bad, "save the project")
	_ok(m == 2, "refused, retried once (2 requests): %d" % m)
	var c2 := _action_card()
	var text := ""
	if not c2.is_empty():
		text = JSON.stringify(c2["info"])
	_ok(text.contains("B6") and (c2["info"].get("valid", []) as Array).is_empty(), "the card shows rule B6 and no valid action")
	_ok(JSON.stringify(_snap()) == JSON.stringify(idle_digest), "nothing moved after the refused proposal")
	_finish()
