extends Node
## Probe for the Android "network access may be off" hint (`shell/llm_net_hint.gd`) in the real shell.
##
##   godot --path . --resolution 1600x1000 _nethint_probe.tscn   (also 1280x800 and 393x852, each with -- --force-touch)
##
## Windowed, never `--headless`; run from a scratch copy with its own `config/name` and a wiped user dir.
## Real Android cannot be reached from a desktop, so the platform check and the JNI call are replaced by
## `LlmNetHint.fake_android` / `fake_opener` (the seams exist for exactly this). What this protects:
##   1. platform gating: with fake_android=0 NO hint text and NO button anywhere, even for a connect failure;
##   2. transport-only: with fake_android=1 a refused connection (code unreachable) gets the exact hint
##      sentence; a 401 and a 500 (the server answered) get none, and the message text is otherwise unchanged;
##   3. the intent helper builds action android.settings.APPLICATION_DETAILS_SETTINGS and data package:<pkg>
##      and reports opened / unavailable; desktop returns unavailable and records nothing;
##   4. the settings window shows the Open app settings button on Android only, and the first-run note once;
##   5. the assistant window's failure card carries the hint plus a button for a connect failure only.
## Result: `RESULT: PASS|FAIL (<n> failed)`.

const Mock := preload("res://_llm_models_mock.gd")
const HINT_LITERAL := "Network access may be off for Cartalith. Turn it on: Settings > Apps > Cartalith > Wi-Fi data usage > Allow network access."
const ACTION_LITERAL := "android.settings.APPLICATION_DETAILS_SETTINGS"
const BTN := "Open app settings"

var _fails := 0
var _app: Node
var _win
var _mock
var _calls: Array = []


func _ok(cond: bool, what: String) -> void:
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _status() -> String:
	var c = _win.controls.get("model_status")
	return (c as Label).text if c is Label else ""


func _reset() -> void:
	DccSettings._ensure_loaded()
	if DccSettings._cfg.has_section("llm"):
		DccSettings._cfg.erase_section("llm")
		DccSettings._save()
	LlmModelList.clear_cache()


func _configure(port: int) -> void:
	_reset()
	var s := {"enabled": true, "prose": true, "local_allowed": true, "preset_id": "lm_studio",
		"base_url": "http://127.0.0.1:%d/v1" % port, "model": "", "key_source": "env", "env_var": "CART_NETHINT_NOKEY", "net_hint_seen": true}
	for k in s:
		DccSettings.set_llm_value(k, s[k])
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % port)


func _open_and_wait() -> void:
	_win.open()
	var t0 := Time.get_ticks_msec()
	await _settle(2)
	while String(_win._ml_state) == "loading" and Time.get_ticks_msec() - t0 < 25000:
		await get_tree().process_frame
	await _settle(3)


func _buttons(root: Node, text: String) -> int:
	var n := 1 if (root is Button and (root as Button).text == text) else 0
	for c in root.get_children():
		n += _buttons(c, text)
	return n


func _labels(n: Node) -> Array:
	var out: Array = []
	if n is Label:
		out.append((n as Label).text)
	for c in n.get_children():
		out.append_array(_labels(c))
	return out


func _last_card_text(aw) -> String:
	for i in range(aw._msgs.get_child_count() - 1, -1, -1):
		var ls := _labels(aw._msgs.get_child(i))
		if not ls.is_empty():
			return String(ls[0])
	return ""


