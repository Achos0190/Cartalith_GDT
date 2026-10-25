extends Node
## Probe for the Assistant panel (`shell/llm_assistant_window.gd`) inside the real shell, against a
## real loopback OpenAI-compatible server (`_llm_mock_server.gd`) -- milestone LLM-3 of
## `LLM_ASSISTANT_SCOPE.md` (U1 prose, U3 story, U6 note assist; streamed, cancellable, written to
## the Vault only on a click).
##
##   desktop: godot --path . --resolution 1600x1000 _llmpanel_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1280x800  _llmpanel_probe.tscn -- --force-touch --shots <dir>
##   phone:   godot --path . --resolution 393x852   _llmpanel_probe.tscn -- --force-touch --shots <dir>
##
## **Windowed, never `--headless`**, from a scratch copy with its own `config/name` and a wiped user
## dir (`MISTAKES.md`). Arguments read: `--force-touch` and `--shots <dir>`; anything else aborts with
## exit 2. It sets `CART_LLM_PROBE_KEYVAR` itself (the secret is never printed) and clears it on exit.
##
## What it protects, in the shell (the LLM-3 bars):
##   1. OFF: with the feature off the panel opens, says why Send is unavailable, a forced Send is
##      refused with words and opens NO connection (counters and `netstat`, against a positive
##      control taken while a reply streams).
##   2. SOURCES: "This will send" names exactly the ticked sources; an unticked source is absent from
##      the request body on the wire, not empty; disabled-reason text for each missing source.
##   3. STREAM: a real click on Send streams tokens into the panel while the server is still sending
##      (a partial text is seen), the reply carries the MODEL-GENERATED chip, the counter shows what
##      the server reported, and the Vault is UNTOUCHED until Save.
##   4. SAVE: a real click on Save writes through `vault_write_file(expect_hash)`: the note keeps its
##      own text as a prefix and gains one marked block; defused if the model forges a machine block.
##   5. CONFLICT: a note edited after it was picked is REFUSED, nothing is written, and Reload then
##      Save succeeds.
##   6. CANCEL: a real click on Stop ends the stream, closes the socket (the server sees it), joins
##      the thread, and leaves the panel usable.
##   7. TASKS: Story sends events, Note sends the chosen note only when its source is ticked.
##   8. CLOSE: hiding the window mid-stream cancels and joins.
##   9. TOUCH: tablet and phone controls meet the density floor and sit inside the window.
##  11. LLM-4 (counter, cap, retry): Reset zeroes the counter line ("not reported yet"); a reply on
##      the Anthropic wire shows its usage ("Tokens 30 in / N out", "no cap"); a cap that the
##      counter has reached disables Send with the reason BEFORE the click and the line says "cap N";
##      Reset lifts it; one 529 shows "Trying once more" and the reply still completes after exactly
##      two requests; the status line says where the endpoint is (hosted or this computer).
##  10. LEAKS: the key is absent from the note, the settings file, every label in the window and
##      `godot.log`.
## Result: `RESULT: PASS|FAIL (<n> failed)`. Exit 0 pass, 1 fail, 2 bad argument or watchdog.

const SECRET := "sk-PROBE-SECRET-0123456789wxyz"
const ENV_VAR := "CART_LLM_PROBE_KEYVAR"
const VAULT_DIR := "user://llm_panel_probe_vault"

var _fails := 0
var _mode := "desktop"
var _shots := ""
var _app: Node
var _bridge
var _win
var _server: LlmMockServer
var _port := 0
var _vault := ""


func _ok(cond: bool, what: String) -> void:
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


