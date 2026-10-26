extends Node
## Probe for the model dropdown in `Preferences ▸ Assistant…` (`shell/llm_settings_window.gd`,
## `shell/llm_models.gd`, the Rust `LlmModels` and `cartalith-llm::models`) -- the owner's
## 2026-10-26 request -- inside the real shell, against an in-process HTTP server.
##
##   desktop: godot --path . --resolution 1600x1000 _llmmodels_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1280x800  _llmmodels_probe.tscn -- --force-touch --shots <dir>
##   phone:   godot --path . --resolution 393x852   _llmmodels_probe.tscn -- --force-touch --shots <dir>
##
## **Windowed, never `--headless`**; run from a scratch project copy with its own `config/name`
## and a wiped user dir (`MISTAKES.md`). Arguments read: `--force-touch`, `--shots <dir>`; anything
## else aborts with exit 2. The timeout section (about 20 s: the 10 s first-byte timeout, then the
## one retry) runs on the desktop pass only; the Rust path it exercises is density-independent.
## It sets up every `[llm]` key it asserts on and clears them at the end.
##
## What it protects, each against the real shell and a real socket:
##   1. GATE OFF = ZERO CONNECTIONS: master switch off, no consent, local off, no provider -- opening
##      the window and pressing Refresh dial nothing (`LlmWire.connect_attempts` and the server's
##      accept count unchanged) and each refusal's reason is drawn;
##   2. LM Studio's native list: `GET /api/v0/models`, loaded model first with its mark and context,
##      the embedding model hidden and counted, no key header when there is no key;
##   3. fallback: native route 404 -> `/v1/models`, two requests in all;
##   4. an empty list says so and shows the typed field; 401 is final (one request) and the key does
##      not appear in any label; a redirect is not followed; an HTML 200 is "not a model list";
##   5. hostile list: 1 200 entries, control characters in names, duplicates -> at most 500 clean rows;
##   6. server down: "could not reach"; the typed field is shown; nothing hangs;
##   7. timeout (desktop): "did not answer in time" after exactly 2 requests (the one retry);
##   8. the last list is remembered in memory with its age and shown, marked stale, when a refresh fails;
##   9. picking a row persists the model and the chat request (`LlmStream.prepare`) carries it; a saved
##      model absent from the list is kept and marked "(not in list)"; "Type a name…" always works;
##  10. refresh triggers: window open, address commit and the Refresh button each ask once; nothing
##      asks on a timer (request count flat over a 3 s idle);
##  11. layout per density: the dropdown and Refresh meet the touch floor, the window body does not
##      overflow, Close stays on screen.
## Result: `RESULT: PASS|FAIL (<n> failed)`. Exit 0 pass, 1 fail, 2 bad argument or watchdog.

const Mock := preload("res://_llm_models_mock.gd")
const SECRET := "sk-PROBE-MODELS-0123456789abcd"
const ENV_VAR := "CART_LLM_MODELS_PROBE_KEY"

var _fails := 0
var _mode := "desktop"
var _shots := ""
var _app: Node
var _bridge
var _win
var _mock


func _ok(cond: bool, what: String) -> void:
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _c(name: String) -> Control:
	return _win.controls.get(name) as Control


func _status() -> String:
	var c := _c("model_status")
	return (c as Label).text if c is Label else ""


