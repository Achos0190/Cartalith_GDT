extends RefCounted
class_name LlmWire

## The Godot half of the assistant's hybrid transport (`LLM_ASSISTANT_SCOPE.md` LLM-3;
## Ruling DD Q4): one worker `Thread` polling one `HTTPClient`, and nothing else.
##
## ## What this does and never does
##
## * **Moves bytes, decides nothing.** It is started with the dictionary `LlmStream.prepare`
##   returned (host, port, tls, path, headers, body) and has no way to build one: a refused
##   gate returns no connection data, so there is nothing to start it with. `start` refuses
##   a dictionary without `ok == true` and a host, as a second line behind the gate.
## * **Never touches a Rust object.** The crate is built without gdext's
##   `experimental-threads`, so a Dictionary/Array/String operation on a worker thread panics
##   (see `cartalith-godot`'s `landmark_run` note). The thread only calls `HTTPClient`, reads
##   a `Mutex`-guarded cancel flag and appends plain items to a `Mutex`-guarded queue;
##   `drain()` is called on the MAIN thread, which feeds the Rust parser.
## * **Cancel closes the socket.** `cancel()` sets the flag; the thread notices within one
##   poll (a few ms), calls `HTTPClient.close()` and returns. `shutdown()` joins it. A wire
##   with a live thread is kept alive by the static `_live` registry until it is joined, and
##   `shutdown_all()` (called by the shell on exit) joins every one, so no thread outlives the
##   library (the `lib.rs` unload rule; `live_threads` is the test's witness).
## * **Holds the credential only in the Thread's argument.** The headers (which carry
##   `Authorization` when a key is set) are bound into the thread callable and dropped when
##   the thread exits. They are never stored on a property, printed or put in a queue item.
## * **No retry, no redirect following, no proxy.** One request, one reply.
##
## Queue items (plain dictionaries; bytes only for the body):
## `{"kind":"head","status":int}`, `{"kind":"chunk","bytes":PackedByteArray}`,
## `{"kind":"end"}`, `{"kind":"error","text":String}`, `{"kind":"cancelled"}`.
## **Listing models.** The same wire carries a `GET` for the settings dropdown: the target
## dictionary (`LlmModels.begin`) may carry `"method": "GET"` (default `POST`) and the short
## timeouts `"connect_ms"` and `"first_byte_ms"` (defaults: the chat constants below). A chat
## target carries none of them, so chat is unchanged. An `error` caused by a timeout also carries
## `"timeout": true`, which the listing maps to its "did not answer in time" reason.
## An `error` raised before the connection was established also carries `"connect": true`
## (LLM-4): the server was never reached, so the one automatic retry is not offered for it --
## a closed port or an unreachable host will fail the same way a second later, and a local
## server that is not running should say so at once.

## Sockets this process has asked `HTTPClient` to open through this class. The "zero
## outbound connections when the feature is off" probe asserts it is unchanged by a refused
## request. Static so it spans instances.
static var connect_attempts := 0
## Worker threads started and not yet joined. Must be 0 whenever no reply is in flight.
static var live_threads := 0
## Every wire whose thread has been started and not joined. **Why a registry:** a `Callable`
## bound to a `RefCounted` holds no reference, so a wire dropped mid-stream would be freed under
## its own running thread, and `NOTIFICATION_PREDELETE` cannot be relied on to join it (calling
## a script method there fails with "null instance", measured 2026-10-25). Holding each live wire
## here keeps it alive until it is joined; `shutdown_all()` is the unload hook.
static var _live: Array = []

## Milliseconds the thread sleeps when a poll produced nothing. Small enough that a
## cancel is noticed in a few ms, large enough not to spin a core. Labelled judgement.
const IDLE_MS := 2
## Longest the wire waits for the connection to be established. Labelled judgement: a
## local server answers in milliseconds; 8 s covers a model server that is mid-start.
const CONNECT_TIMEOUT_MS := 8000
## Longest the wire waits for the first byte of the reply after the request is sent. A
## local model may be loading weights, which takes tens of seconds, so this is generous;
## the user can always press Stop.
const FIRST_BYTE_TIMEOUT_MS := 120000
## Longest silence mid-stream before the wire gives up. Labelled judgement.
const STALL_TIMEOUT_MS := 60000

