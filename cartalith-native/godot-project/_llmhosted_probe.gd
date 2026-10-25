extends Node
## Probe for the LLM-4 additions, through the REAL Godot `HTTPClient` worker thread, the Rust
## `LlmStream` and a REAL loopback server (`_llm_mock_server.gd`): the Anthropic Messages wire,
## the running token counter and its opt-in cap, the one automatic retry, and the hosted request
## shape. Milestone LLM-4 of `LLM_ASSISTANT_SCOPE.md`.
##
##   godot --path . --resolution 800x600 _llmhosted_probe.tscn
##
## **Windowed, from a scratch copy with its own `config/name` and a wiped user dir** (`MISTAKES.md`).
## Takes no arguments (any aborts with exit 2). It sets `CART_LLM_PROBE_KEYVAR` itself to a FAKE key,
## never printed, and clears it on exit.
##
## **No real hosted API is called and none could be: no key exists for this probe.** Hosted
## (https, api.anthropic.com / api.openai.com) requests are only PREPARED (`LlmStream.prepare`
## opens nothing) and inspected; every request that is actually sent goes to 127.0.0.1. TLS is
## therefore NOT exercised by this probe.
##
## What it protects:
##   1. ANTHROPIC over the real wire: the server received `POST /v1/messages`, `x-api-key` carrying
##      the key, `anthropic-version: 2023-06-01`, no `Authorization`, a body with `max_tokens`, the
##      system text lifted to `system`; the typed events decode to the exact text; ONE usage event
##      (30 in, the cumulative out); the counter holds it.
##   2. NO USAGE: a stream with no usage object ends normally, the totals do not move, `calls`
##      does and `calls_reported` does not (never shown as 0).
##   3. HOSTED SHAPE (prepare only, no socket): OpenAI hosted -> tls, 443, `/v1/chat/completions`,
##      Bearer, `max_completion_tokens`; Anthropic hosted -> tls, 443, `/v1/messages`, `x-api-key`;
##      `connect_attempts` unchanged. Plain http to a remote host, no remote switch, no consent and
##      no key each still refuse.
##   4. NO CAP BY DEFAULT: the counter preloaded to ~4.6e18 tokens, TEN sequential requests all go
##      out (the server accepts ten); the same state with a cap set is refused with
##      `token_cap_reached` and opens nothing.
##   5. RETRY: one 529 then success -> two requests, the full text, `retrying` fired once, no
##      `failed`; endless 529 -> exactly two requests then `failed`; 401 -> one request; Stop during
##      the retry pause -> cancelled, one request.
##      A refused connection (nothing listening) is NOT retried.
##   6. RESET: `reset_token_counts` empties the totals (the keys are omitted again).
## Result: `RESULT: PASS|FAIL (<n> failed)`. Exit 0 pass, 1 fail, 2 bad argument or watchdog.

const SECRET := "sk-ant-PROBE-NOTREAL-0123456789wxyz"
const ENV_VAR := "CART_LLM_PROBE_KEYVAR"

var _fails := 0
var _checks := 0
var _server: LlmMockServer
var _run: LlmRun
var _tokens: PackedStringArray = PackedStringArray()
var _done: Array = []
var _failed: Array = []
var _usage: Array = []
var _retrying: Array = []
var _cancelled := false


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _until(cond: Callable, ms: int = 8000) -> bool:
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < ms:
		if cond.call():
			return true
		await get_tree().process_frame
	return bool(cond.call())


func _anthropic(port: int, over: Dictionary = {}) -> Dictionary:
	var s := {
		"enabled": true, "prose": true, "commands": false, "edit": false,
		"local_allowed": true, "remote_allowed": false,
		"preset_id": "custom_anthropic", "base_url": "http://127.0.0.1:%d/v1" % port,
		"model": "mock-model", "key_source": "env", "env_var": ENV_VAR, "token_cap": 0,
	}
	s.merge(over, true)
	return s


func _consent(port: int) -> PackedStringArray:
	return PackedStringArray(["http://127.0.0.1:%d" % port])


