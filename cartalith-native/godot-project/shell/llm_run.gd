extends Node
class_name LlmRun

## One assistant reply from request to last token, on the MAIN thread
## (`LLM_ASSISTANT_SCOPE.md` LLM-3). It joins the two halves of the hybrid transport:
## Rust's `LlmStream` (gate, request, parser) and `LlmWire` (a worker thread polling
## `HTTPClient`). The wire queues raw bytes; this node drains the queue every frame and
## feeds the parser, so each token reaches `token` within one frame of arriving.
##
## ## Rules this node keeps
##
## * **Gated by construction.** `begin` calls `LlmStream.prepare`, which runs the LLM-2
##   `authorize()` and returns connection data only when it passes. The wire is started
##   with that result and nothing else. A refusal returns the reason and starts nothing: no
##   thread, no `HTTPClient`, no socket (the probe asserts `LlmWire.connect_attempts`).
## * **Never writes anywhere.** It produces text and signals. Saving to a note is the panel's
##   click handler; this node has no Vault access.
## * **Cancel stops the stream and closes the socket.** `cancel()` silences the parser first
##   (no token after Stop, whatever the server still sends), then asks the wire to close;
##   `cancelled` fires once the thread has exited.
## * **No thread outlives the node.** `_exit_tree` and `shutdown()` join the worker.
##
## `error_text` and `failed` messages come from the Rust side already scrubbed of the key.

## One text fragment, as it was decoded.
signal token(text: String)
## Token counts, with only the fields the server reported.
signal usage(info: Dictionary)
## The reply ended normally. `info` may carry `finish_reason`.
signal finished(info: Dictionary)
## The reply failed; `message` is plain words with no key in it.
signal failed(message: String)
## Cancel completed: the socket is closed and the thread joined.
signal cancelled()
## The reply is over AND its worker thread is joined: `busy()` is now false. `finished` and
## `failed` fire the moment the last event is decoded, while the thread is still closing its
## socket, so a UI that enables "Save" must wait for this, not for `finished`.
signal idle()

## The `LlmStream` Rust object (null on a build without it).
var stream: Object
## The worker wrapper.
var wire := LlmWire.new()
## Everything streamed so far.
var text_so_far := ""
## Milliseconds from `begin` to the first token; -1 until one arrives. (An internal
## sentinel for the probe, never displayed as a number when -1.)
var first_token_ms := -1
## Token counts as last reported (keys present only when the server sent them).
var last_usage: Dictionary = {}
## The model name the request used, for the note marker.
var model := ""

var _t0 := 0
var _active := false
var _cancelling := false
var _terminal_sent := false


func _init() -> void:
	set_process(false)
	if ClassDB.class_exists("LlmStream"):
		stream = ClassDB.instantiate("LlmStream")


## True when this build has the Rust half.
func available() -> bool:
	return stream != null


## True from a successful `begin` until the worker is joined.
func busy() -> bool:
	return _active


## Starts a reply. Returns `LlmStream.prepare`'s dictionary: on `ok == false` nothing was
## started and `reason` says why. `settings`, `capability`, `consented` and `args` are as
## `LlmStream.prepare` documents.
func begin(settings: Dictionary, capability: String, consented: PackedStringArray, args: Dictionary) -> Dictionary:
	if stream == null:
		return {"ok": false, "code": "no_binding", "reason": "This build has no assistant support. Update the Cartalith library."}
	if _active:
		return {"ok": false, "code": "busy", "reason": "A reply is already running. Stop it first."}
	var prep: Dictionary = stream.prepare(settings, capability, consented, args)
	if not bool(prep.get("ok", false)):
		return prep
	text_so_far = ""
	first_token_ms = -1
	last_usage = {}
	model = String(prep.get("model", ""))
	_cancelling = false
	_terminal_sent = false
	if not wire.start(prep):
		stream.reset()
		return {"ok": false, "code": "wire", "reason": "The connection could not be started."}
	_active = true
	_t0 = Time.get_ticks_msec()
	set_process(true)
	# The headers carry the credential; the caller gets a copy without them.
	var out := prep.duplicate()
	out.erase("headers")
	out.erase("body")
	return out


## Stops the reply. No token is delivered after this returns; `cancelled` fires when the
## socket is closed and the thread is gone. A no-op when idle.
func cancel() -> void:
	if not _active or _cancelling:
		return
	_cancelling = true
	stream.cancel()
	wire.cancel()


## Cancels and blocks until the worker is joined. For close paths.
func shutdown() -> void:
	if stream != null:
		stream.cancel()
	wire.shutdown()
	_active = false
	set_process(false)


func _exit_tree() -> void:
	shutdown()


func _process(_delta: float) -> void:
	pump()


## Drains the wire once. Public so a headless probe can drive it without a frame loop.
func pump() -> void:
	if not _active:
		return
	for item in wire.drain():
		match String(item.get("kind", "")):
			"head":
				stream.on_head(int(item["status"]))
			"chunk":
				_dispatch(stream.on_chunk(item["bytes"]))
			"end":
				_dispatch(stream.on_end())
			"error":
				_dispatch(stream.on_transport_error(String(item["text"])))
			"cancelled":
				pass
	if not wire.is_started():
		_active = false
		set_process(false)
		if _cancelling:
			cancelled.emit()
		idle.emit()


func _dispatch(events: Array) -> void:
	for ev in events:
		match String(ev.get("type", "")):
			"text":
				if first_token_ms < 0:
					first_token_ms = Time.get_ticks_msec() - _t0
				text_so_far += String(ev["text"])
				token.emit(String(ev["text"]))
			"usage":
				last_usage = ev.duplicate()
				last_usage.erase("type")
				usage.emit(last_usage)
			"done":
				_terminal(func(): finished.emit(_without_type(ev)))
			"error":
				_terminal(func(): failed.emit(String(ev.get("message", "The reply failed."))))
			_:
				pass


func _without_type(ev: Dictionary) -> Dictionary:
	var d := ev.duplicate()
	d.erase("type")
	return d


## A terminal event: tell the UI once, then ask the wire to close the socket (the server may
## keep the connection open after its last event).
func _terminal(notify: Callable) -> void:
	if _terminal_sent:
		return
	_terminal_sent = true
	wire.cancel()
	notify.call()
