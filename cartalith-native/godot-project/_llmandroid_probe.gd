extends Node
## On-device probe for LLM-10 (`LLM_ASSISTANT_SCOPE.md`): the Android Keystore key store, the
## phone form of the GPU rule, the LAN `http://` gate, and whether Godot's own `HTTPClient` may
## speak plain HTTP under Android's cleartext policy. **Runs on the handset only** (it refuses on
## any other OS with exit 2): an Android build that ships it is made from a TEMPORARY copy of the
## `Android` preset whose `exclude_filter` no longer drops `_*` and whose
## `command_line/extra_args` is `res://_llmandroid_probe.tscn`; the shipping APK never carries it
## (`ANDROID_BUILD_SCOPE.md`, LLM-10 section, has the exact commands).
##
## It runs in TWO launches, because "the key survives a restart" needs a new process:
##   launch 1 (no marker): save a key in the Keystore store, check it reads back masked, that the
##            ciphertext file has the documented format and does not hold the key, that no file
##            in app storage holds the key, the gate's answers, the GPU plan, a loopback HTTP
##            request; then write the marker `user://llma_phase` and quit.
##   (between launches the operator copies app storage out with `run-as` and greps it for the key)
##   launch 2 (marker present): the key still reads back after the restart; Forget removes the
##            file and the key; the store then reports nothing saved; the marker is removed.
## Output: lines `LLMA ok|FAIL <what>` and `LLMA RESULT: PASS|FAIL (<n> failed) phase <1|2>` in
## logcat (tag `godot`). The key itself is NEVER printed: it is assembled at run time from parts
## so the literal is in no file either, and only its masked tail is compared.
##
## What it protects: the LLM-10 bars "the key survives a restart and is absent from a `run-as`
## copy of app storage", "with no Keystore and no env var, refuses" (the gate refuses a hosted
## call with no saved key), "cleartext/LAN policy decided" (a LAN `http://` address passes the
## gate on the phone as on the desktop; `HTTPClient` cleartext over loopback works), and the phone
## GPU text (no desktop advice). It writes nothing outside `user://` and leaves no key behind on
## any exit path of launch 2.

const ACCOUNT := "openai"
const MARKER := "user://llma_phase"
const STORE_DIR := "user://llm_keystore"

var _fails := 0
var _llm


func _ok(cond: bool, what: String) -> void:
	if cond:
		print("LLMA ok ", what)
	else:
		_fails += 1
		print("LLMA FAIL ", what)


## The probe key, built from parts so it appears verbatim in no file (the script included).
func _key() -> String:
	return "sk-" + "llma" + "-dev" + str(7 * 1234567) + "-" + "q" + "z" + "w" + "x"


## Every regular file under `dir` (recursive), as absolute paths.
func _files(dir: String) -> PackedStringArray:
	var out := PackedStringArray()
	var d := DirAccess.open(dir)
	if d == null:
		return out
	d.include_hidden = true
	d.list_dir_begin()
	var n := d.get_next()
	while n != "":
		var p := dir.path_join(n)
		if d.current_is_dir():
			out.append_array(_files(p))
		else:
			out.append(p)
		n = d.get_next()
	return out


## Whether any file under the app's user dir holds `needle`'s bytes.
func _storage_holds(needle: String) -> String:
	var want := needle.to_utf8_buffer()
	for f in _files(OS.get_user_data_dir()):
		var b := FileAccess.get_file_as_bytes(f)
		if b.size() >= want.size() and _find(b, want) >= 0:
			return f
	return ""


func _find(hay: PackedByteArray, needle: PackedByteArray) -> int:
	var first := needle[0]
	for i in hay.size() - needle.size() + 1:
		if hay[i] == first and hay.slice(i, i + needle.size()) == needle:
			return i
	return -1


func _ready() -> void:
	if not OS.has_feature("android"):
		print("LLMA usage: this probe runs on an Android handset only")
		get_tree().quit(2)
		return
	if not ClassDB.class_exists("LlmBridge"):
		_ok(false, "the LlmBridge class is registered (extension loaded)")
		_finish("?")
		return
	_llm = ClassDB.instantiate("LlmBridge")
	if FileAccess.file_exists(MARKER):
		_phase2()
	else:
		_phase1()


func _status() -> Dictionary:
	return _llm.llm_key_status("store", "", ACCOUNT)