func _shot(tag: String) -> void:
	if _shots == "":
		return
	await _settle(3)
	## Shots numbered 10 and up belong to LLM-4 (counter, cap, retry); the rest are LLM-3's.
	var path := _shots.path_join("llm%s_%s_%s.png" % ["4" if int(tag.get_slice("_", 0)) >= 10 else "3", _mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


func _click(c: Control) -> void:
	var local := c.get_global_rect().get_center()
	var vp := c.get_viewport()
	var at: Vector2 = vp.get_final_transform() * local
	if vp is Window and vp != get_tree().root:
		at += Vector2((vp as Window).position)
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	Input.parse_input_event(mm)
	await _settle(2)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(3)


func _find_row(n: Node, text: String) -> Array:
	if n is PopupMenu:
		var p := n as PopupMenu
		for i in p.item_count:
			if p.get_item_text(i) == text:
				return [p, i]
	for c in n.get_children(true):
		var r := _find_row(c, text)
		if not r.is_empty():
			return r
	return []


func _all_text(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
		out.append((n as Button).tooltip_text)
	elif n is LineEdit:
		out.append((n as LineEdit).text)
	elif n is TextEdit:
		out.append((n as TextEdit).text)
	if n is Control:
		out.append((n as Control).tooltip_text)
	for ch in n.get_children(true):
		_all_text(ch, out)


func _window_text() -> String:
	var parts: Array = []
	_all_text(_win, parts)
	return "\n".join(parts)


func _read(path: String) -> String:
	var f := FileAccess.open(path, FileAccess.READ)
	return f.get_as_text() if f != null else ""


func _bytes(path: String) -> PackedByteArray:
	var f := FileAccess.open(path, FileAccess.READ)
	return f.get_buffer(f.get_length()) if f != null else PackedByteArray()


func _has(hay: PackedByteArray, needle: String) -> bool:
	var n := needle.to_utf8_buffer()
	if n.is_empty() or hay.size() < n.size():
		return false
	for i in range(0, hay.size() - n.size() + 1):
		if hay[i] == n[0]:
			var hit := true
			for j in range(1, n.size()):
				if hay[i + j] != n[j]:
					hit = false
					break
			if hit:
				return true
	return false


func _netstat_other(pid: int) -> int:
	## Both address families: `-p TCP` is IPv4 only, and a request to `localhost` can be IPv6.
	var other := 0
	for proto in ["TCP", "TCPv6"]:
		var out: Array = []
		OS.execute("netstat", ["-ano", "-p", proto], out)
		for line in String("".join(out)).split("\n"):
			var cols := line.strip_edges().split(" ", false)
			if cols.size() >= 5 and cols[0] == "TCP" and int(cols[cols.size() - 1]) == pid and cols[3] != "LISTENING":
				other += 1
	return other


func _reset_llm() -> void:
	DccSettings._ensure_loaded()
	if DccSettings._cfg.has_section("llm"):
		DccSettings._cfg.erase_section("llm")
		DccSettings._save()


func _configure(over: Dictionary = {}) -> void:
	var s := {"enabled": true, "prose": true, "local_allowed": true, "preset_id": "ollama",
		"base_url": "http://127.0.0.1:%d/v1" % _port, "model": "mock-model",
		"key_source": "env", "env_var": ENV_VAR, "ctx_entity": true, "ctx_notes": true}
	s.merge(over, true)
	for k in s:
		DccSettings.set_llm_value(k, s[k])
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % _port)


func _type(text: String) -> void:
	var inp := _c("input") as TextEdit
	inp.text = text
	inp.text_changed.emit()
	await _settle(3)


func _assistant_text() -> String:
	return String(_win._reply_label.text) if _win._reply_label != null else ""


func _last_body() -> Dictionary:
	var b = JSON.parse_string(String(_server.requests[_server.requests.size() - 1]["body"]))
	return b if b is Dictionary else {}


func _user_content(b: Dictionary) -> String:
	for m in b.get("messages", []):
		if m["role"] == "user":
			return String(m["content"])
	return ""


func _system_content(b: Dictionary) -> String:
	for m in b.get("messages", []):
		if m["role"] == "system":
			return String(m["content"])
	return ""


func _finish() -> void:
	OS.unset_environment(ENV_VAR)
	LlmWire.shutdown_all()
	if _server != null:
		_server.stop()
	print("RESULT: %s (%d failed)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--force-touch":
			pass
		elif args[i] == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		else:
			print("ABORT: unrecognised argument %s -- this probe reads --force-touch and --shots <dir> only" % args[i])
			get_tree().quit(2)
			return
		i += 1
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	OS.unset_environment(ENV_VAR)
	_server = LlmMockServer.new()
	add_child(_server)
	_ok(_server.start(), "the mock server listens")
	_port = _server.port
	_reset_llm()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_mode = "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	var vp := get_viewport().get_visible_rect().size
	print("== mode %s, viewport %s ==" % [_mode, vp])
	_bridge = _app.bridge
	_win = _app.assistant_window
	_ok(_bridge.llm_api and _win != null, "the library has LlmBridge and the shell has the panel")
	if not _bridge.llm_api or _win == null:
		_finish()
		return
	var pid := OS.get_process_id()

	# A world, a place, a Vault: the state the tasks need.
	_bridge.generate({"seed": 9137, "width_km": 1200.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _settle(10)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	var places: Array = _bridge.settlements()
	_ok(places.size() > 0, "the generated world has settlements (%d)" % places.size())
	_app.right_dock_ctrl.on_settlement_selected(places[0], 0)
	await _settle(4)
	var place_name := String(places[0].get("name", ""))
	_vault = ProjectSettings.globalize_path(VAULT_DIR)
	DirAccess.make_dir_recursive_absolute(_vault)
	var note_a := "# Veldmark\nMy own notes about the port.\n"
	var f := FileAccess.open(_vault.path_join("Veldmark.md"), FileAccess.WRITE)
	f.store_string(note_a)
	f.close()
	f = FileAccess.open(_vault.path_join("Other.md"), FileAccess.WRITE)
	f.store_string("# Other\nA second note.\n")
	f.close()
	var vinfo: Dictionary = _bridge.vault_connect(_vault, "LLM panel probe")
	_ok(bool(vinfo.get("ok", false)) and bool(_bridge.vault_info().get("bound", false)), "a probe Vault is connected: %s" % str(vinfo.get("error", "")))

	# ------------------------------------------------------------------ 1. OFF
	print("== 1. feature off ==")
	var row := _find_row(_app, "Assistant panel…")
	_ok(not row.is_empty(), "Story ▸ NOTES carries 'Assistant panel…'")
	if row.is_empty():
		_finish()
		return
	var pm: PopupMenu = row[0]
	pm.about_to_popup.emit()
	_ok(not pm.is_item_disabled(row[1]), "the row is enabled (the panel itself says why it cannot send)")
	pm.id_pressed.emit(pm.get_item_id(row[1]))
	await _settle(10)
	_ok(_win.visible, "the row opens the panel")
	var wr := Rect2(Vector2(_win.position), Vector2(_win.size))
	_ok(wr.position.x >= -0.5 and wr.position.y >= -0.5 and wr.end.x <= vp.x + 0.5 and wr.end.y <= vp.y + 0.5, "the window %s is inside the viewport %s" % [wr, vp])
	_ok((_c("status") as Label).text.begins_with("The assistant is off"), "status: '%s'" % (_c("status") as Label).text.left(90))
	await _type("Describe the port.")
	_ok((_c("send") as Button).disabled, "Send is disabled while the feature is off")
	_ok((_c("reason") as Label).visible and (_c("reason") as Label).text.begins_with("The assistant is off"), "...with its reason beside it: '%s'" % (_c("reason") as Label).text.left(60))
	_ok(String((_c("send") as Button).tooltip_text) != "", "...and as its tooltip")
	_ok((_c("save") as Button).disabled and (_c("save_reason") as Label).text == "No reply to save yet.", "Save is disabled: '%s'" % (_c("save_reason") as Label).text)
	_ok(not (_c("stop") as Button).visible, "Stop is hidden when idle")
	## LLM-6b enabled the chip (setting proposals); its own probe is `_llmparams_probe.gd`.
	_ok(not (_c("task_commands") as Button).disabled and (_c("task_commands") as Button).tooltip_text.contains("Apply"), "Commands is enabled and says nothing changes until Apply")
	var att0 := LlmWire.connect_attempts
	_win._on_send()   # a forced Send, bypassing the disabled button
	await _settle(8)
	var last_msg: Node = _win._msgs.get_child(_win._msgs.get_child_count() - 1)
	var last_txt: Array = []
	_all_text(last_msg, last_txt)
	last_txt = last_txt.filter(func(t): return String(t) != "")
	_ok("
".join(last_txt).begins_with("The assistant is off"), "a forced Send is refused with the GATE's reason, not a source complaint: '%s'" % "
".join(last_txt).left(60))
	_ok(LlmWire.connect_attempts == att0 and _server.accepted == 0, "a forced Send opened no connection (attempts %d, accepted %d)" % [LlmWire.connect_attempts, _server.accepted])
	_ok(_win.run.stream != null and not _win.run.busy() and LlmWire.live_threads == 0, "...and started no thread")
	_ok(_netstat_other(pid) == 0, "netstat: 0 non-listening TCP rows for this process with the feature off")
	await _shot("1_off")

	# ------------------------------------------------------------------ 2. sources
	print("== 2. sources ==")
	_configure({"ctx_entity": false})
	_win.open()
	await _settle(8)
	await _type("Describe the port.")
	_ok((_c("place") as Label).text == "Place: %s" % place_name, "the place line names the selected settlement: '%s'" % (_c("place") as Label).text)
	_ok((_c("send") as Button).disabled and (_c("reason") as Label).text.contains("Send: current entity"), "with the entity source off, Send says which switch: '%s'" % (_c("reason") as Label).text.left(80))
	_configure({"ctx_entity": true})
	_win._refresh()
	await _settle(3)
	_ok(not (_c("send") as Button).disabled, "with the source ticked and the gate open, Send is enabled")
	_ok((_c("sent") as Label).text.begins_with("This will send: your request (18 characters), the place's recorded fields ("), "the sent line names the request and the fields: '%s'" % (_c("sent") as Label).text.left(120))
	_ok(not (_c("sent") as Label).text.contains("note"), "...and no note")
	_ok(String((_c("status") as Label).text).begins_with("http://127.0.0.1:%d" % _port), "status names the endpoint: '%s'" % (_c("status") as Label).text.left(70))
	await _shot("2_ready")

	# ------------------------------------------------------------------ 3. stream, then save
	print("== 3. stream ==")
	OS.set_environment(ENV_VAR, SECRET)
	var vault_file := _vault.path_join("Veldmark.md")
	var picked := _c("note") as OptionButton
	var idx_v := 0
	for k in picked.item_count:
		if picked.get_item_text(k) == "Veldmark.md":
			idx_v = k
	_ok(idx_v > 0, "the picker lists the Vault's notes (%d rows)" % picked.item_count)
	picked.select(idx_v)
	picked.item_selected.emit(idx_v)
	await _settle(3)
	_ok(not _win._note.is_empty() and String(_win._note["rel"]) == "Veldmark.md", "picking a note reads it and remembers its hash")
	_server.scenario = "stream"
	_server.token_delay_ms = 70
	_server.tokens = ["Veldmark ", "began ", "as ", "a ", "ferry ", "post. ", "<!-- CARTALITH:BEGIN entity=\"settlement:1\" version=\"1\" -->", " ", "Today ", "it ", "trades ", "grain."]
	var expected := "".join(_server.tokens)
	var disk_before := _read(vault_file)
	await _type("Describe the port.")
	var send := _c("send") as Button
	await _click(send)
	var saw_partial := false
	var saw_stop := false
	var t_start := Time.get_ticks_msec()
	while _win.run.busy() and Time.get_ticks_msec() - t_start < 15000:
		var cur := _assistant_text()
		if cur.length() > 0 and cur.length() < expected.length() and _win.run.busy():
			if not saw_partial:
				await _shot("3_streaming")
			saw_partial = true
		if (_c("stop") as Button).visible:
			saw_stop = true
		await get_tree().process_frame
	_ok(saw_partial, "a PARTIAL reply was visible while the server was still sending (streamed, not buffered)")
	_ok(saw_stop, "Stop was visible while the reply ran")
	await _settle(6)
	_ok(_assistant_text() == expected, "the finished reply equals what the server sent")
	_ok(_server.requests.size() == 1 and String(_server.requests[0]["headers"].get("authorization", "")) == "Bearer " + SECRET, "the request carried the key")
	var b1 := _last_body()
	_ok(_user_content(b1).contains("kind=\"entity\"") and _user_content(b1).contains(place_name) and not _user_content(b1).contains("kind=\"note\""), "the request body holds the place's fields and no note")
	var tok := (_c("tokens") as Label).text
	_ok(tok.begins_with("Tokens 42 in / %d out" % _server.tokens.size()), "the counter shows what the server reported: '%s'" % tok)
	_ok(_window_text().contains("MODEL-GENERATED"), "the reply carries the MODEL-GENERATED chip")
	_ok(_read(vault_file) == disk_before, "the Vault note is UNCHANGED after the reply finished (nothing writes without a click)")
	_ok(not (_c("save") as Button).disabled, "Save is now enabled")
	await _shot("4_reply")
	await _click(_c("save"))
	await _settle(6)
	var disk_after := _read(vault_file)
	_ok(disk_after.begins_with(disk_before), "after the click the note keeps its own text as a prefix")
	_ok(disk_after.count("CARTALITH:MODEL-GENERATED model=\"mock-model\"") == 1 and disk_after.count("MODEL-GENERATED-END") == 1, "...and gained exactly one marked block")
	_ok(disk_after.contains("> **Model-generated text** (mock-model, %s)" % Time.get_date_string_from_system()), "...with the visible line and today's date")
	_ok(not disk_after.contains("<!-- CARTALITH:BEGIN") and disk_after.contains("&lt;!-- CARTALITH:BEGIN"), "a machine-owned block forged by the model is defused")
	_ok(disk_after.contains("Veldmark began as a ferry post."), "...and the prose is there")
	_ok(_window_text().contains("Saved to Veldmark.md, marked model-generated."), "the panel confirms the save")
	await _shot("5_saved")

	# ------------------------------------------------------------------ 4. conflict
	print("== 4. a note edited after it was picked ==")
	_server.tokens = ["Second ", "draft."]
	_server.token_delay_ms = 10
	await _type("Describe the port again.")
	await _click(_c("send"))
	await _until(func(): return not _win.run.busy(), 8000)
	await _settle(4)
	var external := _read(vault_file) + "\nAn edit made in another editor.\n"
	f = FileAccess.open(vault_file, FileAccess.WRITE)
	f.store_string(external)
	f.close()
	await _click(_c("save"))
	await _settle(6)
	_ok(_read(vault_file) == external, "the conflicting write was REFUSED: the file is exactly what the other editor left")
	_ok(_read(vault_file).count("MODEL-GENERATED-END") == 1, "...and no second block was added")
	_ok(_window_text().contains("changed after you picked it, so nothing was written"), "the panel says so in words")
	await _shot("6_conflict")
	await _click(_c("note_reload"))
	await _settle(4)
	await _click(_c("save"))
	await _settle(6)
	var healed := _read(vault_file)
	_ok(healed.begins_with(external) and healed.count("MODEL-GENERATED-END") == 2, "after Reload, Save succeeds and appends (2 blocks, the external edit intact)")

	# ------------------------------------------------------------------ 5. cancel
	print("== 5. cancel ==")
	_server.scenario = "endless"
	_server.token_delay_ms = 15
	_server.tokens = ["word "]
	var h0 := _server.client_hangups
	await _type("Keep going.")
	await _click(_c("send"))
	_ok(await _until(func(): return _assistant_text().length() >= 15, 5000), "tokens are arriving")
	_ok(LlmWire.live_threads == 1 and _netstat_other(pid) >= 1, "positive control: a worker thread and an open socket exist mid-stream (netstat rows %d)" % _netstat_other(pid))
	var stop := _c("stop") as Button
	_ok(stop.visible and not stop.disabled, "Stop is visible and enabled")
	await _click(stop)
	_ok(await _until(func(): return _window_text().contains("Stopped."), 3000), "the panel says 'Stopped.'")
	var frozen := _assistant_text()
	_ok(await _until(func(): return _server.client_hangups > h0, 3000), "the server saw the socket close")
	await _settle(30)
	_ok(_assistant_text() == frozen, "no token was added after Stop")
	_ok(not _win.run.busy() and LlmWire.live_threads == 0, "the worker is joined")
	_ok(not (_c("send") as Button).disabled or (_c("reason") as Label).text != "", "the panel is usable again")
	await _shot("7_stopped")

	# ------------------------------------------------------------------ 6. tasks
	print("== 6. story and note ==")
	_server.scenario = "stream"
	_server.tokens = ["Ok."]
	_server.token_delay_ms = 5
	_win._pick_task("story")
	await _type("Narrate this place.")
	var n_req := _server.requests.size()
	await _click(_c("send"))
	await _until(func(): return _server.requests.size() > n_req and not _win.run.busy(), 8000)
	var bs := _last_body()
	_ok(_user_content(bs).contains("kind=\"events\"") and not _user_content(bs).contains("kind=\"entity\"") and not _user_content(bs).contains("kind=\"note\""), "Story sends the events source only")
	_ok(_system_content(bs).contains("ONLY the dated events"), "...under the story system prompt")
	_win._pick_task("note")
	await _type("Summarise.")
	_configure({"ctx_notes": false})
	_win._refresh()
	await _settle(3)
	_ok((_c("send") as Button).disabled and (_c("reason") as Label).text.contains("Send: named Vault notes"), "with the notes source off, Note assist is disabled saying which switch: '%s'" % (_c("reason") as Label).text.left(80))
	_configure({"ctx_notes": true})
	_win._refresh()
	await _settle(3)
	_ok((_c("sent") as Label).text.contains("the note's text ("), "with it ticked the sent line names the note: '%s'" % (_c("sent") as Label).text.left(110))
	n_req = _server.requests.size()
	await _click(_c("send"))
	await _until(func(): return _server.requests.size() > n_req and not _win.run.busy(), 8000)
	var bn := _last_body()
	_ok(_user_content(bn).contains("kind=\"note\" name=\"Veldmark.md\"") and _user_content(bn).contains("My own notes about the port.") and not _user_content(bn).contains("kind=\"entity\""), "Note assist sends the chosen note's text and its name only")
	_ok(_system_content(bn).contains("Summarise the note"), "...under the summarise prompt")
	await _shot("8_note_task")

	# ------------------------------------------------------------------ 7. closing mid-stream
	print("== 7. closing the window ==")
	_server.scenario = "endless"
	_server.tokens = ["w "]
	h0 = _server.client_hangups
	_win._pick_task("prose")
	await _type("Go on.")
	await _click(_c("send"))
	await _until(func(): return _assistant_text().length() >= 6, 5000)
	_win.hide()
	await _settle(4)
	_ok(LlmWire.live_threads == 0 and not _win.run.busy(), "hiding the window joined the worker")
	_ok(await _until(func(): return _server.client_hangups > h0, 3000), "...and closed the socket")

	# ------------------------------------------------------------------ 7b. LLM-4: counter, cap, retry
	print("== 7b. counter, cap and retry (LLM-4) ==")
	_win.open()
	await _settle(8)
	_configure({"preset_id": "custom_anthropic", "token_cap": 0})
	_server.scenario = "anthropic"
	_server.token_delay_ms = 10
	_server.tokens = ["The ", "river ", "runs ", "south."]
	_win._pick_task("prose")
	await _type("Describe the river.")
	await _click(_c("tokens_reset"))
	await _settle(3)
	_ok((_c("tokens") as Label).text == "Tokens: not reported yet · no cap", "Reset empties the counter line: '%s'" % (_c("tokens") as Label).text)
	var st := (_c("status") as Label).text
	_ok(st.contains("(this computer or network)") and st.begins_with("http://127.0.0.1:%d" % _port), "the status line says where the endpoint is: '%s'" % st.left(90))
	_ok(not (_c("send") as Button).disabled, "Send is enabled on the Anthropic style")
	var n4 := _server.requests.size()
	await _click(_c("send"))
	await _until(func(): return _server.requests.size() > n4 and not _win.run.busy(), 12000)
	await _settle(6)
	_ok(_assistant_text() == "The river runs south.", "the Anthropic reply is shown: '%s'" % _assistant_text())
	_ok(_server.requests[n4]["path"] == "/v1/messages" and String(_server.requests[n4]["headers"].get("x-api-key", "")) == SECRET, "it went to /v1/messages with x-api-key")
	var tok4 := (_c("tokens") as Label).text
	_ok(tok4 == "Tokens 30 in / 4 out · no cap", "the counter shows usage and 'no cap': '%s'" % tok4)
	await _shot("10_counter")
	# A cap the counter has reached: Send is disabled with the reason before any click.
	## `_configure` restores its own defaults for every key it is not given, INCLUDING the provider,
	## so each call in this section names the Anthropic preset again (the first run of this section
	## silently went back to the OpenAI wire and failed against the Anthropic server).
	_configure({"preset_id": "custom_anthropic", "token_cap": 10})
	_win._refresh()
	await _settle(3)
	_ok((_c("send") as Button).disabled and (_c("reason") as Label).text.contains("token cap is reached"), "a reached cap disables Send saying why: '%s'" % (_c("reason") as Label).text.left(70))
	_ok((_c("tokens") as Label).text.ends_with("· cap 10"), "...and the counter line says 'cap 10': '%s'" % (_c("tokens") as Label).text)
	await _shot("11_cap_reached")
	await _type("Describe the river once more.")
	await _click(_c("tokens_reset"))
	await _settle(3)
	_ok(not (_c("send") as Button).disabled and (_c("tokens") as Label).text == "Tokens: not reported yet · cap 10", "Reset lifts it: '%s'" % (_c("tokens") as Label).text)
	_configure({"preset_id": "custom_anthropic", "token_cap": 0})
	_win._refresh()
	# One 529, then success: the retry is visible as words, and the reply completes.
	n4 = _server.requests.size()
	_server.fail_next = 1
	await _type("Describe the river again.")
	await _click(_c("send"))
	await _until(func(): return _server.requests.size() - n4 >= 2 and not _win.run.busy(), 15000)
	await _settle(6)
	_ok(_server.requests.size() - n4 == 2, "exactly two requests for one 529 (%d)" % (_server.requests.size() - n4))
	_ok(_window_text().contains("Overloaded. Trying once more;"), "the panel says it is trying once more")
	_ok(_assistant_text() == "The river runs south.", "...and the reply completed: '%s'" % _assistant_text())
	await _shot("12_retry")

	# ------------------------------------------------------------------ 8. touch and layout
	print("== 8. density ==")
	_win.open()
	await _settle(8)
	var floor_px := 0.0
	if _mode == "phone":
		floor_px = float(DccTheme.PHONE_TAP_MIN)
	elif _mode == "tablet":
		floor_px = float(DccTheme.role_px("btn_min_h"))
	var win_rect := Rect2(Vector2(_win.position), Vector2(_win.size))
	for name in ["send", "stop", "save", "tokens_reset", "note_reload", "settings", "task_prose", "task_story", "task_note", "note", "input"]:
		var c := _c(name)
		if c == null or not c.is_visible_in_tree():
			continue
		if floor_px > 0.0:
			_ok(c.size.y + 0.5 >= floor_px, "'%s' is at least %d px tall: %.0f" % [name, int(floor_px), c.size.y])
		var rr := c.get_global_rect()
		_ok(rr.position.x >= -1.0 and rr.end.x <= float(_win.size.x) + 1.0 or _mode == "desktop" and rr.end.x <= win_rect.end.x + 1.0, "'%s' sits inside the window width" % name)
	await _shot("9_layout")

	# ------------------------------------------------------------------ 9. leaks
	print("== 9. the key is nowhere it should not be ==")
	var cfg := _bytes(DccSettings.CONFIG_PATH)
	_ok(cfg.size() > 20 and not _has(cfg, SECRET), "cartalith_settings.cfg (%d bytes) holds no key" % cfg.size())
	_ok(not _read(vault_file).contains(SECRET), "the note holds no key")
	_ok(not _window_text().contains(SECRET), "no label, button, tooltip or field in the window holds the key")
	var lb := _bytes(ProjectSettings.globalize_path("user://logs/godot.log"))
	_ok(lb.size() > 100 and not _has(lb, SECRET), "godot.log (%d bytes) holds no key" % lb.size())
	# A saved project after a whole session with the key in the environment: every entry of the archive.
	var sdir := ProjectSettings.globalize_path("user://_llmpanel_probe")
	DirAccess.make_dir_recursive_absolute(sdir)
	var zpath := sdir.path_join("session.zip")
	_ok(_bridge.save_project(zpath), "a project was saved after the session")
	var zr := ZIPReader.new()
	_ok(zr.open(zpath) == OK, "the archive opens")
	var entries := zr.get_files()
	var leaked := 0
	for e in entries:
		if _has(zr.read_file(e), SECRET):
			leaked += 1
	zr.close()
	_ok(entries.size() >= 3 and leaked == 0, "none of the archive's %d entries holds the key" % entries.size())
	_ok(LlmWire.live_threads == 0, "no worker thread is live at the end")
	_finish()