func _args() -> Dictionary:
	return {"task": "prose", "request": "Describe the port.", "entity_label": "Veldmark",
		"entity_fields": "population: 4200", "max_tokens": 128, "include_usage": true}


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
	_retrying = []
	_cancelled = false
	_run.token.connect(func(t: String): _tokens.append(t))
	_run.finished.connect(func(i: Dictionary): _done.append(i))
	_run.failed.connect(func(m: String): _failed.append(m))
	_run.usage.connect(func(i: Dictionary): _usage.append(i))
	_run.retrying.connect(func(r: String): _retrying.append(r))
	_run.cancelled.connect(func(): _cancelled = true)


## Runs one request to the end (done or failed or refused); returns the begin() dictionary.
func _go(settings: Dictionary, port: int) -> Dictionary:
	await _reset_run()
	var res: Dictionary = _run.begin(settings, "prose", _consent(port), _args())
	if bool(res.get("ok", false)):
		await _until(func(): return (not _done.is_empty() or not _failed.is_empty() or _cancelled) and not _run.busy(), 12000)
	return res


func _finish() -> void:
	OS.unset_environment(ENV_VAR)
	LlmWire.shutdown_all()
	if _server != null:
		_server.stop()
	print("RESULT: %s (%d failed of %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _ready() -> void:
	if not OS.get_cmdline_user_args().is_empty():
		print("ABORT: this probe takes no arguments")
		get_tree().quit(2)
		return
	get_tree().create_timer(240.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	OS.set_environment(ENV_VAR, SECRET)
	_server = LlmMockServer.new()
	add_child(_server)
	_ok(_server.start(), "the mock server listens on 127.0.0.1:%d" % _server.port)
	await _reset_run()
	_ok(_run.available(), "the loaded library has LlmStream")
	if not _run.available():
		_finish()
		return
	var port := _server.port
	var stream: Object = _run.stream
	stream.reset_token_counts()
	var c0: Dictionary = stream.token_counts()
	_ok(not c0.has("prompt") and not c0.has("completion") and int(c0["calls"]) == 0, "a fresh counter omits both totals (never 0): %s" % str(c0))
	_ok(int(c0["max_retries"]) == 1, "the retry bound the library reports is 1")

	# ------------------------------------------------------------------ 1. Anthropic wire
	print("== 1. Anthropic Messages over the real wire ==")
	_server.scenario = "anthropic"
	_server.token_delay_ms = 5
	_server.tokens = ["The ", "river ", "runs ", "south."]
	var res := await _go(_anthropic(port), port)
	_ok(bool(res.get("ok", false)) and String(res.get("style", "")) == "anthropic", "the request starts on the anthropic style")
	_ok(not bool(res.get("hosted", true)), "...and is marked not hosted (it is 127.0.0.1)")
	_ok(_failed.is_empty(), "no failure: %s" % str(_failed))
	_ok("".join(_tokens) == "The river runs south.", "typed events decode to the exact text: '%s'" % "".join(_tokens))
	_ok(_done.size() == 1 and String(_done[0].get("finish_reason", "")) == "stop", "one finish; end_turn is reported as 'stop'")
	_ok(_usage.size() == 1 and int(_usage[0].get("prompt", -1)) == 30 and int(_usage[0].get("completion", -1)) == 4, "ONE usage event: 30 in, cumulative 4 out (not 1+4): %s" % str(_usage))
	var rq: Dictionary = _server.requests[0]
	_ok(rq["path"] == "/v1/messages", "POST path is /v1/messages: %s" % rq["path"])
	_ok(String(rq["headers"].get("x-api-key", "")) == SECRET, "the server received x-api-key carrying the key")
	_ok(String(rq["headers"].get("anthropic-version", "")) == "2023-06-01", "anthropic-version: 2023-06-01")
	_ok(not rq["headers"].has("authorization"), "no Authorization header on the Anthropic wire")
	var body = JSON.parse_string(String(rq["body"]))
	_ok(body is Dictionary and int(body.get("max_tokens", 0)) == 128 and body.get("stream") == true, "body carries max_tokens 128 and stream true")
	_ok(body is Dictionary and typeof(body.get("system")) == TYPE_STRING and not String(body.get("system")).is_empty(), "the system prompt is the top-level 'system'")
	var roles := []
	for m in body.get("messages", []):
		roles.append(m["role"])
	_ok(roles == ["user"], "messages holds only the user turn: %s" % str(roles))
	_ok(not String(rq["body"]).contains(SECRET), "the body holds no key")
	var c1: Dictionary = stream.token_counts()
	_ok(int(c1.get("prompt", -1)) == 30 and int(c1.get("completion", -1)) == 4 and int(c1["calls"]) == 1 and int(c1["calls_reported"]) == 1, "the counter holds it: %s" % str(c1))

	# ------------------------------------------------------------------ 2. no usage
	print("== 2. a stream with no usage ==")
	_server.scenario = "anthropic_no_usage"
	res = await _go(_anthropic(port), port)
	_ok(_failed.is_empty() and "".join(_tokens) == "The river runs south." and _done.size() == 1, "the reply still completes")
	_ok(_usage.is_empty(), "no usage event was produced")
	var c2: Dictionary = stream.token_counts()
	_ok(int(c2.get("prompt", -1)) == 30 and int(c2.get("completion", -1)) == 4, "the totals did not move: %s" % str(c2))
	_ok(int(c2["calls"]) == 2 and int(c2["calls_reported"]) == 1, "calls 2, reported 1 (the unreported reply is visible, not hidden): %s" % str(c2))

	# ------------------------------------------------------------------ 3. hosted shape, prepare only
	print("== 3. hosted request shape (prepared, never sent) ==")
	var att0 := LlmWire.connect_attempts
	var hosted_openai := {"enabled": true, "prose": true, "local_allowed": false, "remote_allowed": true,
		"preset_id": "openai", "base_url": "https://api.openai.com/v1", "model": "gpt-test",
		"key_source": "env", "env_var": ENV_VAR, "token_cap": 0}
	var p1: Dictionary = stream.prepare(hosted_openai, "prose", PackedStringArray(["https://api.openai.com"]), _args())
	_ok(bool(p1.get("ok", false)) and bool(p1["tls"]) and String(p1["host"]) == "api.openai.com" and int(p1["port"]) == 443, "hosted OpenAI: tls, api.openai.com, 443")
	_ok(String(p1.get("path", "")) == "/v1/chat/completions" and bool(p1.get("hosted", false)), "...path /v1/chat/completions, marked hosted")
	var h1: PackedStringArray = p1.get("headers", PackedStringArray())
	_ok(h1.has("Authorization: Bearer " + SECRET), "...Authorization: Bearer <key>")
	var b1 = JSON.parse_string(PackedByteArray(p1.get("body", PackedByteArray())).get_string_from_utf8())
	_ok(b1 is Dictionary and int(b1.get("max_completion_tokens", 0)) == 128 and not b1.has("max_tokens"), "...max_completion_tokens (OpenAI's current name), not max_tokens")
	_ok(b1 is Dictionary and b1.has("stream_options"), "...usage requested for OpenAI")
	stream.reset()
	var hosted_anth := hosted_openai.duplicate()
	hosted_anth.merge({"preset_id": "anthropic", "base_url": "https://api.anthropic.com/v1"}, true)
	var p2: Dictionary = stream.prepare(hosted_anth, "prose", PackedStringArray(["https://api.anthropic.com"]), _args())
	_ok(bool(p2.get("ok", false)) and bool(p2["tls"]) and String(p2["host"]) == "api.anthropic.com" and int(p2["port"]) == 443, "hosted Anthropic: tls, api.anthropic.com, 443")
	_ok(String(p2.get("path", "")) == "/v1/messages" and String(p2.get("style", "")) == "anthropic", "...path /v1/messages")
	var h2: PackedStringArray = p2.get("headers", PackedStringArray())
	_ok(h2.has("x-api-key: " + SECRET) and h2.has("anthropic-version: 2023-06-01"), "...x-api-key and anthropic-version")
	stream.reset()
	var hosted_router := hosted_openai.duplicate()
	hosted_router.merge({"preset_id": "openrouter", "base_url": "https://openrouter.ai/api/v1"}, true)
	var p3: Dictionary = stream.prepare(hosted_router, "prose", PackedStringArray(["https://openrouter.ai"]), _args())
	var b3 = JSON.parse_string(PackedByteArray(p3.get("body", PackedByteArray())).get_string_from_utf8())
	_ok(bool(p3.get("ok", false)) and b3 is Dictionary and int(b3.get("max_tokens", 0)) == 128 and not b3.has("stream_options"), "OpenRouter: max_tokens, no stream_options (an unverified field is not sent)")
	stream.reset()
	_ok(LlmWire.connect_attempts == att0, "preparing hosted requests opened NO connection (attempts %d -> %d)" % [att0, LlmWire.connect_attempts])
	var refusals := [
		["plain http to a remote host", hosted_anth.merged({"base_url": "http://api.anthropic.com/v1"}, true), PackedStringArray(["http://api.anthropic.com"]), "cleartext_remote"],
		["remote switch off", hosted_anth.merged({"remote_allowed": false}, true), PackedStringArray(["https://api.anthropic.com"]), "remote_not_allowed"],
		["no consent", hosted_anth, PackedStringArray(), "consent_needed"],
		["consent for another origin", hosted_anth, PackedStringArray(["https://api.openai.com"]), "consent_needed"],
		["feature off", hosted_anth.merged({"enabled": false}, true), PackedStringArray(["https://api.anthropic.com"]), "feature_off"],
	]
	for r in refusals:
		var rr: Dictionary = stream.prepare(r[1], "prose", r[2], _args())
		_ok(not bool(rr.get("ok", false)) and String(rr.get("code", "")) == r[3] and not rr.has("host") and not rr.has("headers"), "hosted refused: %s -> %s, no connection data" % [r[0], String(rr.get("code", "?"))])
	OS.unset_environment(ENV_VAR)
	var nokey: Dictionary = stream.prepare(hosted_anth, "prose", PackedStringArray(["https://api.anthropic.com"]), _args())
	_ok(not bool(nokey.get("ok", false)) and String(nokey.get("code", "")) == "key_env_missing", "hosted with no key and no store: refused with a reason, code key_env_missing")
	_ok(not JSON.stringify(nokey).contains(SECRET), "...and the refusal holds no key")
	OS.set_environment(ENV_VAR, SECRET)
	_ok(LlmWire.connect_attempts == att0, "no refusal opened a connection")

	# ------------------------------------------------------------------ 4. no cap by default
	print("== 4. no enforced cap by default ==")
	stream.reset_token_counts()
	stream.debug_record_usage(4611686018427387904, 4611686018427387904)
	_server.scenario = "anthropic"
	_server.token_delay_ms = 0
	_server.tokens = ["ok"]
	var acc0 := _server.accepted
	var sent_ok := 0
	for i in 10:
		res = await _go(_anthropic(port), port)
		if bool(res.get("ok", false)) and _done.size() == 1 and _failed.is_empty():
			sent_ok += 1
	_ok(sent_ok == 10, "ten sequential requests past a counter near 2^63 ALL went out and finished (%d of 10)" % sent_ok)
	_ok(_server.accepted - acc0 == 10, "the server accepted ten connections (%d)" % (_server.accepted - acc0))
	var capped_try: Dictionary = await _go(_anthropic(port, {"token_cap": 1000000}), port)
	_ok(not bool(capped_try.get("ok", false)) and String(capped_try.get("code", "")) == "token_cap_reached", "with a cap set the same counter is REFUSED: code %s" % String(capped_try.get("code", "")))
	_ok(String(capped_try.get("reason", "")).contains("cap"), "...in words naming the cap: '%s'" % String(capped_try.get("reason", "")).left(80))
	_ok(_server.accepted - acc0 == 10, "...and opened no connection")
	var cc: Dictionary = stream.cap_check(1000000)
	_ok(not bool(cc.get("ok", true)) and String(cc.get("code", "")) == "token_cap_reached", "cap_check(1000000) agrees")
	_ok(bool(stream.cap_check(0).get("ok", false)) and bool(stream.cap_check(-5).get("ok", false)), "cap_check(0) and a negative cap mean OFF")
	# A cap just above what is used does NOT refuse; one at it does.
	stream.reset_token_counts()
	stream.debug_record_usage(60, 40)
	_ok(bool(stream.cap_check(101).get("ok", false)) and not bool(stream.cap_check(100).get("ok", true)), "boundary: used 100 passes a cap of 101 and meets a cap of 100")

	# ------------------------------------------------------------------ 5. retry
	print("== 5. the one automatic retry ==")
	_server.tokens = ["The ", "river ", "runs ", "south."]
	_server.token_delay_ms = 5
	stream.reset_token_counts()
	var n0 := _server.requests.size()
	_server.fail_next = 1
	res = await _go(_anthropic(port), port)
	_ok(_server.requests.size() - n0 == 2, "one 529 then success: the server saw TWO requests (%d)" % (_server.requests.size() - n0))
	_ok(_retrying.size() == 1 and String(_retrying[0]).contains("529"), "'retrying' fired once, naming the failure: %s" % str(_retrying))
	_ok(_failed.is_empty() and "".join(_tokens) == "The river runs south." and _done.size() == 1, "...and the reply completed, with no failure shown")
	_ok(int(stream.token_counts()["calls"]) == 2, "...the retry is counted as its own call (2)")
	n0 = _server.requests.size()
	_server.fail_next = 5
	res = await _go(_anthropic(port), port)
	_ok(_server.requests.size() - n0 == 2, "an endless 529: EXACTLY two requests, never a third (%d)" % (_server.requests.size() - n0))
	_ok(_retrying.size() == 1 and _failed.size() == 1 and String(_failed[0]).contains("529"), "...then one failure naming 529: %s" % str(_failed))
	_server.fail_next = 0
	n0 = _server.requests.size()
	_server.fail_next = 1
	_server.fail_status = 401
	res = await _go(_anthropic(port), port)
	_ok(_server.requests.size() - n0 == 1 and _retrying.is_empty() and _failed.size() == 1, "a 401 is NOT retried: one request, one failure")
	_server.fail_status = 529
	_server.fail_next = 0
	# A new user-initiated request after an exhausted retry gets a fresh budget.
	n0 = _server.requests.size()
	_server.fail_next = 1
	res = await _go(_anthropic(port), port)
	_ok(_server.requests.size() - n0 == 2 and _failed.is_empty(), "a NEW request after an exhausted one gets its own single retry")
	# Stop during the pause.
	await _reset_run()
	n0 = _server.requests.size()
	_server.fail_next = 3
	res = _run.begin(_anthropic(port), "prose", _consent(port), _args())
	_ok(await _until(func(): return not _retrying.is_empty(), 6000), "the retry pause began")
	_run.cancel()
	_ok(await _until(func(): return _cancelled and not _run.busy(), 4000), "Stop during the pause ends the reply as cancelled")
	await _frames(70)
	_ok(_server.requests.size() - n0 == 1, "...and the retry never went out (%d request)" % (_server.requests.size() - n0))
	_ok(_failed.is_empty() and LlmWire.live_threads == 0, "...with no failure shown and no thread left")
	_server.fail_next = 0

	# A refused connection never reached a server, so it is NOT retried: one attempt, one failure.
	var dead := TCPServer.new()
	dead.listen(0, "127.0.0.1")
	var dead_port := dead.get_local_port()
	dead.stop()
	var att1 := LlmWire.connect_attempts
	res = await _go(_anthropic(dead_port), dead_port)
	_ok(bool(res.get("ok", false)) and _failed.size() == 1 and _retrying.is_empty(), "a refused connection is reported once and not retried: %s" % str(_failed).left(100))
	_ok(LlmWire.connect_attempts - att1 == 1, "...exactly one connection attempt (%d)" % (LlmWire.connect_attempts - att1))

	# ------------------------------------------------------------------ 6. reset
	print("== 6. reset ==")
	stream.reset_token_counts()
	var c6: Dictionary = stream.token_counts()
	_ok(not c6.has("prompt") and not c6.has("completion") and int(c6["calls"]) == 0, "reset: totals omitted again, calls 0: %s" % str(c6))
	_ok(not JSON.stringify(stream.token_counts()).contains(SECRET), "the counter dictionary holds no key")
	print("CHECKS %d" % _checks)
	_finish()