func _phase1() -> void:
	var key := _key()
	var tail := key.substr(key.length() - 4)
	_ok(LlmGpuRule.is_phone(), "LlmGpuRule.is_phone() on Android")
	var st := _status()
	_ok(bool(st.get("store_available", false)), "the Android Keystore store is available (Android %s)" % OS.get_version())
	_llm.llm_key_forget(ACCOUNT)
	_ok(not bool(_status().get("present", true)), "nothing saved before the test")
	# The gate refuses a hosted call with no key saved and no environment variable: the
	# "no Keystore entry and no env var refuses" half that does not need a broken Keystore.
	var hosted := {"enabled": true, "prose": true, "remote_allowed": true, "preset_id": ACCOUNT,
		"base_url": "https://api.openai.com/v1", "model": "probe", "key_source": "store"}
	var g0: Dictionary = _llm.llm_authorize(hosted, "prose", PackedStringArray(["https://api.openai.com"]))
	_ok(String(g0.get("code", "")) == "key_store_empty", "no key saved: the hosted gate refuses (%s)" % g0.get("code", "?"))
	var env_only := hosted.duplicate()
	env_only["key_source"] = "env"
	env_only["env_var"] = "CART_LLM_KEY"
	var g1: Dictionary = _llm.llm_authorize(env_only, "prose", PackedStringArray(["https://api.openai.com"]))
	_ok(String(g1.get("code", "")) == "key_env_missing", "environment source on a phone: refused (%s)" % g1.get("code", "?"))
	# Save.
	var t0 := Time.get_ticks_usec()
	var sv: Dictionary = _llm.llm_key_save(ACCOUNT, key)
	var save_ms := (Time.get_ticks_usec() - t0) / 1000.0
	_ok(bool(sv.get("ok", false)), "the key saves to the Keystore store (%.1f ms) %s" % [save_ms, sv.get("error", "")])
	t0 = Time.get_ticks_usec()
	st = _status()
	var read_ms := (Time.get_ticks_usec() - t0) / 1000.0
	_ok(bool(st.get("present", false)) and String(st.get("masked", "")) == "****" + tail,
		"it reads back masked as ****%s (%.1f ms)" % [tail, read_ms])
	var g2: Dictionary = _llm.llm_authorize(hosted, "prose", PackedStringArray(["https://api.openai.com"]))
	_ok(bool(g2.get("ok", false)) and bool(g2.get("key_required", false)), "with the key saved the hosted gate passes")
	# The ciphertext on disk.
	var f := STORE_DIR.path_join(ACCOUNT + ".bin")
	_ok(FileAccess.file_exists(f), "the ciphertext file exists at " + f)
	var b := FileAccess.get_file_as_bytes(f)
	_ok(b.size() > 4 + 1 + 12 + 16 and b.slice(0, 4).get_string_from_ascii() == "CLK1" and b[4] == 12,
		"it is CLK1 | 12-byte nonce | ciphertext+tag (%d bytes for a %d-byte key)" % [b.size(), key.length()])
	_ok(b.size() == 4 + 1 + 12 + key.length() + 16, "its size is exactly 33 + key length")
	_ok(_find(b, key.to_utf8_buffer()) < 0, "the ciphertext does not contain the key")
	var holder := _storage_holds(key)
	_ok(holder == "", "no file in app storage holds the key" + ("" if holder == "" else " (found in " + holder + ")"))
	# Re-saving replaces, still masked.
	_ok(bool(_llm.llm_key_save(ACCOUNT, key).get("ok", false)), "saving again replaces it")
	# An invalid account id is refused, never stored under an odd name.
	_ok(not bool(_llm.llm_key_save("Bad Account", key).get("ok", true)), "an invalid account id is refused")
	# The LAN http:// gate on a phone, exactly as the desktop (no network is touched).
	var lan := {"enabled": true, "prose": true, "local_allowed": true, "preset_id": "lm_studio",
		"base_url": "http://192.168.1.238:1234/v1", "model": "probe", "key_source": "env"}
	var gl: Dictionary = _llm.llm_authorize(lan, "prose", PackedStringArray(["http://192.168.1.238:1234"]))
	_ok(bool(gl.get("ok", false)) and String(gl.get("class", "")) == "local_network", "a LAN http:// address passes the gate on the phone (class %s)" % gl.get("class", "?"))
	var gn: Dictionary = _llm.llm_authorize(lan, "prose", PackedStringArray())
	_ok(String(gn.get("code", "")) == "consent_needed", "without consent it asks first (%s)" % gn.get("code", "?"))
	var pub := lan.duplicate()
	pub["preset_id"] = "custom_openai"
	pub["base_url"] = "http://8.8.8.8:1234/v1"
	pub["remote_allowed"] = true
	var gp: Dictionary = _llm.llm_authorize(pub, "prose", PackedStringArray(["http://8.8.8.8:1234"]))
	_ok(String(gp.get("code", "")) == "cleartext_remote", "a public host over http:// is refused (%s)" % gp.get("code", "?"))
	# The phone GPU plan: never a switch, no desktop advice.
	for url in ["http://192.168.1.238:1234/v1", "http://localhost:1234/v1", "https://api.openai.com/v1", "http://myhost.local/v1"]:
		var p: Dictionary = _llm.llm_gpu_plan_phone(url, true)
		var txt := (String(p.get("headline", "")) + " " + String(p.get("effect", ""))).to_lower()
		_ok(bool(p.get("warn", false)) and not bool(p.get("switch", true)) and not p.has("target")
			and not txt.contains("low-power") and not txt.contains("dedicated") and not txt.contains("windows"),
			"phone GPU plan for %s: %s | %s" % [url, p.get("detected", ""), p.get("effect", "")])
	# Cleartext: Godot's HTTPClient over loopback (no packet leaves the phone).
	_loopback_http()
	var m := FileAccess.open(MARKER, FileAccess.WRITE)
	m.store_string("1")
	m.close()
	_finish("1")