func _finish() -> void:
	LlmNetHint.fake_android = -1
	LlmNetHint.fake_opener = Callable()
	_reset()
	if _mock != null:
		_mock.stop()
	print("RESULT: %s (%d failed)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)


func _ready() -> void:
	get_tree().create_timer(240.0).timeout.connect(func() -> void:
		print("WATCHDOG")
		get_tree().quit(2))
	_mock = Mock.new()
	add_child(_mock)
	_ok(_mock.start(), "server listens")
	var dead_port: int = _mock.port
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_win = _app.assistant_settings_window
	_ok(_win != null, "settings window exists")
	LlmNetHint.fake_package = "org.cartalith.probe"
	var rec := func(action: String, data: String) -> bool:
		_calls.append([action, data])
		return true
	LlmNetHint.fake_opener = rec

	print("== 3. the intent helper ==")
	LlmNetHint.fake_android = 0
	_ok(LlmNetHint.open_app_settings() == "unavailable" and _calls.is_empty(), "desktop: unavailable, nothing recorded")
	LlmNetHint.fake_android = 1
	_ok(LlmNetHint.open_app_settings() == "opened", "android: opened")
	_ok(_calls.size() == 1 and _calls[0][0] == ACTION_LITERAL and _calls[0][1] == "package:org.cartalith.probe",
		"intent is %s with data package:<pkg>: %s" % [ACTION_LITERAL, str(_calls)])
	LlmNetHint.fake_opener = func(_a: String, _d: String) -> bool: return false
	_ok(LlmNetHint.open_app_settings() == "unavailable", "opener failure reports unavailable")
	LlmNetHint.fake_opener = rec

	print("== 1/2. the rule, with literals ==")
	for code in ["unreachable", "timeout"]:
		LlmNetHint.fake_android = 1
		_ok(LlmNetHint.applies(code), "android + %s applies" % code)
		LlmNetHint.fake_android = 0
		_ok(not LlmNetHint.applies(code), "desktop + %s does not" % code)
	LlmNetHint.fake_android = 1
	for code in ["unauthorized", "http_status", "transport", "not_a_list", "too_large", ""]:
		_ok(not LlmNetHint.applies(code), "android + '%s' does not apply" % code)
	_ok(LlmNetHint.with_hint("Could not connect.", true) == "Could not connect. " + HINT_LITERAL, "with_hint appends the exact sentence")
	_ok(LlmNetHint.with_hint("HTTP 500.", false) == "HTTP 500.", "non-transport message is unchanged")
	LlmNetHint.fake_android = 0
	_ok(LlmNetHint.with_hint("Could not connect.", true) == "Could not connect.", "desktop message is unchanged")

	print("== 4/1/2. settings window, refused connection ==")
	_mock.stop()
	for android in [0, 1]:
		LlmNetHint.fake_android = android
		_configure(dead_port)
		await _open_and_wait()
		var st := _status()
		_ok(st.begins_with("Could not list:"), "(android=%d) listing failed: %s" % [android, st.left(80)])
		_ok(st.contains(HINT_LITERAL) == (android == 1), "(android=%d) hint present iff Android" % android)
		_ok((_buttons(_win, BTN) == 1) == (android == 1) and _buttons(_win, BTN) <= 1, "(android=%d) one settings button iff Android" % android)

	print("== 2. HTTP status errors get no hint ==")
	_mock = Mock.new()
	add_child(_mock)
	_ok(_mock.start(), "second server")
	LlmNetHint.fake_android = 1
	for status in [401, 500]:
		_mock.routes = {"/api/v0/models": {"status": status, "body": "{}"}, "/v1/models": {"status": status, "body": "{}"}}
		_configure(_mock.port)
		await _open_and_wait()
		_ok(_status().begins_with("Could not list:") and not _status().contains("Wi-Fi data usage"),
			"status %d: no hint: %s" % [status, _status().left(90)])

	print("== 5. assistant failure card ==")
	var aw = _app.assistant_window
	_ok(aw != null, "assistant window exists")
	aw.open()
	await _settle(4)
	for c in [[1, true, true], [1, false, false], [0, true, false]]:
		LlmNetHint.fake_android = c[0]
		aw.run.last_failure_connect = c[1]
		var cards: int = aw._msgs.get_child_count()
		var btns := _buttons(aw._msgs, BTN)
		aw._on_failed("Timed out connecting to x:1.")
		await _settle(2)
		var txt := _last_card_text(aw)
		_ok(txt.contains(HINT_LITERAL) == c[2], "android=%d connect=%s -> hint %s" % [c[0], c[1], c[2]])
		_ok(txt.begins_with("Timed out connecting to x:1."), "original text kept")
		_ok(_buttons(aw._msgs, BTN) - btns == (1 if c[2] else 0), "button added iff hint")
		_ok(aw._msgs.get_child_count() > cards, "a card was added")

	print("== 4. first-run note ==")
	LlmNetHint.fake_android = 1
	_configure(_mock.port)
	DccSettings.set_llm_value("net_hint_seen", false)
	_win.open()
	await _settle(4)
	_ok(_labels(_win).has(LlmNetHint.FIRST_RUN_NOTE), "first-run note shown while enabled and unseen")
	_win.hide()
	await _settle(2)
	_ok(bool(DccSettings.llm_value("net_hint_seen")), "closing records it as seen")
	_win.open()
	await _settle(4)
	_ok(not _labels(_win).has(LlmNetHint.FIRST_RUN_NOTE), "not shown again")
	LlmNetHint.fake_android = 0
	DccSettings.set_llm_value("net_hint_seen", false)
	_win.open()
	await _settle(4)
	_ok(not _labels(_win).has(LlmNetHint.FIRST_RUN_NOTE), "desktop: never shown")
	_finish()