var _thread: Thread
var _mutex := Mutex.new()
var _queue: Array = []
var _cancel := false
var _started := false


## Starts the worker for `target` (a successful `LlmStream.prepare` result). Returns false,
## starting nothing, for anything else or if a reply is already in flight.
func start(target: Dictionary) -> bool:
	if _started or not bool(target.get("ok", false)) or String(target.get("host", "")) == "":
		return false
	_started = true
	_live.append(self)
	_cancel = false
	## Counted here, on the main thread, so the static is never written from the worker.
	connect_attempts += 1
	_queue.clear()
	_thread = Thread.new()
	live_threads += 1
	var headers: PackedStringArray = target.get("headers", PackedStringArray())
	## A listing is a GET with short timeouts (`LlmModels.begin`); a chat target has neither key.
	var method := HTTPClient.METHOD_GET if String(target.get("method", "POST")) == "GET" else HTTPClient.METHOD_POST
	var err := _thread.start(_run.bind(String(target["host"]), int(target["port"]),
			bool(target.get("tls", false)), String(target.get("path", "/")), headers,
			target.get("body", PackedByteArray()), method,
			int(target.get("connect_ms", CONNECT_TIMEOUT_MS)), int(target.get("first_byte_ms", FIRST_BYTE_TIMEOUT_MS))))
	if err != OK:
		live_threads -= 1
		_started = false
		_thread = null
		_live.erase(self)
		return false
	return true


## Asks the worker to stop. It closes the socket and exits within a few milliseconds.
func cancel() -> void:
	_mutex.lock()
	_cancel = true
	_mutex.unlock()


## True while the worker thread is running.
func is_running() -> bool:
	return _thread != null and _thread.is_alive()


## True from `start` until the thread has been joined.
func is_started() -> bool:
	return _started


## Takes every queued item (main thread only). When the worker has exited and its queue is
## empty, joins it, so `is_started()` becomes false and `live_threads` drops.
func drain() -> Array:
	_mutex.lock()
	var out := _queue
	_queue = []
	_mutex.unlock()
	if _started and _thread != null and not _thread.is_alive():
		_mutex.lock()
		var more := _queue
		_queue = []
		_mutex.unlock()
		out.append_array(more)
		_join()
	return out


## Cancels and joins EVERY live wire, blocking until all threads are gone. The library-unload
## hook: `app.gd` calls it when the shell exits, and the probes call it in place of an unload.
static func shutdown_all() -> void:
	for w in _live.duplicate():
		(w as LlmWire).shutdown()


## Cancels and joins, blocking until the thread is gone. Safe to call twice and when idle.
func shutdown() -> void:
	cancel()
	_join()


func _join() -> void:
	if _thread != null and _thread.is_started():
		_thread.wait_to_finish()
	if _started:
		live_threads -= 1
	_started = false
	_thread = null
	_live.erase(self)


func _cancelled() -> bool:
	_mutex.lock()
	var c := _cancel
	_mutex.unlock()
	return c


func _push(item: Dictionary) -> void:
	_mutex.lock()
	_queue.append(item)
	_mutex.unlock()


## One connection attempt to `h`: `"ok"`, `"start"` (could not even begin), `"timeout"`, `"cancelled"` or
## `"fail"` (refused or unreachable). Worker thread only; touches nothing but `client` and the cancel flag.
func _connect(client: HTTPClient, h: String, port: int, tls_opts: TLSOptions, connect_ms: int) -> String:
	client.close()
	if client.connect_to_host(h, port, tls_opts) != OK:
		return "start"
	var t0 := Time.get_ticks_msec()
	while client.get_status() == HTTPClient.STATUS_RESOLVING or client.get_status() == HTTPClient.STATUS_CONNECTING:
		if _cancelled():
			return "cancelled"
		client.poll()
		if Time.get_ticks_msec() - t0 > connect_ms:
			return "timeout"
		OS.delay_msec(IDLE_MS)
	return "ok" if client.get_status() == HTTPClient.STATUS_CONNECTED else "fail"