func _phase2() -> void:
	var key := _key()
	var tail := key.substr(key.length() - 4)
	var st := _status()
	_ok(bool(st.get("present", false)) and String(st.get("masked", "")) == "****" + tail,
		"after a restart the key still reads back as ****" + tail)
	var holder := _storage_holds(key)
	_ok(holder == "", "after a restart no file in app storage holds the key")
	var fg: Dictionary = _llm.llm_key_forget(ACCOUNT)
	_ok(bool(fg.get("ok", false)), "Forget succeeds")
	_ok(not FileAccess.file_exists(STORE_DIR.path_join(ACCOUNT + ".bin")), "Forget removes the ciphertext file")
	st = _status()
	_ok(not bool(st.get("present", true)) and bool(st.get("store_available", false)), "the store then reports nothing saved")
	_ok(bool(_llm.llm_key_forget(ACCOUNT).get("ok", false)), "forgetting nothing is not an error")
	DirAccess.remove_absolute(ProjectSettings.globalize_path(MARKER))
	_finish("2")


## A real HTTP/1.1 exchange between a TCPServer and an HTTPClient inside this process, over
## 127.0.0.1. Android's cleartext policy (`usesCleartextTraffic`, network security config) is
## enforced by the Java HTTP stacks; Godot's client is native sockets, so this is expected to work
## with only the INTERNET permission (which loopback also needs).
func _loopback_http() -> void:
	var srv := TCPServer.new()
	var err := srv.listen(0, "127.0.0.1")
	_ok(err == OK, "a loopback TCPServer listens (INTERNET permission present) err=%d" % err)
	if err != OK:
		return
	var port := srv.get_local_port()
	var cl := HTTPClient.new()
	_ok(cl.connect_to_host("127.0.0.1", port) == OK, "HTTPClient connects to http://127.0.0.1:%d" % port)
	var peer: StreamPeerTCP = null
	var sent := false
	var body := PackedByteArray()
	var deadline := Time.get_ticks_msec() + 5000
	var requested := false
	while Time.get_ticks_msec() < deadline:
		cl.poll()
		if peer == null and srv.is_connection_available():
			peer = srv.take_connection()
		if peer != null:
			peer.poll()
		var s := cl.get_status()
		if s == HTTPClient.STATUS_CONNECTED and not requested:
			cl.request(HTTPClient.METHOD_GET, "/probe", [])
			requested = true
		if peer != null and requested and not sent and peer.get_available_bytes() > 0:
			peer.get_data(peer.get_available_bytes())
			peer.put_data("HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok".to_ascii_buffer())
			sent = true
		if s == HTTPClient.STATUS_BODY:
			var chunk := cl.read_response_body_chunk()
			body.append_array(chunk)
		if body.size() >= 2:
			break
		## A blocking 2 ms step, not a frame: the exchange is one process talking to itself, and
		## a frame-paced loop measured code 0 on the handset within its 5 s (2026-10-26) where
		## this tight loop answers at once on the desktop. Diagnosed below if it still fails.
		OS.delay_msec(2)
	_ok(cl.get_response_code() == 200 and body.get_string_from_ascii() == "ok",
		"plain http:// through HTTPClient works on Android (code %d, body %s, client status %d, server peer %s, request sent %s, reply sent %s)" % [cl.get_response_code(), body.get_string_from_ascii(), cl.get_status(), "none" if peer == null else str(peer.get_status()), requested, sent])
	cl.close()
	if peer != null:
		peer.disconnect_from_host()
	srv.stop()
	_tcp_diag()


## Diagnostic only (asserts nothing): a bare StreamPeerTCP connect to an in-process TCPServer,
## bound to "127.0.0.1" and to "*", reported as the status reached within 3 s. Added when the
## HTTPClient exchange above stayed CONNECTING on the handset (2026-10-26), to tell a socket-level
## block from an HTTPClient one.
func _tcp_diag() -> void:
	for bind in ["127.0.0.1", "*"]:
		for target in ["127.0.0.1", "::1", "localhost"]:
			var srv := TCPServer.new()
			var e := srv.listen(0, bind)
			if e != OK:
				print("LLMA diag bind %s -> %s: listen err %d" % [bind, target, e])
				continue
			var t := StreamPeerTCP.new()
			var ce := t.connect_to_host(target, srv.get_local_port())
			var accepted := false
			var t0 := Time.get_ticks_msec()
			while Time.get_ticks_msec() - t0 < 3000:
				t.poll()
				if srv.is_connection_available():
					srv.take_connection()
					accepted = true
				if accepted and t.get_status() == StreamPeerTCP.STATUS_CONNECTED:
					break
				OS.delay_msec(2)
			print("LLMA diag bind %s -> %s: connect err %d, status %d, accepted %s, %d ms" % [bind, target, ce, t.get_status(), accepted, Time.get_ticks_msec() - t0])
			t.disconnect_from_host()
			srv.stop()


func _finish(phase: String) -> void:
	print("LLMA RESULT: %s (%d failed) phase %s" % ["PASS" if _fails == 0 else "FAIL", _fails, phase])
	get_tree().quit(0 if _fails == 0 else 1)
