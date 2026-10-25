extends Node
## Probe for the assistant's wire: `shell/llm_wire.gd` (a worker `Thread` polling `HTTPClient`),
## `shell/llm_run.gd` (the main-thread coordinator) and the Rust `LlmStream` class
## (`cartalith-godot/src/llm_stream.rs`), against a REAL loopback HTTP/1.1 server
## (`_llm_mock_server.gd`) -- milestone LLM-3 of `LLM_ASSISTANT_SCOPE.md`.
##
##   godot --path . --resolution 800x600 _llmstream_probe.tscn
##
## **Windowed, from a scratch copy with its own `config/name` and a wiped user dir** (`MISTAKES.md`).
## Takes no arguments; any argument aborts with exit 2. It sets the environment variable
## `CART_LLM_PROBE_KEYVAR` itself (never printed) and clears it on exit.
##
## What it protects (each against the real thread, socket and Rust parser):
##   1. GATE: every refusal (feature off, capability off, local not allowed, no consent, hosted
##      endpoint, unknown task) starts NOTHING -- `LlmWire.connect_attempts` and the server's
##      `accepted` do not move, no thread is live. The happy path right after is the positive
##      control: the same counters DO move.
##   2. STREAM: tokens arrive incrementally (the first before the server has sent them all), their
##      concatenation equals what the server sent, usage and finish_reason are reported, the
##      request on the wire has the chunked-SSE shape, the right path and body, and carries no
##      Authorization header when no key is set.
##   3. SPLIT: an SSE event cut mid-line across two TCP writes decodes to the same text.
##   4. KEY: with a key set the server sees `Authorization: Bearer <key>`; a 401 that ECHOES it
##      reaches the user with the key replaced; the key is absent from `godot.log`.
##   5. CANCEL: Stop silences the parser at once (no token after it), closes the socket (the server
##      sees the hang-up), joins the thread (`live_threads` 0, `is_started()` false) -- latency
##      printed over 5 runs.
##   6. UNLOAD: removing the node mid-stream joins the worker; a bare `LlmWire` dropped with its
##      thread running is kept alive by a registry and joined by `LlmWire.shutdown_all()`.
##   7. FAILURES: a dead port, a stream with no [DONE] but a finish_reason, a stream cut early.
##   8. NOTE: `compose_note` marks, appends and defuses; a vault write is not made by anything here.
## Result: `RESULT: PASS|FAIL (<n> failed)`. Exit 0 pass, 1 fail, 2 bad argument or watchdog.

const SECRET := "sk-PROBE-SECRET-0123456789wxyz"
const ENV_VAR := "CART_LLM_PROBE_KEYVAR"

var _fails := 0
var _server: LlmMockServer
var _run: LlmRun
var _tokens: PackedStringArray = PackedStringArray()
var _done: Array = []
var _failed: Array = []
var _cancelled := false
var _usage: Array = []
var _first_tokens_sent := -1


func _ok(cond: bool, what: String) -> void:
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


## Waits (pumping frames) until `cond` is true or `ms` pass; returns whether it became true.
func _until(cond: Callable, ms: int = 5000) -> bool:
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < ms:
		if cond.call():
			return true
		await get_tree().process_frame
	return bool(cond.call())


func _settings(port: int, over: Dictionary = {}) -> Dictionary:
	var s := {
		"enabled": true, "prose": true, "commands": false, "edit": false,
		"local_allowed": true, "remote_allowed": false,
		"preset_id": "ollama", "base_url": "http://127.0.0.1:%d/v1" % port,
		"model": "mock-model", "key_source": "env", "env_var": ENV_VAR,
	}
	s.merge(over, true)
	return s


func _consent(port: int) -> PackedStringArray:
	return PackedStringArray(["http://127.0.0.1:%d" % port])


func _args(over: Dictionary = {}) -> Dictionary:
	var a := {"task": "prose", "request": "Describe the port.", "entity_label": "Veldmark",
		"entity_fields": "population: 4200\nculture: Marsh", "max_tokens": 128, "include_usage": true}
	a.merge(over, true)
	return a


