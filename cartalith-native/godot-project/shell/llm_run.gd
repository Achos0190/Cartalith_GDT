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
##
## ## Setting proposals (LLM-6a/6b)
##
## With capability `commands` one user request is SEVERAL HTTP requests: Rust replaces the
## reply's `done` with a step. `params_continue` / `params_retry` mean "the model was answered
## (a schema, or the errors of a refused proposal); ask it again": this node waits for the
## worker to be joined, then calls `LlmStream.prepare_continue`, which asks the WHOLE gate again
## and rebuilds the catalogue from the registry and a fresh settings snapshot (`params_source`),
## and starts the wire with that. `proposal` is the validated card data, emitted once, and
## nothing here acts on it: the panel's Apply button is the only writer. `params_answer` and
## `params_failed` end the request in words.

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
## A transient failure before any text is being retried ONCE (LLM-4). `reason` is the
## failed attempt's message, already free of the key. `failed` is not emitted for it.
signal retrying(reason: String)
## The reply is over AND its worker thread is joined: `busy()` is now false. `finished` and
## `failed` fire the moment the last event is decoded, while the thread is still closing its
## socket, so a UI that enables "Save" must wait for this, not for `finished`.
signal idle()
## A setting request moved on (`info.type` is `params_continue` or `params_retry`; a retry
## carries the refused `verdict`). The next request is already on its way.
signal params_step(info: Dictionary)
## The validated proposal (Rust's `verdict_dict`). DATA: nothing is applied by this signal.
signal proposal(info: Dictionary)
## The model answered a setting request in words, without proposing (`text`).
signal answered(text: String)
## A tool call the decoder accepted (`name`, `arguments` as JSON TEXT), for transcripts only
## (the stage-2 driver `_llmparams_cli.gd`). Nothing acts on it: the card comes from `proposal`.
signal tool_called(info: Dictionary)

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

## The arguments of the user-initiated request, kept so the one automatic retry can run
## `prepare` again (and so pass the whole gate a second time).
var _last: Dictionary = {}
## Set when an error carried `will_retry`; consumed once the worker is joined.
var _retry_pending := false
var _retry_reason := ""
## Set when Rust answered a setting request's turn and wants the next one; consumed once the
## worker is joined (`_continue_after_join`).
var _continue_pending := false
## Returns the CURRENT settings snapshot (`get_params()`) for the next turn's catalogue; when
## invalid, the snapshot passed to `begin` is reused.
var params_source: Callable

## Pause before the one automatic retry. Labelled judgement: long enough for a
## momentary 529 or 429 to clear, short enough to feel like part of the same request.
## The server's `Retry-After` header is not honoured (the wire passes the status only).
const RETRY_DELAY_S := 1.0

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
	_last = {"settings": settings, "capability": capability, "consented": consented, "args": args}
	_retry_pending = false
	_continue_pending = false
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
	_retry_pending = false
	_continue_pending = false
	stream.cancel()
	wire.cancel()


## Cancels and blocks until the worker is joined. For close paths.
func shutdown() -> void:
	_retry_pending = false
	_continue_pending = false
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
				## A connect-phase failure never reached a server, so it is never retried.
				if bool(item.get("connect", false)):
					_dispatch(stream.on_connect_error(String(item["text"])))
				else:
					_dispatch(stream.on_transport_error(String(item["text"])))
			"cancelled":
				pass
	if not wire.is_started():
		if _continue_pending and not _cancelling:
			_continue_pending = false
			_continue_after_join()
			return
		if _retry_pending and not _cancelling:
			_retry_pending = false
			set_process(false)
			_retry_after_pause()
			return
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
			"tool_call":
				tool_called.emit(_without_type(ev))
			"params_continue", "params_retry":
				## Rust answered the model's tool call(s); the next turn starts once this
				## socket is closed and the worker joined.
				if not _cancelling:
					_continue_pending = true
					wire.cancel()
					params_step.emit(ev.duplicate())
			"proposal":
				var info: Dictionary = ev.duplicate()
				_terminal(func(): proposal.emit(info); finished.emit({}))
			"params_answer":
				var said := String(ev.get("text", ""))
				_terminal(func(): answered.emit(said); finished.emit({}))
			"params_failed":
				_terminal(func(): failed.emit(String(ev.get("reason", "The setting request failed."))))
			"error":
				if bool(ev.get("will_retry", false)) and not _cancelling:
					## Rust granted the one retry (transient, nothing shown yet): hold the
					## failure, close this socket, and let `pump` start the second attempt
					## once the worker is joined.
					_retry_pending = true
					_retry_reason = String(ev.get("message", "The reply failed."))
					wire.cancel()
				else:
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


## The next turn of a setting request (LLM-6a/6b), once the previous socket is closed: the
## gate is asked again (`prepare_continue`), with a fresh settings snapshot.
func _continue_after_join() -> void:
	var args := {"current": params_source.call() if params_source.is_valid() else Dictionary(_last["args"]).get("current", {})}
	var prep: Dictionary = stream.prepare_continue(_last["settings"], _last["consented"], args)
	if not bool(prep.get("ok", false)):
		_active = false
		set_process(false)
		failed.emit(String(prep.get("reason", "The setting request could not continue.")))
		idle.emit()
		return
	_terminal_sent = false
	if not wire.start(prep):
		stream.reset()
		_active = false
		set_process(false)
		failed.emit("The connection could not be restarted.")
		idle.emit()
		return
	_t0 = Time.get_ticks_msec()


## The one automatic retry (LLM-4; scope s.4 rule 7). Runs only after Rust marked the error
## `will_retry`: a transient provider or transport failure, before any text was shown, within
## the budget `cartalith_llm::retry` owns. It waits `RETRY_DELAY_S`, then calls `prepare`
## AGAIN, so the whole gate (switches, https, key, consent, the opt-in cap) is asked a second
## time: nothing here goes round it. Stop during the wait ends the reply as cancelled.
func _retry_after_pause() -> void:
	retrying.emit(_retry_reason)
	await get_tree().create_timer(RETRY_DELAY_S).timeout
	if not _active:
		return ## `shutdown()` ran during the wait.
	if _cancelling:
		_active = false
		cancelled.emit()
		idle.emit()
		return
	var prep: Dictionary = stream.prepare(_last["settings"], _last["capability"], _last["consented"], _last["args"])
	if not bool(prep.get("ok", false)):
		_active = false
		failed.emit(String(prep.get("reason", _retry_reason)))
		idle.emit()
		return
	_terminal_sent = false
	if not wire.start(prep):
		stream.reset()
		_active = false
		failed.emit("The connection could not be restarted.")
		idle.emit()
		return
	_t0 = Time.get_ticks_msec()
	set_process(true)
