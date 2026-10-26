extends Node
## A tiny HTTP/1.1 server on 127.0.0.1 that answers the model-list requests of
## `_llmmodels_probe.gd`. **Test support only**: nothing in `shell/` references it, and it is
## not a `class_name` (the probe `preload`s it).
##
## WHY its own server rather than `_llm_mock_server.gd`: that one speaks chat SSE for the LLM-3
## probes and is shared with them; listing needs plain Content-Length replies, per-path routes, a
## reply that never comes (the timeout bar) and a recorded method per request, none of which that
## file has, and a shared file edited for one probe is how another probe breaks.
##
## `routes`: `path -> {status: int, body: String, ctype: String, hang: bool, echo_auth: bool,
## location: String}`. The query string is ignored when matching. An unrouted path answers 404.
## Recorded: `accepted` (connections), `requests` (`{method, path, headers}`, headers lower-cased).

var port := 0
var routes: Dictionary = {}
var accepted := 0
var requests: Array = []

var _server := TCPServer.new()
var _conns: Array = []


func start() -> bool:
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


## Counts requests whose path (without the query) equals `path`.
func count(path: String) -> int:
	var n := 0
	for r in requests:
		if String(r["path"]).split("?")[0] == path:
			n += 1
	return n


func pump() -> void:
	while _server.is_connection_available():
		var p := _server.take_connection()
		accepted += 1
		_conns.append({"peer": p, "buf": PackedByteArray(), "hung": false})
	var keep: Array = []
	for c in _conns:
		if _service(c):
			keep.append(c)
		else:
			(c["peer"] as StreamPeerTCP).disconnect_from_host()
	_conns = keep


func _service(c: Dictionary) -> bool:
	var p: StreamPeerTCP = c["peer"]
	p.poll()
	if p.get_status() != StreamPeerTCP.STATUS_CONNECTED:
		return false
	if c["hung"]:
		return true
	## A reply already written: hold the socket open briefly, then close. Closing in the same
	## frame as the write can reset the connection under a client that has not read yet.
	if c.has("done_at"):
		return Time.get_ticks_msec() - int(c["done_at"]) < 300
	var n := p.get_available_bytes()
	if n > 0:
		var got := p.get_data(n)
		if got[0] == OK:
			var buf: PackedByteArray = c["buf"]
			buf.append_array(got[1])
			c["buf"] = buf
	var text := (c["buf"] as PackedByteArray).get_string_from_utf8()
	var head_end := text.find("\r\n\r\n")
	if head_end < 0:
		return true
	var lines := text.substr(0, head_end).split("\r\n")
	var req := String(lines[0]).split(" ")
	var headers := {}
	for i in range(1, lines.size()):
		var kv := String(lines[i]).split(":", true, 1)
		if kv.size() == 2:
			headers[kv[0].strip_edges().to_lower()] = kv[1].strip_edges()
	var path := req[1] if req.size() > 1 else ""
	requests.append({"method": req[0], "path": path, "headers": headers})
	var route: Dictionary = routes.get(path.split("?")[0], {"status": 404, "body": "{}"})
	if bool(route.get("hang", false)):
		c["hung"] = true
		return true
	var body := String(route.get("body", ""))
	if bool(route.get("echo_auth", false)):
		## The realistic key leak: an error body that repeats the Authorization header.
		body = JSON.stringify({"error": {"message": "Incorrect API key provided: %s" % String(headers.get("authorization", headers.get("x-api-key", "(none)")))}})
	var b := body.to_utf8_buffer()
	var extra := ""
	if String(route.get("location", "")) != "":
		extra = "Location: %s\r\n" % String(route["location"])
	p.put_data(("HTTP/1.1 %d X\r\nContent-Type: %s\r\n%sContent-Length: %d\r\nConnection: close\r\n\r\n" % [
			int(route.get("status", 200)), String(route.get("ctype", "application/json")), extra, b.size()]).to_utf8_buffer())
	p.put_data(b)
	c["done_at"] = Time.get_ticks_msec()
	return true