## The worker body. Runs off the main thread: only `HTTPClient`, `OS.delay_msec`, `Time` and
## this object's mutex-guarded members are touched.
func _run(host: String, port: int, tls: bool, path: String, headers: PackedStringArray,
		body: PackedByteArray, method: int = HTTPClient.METHOD_POST, connect_ms: int = CONNECT_TIMEOUT_MS,
		first_byte_ms: int = FIRST_BYTE_TIMEOUT_MS) -> void:
	var client := HTTPClient.new()
	var tls_opts: TLSOptions = TLSOptions.client() if tls else null
	## **`localhost` is tried as `127.0.0.1`, then `::1`.** Godot's `HTTPClient` resolves `localhost`
	## to `::1` and does not fall back to IPv4, so a server bound to 127.0.0.1 only (LM Studio, Ollama
	## and llama.cpp by default) never answered: measured 2026-10-26, `localhost:1234` timed out after
	## 20 s against a live LM Studio while `127.0.0.1:1234` answered in 64 ms. Every local preset's
	## default address is `localhost`, so the chat wire had the same defect; the gate and the consent
	## origin still see the name the user typed.
	var candidates: Array = ["127.0.0.1", "::1"] if host.to_lower() == "localhost" else [host]
	var outcome := ""
	for cand in candidates:
		outcome = _connect(client, String(cand), port, tls_opts, connect_ms)
		if outcome == "ok" or outcome == "cancelled":
			break
	if outcome == "cancelled":
		client.close()
		_push({"kind": "cancelled"})
		return
	if outcome == "start":
		_push({"kind": "error", "connect": true, "text": "Could not start a connection to %s:%d." % [host, port]})
		return
	if outcome == "timeout":
		client.close()
		_push({"kind": "error", "connect": true, "timeout": true, "text": "Timed out connecting to %s:%d. Is the server running?" % [host, port]})
		return
	if outcome != "ok":
		client.close()
		_push({"kind": "error", "connect": true, "text": "Could not connect to %s:%d. Is the server running?" % [host, port]})
		return
	var err := client.request_raw(method, path, headers, body)
	if err != OK:
		client.close()
		_push({"kind": "error", "text": "The request could not be sent."})
		return
	var last_data := Time.get_ticks_msec()
	var got_head := false
	while true:
		if _cancelled():
			client.close()
			_push({"kind": "cancelled"})
			return
		client.poll()
		var st := client.get_status()
		if st == HTTPClient.STATUS_REQUESTING:
			pass
		elif st == HTTPClient.STATUS_BODY or st == HTTPClient.STATUS_CONNECTED or st == HTTPClient.STATUS_DISCONNECTED:
			if not got_head and client.has_response():
				got_head = true
				_push({"kind": "head", "status": client.get_response_code()})
				last_data = Time.get_ticks_msec()
			if st == HTTPClient.STATUS_BODY:
				var chunk := client.read_response_body_chunk()
				if chunk.size() > 0:
					_push({"kind": "chunk", "bytes": chunk})
					last_data = Time.get_ticks_msec()
					continue
			else:
				# CONNECTED or DISCONNECTED after the head means the body is complete.
				if got_head:
					client.close()
					_push({"kind": "end"})
					return
				client.close()
				_push({"kind": "error", "text": "The server closed the connection without replying."})
				return
		elif st == HTTPClient.STATUS_CONNECTION_ERROR or st == HTTPClient.STATUS_TLS_HANDSHAKE_ERROR \
				or st == HTTPClient.STATUS_CANT_CONNECT or st == HTTPClient.STATUS_CANT_RESOLVE:
			client.close()
			_push({"kind": "error", "text": "The connection to the server failed or was lost."})
			return
		var waited := Time.get_ticks_msec() - last_data
		if (got_head and waited > STALL_TIMEOUT_MS) or (not got_head and waited > first_byte_ms):
			client.close()
			_push({"kind": "error", "timeout": true, "text": "The server stopped sending. Press Stop, or check that the model is loaded."})
			return
		OS.delay_msec(IDLE_MS)
