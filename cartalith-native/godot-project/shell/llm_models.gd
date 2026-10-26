extends Node
class_name LlmModelList

## One model listing for the Assistant settings dropdown, and the in-memory cache of the last
## list per endpoint (the owner's 2026-10-26 request; `cartalith-llm`'s `models.rs` and the Rust
## class `LlmModels` decide everything, this file moves bytes and keeps time).
##
## ## What this does and never does
##
## * **Gated by construction.** `refresh` calls `LlmModels.begin`, which runs the chat gate minus
##   the model name (master switch, local or remote switch, https rule, key, per-origin consent).
##   A refusal returns a reason and NO connection data, so there is nothing to start the wire with:
##   with the assistant off, no thread, no `HTTPClient`, no socket (`LlmWire.connect_attempts` is
##   unchanged; `_llmmodels_probe.gd` asserts it).
## * **Rides the chat wire.** `LlmWire` (a worker thread polling `HTTPClient`) with `method: GET`
##   and the short timeouts `begin` names. One extra request, never a loop: the single retry and the
##   fallback from a native source to `/models` are `ModelListJob`'s, in Rust.
## * **Never polls in the background.** A refresh runs only when `refresh` is called (the window
##   opening, the endpoint, key or a switch changing, or the Refresh button). `_process` only drains
##   an in-flight reply and stops itself when there is none.
## * **The key.** It exists only in the request dictionary Rust returns, which is handed to the wire
##   and then dropped. It is never stored on this node, printed, cached or put in a signal.
## * **The cache is memory only.** `last_for` returns the last good list for an endpoint with its
##   age; nothing is written to `cartalith_settings.cfg` or anywhere else.
##
## `finished(result)`: `result` is Rust's last step (`{ok: true, step: "done", models, hidden_embeddings,
## skipped, truncated}` or `{ok: false, step: "failed", code, reason, status?}`), plus `key` (the cache
## key) and `origin`.

## Emitted once per started refresh, unless it was cancelled or replaced.
signal finished(result: Dictionary)

## `cache key -> {models: Array, hidden: int, truncated: bool, skipped: int, unix: float}`. Static so
## it outlives the window, and memory only. Capped at 16 endpoints (oldest dropped).
static var _cache: Dictionary = {}
const _CACHE_MAX := 16

var _rust: Object
var _wire: LlmWire
var _buf := PackedByteArray()
var _status := 0
var _max_body := 2 * 1024 * 1024
var _key := ""
var _origin := ""
var _active := false


## The cache key for an endpoint: provider id plus base URL (the model list belongs to the server).
static func key_for(settings: Dictionary) -> String:
	return "%s|%s" % [String(settings.get("preset_id", "")), String(settings.get("base_url", "")).strip_edges()]


## The last good list for `key`: `{models, hidden, truncated, skipped, age_s}` or `{}`.
static func last_for(key: String) -> Dictionary:
	if not _cache.has(key):
		return {}
	var c: Dictionary = (_cache[key] as Dictionary).duplicate()
	c["age_s"] = maxf(0.0, Time.get_unix_time_from_system() - float(c["unix"]))
	return c


## Forgets every cached list (the probe's reset; no UI calls it).
static func clear_cache() -> void:
	_cache.clear()


## True while a request is in flight.
func is_active() -> bool:
	return _active


## Starts a listing for `settings` (`DccSettings.llm_gate_dict()`), or says why not. Returns
## Rust's `begin` dictionary with the connection data and headers removed: `{ok: false, code, reason}`
## when refused (nothing was opened), `{ok: true, origin}` when a request is in flight.
func refresh(settings: Dictionary, consented: PackedStringArray) -> Dictionary:
	cancel()
	if not ClassDB.class_exists("LlmModels"):
		return {"ok": false, "code": "no_binding", "reason": "The assistant is not available in this version of Cartalith."}
	_rust = ClassDB.instantiate("LlmModels")
	_max_body = int(_rust.max_body_bytes())
	_key = key_for(settings)
	var r: Dictionary = _rust.begin(settings, consented)
	if not bool(r.get("ok", false)):
		_rust.cancel()
		_rust = null
		return r
	_origin = String(r.get("origin", ""))
	if not _launch(r):
		_rust.cancel()
		_rust = null
		return {"ok": false, "code": "wire", "reason": "The connection could not be started."}
	_active = true
	set_process(true)
	return {"ok": true, "origin": _origin}


