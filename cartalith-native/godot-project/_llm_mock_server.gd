extends Node
class_name LlmMockServer

## An in-process OpenAI-compatible SSE server on 127.0.0.1, for the LLM-3 probes
## (`_llmstream_probe.gd`, `_llmpanel_probe.gd`). **Test support only** -- nothing in `shell/`
## references it.
##
## WHY a server inside the probe rather than a Python process: the bars are about the real
## Godot `HTTPClient` and the real worker thread, so the server must speak real HTTP/1.1 over a
## real TCP socket, but it needs no second runtime and it can count and record what the client
## did (connections accepted, the request it sent, whether the client hung up). It is polled
## from the main thread; the client under test runs on its own worker thread, so the two never
## share a stack.
##
## Scenarios (set `scenario` before the request arrives; each request reads it once):
##   "stream"        chunked SSE of `tokens`, `token_delay_ms` apart, a finish chunk, a usage
##                   chunk, `[DONE]`, then the terminating chunk.
##   "stream_split"  the same bytes, but every SSE event is written in two TCP writes cut in the
##                   middle of its `data:` line (the split-chunk case on a real socket).
##   "endless"       tokens forever, `token_delay_ms` apart, until the client hangs up.
##   "unauthorized"  401 with a JSON error whose message ECHOES the Authorization header (the
##                   realistic key leak the scrubber exists for), Content-Length framed.
##   "no_done"       chunked SSE that ends with a finish_reason chunk and closes with no [DONE].
##
## Recorded: `accepted` (connections), `requests` (each: path, headers lower-cased, body text),
## `client_hangups` (connections the client closed before the server finished), `tokens_sent`.

var port := 0
var scenario := "stream"
var tokens: Array = ["The ", "port ", "of ", "Veldmark ", "began ", "as ", "a ", "ferry ", "post."]
var token_delay_ms := 15
var accepted := 0
var requests: Array = []
var client_hangups := 0
var tokens_sent := 0

var _server := TCPServer.new()
var _conns: Array = []


func start() -> bool:
	## Port 0 asks the OS for a free one; `get_local_port` reads it back.
	if _server.listen(0, "127.0.0.1") != OK:
		return false
	port = _server.get_local_port()
	return port > 0


func stop() -> void:
	for c in _conns:
		(c["peer"] as StreamPeerTCP).disconnect_from_host()
	_conns.clear()
	_server.stop()


func _process(_delta: float) -> void:
	pump()


## Accepts and services connections once; public so a loop can call it directly.
func pump() -> void:
	while _server.is_connection_available():
		var p := _server.take_connection()
		accepted += 1
		_conns.append({"peer": p, "buf": PackedByteArray(), "state": "reading", "t": 0, "i": 0,
				"scenario": scenario, "need": -1, "head_len": -1})
	var keep: Array = []
	for c in _conns:
		if _service(c):
			keep.append(c)
		else:
			(c["peer"] as StreamPeerTCP).disconnect_from_host()
	_conns = keep


## Services one connection; returns false when it is finished and may be dropped.
func _service(c: Dictionary) -> bool:
	var p: StreamPeerTCP = c["peer"]
	p.poll()
	if p.get_status() != StreamPeerTCP.STATUS_CONNECTED:
		if c["state"] == "streaming":
			client_hangups += 1
		return false
	if c["state"] == "reading":
		var n := p.get_available_bytes()
		if n > 0:
			var got := p.get_data(n)
			if got[0] == OK:
				## Re-store the array: indexing a dictionary hands back a copy, so appending to
				## `c["buf"]` in place would be lost (the mistake that stalled the first run).
				var buf: PackedByteArray = c["buf"]
				buf.append_array(got[1])
				c["buf"] = buf
		_try_parse(c)
		return true
	if c["state"] == "streaming":
		return _stream(c, p)
	return false


func _try_parse(c: Dictionary) -> void:
	var buf: PackedByteArray = c["buf"]
	var text := buf.get_string_from_utf8()
	var head_end := text.find("\r\n\r\n")
	if head_end < 0:
		return
	var head := text.substr(0, head_end)
	var lines := head.split("\r\n")
	var req := String(lines[0]).split(" ")
	var headers := {}
	for i in range(1, lines.size()):
		var kv := String(lines[i]).split(":", true, 1)
		if kv.size() == 2:
			headers[kv[0].strip_edges().to_lower()] = kv[1].strip_edges()
	var need := int(headers.get("content-length", "0"))
	var body_start := buf.slice(0, head_end).size() + 4
	if buf.size() - body_start < need:
		return
	var body := buf.slice(body_start, body_start + need).get_string_from_utf8()
	requests.append({"path": req[1] if req.size() > 1 else "", "headers": headers, "body": body})
	c["scenario"] = scenario
	c["t"] = Time.get_ticks_msec()
	c["i"] = 0
	c["state"] = "streaming"
	var p: StreamPeerTCP = c["peer"]
	if c["scenario"] == "unauthorized":
		var msg := "Incorrect API key provided: %s" % String(headers.get("authorization", "(none)"))
		var payload := JSON.stringify({"error": {"message": msg, "type": "invalid_request_error"}})
		var b := payload.to_utf8_buffer()
		p.put_data(("HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: %d\r\n\r\n" % b.size()).to_utf8_buffer())
		p.put_data(b)
		c["state"] = "done_soon"
		c["i"] = 1_000_000
		return
	p.put_data("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nTransfer-Encoding: chunked\r\n\r\n".to_utf8_buffer())


func _chunk(p: StreamPeerTCP, data: String) -> void:
	var b := data.to_utf8_buffer()
	p.put_data(("%x\r\n" % b.size()).to_utf8_buffer())
	p.put_data(b)
	p.put_data("\r\n".to_utf8_buffer())


func _delta_event(text: String) -> String:
	return "data: " + JSON.stringify({"choices": [{"index": 0, "delta": {"content": text}, "finish_reason": null}]}) + "\n\n"


## One step of a streaming reply. Returns false when the reply is complete.
func _stream(c: Dictionary, p: StreamPeerTCP) -> bool:
	var now := Time.get_ticks_msec()
	if now - int(c["t"]) < token_delay_ms:
		return true
	c["t"] = now
	var i: int = c["i"]
	var sc: String = c["scenario"]
	if i >= 1_000_000:
		return false
	if sc == "endless":
		_chunk(p, _delta_event(String(tokens[i % tokens.size()])))
		tokens_sent += 1
		c["i"] = i + 1
		return true
	if i < tokens.size():
		var ev := _delta_event(String(tokens[i]))
		if sc == "stream_split":
			var cut := ev.length() / 2
			_chunk(p, ev.substr(0, cut))
			_chunk(p, ev.substr(cut))
		else:
			_chunk(p, ev)
		tokens_sent += 1
		c["i"] = i + 1
		return true
	var fin := "data: " + JSON.stringify({"choices": [{"index": 0, "delta": {}, "finish_reason": "stop"}]}) + "\n\n"
	_chunk(p, fin)
	if sc != "no_done":
		_chunk(p, "data: " + JSON.stringify({"choices": [], "usage": {"prompt_tokens": 42, "completion_tokens": tokens.size(), "total_tokens": 42 + tokens.size()}}) + "\n\n")
		_chunk(p, "data: [DONE]\n\n")
	p.put_data("0\r\n\r\n".to_utf8_buffer())
	c["i"] = 1_000_000
	return true