func _reset_run() -> void:
	if _run != null:
		_run.shutdown()
		_run.queue_free()
		await _frames(2)
	_run = LlmRun.new()
	add_child(_run)
	_tokens = PackedStringArray()
	_done = []
	_failed = []
	_usage = []
	_cancelled = false
	_first_tokens_sent = -1
	_run.token.connect(func(t: String):
		if _tokens.is_empty():
			_first_tokens_sent = _server.tokens_sent
		_tokens.append(t))
	_run.finished.connect(func(i: Dictionary): _done.append(i))
	_run.failed.connect(func(m: String): _failed.append(m))
	_run.usage.connect(func(i: Dictionary): _usage.append(i))
	_run.cancelled.connect(func(): _cancelled = true)


func _contains(hay: PackedByteArray, needle: String) -> bool:
	var n := needle.to_utf8_buffer()
	if n.is_empty() or hay.size() < n.size():
		return false
	for i in range(0, hay.size() - n.size() + 1):
		if hay[i] == n[0]:
			var hit := true
			for j in range(1, n.size()):
				if hay[i + j] != n[j]:
					hit = false
					break
			if hit:
				return true
	return false


func _finish() -> void:
	OS.unset_environment(ENV_VAR)
	if _run != null:
		_run.shutdown()
	if _server != null:
		_server.stop()
	print("RESULT: %s (%d failed)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)


func _ready() -> void:
	if not OS.get_cmdline_user_args().is_empty():
		print("ABORT: this probe takes no arguments")
		get_tree().quit(2)
		return
	get_tree().create_timer(240.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	OS.unset_environment(ENV_VAR)
	_server = LlmMockServer.new()
	add_child(_server)
	_ok(_server.start(), "the mock server listens on 127.0.0.1:%d" % _server.port)
	await _reset_run()
	_ok(_run.available(), "the loaded library has LlmStream")
	if not _run.available():
		_finish()
		return
	var port := _server.port
	var ok_settings := _settings(port)

	# ------------------------------------------------------------------ 1. gate refusals start nothing
	print("== 1. refusals start nothing ==")
	var attempts0 := LlmWire.connect_attempts
	var rows := [
		["feature off", _settings(port, {"enabled": false}), "prose", _consent(port), _args(), "feature_off"],
		["capability off", _settings(port, {"prose": false}), "prose", _consent(port), _args(), "capability_off"],
		["local not allowed", _settings(port, {"local_allowed": false}), "prose", _consent(port), _args(), "local_not_allowed"],
		["no consent", ok_settings, "prose", PackedStringArray(), _args(), "consent_needed"],
		["other origin consented", ok_settings, "prose", PackedStringArray(["http://127.0.0.1:1"]), _args(), "consent_needed"],
		["no model", _settings(port, {"model": ""}), "prose", _consent(port), _args(), "model_missing"],
		["no provider", _settings(port, {"preset_id": ""}), "prose", _consent(port), _args(), "no_provider"],
		["userinfo in URL", _settings(port, {"base_url": "http://u:p@127.0.0.1:%d/v1" % port}), "prose", _consent(port), _args(), "base_url"],
		["hosted endpoint", _settings(port, {"preset_id": "openai", "base_url": "https://api.openai.com/v1", "remote_allowed": true, "local_allowed": false}), "prose", PackedStringArray(["https://api.openai.com"]), _args(), "key_env_missing"],
		["unknown capability", ok_settings, "edit", _consent(port), _args(), ""],
		["unknown task", ok_settings, "prose", _consent(port), _args({"task": "commands"}), ""],
		["empty request", ok_settings, "prose", _consent(port), _args({"request": "  "}), "empty_request"],
		["prose without its source", ok_settings, "prose", _consent(port), {"task": "prose", "request": "go"}, "missing_source"],
	]
	for r in rows:
		var res: Dictionary = _run.begin(r[1], r[2], r[3], r[4])
		_ok(not bool(res.get("ok", false)), "refused: %s" % r[0])
		_ok(String(res.get("reason", "")) != "", "...with a reason: '%s'" % String(res.get("reason", "")).left(70))
		if r[5] != "":
			_ok(String(res.get("code", "")) == r[5], "...code %s (got %s)" % [r[5], String(res.get("code", ""))])
		_ok(not res.has("headers") and not res.has("host") and not res.has("body"), "...and returns no connection data")
	await _frames(10)
	_ok(LlmWire.connect_attempts == attempts0, "no connection was attempted by any refusal (%d -> %d)" % [attempts0, LlmWire.connect_attempts])
	_ok(_server.accepted == 0, "the server accepted 0 connections")
	_ok(LlmWire.live_threads == 0, "no worker thread is live")
	_ok(not _run.busy(), "the run is idle")

	# ------------------------------------------------------------------ 2. streaming happy path
	print("== 2. streaming ==")
	_server.scenario = "stream"
	_server.token_delay_ms = 25
	var expected := "".join(_server.tokens)
	var t0 := Time.get_ticks_msec()
	var res: Dictionary = _run.begin(ok_settings, "prose", _consent(port), _args())
	_ok(bool(res.get("ok", false)), "a gated request starts")
	_ok(not res.has("headers") and not res.has("body"), "...and the caller is not handed the headers or body")
	_ok(String(res.get("host", "")) == "127.0.0.1" and int(res.get("port", 0)) == port, "...for the chosen host and port")
	_ok(bool(res["sent"]["entity"]) and not bool(res["sent"]["note"]) and not bool(res["sent"]["events"]), "...sending the place's fields only")
	_ok(LlmWire.connect_attempts == attempts0 + 1, "one connection attempt (positive control for section 1)")
	_ok(await _until(func(): return not _done.is_empty() or not _failed.is_empty(), 8000), "the stream finished")
	var total_ms := Time.get_ticks_msec() - t0
	_ok(_failed.is_empty(), "no failure: %s" % str(_failed))
	_ok("".join(_tokens) == expected, "the tokens concatenate to what the server sent: '%s'" % "".join(_tokens))
	_ok(_tokens.size() == _server.tokens.size(), "one token event per SSE event (%d)" % _tokens.size())
	_ok(_first_tokens_sent >= 0 and _first_tokens_sent < _server.tokens.size(), "the first token was shown before the server had sent them all (server had sent %d of %d)" % [_first_tokens_sent, _server.tokens.size()])
	_ok(_run.first_token_ms >= 0, "first token after %d ms (stream total %d ms)" % [_run.first_token_ms, total_ms])
	_ok(_done.size() == 1 and String(_done[0].get("finish_reason", "")) == "stop", "one finish with finish_reason 'stop'")
	_ok(_usage.size() == 1 and int(_usage[0].get("prompt", -1)) == 42 and int(_usage[0].get("completion", -1)) == _server.tokens.size(), "usage reported: %s" % str(_usage))
	var req: Dictionary = _server.requests[0]
	_ok(req["path"] == "/v1/chat/completions", "POST path is /v1/chat/completions: %s" % req["path"])
	_ok(not req["headers"].has("authorization"), "no Authorization header without a key")
	_ok(String(req["headers"].get("accept", "")) == "text/event-stream", "Accept: text/event-stream")
	var body = JSON.parse_string(String(req["body"]))
	_ok(body is Dictionary and body.get("stream") == true and body.get("model") == "mock-model", "body: stream true, model as set")
	_ok(body is Dictionary and int(body.get("max_tokens", 0)) == 128, "body carries max_tokens 128 (every request has a cap)")
	var user_msg := ""
	for m in body.get("messages", []):
		if m["role"] == "user":
			user_msg = String(m["content"])
	_ok(user_msg.contains("population: 4200") and user_msg.contains("Describe the port."), "the user message holds the opted-in fields and the request")
	_ok(not user_msg.contains("kind=\"note\"") and not user_msg.contains("kind=\"events\""), "...and no source that was not provided")
	await _until(func(): return not _run.busy(), 3000)
	_ok(not _run.busy() and LlmWire.live_threads == 0 and not _run.wire.is_started(), "after the stream the worker is joined (live_threads %d)" % LlmWire.live_threads)

	# ------------------------------------------------------------------ 3. split events
	print("== 3. SSE events split across TCP writes ==")
	await _reset_run()
	_server.scenario = "stream_split"
	_server.token_delay_ms = 10
	res = _run.begin(ok_settings, "prose", _consent(port), _args())
	await _until(func(): return not _done.is_empty() or not _failed.is_empty(), 8000)
	_ok(_failed.is_empty() and "".join(_tokens) == expected, "split events decode to the same text: '%s'" % "".join(_tokens))

	# ------------------------------------------------------------------ 4. the key
	print("== 4. the key ==")
	OS.set_environment(ENV_VAR, SECRET)
	await _reset_run()
	_server.scenario = "stream"
	_server.token_delay_ms = 5
	res = _run.begin(ok_settings, "prose", _consent(port), _args())
	await _until(func(): return not _done.is_empty() or not _failed.is_empty(), 8000)
	var req2: Dictionary = _server.requests[_server.requests.size() - 1]
	_ok(String(req2["headers"].get("authorization", "")) == "Bearer " + SECRET, "the server received the Authorization header with the key")
	_ok(not JSON.stringify(res).contains(SECRET), "the dictionary returned to the shell holds no key")
	_ok(not String(req2["body"]).contains(SECRET), "the request body holds no key")
	await _reset_run()
	_server.scenario = "unauthorized"
	res = _run.begin(ok_settings, "prose", _consent(port), _args())
	await _until(func(): return not _failed.is_empty() or not _done.is_empty(), 8000)
	_ok(_failed.size() == 1, "a 401 is reported as one failure")
	var msg := String(_failed[0]) if not _failed.is_empty() else ""
	_ok(msg.contains("401"), "...naming the status: '%s'" % msg.left(120))
	_ok(not msg.contains(SECRET), "...and the key the server echoed is NOT in it")
	_ok(msg.contains("[redacted-key]"), "...it was replaced by the redaction marker (positive control for the scrub)")
	_ok(_tokens.is_empty(), "no token was shown for an error reply")
	await _until(func(): return not _run.busy(), 3000)
	var log_bytes := _file_bytes(ProjectSettings.globalize_path("user://logs/godot.log"))
	_ok(log_bytes.size() > 100, "godot.log was readable (%d bytes), so the grep is not over nothing" % log_bytes.size())
	_ok(not _contains(log_bytes, SECRET), "godot.log holds no key")
	OS.unset_environment(ENV_VAR)

	# ------------------------------------------------------------------ 5. cancel
	print("== 5. cancel ==")
	var lat_cancel: Array = []
	var lat_hangup: Array = []
	for round in 5:
		await _reset_run()
		_server.scenario = "endless"
		_server.token_delay_ms = 10
		var hangups0 := _server.client_hangups
		res = _run.begin(ok_settings, "prose", _consent(port), _args())
		_ok(bool(res.get("ok", false)), "round %d: endless stream started" % round)
		_ok(await _until(func(): return _tokens.size() >= 3, 5000), "round %d: three tokens arrived" % round)
		var before := _tokens.size()
		var sent_before := _server.tokens_sent
		var tc := Time.get_ticks_msec()
		_run.cancel()
		_ok(await _until(func(): return _cancelled, 3000), "round %d: 'cancelled' fired" % round)
		lat_cancel.append(Time.get_ticks_msec() - tc)
		_ok(await _until(func(): return _server.client_hangups > hangups0, 3000), "round %d: the server saw the socket close" % round)
		lat_hangup.append(Time.get_ticks_msec() - tc)
		await _frames(20)
		_ok(_tokens.size() == before, "round %d: no token after Stop (%d before, %d after; the server had sent %d more by then)" % [round, before, _tokens.size(), _server.tokens_sent - sent_before])
		_ok(_done.is_empty() and _failed.is_empty(), "round %d: a cancelled reply is neither 'done' nor 'failed'" % round)
		_ok(not _run.busy() and not _run.wire.is_started() and not _run.wire.is_running(), "round %d: the thread is joined" % round)
		_ok(LlmWire.live_threads == 0, "round %d: live_threads is 0" % round)
	lat_cancel.sort()
	lat_hangup.sort()
	print("MEASURE cancel-to-'cancelled' ms over 5: min %d median %d max %d" % [lat_cancel[0], lat_cancel[2], lat_cancel[4]])
	print("MEASURE cancel-to-server-saw-close ms over 5: min %d median %d max %d" % [lat_hangup[0], lat_hangup[2], lat_hangup[4]])
	_ok(lat_hangup[4] < 1000, "the socket was closed within 1 s every time")

	# ------------------------------------------------------------------ 6. unload
	print("== 6. no thread left on free or unload ==")
	await _reset_run()
	_server.scenario = "endless"
	var h0 := _server.client_hangups
	res = _run.begin(ok_settings, "prose", _consent(port), _args())
	await _until(func(): return _tokens.size() >= 2, 5000)
	_ok(LlmWire.live_threads == 1, "one worker thread is live mid-stream")
	var node := _run
	_run = null
	remove_child(node)   # _exit_tree: the same path a closing window or a freed scene takes
	_ok(LlmWire.live_threads == 0, "removing the node from the tree joined the worker at once (live_threads %d)" % LlmWire.live_threads)
	node.free()
	_ok(await _until(func(): return _server.client_hangups > h0, 3000), "...and the socket was closed")
	# A bare wire dropped with its thread running (the library-unload shape: nothing calls shutdown).
	var wire := LlmWire.new()
	var tgt := {"ok": true, "host": "127.0.0.1", "port": port, "tls": false, "path": "/v1/chat/completions",
		"headers": PackedStringArray(["Content-Type: application/json"]),
		"body": JSON.stringify({"model": "m", "stream": true, "messages": []}).to_utf8_buffer()}
	_server.scenario = "endless"
	_ok(wire.start(tgt), "a bare wire starts")
	await _frames(10)
	_ok(LlmWire.live_threads == 1 and wire.is_running(), "its thread is running")
	wire = null   # dropped with its thread running: the static registry keeps it alive, so no use-after-free
	await _frames(5)
	_ok(LlmWire.live_threads == 1, "a dropped wire's thread is still tracked, not orphaned (live_threads %d)" % LlmWire.live_threads)
	LlmWire.shutdown_all()   # what the shell does when it exits, before the library unloads
	_ok(LlmWire.live_threads == 0, "shutdown_all() joined it (live_threads %d)" % LlmWire.live_threads)
	_ok(await _until(func(): return _server.client_hangups > h0 + 1, 3000), "...and its socket was closed")
	var refused := LlmWire.new()
	_ok(not refused.start({"ok": false, "host": "127.0.0.1"}), "a wire refuses a target that was not 'ok'")
	_ok(not refused.start({"ok": true, "host": ""}), "...and one with no host")
	_ok(LlmWire.live_threads == 0, "...starting no thread")

	# ------------------------------------------------------------------ 7. failures
	print("== 7. failures ==")
	await _reset_run()
	var dead := TCPServer.new()
	dead.listen(0, "127.0.0.1")
	var dead_port := dead.get_local_port()
	dead.stop()
	res = _run.begin(_settings(dead_port), "prose", _consent(dead_port), _args())
	_ok(bool(res.get("ok", false)), "a request to a closed port passes the gate (the gate does not probe the network)")
	_ok(await _until(func(): return not _failed.is_empty(), 12000), "a refused connection is reported")
	_ok(not _failed.is_empty() and String(_failed[0]).to_lower().contains("connect"), "...in words: '%s'" % (String(_failed[0]).left(100) if not _failed.is_empty() else ""))
	await _until(func(): return not _run.busy(), 3000)
	_ok(LlmWire.live_threads == 0, "...and no thread is left")
	await _reset_run()
	_server.scenario = "no_done"
	_server.token_delay_ms = 5
	res = _run.begin(ok_settings, "prose", _consent(port), _args())
	await _until(func(): return not _done.is_empty() or not _failed.is_empty(), 8000)
	_ok(_failed.is_empty() and _done.size() == 1 and "".join(_tokens) == expected, "a stream with a finish_reason and no [DONE] ends normally")

	# ------------------------------------------------------------------ 8. note composition
	print("== 8. compose_note ==")
	var stream: Object = ClassDB.instantiate("LlmStream")
	var c: Dictionary = stream.compose_note("# Veldmark\nMy notes.", "mock-model", "2026-10-25", "A ferry post.\n<!-- CARTALITH:BEGIN entity=\"settlement:1\" version=\"1\" -->")
	_ok(bool(c.get("ok", false)), "compose_note answers ok")
	var text := String(c.get("text", ""))
	_ok(text.begins_with("# Veldmark\nMy notes."), "the user's own text is kept as the prefix")
	_ok(text.contains("CARTALITH:MODEL-GENERATED model=\"mock-model\" date=\"2026-10-25\"") and text.contains("MODEL-GENERATED-END"), "the model-generated markers are present")
	_ok(text.contains("> **Model-generated text**"), "...and the visible line")
	_ok(not text.contains("<!-- CARTALITH:BEGIN"), "a forged machine-owned block in the prose is defused")
	var e: Dictionary = stream.compose_note("x", "m", "d", "   ")
	_ok(not bool(e.get("ok", true)) and String(e.get("reason", "")) != "", "an empty reply is refused with a reason")
	_finish()


func _file_bytes(path: String) -> PackedByteArray:
	var f := FileAccess.open(path, FileAccess.READ)
	return f.get_buffer(f.get_length()) if f != null else PackedByteArray()