## Drops an in-flight listing. No signal. Joins the worker, which notices within a few ms.
func cancel() -> void:
	if _wire != null:
		_wire.shutdown()
		_wire = null
	if _rust != null:
		_rust.cancel()
		_rust = null
	_active = false
	_buf = PackedByteArray()
	set_process(false)


func _exit_tree() -> void:
	cancel()


func _process(_delta: float) -> void:
	pump()


## Starts the wire for one request dictionary. The request (it holds the credential) is not kept.
func _launch(req: Dictionary) -> bool:
	if _wire != null:
		_wire.shutdown()
	_wire = LlmWire.new()
	_buf = PackedByteArray()
	_status = 0
	return _wire.start(req)


## Drains the wire once and feeds Rust. Public so a probe can drive it without a frame loop.
func pump() -> void:
	if not _active or _wire == null:
		return
	for item in _wire.drain():
		match String(item.get("kind", "")):
			"head":
				_status = int(item["status"])
			"chunk":
				_buf.append_array(item["bytes"])
				if _buf.size() > _max_body:
					## Hand Rust the oversize body: it answers `too_large`. Stop reading.
					_wire.cancel()
					_handle(_rust.on_response(_status, _buf))
					return
			"end":
				_handle(_rust.on_response(_status, _buf))
				return
			"error":
				var kind := "other"
				if bool(item.get("connect", false)):
					kind = "connect"
				elif bool(item.get("timeout", false)):
					kind = "timeout"
				_handle(_rust.on_fault(kind, String(item.get("text", ""))))
				return
			"cancelled":
				return


## Acts on Rust's next step: send again (the retry or the fallback) or finish.
func _handle(step: Dictionary) -> void:
	if String(step.get("step", "")) == "send":
		if not _launch(step):
			_finish({"ok": false, "step": "failed", "code": "wire", "reason": "The connection could not be started."})
		return
	_finish(step)


func _finish(step: Dictionary) -> void:
	if _wire != null:
		_wire.shutdown()
		_wire = null
	if _rust != null:
		_rust.cancel()
		_rust = null
	_active = false
	set_process(false)
	var out := step.duplicate()
	out["key"] = _key
	out["origin"] = _origin
	if bool(step.get("ok", false)) and String(step.get("step", "")) == "done":
		_remember(_key, step)
	finished.emit(out)


## Keeps the list in memory (oldest endpoint dropped past the cap).
static func _remember(key: String, step: Dictionary) -> void:
	_cache[key] = {"models": step.get("models", []), "hidden": int(step.get("hidden_embeddings", 0)),
		"truncated": bool(step.get("truncated", false)), "skipped": int(step.get("skipped", 0)),
		"unix": Time.get_unix_time_from_system()}
	while _cache.size() > _CACHE_MAX:
		var oldest := ""
		var t := INF
		for k in _cache:
			if float(_cache[k]["unix"]) < t:
				t = float(_cache[k]["unix"])
				oldest = String(k)
		_cache.erase(oldest)


## `262144 -> "256K"`, `40960 -> "40K"`, `1000000 -> "1M"`, `8192 -> "8K"`; `""` for none (a
## context length the server did not send is not shown, never "0K").
static func context_text(n: int) -> String:
	if n <= 0:
		return ""
	if n % 1024 == 0 and n < 1024 * 1024:
		return "%dK" % (n / 1024)
	if n >= 1000000:
		var m := float(n) / 1000000.0
		return ("%dM" % int(m)) if is_equal_approx(m, floorf(m)) else ("%.1fM" % m)
	return "%dK" % int(roundf(float(n) / 1000.0))


## The dropdown row text for one model entry: loaded mark, label, context.
static func item_text(m: Dictionary) -> String:
	var id := String(m["id"])
	var shown := String(m.get("display_name", ""))
	var label := id if shown == "" or shown == id else "%s (%s)" % [shown, id]
	var parts: Array = []
	if bool(m.get("loaded", false)):
		label = "● " + label
		parts.append("loaded")
	var ctx := context_text(int(m.get("context_length", 0)))
	if ctx != "":
		parts.append(ctx + " ctx")
	return label if parts.is_empty() else label + " · " + " · ".join(parts)
