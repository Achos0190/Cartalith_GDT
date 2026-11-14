extends Node
## Private notes are never sent to the assistant (Ruling DP; owner choice "Never send private", 2026-11-14;
## `OUTSTANDING_WORK.md` row "Marker card click-through, and private notes never sent to the assistant").
##
##   desktop: godot --path . --resolution 1600x1000 _privnote_probe.tscn
##
## **Windowed, never `--headless`**, from a wiped scratch user dir (`APPDATA` pointed at an empty folder; Godot 4.7
## has no `--user-data-dir`). A throw-away Vault and a loopback OpenAI-compatible server (`_llm_mock_server.gd`)
## are made here; NO real model is ever contacted. The check is on the request body the server actually received.
##
## What it protects. The Assistant's Note task sends the picked note's text when "Send: named Vault notes"
## (`ctx_notes`) is ticked. The Vault has three kinds of private note (a `private: true` frontmatter key, a
## `DM/` folder, a `#secret` tag) and one ordinary note, each holding a canary word:
##   P  the picker LISTS the private notes greyed (disabled, badge "private", reason as the tooltip) and the
##      ordinary one enabled;
##   F  a forced pick of a private note (the signal emitted directly) holds nothing: `_note` is empty, the picker
##      is back on "No note chosen", and a message says why;
##   S  a normal pick still sends its text (the control: the canary of the ordinary note IS in the body, so a
##      green private leg cannot be a request that never carried notes);
##   L  a note picked while public and made private afterwards (the file rewritten) is refused at Send: the
##      button is disabled with the reason, a forced `_on_send()` sends NOTHING, and `_args()` has no
##      `note_text`;
##   T  with `ctx_notes` off no note text goes either (the existing source switch still rules);
##   B  across every request the server received in the whole run, no private canary appears anywhere in any
##      body (the sweep that would catch a path this probe did not think of).
## Exit 0 pass, 1 fail, 2 bad argument or watchdog.

const SECRET := "sk-PROBE-SECRET-0123456789wxyz"
const ENV_VAR := "CART_LLM_PROBE_KEYVAR"
const VAULT_DIR := "user://privnote_probe_vault"
const CANARIES := ["CANARYFRONTMATTER", "CANARYFOLDER", "CANARYTAG"]
const PUBLIC_CANARY := "CANARYPUBLIC"

var _fails := 0
var _checks := 0
var _app: Node
var _bridge
var _win
var _server: LlmMockServer
var _port := 0
var _vault := ""


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _until(cond: Callable, ms: int = 8000) -> bool:
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < ms:
		if cond.call():
			return true
		await get_tree().process_frame
	return bool(cond.call())


func _c(name: String) -> Control:
	return _win.controls.get(name) as Control


func _write(rel: String, text: String) -> void:
	var path := _vault.path_join(rel)
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _type(text: String) -> void:
	var inp := _c("input") as TextEdit
	inp.text = text
	inp.text_changed.emit()
	await _settle(3)


func _configure(over: Dictionary = {}) -> void:
	var s := {"enabled": true, "prose": true, "local_allowed": true, "preset_id": "ollama",
		"base_url": "http://127.0.0.1:%d/v1" % _port, "model": "mock-model",
		"key_source": "env", "env_var": ENV_VAR, "ctx_entity": true, "ctx_notes": true}
	s.merge(over, true)
	for k in s:
		DccSettings.set_llm_value(k, s[k])
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % _port)


func _item_index(text_start: String) -> int:
	var ob := _c("note") as OptionButton
	for k in ob.item_count:
		if ob.get_item_text(k).begins_with(text_start):
			return k
	return -1


func _pick(k: int) -> void:
	var ob := _c("note") as OptionButton
	ob.select(k)
	ob.item_selected.emit(k)
	await _settle(4)


## Every body the server has received, concatenated: the sweep leg B reads it.
func _all_bodies() -> String:
	var parts: Array = []
	for r in _server.requests:
		parts.append(String(r["body"]))
	return "\n".join(parts)


