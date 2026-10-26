extends Node
## LIVE check of the Local profile against a REAL LM Studio (Ruling DQ; no mock model).
##
##   godot --path . --resolution 1280x800 _llmprofiles_live_probe.tscn -- --model qwen3-14b [--url http://127.0.0.1:1234/v1]
##
## NOT part of any suite: needs LM Studio running with the model loaded (`lms server start --port 1234`,
## `lms load qwen3-14b --gpu max -c 8192 --parallel 1 -y`). With no server it prints SKIP and exits 0.
## Synthetic prompt only. It proves, on a real model, what `_llmprofiles_probe.gd` proves from the request
## bytes: the Local profile's pinned reply cap and thinking-off setting reach the wire and are honoured --
## the reply finishes with visible text, and the completion tokens the SERVER reports stay at or under the
## pinned cap. A reasoning model left thinking would spend a 48-token cap on hidden reasoning and answer
## with nothing, so a non-empty reply under the cap is the evidence. Cleans up its settings on exit.
## Result: RESULT: PASS|FAIL (<n> failed) / SKIP. Exit 0, 1, or 2 (bad argument).

const CAP := 48
var _fails := 0
var _done: Array = []
var _failed: Array = []
var _usage: Array = []


func _ok(cond: bool, what: String) -> void:
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _until(cond: Callable, ms: int) -> bool:
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < ms:
		if cond.call():
			return true
		await get_tree().process_frame
	return bool(cond.call())


func _ready() -> void:
	var model := ""
	var url := "http://127.0.0.1:1234/v1"
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--model" and i + 1 < args.size():
			i += 1
			model = args[i]
		elif args[i] == "--url" and i + 1 < args.size():
			i += 1
			url = args[i]
		else:
			print("ABORT: unrecognised argument %s -- this probe reads --model <name> and --url <base> only" % args[i])
			get_tree().quit(2)
			return
		i += 1
	if model == "":
		print("ABORT: --model <name> is required")
		get_tree().quit(2)
		return
	get_tree().create_timer(300.0).timeout.connect(func() -> void:
		print("WATCHDOG")
		get_tree().quit(2))
	var origin := url.trim_suffix("/v1")
	var c := HTTPClient.new()
	c.connect_to_host(origin.get_slice("://", 1).get_slice(":", 0), int(origin.get_slice(":", 2)))
	var t0 := Time.get_ticks_msec()
	while c.get_status() in [HTTPClient.STATUS_RESOLVING, HTTPClient.STATUS_CONNECTING] and Time.get_ticks_msec() - t0 < 3000:
		c.poll()
		await get_tree().process_frame
	if c.get_status() != HTTPClient.STATUS_CONNECTED:
		print("RESULT: SKIP (no server answered at %s)" % origin)
		get_tree().quit(0)
		return
	c.close()

	DccSettings._ensure_loaded()
	for sec in ["llm", "llm_profiles"]:
		if DccSettings._cfg.has_section(sec):
			DccSettings._cfg.erase_section(sec)
	DccSettings._save()
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await get_tree().process_frame
	var panel = app.assistant_window
	# The real path: global switches (the user's), then the Local profile through the switch, then its edits.
	for k in ["enabled", "prose", "local_allowed"]:
		DccSettings.set_llm_value(k, true)
	var r := LlmProfiles.switch_to("local")
	_ok(bool(r.get("ok", false)) and bool(r.get("pending", false)), "switching to Local with the master switch on asks for consent first")
	var card := LlmProfiles.last_consent
	(card["choices"].get("safe") as Button).pressed.emit()
	await get_tree().create_timer(0.3).timeout
	_ok(LlmProfiles.active_id() == "local", "Local is active after agreeing")
	DccSettings.set_llm_value("model", model)
	DccSettings.set_llm_value("base_url", url)
	DccSettings.set_llm_value("max_tokens", CAP)
	var live := LlmProfiles.live_values()
	_ok(String(live["thinking"]) == "none" and int(live["max_tokens"]) == CAP, "Local carries thinking off and the pinned cap %d" % CAP)

	var run := LlmRun.new()
	add_child(run)
	run.finished.connect(func(i: Dictionary): _done.append(i))
	run.failed.connect(func(m: String): _failed.append(m))
	run.usage.connect(func(u: Dictionary): _usage.append(u))
	var fields := "population: 4200\nculture: Marsh folk\nreligion: River saints\ngovernment: council of ferrymen"
	var req := {"task": "prose", "request": "Describe this fictional town in two plain sentences.", "entity_label": "Veldmark",
		"entity_fields": fields, "max_tokens": panel._reply_cap(1024), "include_usage": true}
	_ok(int(req["max_tokens"]) == CAP, "the panel's reply cap is the profile's pin: %d" % int(req["max_tokens"]))
	var res: Dictionary = run.begin(DccSettings.llm_gate_dict(), "prose", DccSettings.llm_consented_origins(), req)
	_ok(bool(res.get("ok", false)), "the gate lets the Local profile's request through: %s" % str(res.get("reason", "ok")))
	_ok(await _until(func(): return not _done.is_empty() or not _failed.is_empty(), 240000), "the reply finished")
	_ok(_failed.is_empty(), "no failure: %s" % str(_failed))
	print("REPLY: " + run.text_so_far.replace("\n", " ").left(300))
	_ok(run.text_so_far.strip_edges().length() > 10, "the reply has visible text (a thinking model would have spent the cap on hidden reasoning)")
	_ok(_usage.size() == 1 and int(_usage[0].get("completion", 99999)) <= CAP, "the server reports completion tokens within the pinned cap: %s" % str(_usage))
	run.shutdown()
	for sec in ["llm", "llm_profiles"]:
		if DccSettings._cfg.has_section(sec):
			DccSettings._cfg.erase_section(sec)
	DccSettings._save()
	print("RESULT: %s (%d failed)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