func _shot(tag: String) -> void:
	if _shots == "":
		return
	if _c("model_pick") != null:
		(_win._scroll as ScrollContainer).ensure_control_visible(_c("model_status"))
	await _settle(3)
	var path := _shots.path_join("llmmodels_%s_%s.png" % [_mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


## Waits (frames) until the window's listing state leaves `loading`, at most `ms`.
func _wait_done(ms: int = 8000) -> void:
	var t0 := Time.get_ticks_msec()
	await _settle(2)
	while String(_win._ml_state) == "loading" and Time.get_ticks_msec() - t0 < ms:
		await get_tree().process_frame
	await _settle(3)


## The dropdown's visible row texts (the real control's items).
func _rows() -> Array:
	var ob := _c("model_pick") as OptionButton
	var out: Array = []
	for i in ob.item_count:
		out.append(ob.get_item_text(i))
	return out


func _model_rows() -> Array:
	## The rows that are models: not the placeholder, not "Type a name…".
	var out: Array = []
	for r in _rows():
		if r != "Choose a model…" and r != "Type a name…":
			out.append(r)
	return out


func _reset_llm() -> void:
	DccSettings._ensure_loaded()
	if DccSettings._cfg.has_section("llm"):
		DccSettings._cfg.erase_section("llm")
		DccSettings._save()
	LlmModelList.clear_cache()


## Stores the gate settings for the mock endpoint and consents to it.
func _configure(preset: String, extra: Dictionary = {}) -> void:
	_reset_llm()
	var base := "http://127.0.0.1:%d/v1" % _mock.port
	var s := {"enabled": true, "prose": true, "local_allowed": true, "preset_id": preset, "base_url": base,
		"model": "", "key_source": "env", "env_var": ENV_VAR}
	s.merge(extra, true)
	for k in s:
		DccSettings.set_llm_value(k, s[k])
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % _mock.port)


func _open() -> void:
	_win.open()
	await _wait_done()


func _lm_v0(loaded_first := false) -> String:
	## The shape recorded from LM Studio 0.4.26 on 2026-10-26, with one model marked loaded.
	var rows := [
		{"id": "qwen3.8-27b", "object": "model", "type": "vlm", "publisher": "unsloth", "arch": "qwen35", "compatibility_type": "gguf",
			"quantization": "Q3_K_XL", "state": "not-loaded", "max_context_length": 262144, "capabilities": ["tool_use"]},
		{"id": "qwen3-14b", "object": "model", "type": "llm", "publisher": "Qwen", "arch": "qwen3", "compatibility_type": "gguf",
			"quantization": "Q4_K_M", "state": "loaded", "max_context_length": 40960, "capabilities": ["tool_use"]},
		{"id": "text-embedding-nomic-embed-text-v1.5", "object": "model", "type": "embeddings", "publisher": "nomic-ai", "arch": "nomic-bert",
			"compatibility_type": "gguf", "quantization": "Q4_K_M", "state": "not-loaded", "max_context_length": 2048},
	]
	return JSON.stringify({"object": "list", "data": rows})


func _openai_list(ids: Array) -> String:
	var rows: Array = []
	for i in ids:
		rows.append({"id": i, "object": "model", "owned_by": "organization_owner"})
	return JSON.stringify({"object": "list", "data": rows})


func _all_label_text(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
	elif n is OptionButton:
		for i in (n as OptionButton).item_count:
			out.append((n as OptionButton).get_item_text(i))
	for ch in n.get_children(true):
		_all_label_text(ch, out)


func _finish() -> void:
	OS.unset_environment(ENV_VAR)
	_reset_llm()
	if _mock != null:
		_mock.stop()
	print("RESULT: %s (%d failed)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := args[i]
		if a == "--force-touch":
			pass
		elif a == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		else:
			print("ABORT: unrecognised argument %s -- this probe reads --force-touch and --shots <dir> only" % a)
			get_tree().quit(2)
			return
		i += 1
	get_tree().create_timer(300.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	_mock = Mock.new()
	add_child(_mock)
	_ok(_mock.start(), "the in-process server listens on 127.0.0.1:%d" % _mock.port)

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_mode = "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s, viewport %s ==" % [_mode, get_viewport().get_visible_rect().size])
	_bridge = _app.bridge
	_win = _app.assistant_settings_window
	_ok(_bridge.llm_api and _win != null, "the loaded library has LlmBridge and the window exists")
	_ok(ClassDB.class_exists("LlmModels"), "the loaded library has LlmModels (a stale DLL fails here)")
	if not _bridge.llm_api or _win == null or not ClassDB.class_exists("LlmModels"):
		_finish()
		return
	OS.unset_environment(ENV_VAR)
	_mock.routes = {"/api/v0/models": {"status": 200, "body": _lm_v0()}, "/v1/models": {"status": 200, "body": _openai_list(["a", "b"])}}

	# ---------------------------------------------------------------- 1. gate off = zero connections
	print("== 1. the gate refuses: nothing is dialled ==")
	var refusals := [
		["feature off", {"enabled": false}, "assistant is off"],
		["local providers off", {"local_allowed": false}, "Local providers allowed"],
		["no provider", {"preset_id": "", "base_url": ""}, "No provider"],
	]
	for r in refusals:
		_configure("lm_studio")
		var extra: Dictionary = r[1]
		for k in extra:
			DccSettings.set_llm_value(k, extra[k])
		var attempts0 := LlmWire.connect_attempts
		var acc0: int = _mock.accepted
		_win.open()
		await _settle(8)
		(_c("model_refresh") as Button).pressed.emit()
		await _settle(8)
		_ok(String(_status()).contains(String(r[2])), "%s: the status says why (%s)" % [r[0], _status().left(90)])
		_ok(LlmWire.connect_attempts == attempts0 and _mock.accepted == acc0 and _mock.requests.is_empty(),
			"%s: zero connection attempts (wire %d -> %d, server accepted %d)" % [r[0], attempts0, LlmWire.connect_attempts, _mock.accepted])
		_ok(_c("model") != null and _c("model").is_visible_in_tree(), "%s: the typed name field is shown, so a name can always be entered" % r[0])
	_configure("lm_studio")
	DccSettings.llm_withdraw_consent()
	var a1 := LlmWire.connect_attempts
	_win.open()
	await _settle(8)
	_ok(_status().contains("Consent needed") and LlmWire.connect_attempts == a1 and _mock.requests.is_empty(),
		"no consent for the origin: refused with 'Consent needed', nothing dialled (%s)" % _status().left(80))
	await _shot("refused")

	# ---------------------------------------------------------------- 2. LM Studio native list
	print("== 2. LM Studio native list ==")
	_configure("lm_studio")
	await _open()
	_ok(_mock.requests.size() == 1 and _mock.requests[0]["method"] == "GET" and _mock.requests[0]["path"] == "/api/v0/models",
		"one GET /api/v0/models (%s)" % str(_mock.requests.map(func(r): return r["method"] + " " + r["path"])))
	_ok(not _mock.requests[0]["headers"].has("authorization") and not _mock.requests[0]["headers"].has("x-api-key"), "no key set: no credential header")
	var rows := _model_rows()
	_ok(rows.size() == 2, "2 chat models listed, the embedding model hidden: %s" % str(rows))
	_ok(rows.size() == 2 and String(rows[0]).begins_with("● qwen3-14b") and String(rows[0]).contains("loaded") and String(rows[0]).contains("40K ctx"),
		"the loaded model is first, marked, with its context: %s" % (rows[0] if rows.size() > 0 else "-"))
	_ok(rows.size() == 2 and String(rows[1]) == "qwen3.8-27b · 256K ctx", "the not-loaded model shows its context only: %s" % (rows[1] if rows.size() > 1 else "-"))
	_ok(_status().contains("2 models") and _status().contains("1 embedding model hidden") and _status().contains("just now"),
		"status: count, hidden embedding, age: %s" % _status())
	_ok(not _c("model").is_visible_in_tree(), "with a list, the typed field is hidden until asked for")
	await _shot("list_ok")
	var ob := _c("model_pick") as OptionButton
	if _shots != "":
		ob.show_popup()
		await _settle(4)
		await _shot("list_popup")
		ob.get_popup().hide()
		await _settle(2)

	# ---------------------------------------------------------------- 9. pick persists, chat carries it
	print("== 9. picking a model ==")
	var idx := -1
	for k in ob.item_count:
		if ob.get_item_text(k).contains("qwen3.8-27b"):
			idx = k
	ob.select(idx)
	ob.item_selected.emit(idx)
	await _settle(6)
	_ok(String(DccSettings.llm_value("model")) == "qwen3.8-27b", "the pick is persisted: '%s'" % DccSettings.llm_value("model"))
	var stream = ClassDB.instantiate("LlmStream")
	var prep: Dictionary = stream.prepare(DccSettings.llm_gate_dict(), "prose", DccSettings.llm_consented_origins(),
		{"task": "prose", "request": "Describe the port.", "entity_label": "Veldmark", "entity_fields": "population: 4200"})
	var body_model := ""
	if bool(prep.get("ok", false)):
		var parsed = JSON.parse_string((prep["body"] as PackedByteArray).get_string_from_utf8())
		body_model = String(parsed.get("model", "")) if parsed is Dictionary else ""
	_ok(bool(prep.get("ok", false)) and String(prep.get("model", "")) == "qwen3.8-27b" and body_model == "qwen3.8-27b",
		"the chat request carries the picked model (prepare.model '%s', body.model '%s')" % [prep.get("model", ""), body_model])
	_win.open()
	await _wait_done()
	_ok(_c("model_pick") is OptionButton and (_c("model_pick") as OptionButton).get_item_text((_c("model_pick") as OptionButton).selected).begins_with("qwen3.8-27b"),
		"reopening shows the saved model selected in the list")
	DccSettings.set_llm_value("model", "ghost-model")
	_win._models_render()
	await _settle(3)
	ob = _c("model_pick") as OptionButton
	_ok(ob.get_item_text(ob.selected) == "ghost-model (not in list)", "a saved model absent from the list is kept and marked: '%s'" % ob.get_item_text(ob.selected))
	var ti := ob.item_count - 1
	_ok(ob.get_item_text(ti) == "Type a name…", "the last row is 'Type a name…'")
	ob.select(ti)
	ob.item_selected.emit(ti)
	await _settle(4)
	_ok(_c("model").is_visible_in_tree(), "'Type a name…' shows the typed field")
	var le := _c("model") as LineEdit
	le.text = "my-own-model"
	le.text_submitted.emit("my-own-model")
	await _settle(4)
	_ok(String(DccSettings.llm_value("model")) == "my-own-model", "a typed name is stored: '%s'" % DccSettings.llm_value("model"))
	await _shot("typed")
	_win._ml_manual = false

	# ---------------------------------------------------------------- 3. fallback
	print("== 3. a native route that 404s falls back to /v1/models ==")
	_mock.requests.clear()
	_mock.routes["/api/v0/models"] = {"status": 404, "body": "{}"}
	_configure("lm_studio")
	await _open()
	_ok(_mock.count("/api/v0/models") == 1 and _mock.count("/v1/models") == 1 and _mock.requests.size() == 2,
		"exactly two requests: native then standard (%s)" % str(_mock.requests.map(func(r): return r["path"])))
	_ok(_model_rows() == ["a", "b"], "the standard list is shown: %s" % str(_model_rows()))
	_mock.routes["/api/v0/models"] = {"status": 200, "body": _lm_v0()}

	# ---------------------------------------------------------------- 4. empty, 401, redirect, html
	print("== 4. empty, 401, redirect, not-a-list ==")
	_mock.routes["/v1/models"] = {"status": 200, "body": _openai_list([])}
	_configure("custom_openai")
	_mock.requests.clear()
	await _open()
	_ok(_status().contains("lists no chat models") and _model_rows().is_empty() and _c("model").is_visible_in_tree(), "empty list: says so and shows the typed field (%s)" % _status().left(70))
	_mock.routes["/v1/models"] = {"status": 401, "body": "{}", "echo_auth": true}
	OS.set_environment(ENV_VAR, SECRET)
	_mock.requests.clear()
	await _open()
	_ok(_status().begins_with("Could not list:") and _status().contains("401") and _status().contains("key"), "401: names the wrong key: %s" % _status())
	_ok(_mock.count("/v1/models") == 1, "401 is final: one request, no retry (%d)" % _mock.count("/v1/models"))
	_ok(String(_mock.requests[0]["headers"].get("authorization", "")) == "Bearer " + SECRET, "the key travelled in the Authorization header")
	var texts: Array = []
	_all_label_text(_win, texts)
	_ok(not " ".join(texts).contains(SECRET), "the key appears in no label, button or row of the window")
	await _shot("unauthorized")
	_mock.routes["/v1/models"] = {"status": 302, "body": "", "location": "http://127.0.0.1:1/elsewhere"}
	_mock.requests.clear()
	await _open()
	_ok(_status().contains("redirect") and _mock.requests.size() == 1, "a redirect is reported, not followed (%d request)" % _mock.requests.size())
	_mock.routes["/v1/models"] = {"status": 200, "body": "<!doctype html><html><body>Sign in</body></html>", "ctype": "text/html"}
	await _open()
	_ok(_status().contains("did not answer with a model list"), "an HTML page with status 200: %s" % _status().left(80))
	OS.unset_environment(ENV_VAR)

	# ---------------------------------------------------------------- 5. hostile
	print("== 5. a hostile list ==")
	var items: Array = []
	for n in 1200:
		items.append({"id": "model-%d" % n, "name": "Name %d" % n})
	items.append({"id": "bad\nid"})
	items.append({"id": "model-3", "name": "dup"})
	items.append({"id": "esc\u001b[31m"})
	items[0]["name"] = "Evil\u0007\u202e name\n"
	## Godot's JSON.stringify writes some control characters raw; a strict parser rightly refuses
	## the whole document then (asserted below), so the sanitising test escapes them by hand.
	var raw_json := JSON.stringify({"data": items})
	var escaped_json := raw_json.replace(char(7), "\\u0007").replace(char(27), "\\u001b")
	_mock.routes["/v1/models"] = {"status": 200, "body": raw_json}
	_configure("custom_openai")
	await _open()
	_ok(_status().contains("did not answer with a model list") and _model_rows().is_empty(),
		"raw control characters make the document invalid JSON: refused whole, nothing half-read (%s)" % _status().left(60))
	_mock.routes["/v1/models"] = {"status": 200, "body": escaped_json}
	_configure("custom_openai")
	await _open()
	var hrows := _model_rows()
	_ok(hrows.size() == 500, "capped at exactly 500 rows (%d)" % hrows.size())
	var dirty := false
	for t in _rows():
		for ch in String(t):
			if ch.unicode_at(0) < 0x20 or ch.unicode_at(0) == 0x202E:
				dirty = true
	_ok(not dirty, "no control or bidi character reaches a row")
	_ok(_status().contains("Only the first 500"), "the cut is said: %s" % _status())
	await _shot("hostile")

	# ---------------------------------------------------------------- 8. cache and age
	print("== 8. the last list is remembered, and shown stale when a refresh fails ==")
	_mock.routes["/v1/models"] = {"status": 200, "body": _openai_list(["alpha", "beta"])}
	_configure("custom_openai")
	await _open()
	_ok(_model_rows() == ["alpha", "beta"], "a clean list first: %s" % str(_model_rows()))
	_mock.routes["/v1/models"] = {"status": 401, "body": "{}"}
	(_c("model_refresh") as Button).pressed.emit()
	await _wait_done()
	_ok(_status().begins_with("Could not list:") and _status().contains("Showing the list from") and _model_rows() == ["alpha", "beta"],
		"a failed refresh keeps the last list, marked: %s" % _status().left(120))
	await _shot("stale")

	# ---------------------------------------------------------------- 10. triggers, no polling
	print("== 10. what asks, and what never does ==")
	_mock.routes["/v1/models"] = {"status": 200, "body": _openai_list(["alpha", "beta"])}
	_configure("custom_openai")
	_mock.requests.clear()
	await _open()
	_ok(_mock.requests.size() == 1, "opening the window asks once (%d)" % _mock.requests.size())
	await get_tree().create_timer(3.0).timeout
	_ok(_mock.requests.size() == 1, "3 s idle: still one request, nothing polls in the background (%d)" % _mock.requests.size())
	(_c("model_refresh") as Button).pressed.emit()
	await _wait_done()
	_ok(_mock.requests.size() == 2, "Refresh asks once more (%d)" % _mock.requests.size())
	_win._on_url_commit("http://127.0.0.1:%d/v1/" % _mock.port)
	await _wait_done()
	_ok(_mock.requests.size() == 3, "committing a changed address asks again (%d)" % _mock.requests.size())
	_win._on_url_commit("http://127.0.0.1:%d/v1/" % _mock.port)
	await _settle(6)
	_ok(_mock.requests.size() == 3, "committing the SAME address asks nothing (%d)" % _mock.requests.size())
	OS.set_environment(ENV_VAR, SECRET)
	_win._on_env_commit(ENV_VAR + "_X")
	await _wait_done()
	_ok(_mock.requests.size() == 4, "changing the key's variable name asks again (%d)" % _mock.requests.size())
	_win._on_env_commit(ENV_VAR)
	await _wait_done()
	OS.unset_environment(ENV_VAR)

	# ---------------------------------------------------------------- 12. `localhost` reaches a 127.0.0.1-only server
	print("== 12. the name localhost ==")
	## Godot's HTTPClient resolves `localhost` to ::1 and does not fall back, so before `llm_wire.gd`
	## tried 127.0.0.1 first this timed out against LM Studio, Ollama and llama.cpp (their default
	## binding). The mock listens on 127.0.0.1 only, like they do: the probe's other sections use the
	## literal address, which is why a mock alone never showed the defect (found by the live check).
	_mock.routes["/v1/models"] = {"status": 200, "body": _openai_list(["via-localhost"])}
	_configure("custom_openai", {"base_url": "http://localhost:%d/v1" % _mock.port})
	DccSettings.llm_add_consent("http://localhost:%d" % _mock.port)
	var tl0 := Time.get_ticks_msec()
	await _open()
	_ok(_model_rows() == ["via-localhost"], "a list over http://localhost:<port> arrives: %s (%s)" % [str(_model_rows()), _status().left(70)])
	_ok(Time.get_ticks_msec() - tl0 < 3000, "and quickly (%d ms), not after a connect timeout" % (Time.get_ticks_msec() - tl0))

	# ---------------------------------------------------------------- 11. layout
	print("== 11. layout at this density ==")
	_configure("lm_studio")
	await _open()
	var vp := get_viewport().get_visible_rect()
	var floor_px := 44.0 if _mode == "phone" else (float(DccTheme.role_px("btn_min_h")) if _mode == "tablet" else 0.0)
	var pick := _c("model_pick") as Control
	var rb := _c("model_refresh") as Control
	if floor_px > 0.0:
		_ok(pick.size.y >= floor_px - 0.5 and rb.size.y >= floor_px - 0.5, "dropdown %.0f and Refresh %.0f px high, floor %.0f" % [pick.size.y, rb.size.y, floor_px])
	var scroll := _win._scroll as ScrollContainer
	_ok((_win._body as Control).get_combined_minimum_size().x <= scroll.size.x + 1.0, "the body's minimum width %.0f fits the scroller %.0f" % [(_win._body as Control).get_combined_minimum_size().x, scroll.size.x])
	scroll.ensure_control_visible(rb)
	await _settle(3)
	var rr := Rect2(rb.global_position, rb.size)
	_ok(vp.encloses(rr) or Rect2(Vector2.ZERO, vp.size).encloses(rr), "Refresh is inside the viewport after scrolling to it: %s in %s" % [rr, vp])
	var okb: Button = _win.get_ok_button()
	var orr := Rect2(okb.global_position, okb.size)
	_ok(okb.is_visible_in_tree() and Rect2(Vector2.ZERO, vp.size).encloses(orr), "Close stays on screen: %s" % orr)
	await _shot("layout")

	# ---------------------------------------------------------------- 6. server down
	print("== 6. the server is down ==")
	_configure("lm_studio")
	var dead_port: int = _mock.port
	_mock.stop()
	LlmModelList.clear_cache()
	var down_a := LlmWire.connect_attempts
	await _open()
	_ok(_status().begins_with("Could not list:") and _status().contains("Is it running"), "a closed port: %s" % _status().left(100))
	_ok(LlmWire.connect_attempts - down_a == 1, "a refused connection is not retried (%d attempts)" % (LlmWire.connect_attempts - down_a))
	_ok(_c("model").is_visible_in_tree(), "the typed field is shown")
	await _shot("down")
	_ok(dead_port > 0, "(the dead port was %d)" % dead_port)

	# ---------------------------------------------------------------- 7. timeout (desktop)
	if _mode == "desktop":
		print("== 7. the server accepts and never answers ==")
		_mock = Mock.new()
		add_child(_mock)
		_ok(_mock.start(), "a second server listens")
		_mock.routes = {"/v1/models": {"hang": true}}
		_configure("custom_openai")
		var t0 := Time.get_ticks_msec()
		_win.open()
		await _wait_done(40000)
		var dt := Time.get_ticks_msec() - t0
		_ok(_status().contains("in time"), "timeout reason: %s" % _status().left(80))
		_ok(_mock.count("/v1/models") == 2, "exactly one retry: %d requests" % _mock.count("/v1/models"))
		_ok(dt > 15000 and dt < 30000, "took %d ms (two 10 s waits)" % dt)
	else:
		print("  note  the timeout section runs on the desktop pass only")
	_finish()