func _finish() -> void:
	OS.unset_environment(ENV_VAR)
	LlmWire.shutdown_all()
	if _server != null:
		_server.stop()
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _ready() -> void:
	get_tree().create_timer(300.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	OS.unset_environment(ENV_VAR)
	_server = LlmMockServer.new()
	add_child(_server)
	_ok(_server.start(), "the mock server listens")
	_port = _server.port
	DccSettings._ensure_loaded()
	if DccSettings._cfg.has_section("llm"):
		DccSettings._cfg.erase_section("llm")
		DccSettings._save()
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_bridge = _app.bridge
	_win = _app.assistant_window
	if not _bridge.llm_api or _win == null:
		print("[FATAL] no LlmBridge or no assistant window")
		_finish()
		return
	_bridge.generate({"seed": 9137, "width_km": 1200.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _settle(10)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	_app.select("settlement", 0)
	await _settle(4)
	_vault = ProjectSettings.globalize_path(VAULT_DIR)
	DirAccess.make_dir_recursive_absolute(_vault)
	_write("Public.md", "# Public\nAn ordinary note. %s\n" % PUBLIC_CANARY)
	_write("Hidden.md", "---\nprivate: true\n---\n# Hidden\nA frontmatter-private note. %s\n" % CANARIES[0])
	_write("DM/Plot.md", "# Plot\nA note in a DM folder. %s\n" % CANARIES[1])
	_write("Tagged.md", "# Tagged\nA tagged note #secret. %s\n" % CANARIES[2])
	var vinfo: Dictionary = _bridge.vault_connect(_vault, "private-note probe")
	_ok(bool(vinfo.get("ok", false)) and bool(_bridge.vault_info().get("bound", false)), "a throw-away Vault is connected: %s" % str(vinfo.get("error", "")))
	OS.set_environment(ENV_VAR, SECRET)
	_server.scenario = "stream"
	_server.tokens = ["Ok."]
	_server.token_delay_ms = 5
	_configure()
	_win.open_window()
	await _settle(8)
	_win._pick_task("note")
	await _settle(3)

	print("== P: the picker greys private notes and reasons them ==")
	var ob := _c("note") as OptionButton
	var ipub := _item_index("Public.md")
	var ihid := _item_index("Hidden.md")
	var idm := _item_index("DM/Plot.md")
	var itag := _item_index("Tagged.md")
	_ok(ipub > 0 and ihid > 0 and idm > 0 and itag > 0, "all four notes are listed (%d %d %d %d)" % [ipub, ihid, idm, itag])
	_ok(not ob.is_item_disabled(ipub) and not ob.get_item_text(ipub).contains("private"), "the ordinary note is enabled and unbadged ('%s')" % ob.get_item_text(ipub))
	for pair in [[ihid, "frontmatter"], [idm, "folder"], [itag, "tag"]]:
		var k: int = pair[0]
		_ok(ob.is_item_disabled(k) and ob.get_item_text(k).ends_with("private"), "item %d '%s' is greyed and badged" % [k, ob.get_item_text(k)])
		_ok(ob.get_item_tooltip(k).contains("Never sent to the assistant"), "...with the reason as its tooltip ('%s')" % ob.get_item_tooltip(k))

	print("== F: a forced pick of a private note holds nothing ==")
	for k in [ihid, idm, itag]:
		await _pick(k)
		_ok(_win._note.is_empty(), "forced pick of item %d: no note is held" % k)
		_ok(ob.selected == 0, "...and the picker is back on 'No note chosen' (selected %d)" % ob.selected)
	var said := false
	for l in _win._msgs.find_children("*", "Label", true, false):
		if (l as Label).text.contains("is private") and (l as Label).text.contains("never sent"):
			said = true
	_ok(said, "...and the transcript says why")

	print("== S: a normal pick sends its text (the control) ==")
	await _pick(ipub)
	_ok(not _win._note.is_empty() and String(_win._note["rel"]) == "Public.md", "the ordinary note is picked")
	await _type("Summarise.")
	_ok(not (_c("send") as Button).disabled, "Send is enabled for the ordinary note ('%s')" % (_c("reason") as Label).text.left(80))
	var n := _server.requests.size()
	_win._on_send()
	await _until(func(): return _server.requests.size() > n and not _win.run.busy(), 8000)
	_ok(_server.requests.size() == n + 1 and String(_server.requests[n]["body"]).contains(PUBLIC_CANARY), "the request body carries the ordinary note's text")

	print("== L: a note made private after it was picked is refused at Send ==")
	_write("Public.md", "---\nprivate: true\n---\n# Public\nNow marked private. %s\n" % PUBLIC_CANARY)
	await _type("Summarise.")
	_ok((_c("send") as Button).disabled, "Send is disabled once the note is private")
	_ok((_c("reason") as Label).text.contains("private") and (_c("reason") as Label).text.contains("never sent"), "...with the reason: '%s'" % (_c("reason") as Label).text.left(120))
	var a: Dictionary = _win._args("Summarise.")
	_ok(not a.has("note_text") and not a.has("note_path"), "`_args()` carries no note_text and no note_path")
	n = _server.requests.size()
	_win._on_send()
	await _settle(30)
	_ok(_server.requests.size() == n, "a forced Send sends NOTHING (%d new requests)" % (_server.requests.size() - n))

	print("== T: with the notes source off no text goes either ==")
	await _pick(ipub)  # refused: it is private now
	_write("Public.md", "# Public\nOrdinary again. %s\n" % PUBLIC_CANARY)
	_win._fill_notes()
	await _pick(_item_index("Public.md"))
	_configure({"ctx_notes": false})
	await _type("Summarise.")
	_ok((_c("send") as Button).disabled and (_c("reason") as Label).text.contains("Send: named Vault notes"), "the source switch still gates Note assist ('%s')" % (_c("reason") as Label).text.left(70))
	_ok(not _win._args("Summarise.").has("note_text"), "`_args()` has no note_text with the source off")
	_configure({"ctx_notes": true})

	print("== B: no private canary in any request body of the whole run ==")
	var all := _all_bodies()
	for cn in CANARIES:
		_ok(not all.contains(cn), "no body contains %s" % cn)
	_ok(all.contains(PUBLIC_CANARY), "(control) the sweep can see a note word: the public canary is in the bodies")
	_finish()
