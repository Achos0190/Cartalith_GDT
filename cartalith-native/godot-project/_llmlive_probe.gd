extends Node
## LIVE check of the assistant's wire against a REAL local server (LLM-3's "tested with" entry).
##
##   godot --path . --resolution 800x600 _llmlive_probe.tscn -- --model phi4:latest [--url http://localhost:11434/v1]
##
## NOT part of any suite: it needs an OpenAI-compatible server already running (Ollama or LM Studio)
## and a model already pulled; it downloads nothing and starts nothing. With no server it prints
## `SKIP` and exits 0 -- it never fails a build for want of a model. **Synthetic prompts only**: the
## place's "fields" are written in this file, so nothing private can reach the model.
##
## It drives the same `LlmRun` -> `LlmStream` -> `LlmWire` path the panel uses (gate, worker thread,
## `HTTPClient`, Rust parser) and prints the numbers a dated "tested with" line needs: the server's
## own version, time to first token, token events, the usage the server reported, and the cancel
## latency and socket check. Run from a scratch copy like the other probes.
## Result: `RESULT: PASS|FAIL (<n> failed)`, `RESULT: SKIP (<why>)`. Exit 0, 1, or 2 (bad argument).

var _fails := 0
var _run: LlmRun
var _tokens := 0
var _done: Array = []
var _failed: Array = []
var _usage: Array = []
var _cancelled := false


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


func _netstat_to(pid: int, port: int) -> int:
	## `netstat -p TCP` lists IPv4 only; `localhost` resolves to ::1 first on this machine, so the
	## IPv6 table (`TCPv6`) is read too (this probe's first run saw 0 rows for that reason).
	var n := 0
	for proto in ["TCP", "TCPv6"]:
		var out: Array = []
		OS.execute("netstat", ["-ano", "-p", proto], out)
		for line in String("".join(out)).split("\n"):
			var cols := line.strip_edges().split(" ", false)
			if cols.size() >= 5 and cols[0] == "TCP" and int(cols[cols.size() - 1]) == pid \
					and cols[2].ends_with(":%d" % port) and cols[3] != "LISTENING":
				n += 1
	return n


func _reset() -> void:
	if _run != null:
		_run.shutdown()
		_run.queue_free()
	_run = LlmRun.new()
	add_child(_run)
	_tokens = 0
	_done = []
	_failed = []
	_usage = []
	_cancelled = false
	_run.token.connect(func(_t: String): _tokens += 1)
	_run.finished.connect(func(i: Dictionary): _done.append(i))
	_run.failed.connect(func(m: String): _failed.append(m))
	_run.usage.connect(func(i: Dictionary): _usage.append(i))
	_run.cancelled.connect(func(): _cancelled = true)


func _ready() -> void:
	var model := ""
	var url := "http://localhost:11434/v1"
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
		print("ABORT: --model <name> is required (a model already pulled; this probe downloads nothing)")
		get_tree().quit(2)
		return
	get_tree().create_timer(300.0).timeout.connect(func() -> void:
		print("WATCHDOG")
		get_tree().quit(2))
	var origin := url.trim_suffix("/v1").trim_suffix("/")
	var port := int(origin.get_slice(":", 2))
	# Is a server there? A plain GET of the model list, through HTTPClient on this thread (a probe-only check).
	var c := HTTPClient.new()
	c.connect_to_host(origin.get_slice("://", 1).get_slice(":", 0), port)
	var t0 := Time.get_ticks_msec()
	while c.get_status() in [HTTPClient.STATUS_RESOLVING, HTTPClient.STATUS_CONNECTING] and Time.get_ticks_msec() - t0 < 3000:
		c.poll()
		await get_tree().process_frame
	if c.get_status() != HTTPClient.STATUS_CONNECTED:
		print("RESULT: SKIP (no server answered at %s)" % origin)
		get_tree().quit(0)
		return
	c.close()

	var settings := {"enabled": true, "prose": true, "commands": false, "edit": false,
		"local_allowed": true, "remote_allowed": false, "preset_id": "ollama" if port == 11434 else "lmstudio",
		"base_url": url, "model": model, "key_source": "env", "env_var": "CART_LLM_NO_SUCH_VAR"}
	var consent := PackedStringArray([origin])
	var fields := "population: 4200\nculture: Marsh folk\nreligion: River saints\ngovernment: council of ferrymen\ntrade: grain, salt"
	var base_args := {"task": "prose", "request": "Describe this fictional town in three plain sentences.",
		"entity_label": "Veldmark", "entity_fields": fields, "max_tokens": 160, "include_usage": true}

	print("== live: %s model %s ==" % [origin, model])
	_reset()
	var res: Dictionary = _run.begin(settings, "prose", consent, base_args)
	_ok(bool(res.get("ok", false)), "the gate lets a local request through: %s" % str(res.get("reason", "ok")))
	var t_begin := Time.get_ticks_msec()
	_ok(await _until(func(): return not _done.is_empty() or not _failed.is_empty(), 240000), "the reply finished")
	var total := Time.get_ticks_msec() - t_begin
	_ok(_failed.is_empty(), "no failure: %s" % str(_failed))
	_ok(_tokens > 3, "streamed as %d token events" % _tokens)
	_ok(_run.first_token_ms >= 0, "first token after %d ms; whole reply %d ms" % [_run.first_token_ms, total])
	_ok(_run.first_token_ms < total, "the first token came before the end (streamed, not buffered)")
	print("REPLY: " + _run.text_so_far.replace("\n", " ").left(400))
	_ok(_usage.size() == 1 and _usage[0].has("prompt") and _usage[0].has("completion"), "usage reported by the server: %s" % str(_usage))
	_ok(_done.size() == 1, "one finish: %s" % str(_done))
	await _until(func(): return not _run.busy(), 5000)

	# Cancel against the real server, mid-stream.
	print("== live: cancel ==")
	_reset()
	var long_args := base_args.duplicate()
	long_args["request"] = "Write a long, detailed history of this fictional town, many paragraphs."
	long_args["max_tokens"] = 1500
	res = _run.begin(settings, "prose", consent, long_args)
	_ok(bool(res.get("ok", false)), "a long request starts")
	_ok(await _until(func(): return _tokens >= 5, 120000), "tokens are flowing (%d)" % _tokens)
	var pid := OS.get_process_id()
	var open_before := _netstat_to(pid, port)
	_ok(open_before >= 1, "positive control: netstat shows %d open connection(s) to port %d mid-stream" % [open_before, port])
	var before := _tokens
	var tc := Time.get_ticks_msec()
	_run.cancel()
	_ok(await _until(func(): return _cancelled, 5000), "cancelled")
	var lat := Time.get_ticks_msec() - tc
	await get_tree().create_timer(0.5).timeout
	var open_after := _netstat_to(pid, port)
	print("MEASURE live cancel latency %d ms; open connections to the server %d -> %d" % [lat, open_before, open_after])
	_ok(open_after == 0, "the socket to the real server is closed after Stop (%d -> %d rows)" % [open_before, open_after])
	_ok(_tokens == before, "no token after Stop (%d before, %d after)" % [before, _tokens])
	_ok(LlmWire.live_threads == 0 and not _run.busy(), "the worker is joined")
	_ok(_done.is_empty() and _failed.is_empty(), "a cancelled reply is neither done nor failed")
	_run.shutdown()
	print("RESULT: %s (%d failed)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
