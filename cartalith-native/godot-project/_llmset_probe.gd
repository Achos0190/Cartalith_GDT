extends Node
## Probe for `Preferences ▸ Assistant…` (`shell/llm_settings_window.gd`), the consent card
## (`shell/llm_consent.gd`) and the local-provider GPU rule (`shell/llm_gpu_rule.gd`) --
## milestone LLM-2 of `LLM_ASSISTANT_SCOPE.md` -- inside the real shell.
##
##   desktop: godot --path . --resolution 1600x1000 _llmset_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1280x800  _llmset_probe.tscn -- --force-touch --shots <dir>
##   phone:   godot --path . --resolution 393x852   _llmset_probe.tscn -- --force-touch --shots <dir>
##
## **Windowed, never `--headless`** (`MISTAKES.md`: headless manufactures its own SCRIPT ERRORs and
## draws nothing). The arguments read are `--force-touch` and `--shots <dir>`; anything else aborts
## with exit 2 (a usage header that names a flag the body never reads is a recorded mistake).
## **Run it from a scratch copy with its own `config/name` and a wiped user dir** (`MISTAKES.md`:
## a probe that reads `user://` goes green for reasons unrelated to the code; Godot 4.7.1 has no
## `--user-data-dir`). It sets up the state it asserts on: every `[llm]` key it reads it first
## clears, and it forgets the credential it saves on every exit path.
##
## Optional environment: `CART_LLM_PROBE_KEY` (any value >= 9 characters, no spaces). When set,
## section 9 asserts the environment-variable source reports "present" with the masked tail; the
## value is never printed. Without it that section prints a `note` and skips, and says so.
##
## What it protects (the LLM-2 bars, each against the real shell):
##   1. the menu row exists, opens the window, and the window fits the viewport;
##   2. DEFAULTS: every switch off, provider/address/model empty, GPU box hidden, key source
##      environment, nothing written to `cartalith_settings.cfg` by merely opening;
##   3. disabled-reason text: Commands and Edit are disabled with a reason; dependents are disabled
##      with "Turn on Assistant enabled first" while it is off;
##   4. the gate's reasons, one at a time, in the order a user fixes them (no provider, local off,
##      consent needed, ready), and each is DISTINCT text;
##   5. the consent card: names the endpoint, says "No spend limit is enforced", has no terms link
##      for a local server and the right one for OpenAI; Esc, the close box and "Not now" record
##      nothing; only "I understand, enable" records; enabling the master switch asks first and a
##      decline leaves it off; withdrawing makes it ask again;
##   6. the GPU rule: warning with a "Detected as" line for every class, the three override values
##      against this machine's real adapters, the engine switched and RESTORED, never persisted;
##   7. the provider picker is the presets table (8 + 2 custom), picking one fills its address;
##   8. the key: a pasted key is cleared from the field on Save, the status shows only the masked
##      tail, and the key is ABSENT from `cartalith_settings.cfg`, a saved project (every entry),
##      an exported PNG and `godot.log`; Forget removes it; with no store Save is disabled with a
##      reason;
##   9. the environment-variable source (when `CART_LLM_PROBE_KEY` is set);
##  10. touch: on tablet and phone every control is at least the density's floor, drawn inside the
##      scroller's visible rect after scrolling to it, and a REAL mouse press at the control's
##      edges (not its centre) fires it;
##  11. ZERO OUTBOUND CONNECTIONS with the feature off over a full session (generate, save, export,
##      open the Vault, open this window) -- `netstat -ano` rows for this process, against a
##      positive control (a loopback connection this probe opens is counted).
## Result: `RESULT: PASS|FAIL (<n> failed)`. Exit 0 pass, 1 fail, 2 bad argument or watchdog.

const SECRET := "sk-PROBE-SECRET-0123456789abcd"
const SECRET_TAIL := "abcd"
const OPENAI_TERMS := "https://openai.com/policies/terms-of-use"

var _fails := 0
var _mode := "desktop"
var _shots := ""
var _app: Node
var _bridge
var _win
var _cleanup_account := ""


func _ok(cond: bool, what: String) -> void:
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


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


func _shot(tag: String, show: Control = null) -> void:
	if _shots == "":
		return
	if show != null and is_instance_valid(show):
		(_win._scroll as ScrollContainer).ensure_control_visible(show)
	await _settle(3)
	var path := _shots.path_join("llm2_%s_%s.png" % [_mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


## A control by its stable name in the window's `controls` map.
func _c(name: String) -> Control:
	return _win.controls.get(name) as Control


func _text_of(name: String) -> String:
	var c := _c(name)
	if c is Label:
		return (c as Label).text
	return ""


## Writes the cfg back to the file Godot would read, then returns the whole file as text.
func _cfg_text() -> String:
	var f := FileAccess.open(DccSettings.CONFIG_PATH, FileAccess.READ)
	return f.get_as_text() if f != null else ""


## True when `needle` occurs in the bytes of `hay`.
func _contains(hay: PackedByteArray, needle: String) -> bool:
	var n := needle.to_utf8_buffer()
	if n.is_empty() or hay.size() < n.size():
		return false
	var first := n[0]
	for i in range(0, hay.size() - n.size() + 1):
		if hay[i] != first:
			continue
		var hit := true
		for j in range(1, n.size()):
			if hay[i + j] != n[j]:
				hit = false
				break
		if hit:
			return true
	return false


func _file_bytes(path: String) -> PackedByteArray:
	var f := FileAccess.open(path, FileAccess.READ)
	return f.get_buffer(f.get_length()) if f != null else PackedByteArray()


## Clears every `[llm]` key and every recorded consent, so a section starts from the shipped state.
func _reset_llm() -> void:
	DccSettings._ensure_loaded()
	if DccSettings._cfg.has_section("llm"):
		DccSettings._cfg.erase_section("llm")
		DccSettings._save()


## Reopens the window on the stored state and lets deferred rebuilds finish.
func _reopen() -> void:
	_win.open()
	await _settle(6)


## Picks provider `id` through the real picker control (the OptionButton), as a click would.
func _pick_provider(id: String) -> void:
	var ob := _c("preset") as OptionButton
	var idx := 0
	var rows: Array = _bridge.llm_presets()
	for i in rows.size():
		if String(rows[i]["id"]) == id:
			idx = i + 1
	ob.select(idx)
	ob.item_selected.emit(idx)
	await _settle(6)


## Picks a key-source row through the live picker (re-fetched: a rebuild replaces the control).
func _choose_key_source(index: int) -> void:
	var ob := _c("key_source") as OptionButton
	ob.select(index)
	ob.item_selected.emit(index)
	await _settle(6)


## Picks a GPU override row through the live picker (re-fetched: a rebuild replaces the control).
func _choose_override(index: int) -> void:
	var ob := _c("gpu_override") as OptionButton
	ob.select(index)
	ob.item_selected.emit(index)
	await _settle(4)


func _set_toggle(name: String, on: bool) -> void:
	var cb := _c(name) as CheckBox
	cb.button_pressed = on
	await _settle(6)


func _type(name: String, text: String) -> void:
	var le := _c(name) as LineEdit
	le.text = text
	le.text_submitted.emit(text)
	await _settle(4)


## The consent card currently open, as `LlmConsent.present` returned it, or `{}`.
func _card() -> Dictionary:
	return _win.last_consent


func _card_text() -> String:
	var parts: Array = []
	var c := _card()
	if c.is_empty() or not is_instance_valid(c["card"]["dialog"]):
		return ""
	_collect_text(c["card"]["dialog"], parts)
	return " ".join(parts)


func _collect_text(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is LinkButton:
		out.append((n as LinkButton).text)
	elif n is Button:
		out.append((n as Button).text)
	for ch in n.get_children(true):
		_collect_text(ch, out)


func _card_open() -> bool:
	var c := _card()
	return not c.is_empty() and is_instance_valid(c["card"]["dialog"]) \
		and not (c["card"]["dialog"] as Node).is_queued_for_deletion() and (c["card"]["dialog"] as Window).visible


func _card_button(kind: String) -> Button:
	var c := _card()
	if c.is_empty() or not is_instance_valid(c["card"]["dialog"]):
		return null
	return c["choices"].get(kind) as Button


## Closes an open card the way the user declines: its "Not now" button.
func _decline_card() -> void:
	var b := _card_button("cancel")
	if b != null:
		b.pressed.emit()
	await _settle(4)


func _agree_card() -> void:
	var b := _card_button("safe")
	if b != null:
		b.pressed.emit()
	await _settle(6)


## A real Escape key press and release through the input system.
func _press_escape() -> void:
	for pressed in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = KEY_ESCAPE
		ev.physical_keycode = KEY_ESCAPE
		ev.pressed = pressed
		Input.parse_input_event(ev)
		await get_tree().process_frame
	await _settle(6)


## A REAL mouse press and release at `local` (a point in `c`'s own viewport space, e.g. a point
## of `c.get_global_rect()`): a motion event first (hover state), then press and release, each a
## couple of frames apart, through the input system -- not `pressed.emit()`. A control inside an
## embedded window is offset by that window's position, which is added here.
func _click(c: Control, local: Vector2) -> void:
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


## The control's rect clipped to its scroller's visible rect, after scrolling it into view.
func _visible_rect(c: Control) -> Rect2:
	(_win._scroll as ScrollContainer).ensure_control_visible(c)
	await _settle(2)
	return c.get_global_rect().intersection((_win._scroll as ScrollContainer).get_global_rect())


func _floor_px() -> float:
	if _mode == "phone":
		return float(DccTheme.PHONE_TAP_MIN)
	if _mode == "tablet":
		return float(DccTheme.role_px("btn_min_h"))
	return 20.0


## Parses `netstat -ano -p TCP` and `-p TCPv6` rows for `pid`: returns {"listening": n, "other": n, "rows": [...]}.
## **Both families since LLM-3 (2026-10-25):** `-p TCP` lists IPv4 only, so a connection to `::1` (where
## `localhost` resolves first on this machine) was invisible to this check; the LLM-3 live run saw 0 rows
## for a real open socket until the IPv6 table was read too.
func _netstat_tcp(pid: int) -> Dictionary:
	var listening := 0
	var other := 0
	var rows: Array = []
	for proto in ["TCP", "TCPv6"]:
		var out: Array = []
		OS.execute("netstat", ["-ano", "-p", proto], out)
		for line in String("".join(out)).split("
"):
			var cols := line.strip_edges().split(" ", false)
			if cols.size() < 5 or cols[0] != "TCP" or int(cols[cols.size() - 1]) != pid:
				continue
			if cols[3] == "LISTENING":
				listening += 1
			else:
				other += 1
				rows.append(line.strip_edges())
	return {"listening": listening, "other": other, "rows": rows}


func _netstat_udp(pid: int) -> int:
	var out: Array = []
	OS.execute("netstat", ["-ano", "-p", "UDP"], out)
	var n := 0
	for line in String("".join(out)).split("\n"):
		var cols := line.strip_edges().split(" ", false)
		if cols.size() >= 4 and cols[0] == "UDP" and int(cols[cols.size() - 1]) == pid:
			n += 1
	return n


func _finish() -> void:
	if _cleanup_account != "":
		_bridge.llm_key_forget(_cleanup_account)
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
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)

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
	_win = _app.assistant_settings_window
	_ok(_bridge.llm_api, "the loaded library has LlmBridge (llm_api)")
	if not _bridge.llm_api or _win == null:
		_finish()
		return
	var pid := OS.get_process_id()

	# ------------------------------------------------------------------ 11a. a baseline for the socket check
	# The positive control comes first so a broken netstat parse cannot read as "0 connections".
	print("== 11a. socket check: positive control ==")
	var server := TCPServer.new()
	var port := 0
	for p in range(49000, 49100):
		if server.listen(p, "127.0.0.1") == OK:
			port = p
			break
	_ok(port != 0, "a loopback listener opened for the control")
	var client := StreamPeerTCP.new()
	client.connect_to_host("127.0.0.1", port)
	for _t in 40:
		client.poll()
		if server.is_connection_available():
			break
		await get_tree().process_frame
	var accepted := server.take_connection()
	await _settle(3)
	var ctl := _netstat_tcp(pid)
	_ok(int(ctl["other"]) >= 1, "netstat sees a non-listening TCP row for this process while a connection is open: %d" % int(ctl["other"]))
	_ok(int(ctl["listening"]) >= 1, "netstat sees the listener: %d" % int(ctl["listening"]))
	client.disconnect_from_host()
	if accepted != null:
		accepted.disconnect_from_host()
	server.stop()
	await get_tree().create_timer(0.5).timeout

	# ------------------------------------------------------------------ 1. menu row, opens, fits
	print("== 1. menu row and window ==")
	var row := _find_row(_app, "Assistant…")
	_ok(not row.is_empty(), "Preferences carries 'Assistant…'")
	if row.is_empty():
		_finish()
		return
	var pm: PopupMenu = row[0]
	var idx: int = row[1]
	pm.about_to_popup.emit()
	_ok(not pm.is_item_disabled(idx), "the row is enabled")
	var cfg_before := _cfg_text()
	pm.id_pressed.emit(pm.get_item_id(idx))
	await _settle(8)
	_ok(_win.visible, "the row's id opens the window")
	var wrect := Rect2(Vector2(_win.position), Vector2(_win.size))
	_ok(wrect.position.x >= -0.5 and wrect.position.y >= -0.5 and wrect.end.x <= vp.x + 0.5 and wrect.end.y <= vp.y + 0.5,
		"the window %s is inside the viewport %s" % [wrect, vp])
	_ok(_cfg_text().find("[llm]") < 0, "merely opening wrote no [llm] section")
	_ok(not cfg_before.contains("[llm]"), "the user dir started with no [llm] section (a clean scratch run)")
	await _shot("1_defaults")

	# ------------------------------------------------------------------ 2. defaults
	print("== 2. defaults ==")
	for k in ["enabled", "prose", "commands", "edit", "local_allowed", "remote_allowed", "ctx_entity", "ctx_selection", "ctx_notes", "ctx_params"]:
		var cb := _c(k) as CheckBox
		_ok(cb != null and not cb.button_pressed, "switch '%s' ships OFF" % k)
	_ok((_c("token_counter") as CheckBox).button_pressed, "the token counter ships visible")
	## LLM-4: the opt-in cap ships OFF (Ruling DD Q8: no enforced cap by default).
	_ok(_c("token_cap") is SpinBox and int((_c("token_cap") as SpinBox).value) == 0, "the optional token cap ships at 0 = OFF")
	_ok(int(DccSettings.llm_value("token_cap")) == 0 and int(DccSettings.llm_gate_dict().get("token_cap", -1)) == 0, "...in the stored default and in the gate dictionary the library reads")
	_ok((_c("token_cap") as SpinBox).min_value == 0.0, "...and its minimum is 0, so it can always be turned back off")
	_ok((_c("preset") as OptionButton).selected == 0, "provider ships as 'None chosen'")
	_ok((_c("preset") as OptionButton).item_count == 1 + _bridge.llm_presets().size(), "the picker is fed by the presets table (%d rows + None)" % _bridge.llm_presets().size())
	_ok(_bridge.llm_presets().size() == 10, "8 presets + 2 custom entries")
	_ok((_c("base_url") as LineEdit).text == "" and (_c("model") as LineEdit).text == "", "address and model ship empty")
	_ok((_c("key_source") as OptionButton).selected == 0, "key source ships as the environment variable")
	_ok((_c("env_var") as LineEdit).text == "CART_LLM_KEY", "the variable NAME defaults to CART_LLM_KEY")
	_ok(not _c("gpu_box").visible, "the GPU warning is hidden with no local provider selected")
	_ok(DccSettings.llm_consented_origins().is_empty(), "no consent is recorded")
	_ok(_text_of("status").contains("off") and _text_of("status").contains("Nothing is sent"),
		"status says it is off: '%s'" % _text_of("status"))

	# ------------------------------------------------------------------ 3. disabled reasons
	print("== 3. disabled-reason text ==")
	## LLM-6b made Commands usable (setting proposals); only Edit is still unavailable.
	for k in ["edit"]:
		var cb := _c(k) as CheckBox
		_ok(cb.disabled and not cb.button_pressed, "'%s' is disabled and off" % k)
		_ok(_text_of(k + "_note") == "Coming later.", "'%s' says why: '%s'" % [k, _text_of(k + "_note")])
	for k in ["prose", "commands", "local_allowed", "remote_allowed"]:
		_ok((_c(k) as CheckBox).disabled, "'%s' is disabled while the master switch is off" % k)
		_ok(_text_of(k + "_note").ends_with("Turn on Assistant enabled first."), "'%s' says why: '%s'" % [k, _text_of(k + "_note")])
	_ok((_c("base_url") as LineEdit).editable and not (_c("preset") as OptionButton).disabled,
		"provider, address, model and key fields stay editable while the master switch is off (2026-10-26: setting them up sends nothing)")
	_ok(not (_c("consent_review") as Button).visible or (_c("consent_review") as Button).disabled, "Review consent is disabled with no endpoint")
	_ok(String((_c("consent_review") as Button).tooltip_text) != "", "...and carries its reason")

	# ------------------------------------------------------------------ 4. gate reasons in order
	print("== 4. the gate's reasons ==")
	var seen := {}
	await _set_toggle("enabled", true)
	_ok(DccSettings.llm_value("enabled") == true, "the master switch stores on (no endpoint, so no consent card yet)")
	_ok(not _card_open(), "no consent card with no endpoint to name")
	seen[_text_of("status")] = true
	_ok(_text_of("status").begins_with("No provider is chosen"), "reason: '%s'" % _text_of("status"))
	await _pick_provider("ollama")
	_ok(DccSettings.llm_value("preset_id") == "ollama", "picking Ollama stores its id")
	_ok((_c("base_url") as LineEdit).text == "http://localhost:11434/v1", "...and fills its default address")
	_ok(DccSettings.llm_value("base_url") == "http://localhost:11434/v1", "...and stores it")
	seen[_text_of("status")] = true
	_ok(_text_of("endpoint").contains("same machine"), "endpoint line: '%s'" % _text_of("endpoint"))
	_ok(_text_of("status").contains("No model is set"), "reason: '%s'" % _text_of("status"))
	await _type("model", "llama3")
	seen[_text_of("status")] = true
	_ok(DccSettings.llm_value("model") == "llama3", "the model name stores (free text)")
	_ok(_text_of("status").contains("Local providers allowed"), "reason: '%s'" % _text_of("status"))
	await _set_toggle("local_allowed", true)
	seen[_text_of("status")] = true
	_ok(_text_of("status").contains("Consent needed for http://localhost:11434"), "reason: '%s'" % _text_of("status"))
	_ok(_text_of("consent_state").contains("No consent recorded"), "consent line: '%s'" % _text_of("consent_state"))
	_ok(seen.size() == 4, "four distinct reasons, got %d" % seen.size())
	await _shot("4_consent_needed")

	# ------------------------------------------------------------------ 5. consent card
	print("== 5. consent ==")
	(_c("consent_review") as Button).pressed.emit()
	await _settle(8)
	_ok(_card_open(), "Review consent opens the card")
	var t := _card_text()
	_ok(t.contains("http://localhost:11434") and t.contains("same machine"), "the card names the endpoint and its class")
	_ok(t.contains("No spend limit is enforced"), "the card says no spend limit is enforced")
	_ok(t.contains("Never sent") and t.contains("machine name"), "the card says what is never sent")
	_ok(t.contains("the text you type") and t.contains("none"), "the card lists what is sent, context 'none' by default")
	_ok(_card()["terms"] == null, "a local server gets no terms link (none is invented)")
	_ok(not t.contains("commands the assistant may propose"), "no claim about commands while Commands is off")
	var cardrect := Rect2((_card()["card"]["dialog"] as Window).position, (_card()["card"]["dialog"] as Window).size)
	_ok(Rect2(Vector2.ZERO, vp).grow(0.5).encloses(cardrect), "the card %s is inside the viewport" % cardrect)
	for b in ["cancel", "safe"]:
		var bt := _card_button(b)
		_ok(bt != null and bt.get_global_rect().size.y >= (44.0 if _mode == "phone" else 20.0), "card button '%s' is tall enough: %s" % [b, bt.get_global_rect().size if bt else "-"])
	await _shot("5_consent_card")
	# Esc
	await _press_escape()
	_ok(not _card_open() and DccSettings.llm_consented_origins().is_empty(), "Esc closes the card and records nothing")
	(_c("consent_review") as Button).pressed.emit()
	await _settle(6)
	((_card()["card"]["close"]) as Button).pressed.emit()
	await _settle(6)
	_ok(not _card_open() and DccSettings.llm_consented_origins().is_empty(), "the close box records nothing")
	(_c("consent_review") as Button).pressed.emit()
	await _settle(6)
	await _decline_card()
	_ok(not _card_open() and DccSettings.llm_consented_origins().is_empty(), "'Not now' records nothing")
	_ok(_text_of("status").contains("Consent needed"), "...and the gate still refuses: '%s'" % _text_of("status"))
	(_c("consent_review") as Button).pressed.emit()
	await _settle(6)
	await _agree_card()
	_ok(not _card_open() and DccSettings.llm_consented_origins() == PackedStringArray(["http://localhost:11434"]),
		"'I understand, enable' records exactly the endpoint: %s" % DccSettings.llm_consented_origins())
	_ok(_text_of("status").begins_with("Ready: requests go to http://localhost:11434"), "status: '%s'" % _text_of("status"))
	_ok(_text_of("consent_state").begins_with("Consent given for http://localhost:11434"), "consent line: '%s'" % _text_of("consent_state"))
	# The gate's own answer for each capability, through the Rust function.
	var gd := DccSettings.llm_gate_dict()
	_ok(bool(_bridge.llm_authorize(gd, "", DccSettings.llm_consented_origins()).get("ok", false)), "gate: ready (status check)")
	_ok(String(_bridge.llm_authorize(gd, "prose", DccSettings.llm_consented_origins()).get("code", "")) == "capability_off", "gate: prose refused while its switch is off")
	# Withdraw makes it ask again.
	(_c("consent_withdraw") as Button).pressed.emit()
	await _settle(4)
	_ok(DccSettings.llm_consented_origins().is_empty() and _text_of("status").contains("Consent needed"), "Withdraw all consent: the next request asks again")
	# Master switch asks first; a decline leaves it off.
	await _set_toggle("enabled", false)
	_ok(DccSettings.llm_value("enabled") == false, "master off stores off")
	await _set_toggle("enabled", true)
	_ok(_card_open(), "turning the master switch ON with a ready endpoint opens the consent card first")
	_ok(DccSettings.llm_value("enabled") == false, "...and nothing is enabled until the user agrees")
	await _decline_card()
	_ok(DccSettings.llm_value("enabled") == false and not (_c("enabled") as CheckBox).button_pressed,
		"declining leaves the master switch OFF (stored and shown)")
	await _set_toggle("enabled", true)
	await _agree_card()
	_ok(DccSettings.llm_value("enabled") == true and (_c("enabled") as CheckBox).button_pressed, "agreeing turns it on")
	_ok(_text_of("status").begins_with("Ready"), "status: '%s'" % _text_of("status"))

	# ------------------------------------------------------------------ 6. GPU rule
	print("== 6. the GPU rule ==")
	_ok(_c("gpu_box").visible, "the warning shows for a local preset")
	_ok(_text_of("gpu_detected") == "Detected as: same machine.", "detected line: '%s'" % _text_of("gpu_detected"))
	_ok(_text_of("gpu_headline").contains("compete for the GPU"), "headline: '%s'" % _text_of("gpu_headline"))
	_ok(_text_of("gpu_effect").contains("display adapter is not changed"), "the display adapter is not promised")
	var devs: Array = _bridge.gpu_devices() if _bridge.gpu_api else []
	var integrated := ""
	for d in devs:
		if String(d["kind"]) == "integrated" and integrated == "":
			integrated = String(d["key"])
	print("  note  this machine's adapters: %s" % str(devs.map(func(d): return "%s/%s" % [d["kind"], d["name"]])))
	var user_keys: PackedStringArray = DccSettings.gpu_devices()
	var sel_before: PackedStringArray = _bridge.gpu_selected_devices()
	_ok((_c("gpu_override") as OptionButton).selected == 0, "override ships as Automatic")
	if integrated != "":
		_ok(LlmGpuRule.is_applied() and _bridge.gpu_selected_devices() == PackedStringArray([integrated]),
			"Automatic + same machine: the compute path is on the integrated GPU (%s)" % integrated)
		_ok(_text_of("gpu_effect").contains("uses the integrated GPU"), "effect says what happens: '%s'" % _text_of("gpu_effect").substr(0, 70))
	else:
		_ok(not LlmGpuRule.is_applied() and _text_of("gpu_effect").contains("No integrated GPU"), "no integrated adapter: nothing is switched and it says so")
	_ok(DccSettings.gpu_devices() == user_keys, "the user's persisted device choice is untouched while switched")
	# Never change -> restored.
	await _choose_override(2)
	_ok(DccSettings.llm_value("gpu_override") == "never", "override 'Never change' stores")
	_ok(not LlmGpuRule.is_applied() and _bridge.gpu_selected_devices() == user_keys, "Never change: the user's own choice is back (%s)" % str(_bridge.gpu_selected_devices()))
	# LAN address: Automatic does not switch, Always does.
	await _choose_override(0)
	await _pick_provider("custom_openai")
	await _type("base_url", "http://192.168.1.9:1234/v1")
	await _type("model", "m")
	_ok(_text_of("gpu_detected") == "Detected as: local network.", "LAN detected line: '%s'" % _text_of("gpu_detected"))
	_ok(not LlmGpuRule.is_applied() and _bridge.gpu_selected_devices() == user_keys, "Automatic + LAN: nothing switched")
	await _choose_override(1)
	_ok(DccSettings.llm_value("gpu_override") == "always", "override 'Always low-power GPU' stores")
	if integrated != "":
		_ok(LlmGpuRule.is_applied() and _bridge.gpu_selected_devices() == PackedStringArray([integrated]), "Always + LAN: switched to the integrated GPU")
	_ok(_text_of("gpu_detected") == "Detected as: local network.", "...and the detected line still says what the address is")
	await _shot("6_gpu_always_lan", _c("gpu_override"))
	# Deselect the provider: restored.
	await _set_toggle("enabled", false)
	_ok(not LlmGpuRule.is_applied() and _bridge.gpu_selected_devices() == user_keys, "master off: the previous selection is restored")
	_ok(DccSettings.gpu_devices() == user_keys, "the persisted choice is still the user's own after the whole rule ran")
	await _choose_override(0)
	# Every class has a warning with a detected line (pure plan over fake adapters).
	var fake: Array = [{"key": "d0", "kind": "discrete"}, {"key": "i0", "kind": "integrated"}]
	var cases := {"http://localhost:1/v1": "same machine", "http://10.1.2.3/v1": "local network",
		"https://api.openai.com/v1": "remote", "http://myhost.local:1/v1": "unknown"}
	for u in cases:
		var pl: Dictionary = _bridge.llm_gpu_plan(u, true, "automatic", fake)
		_ok(bool(pl.get("warn", false)) and String(pl["detected"]) == "Detected as: %s." % cases[u], "plan for %s: %s" % [u, pl.get("detected", "?")])
		_ok(bool(pl["switch"]) == (cases[u] == "same machine"), "...switches only for the same machine")
		_ok((String(pl.get("target", "")) == "i0") == bool(pl["switch"]), "...and targets the integrated adapter, never the discrete one")
	var bad: Dictionary = _bridge.llm_gpu_plan("http://localhost/v1", true, "bogus", fake)
	_ok(not bool(bad.get("ok", true)), "an unknown override id is an error, not a guess")

	# ------------------------------------------------------------------ 7. provider picker
	print("== 7. the provider picker ==")
	_reset_llm()
	await _reopen()
	var names: Array = []
	var po := _c("preset") as OptionButton
	for j in po.item_count:
		names.append(po.get_item_text(j))
	print("  note  picker: %s" % str(names))
	var rows: Array = _bridge.llm_presets()
	for j in rows.size():
		_ok(po.get_item_text(j + 1) == String(rows[j]["name"]), "row %d is '%s'" % [j + 1, rows[j]["name"]])
	var by_id := {}
	for r in rows:
		by_id[String(r["id"])] = r
	_ok(by_id["openai"]["base_url"] == "https://api.openai.com/v1" and not bool(by_id["openai"]["is_local"]), "OpenAI is remote with its https address")
	_ok(by_id["llama_cpp"]["base_url"] == "http://localhost:9931/v1", "llama.cpp keeps the verified port 9931")
	_ok(by_id["openai"].has("terms_url") and not by_id["ollama"].has("terms_url"), "terms links exist for hosted presets only")
	await _set_toggle("enabled", true)
	await _pick_provider("openai")
	_ok((_c("base_url") as LineEdit).text == "https://api.openai.com/v1", "picking OpenAI fills its address")
	await _type("base_url", "https://example.org/v1")
	_ok(DccSettings.llm_value("base_url") == "https://example.org/v1", "the user's own address wins and is stored")
	await _type("base_url", "https://user:hunter2@example.org/v1")
	_ok(_text_of("endpoint").begins_with("This address cannot be used") and not _text_of("endpoint").contains("hunter2"),
		"userinfo is refused and never echoed: '%s'" % _text_of("endpoint"))
	await _type("base_url", "http://api.example.com/v1")
	await _type("model", "gpt-x")
	await _set_toggle("remote_allowed", true)
	_ok(_text_of("status").contains("https://"), "a remote http:// address is refused: '%s'" % _text_of("status"))
	await _type("base_url", "https://api.openai.com/v1")
	await _shot("7_provider", _c("endpoint"))

	# ------------------------------------------------------------------ 8. the key
	print("== 8. the key ==")
	# The settings file's setter is an allow-list: it cannot be made to hold a key by naming one.
	var cfg_pre := _cfg_text()
	_ok(not DccSettings.set_llm_value("api_key", SECRET), "set_llm_value refuses a key name outside the allow-list")
	_ok(not DccSettings.set_llm_value("model", 42), "set_llm_value refuses a value of the wrong type")
	_ok(not DccSettings.set_llm_value("token_cap", "many") and not DccSettings.set_llm_value("token_cap", 1.5), "...and a token cap that is not an integer")
	_ok(DccSettings.llm_value("api_key") == null, "an unknown [llm] key reads as null, never a value")
	_ok(_cfg_text() == cfg_pre and not _cfg_text().contains("api_key"), "...and the refused writes changed nothing on disk")
	var verdict_0: Dictionary = _bridge.llm_authorize(DccSettings.llm_gate_dict(), "", PackedStringArray())
	_ok(DccSettings.set_llm_value("token_cap", 5000) and int(DccSettings.llm_value("token_cap")) == 5000 and int(DccSettings.llm_gate_dict()["token_cap"]) == 5000, "a cap of 5000 stores and reaches the gate dictionary")
	_ok(_bridge.llm_authorize(DccSettings.llm_gate_dict(), "", PackedStringArray()) == verdict_0, "...and a cap does not change the gate's own verdict (it is the shell's own check, in `prepare`)")
	_ok(DccSettings.set_llm_value("token_cap", 0) and int(DccSettings.llm_value("token_cap")) == 0, "a cap set back to 0 is off again")
	for k in DccSettings.LLM_DEFAULTS:
		_ok(not String(k).contains("secret") and k != "key" and k != "api_key" and k != "token", "allow-list entry '%s' is not a secret slot" % k)
	_cleanup_account = "openai"
	await _type("env_var", "CART_LLM_PROBE_UNSET_VAR")
	_ok(_text_of("status").contains("environment variable CART_LLM_PROBE_UNSET_VAR is not set"), "env source, nothing set: '%s'" % _text_of("status"))
	_ok(_text_of("key_status").contains("environment variable CART_LLM_PROBE_UNSET_VAR is not set"), "key line says so: '%s'" % _text_of("key_status"))
	var store_ok := bool(_bridge.llm_key_status("env", "X", "x").get("store_available", false))
	var ks := _c("key_source") as OptionButton
	_ok(ks.is_item_disabled(1) == (not store_ok), "the OS-store source is %s" % ("offered" if store_ok else "disabled with '(unavailable here)'"))
	if store_ok:
		_bridge.llm_key_forget("openai")
		await _choose_key_source(1)
		_ok(DccSettings.llm_value("key_source") == "store", "key source 'store' stores")
		_ok(_text_of("status").contains("No key is saved in the OS credential store"), "store empty: '%s'" % _text_of("status"))
		var kf := _c("key_field") as LineEdit
		_ok(kf != null and kf.secret, "the key field is a secret field")
		kf.text = SECRET
		(_c("key_save") as Button).pressed.emit()
		await _settle(6)
		kf = _c("key_field") as LineEdit
		_ok(kf.text == "", "the field is CLEARED the moment Save is pressed")
		_ok(_text_of("key_status").contains("****" + SECRET_TAIL) and not _text_of("key_status").contains(SECRET),
			"status shows only the masked tail: '%s'" % _text_of("key_status"))
		_ok(not _text_of("status").contains(SECRET) and not _text_of("key_message").contains(SECRET), "no label holds the key")
		_ok(_all_text_free_of(_win, SECRET), "no control anywhere in the window holds the key")
		# Right after Save, before anything else can rewrite the file: the key is not on disk.
		_ok(not _cfg_text().contains(SECRET), "cartalith_settings.cfg holds no key immediately after Save")
		_ok(_text_of("key_message") == "Key saved in the OS credential store.", "message: '%s'" % _text_of("key_message"))
		var ps: Dictionary = _bridge.llm_key_status("store", "X", "openai")
		_ok(bool(ps["present"]) and ps["masked"] == "****" + SECRET_TAIL and not str(ps).contains(SECRET), "a re-read from Credential Manager round-trips, masked: %s" % str(ps))
		# Bad keys are refused with a static reason that does not echo them.
		kf.text = "two words"
		(_c("key_save") as Button).pressed.emit()
		await _settle(4)
		_ok(_text_of("key_message").begins_with("That is not a usable key") and not _text_of("key_message").contains("two words"), "an unusable key is refused without an echo")
		# Consent for the hosted endpoint, with the terms LINK.
		(_c("consent_review") as Button).pressed.emit()
		await _settle(8)
		_ok(_card_open() and _card_text().contains("https://api.openai.com"), "the hosted card names https://api.openai.com")
		var tl := _card()["terms"] as LinkButton
		_ok(tl != null and tl.uri == OPENAI_TERMS, "the card links the provider's terms: %s" % (tl.uri if tl else "none"))
		await _shot("8_consent_hosted")
		await _agree_card()
		_ok(_text_of("status").begins_with("Ready: requests go to https://api.openai.com"), "hosted + key + consent: '%s'" % _text_of("status"))
		await _shot("8_key_saved", _c("key_message"))

		# --- the key is absent from every place it could leak ---
		print("== 8b. the key is absent from cfg, a save, an export and the log ==")
		await get_tree().process_frame
		var cfg := _cfg_text()
		_ok(cfg.find("[llm]") >= 0, "the [llm] section was written, so the grep is not over an empty file")
		_ok(cfg.find(SECRET) < 0, "cartalith_settings.cfg holds no key")
		_ok(not _contains(_file_bytes(DccSettings.CONFIG_PATH), SECRET), "byte search of the cfg file: absent")
		var fails_before := _fails
		await _leak_checks(SECRET)
		_ok(_fails == fails_before, "save / export / log grep done")
		# Forget removes it.
		(_c("key_forget") as Button).pressed.emit()
		await _settle(4)
		_ok(not bool(_bridge.llm_key_status("store", "X", "openai")["present"]), "Forget key removes it from the credential store")
		_ok(_text_of("status").contains("No key is saved"), "...and the gate says so: '%s'" % _text_of("status"))
	else:
		_ok((_c("key_source") as OptionButton).get_item_text(1).ends_with("(unavailable here)"), "no store: the source says '(unavailable here)'")
		var r: Dictionary = _bridge.llm_key_save("openai", SECRET)
		_ok(not bool(r["ok"]) and String(r["error"]).contains("never saves a key to a file") and not str(r).contains(SECRET), "no store: Save is refused with a reason")
		_ok(_c("key_field") == null, "...and there is no key field to type into")

	# ------------------------------------------------------------------ 9. env source
	print("== 9. the environment-variable source ==")
	if OS.has_environment("CART_LLM_PROBE_KEY"):
		var envv := OS.get_environment("CART_LLM_PROBE_KEY")
		await _choose_key_source(0)
		await _type("env_var", "CART_LLM_PROBE_KEY")
		var tail := envv.substr(envv.length() - 4)
		_ok(_text_of("key_status").contains("****" + tail) and not _text_of("key_status").contains(envv), "env key present, masked: '%s'" % _text_of("key_status"))
		_ok(_text_of("status").begins_with("Ready"), "gate ready with an env key: '%s'" % _text_of("status"))
		_ok(_all_text_free_of(_win, envv), "no control holds the env key")
		_ok(_cfg_text().find(envv) < 0, "the cfg holds only the variable NAME")
	else:
		print("  note  CART_LLM_PROBE_KEY not set; the env-present case is covered by cartalith-llm's resolve_key tests only")

	# ------------------------------------------------------------------ 10. touch and reachability
	print("== 10. every control: size, drawn rect, real press at the edges ==")
	_reset_llm()
	await _reopen()
	await _set_toggle("enabled", true)
	await _pick_provider("openai")
	await _set_toggle("remote_allowed", true)
	if store_ok:
		await _choose_key_source(1)
	var floor_px := _floor_px()
	var checked := 0
	for name in _win.controls:
		var c: Control = _win.controls[name]
		if not (c is BaseButton or c is LineEdit or c is OptionButton):
			continue
		if not c.is_visible_in_tree():
			continue
		var vr := await _visible_rect(c)
		var full := c.get_global_rect()
		checked += 1
		_ok(vr.size.x >= full.size.x - 1.0 and vr.size.y >= full.size.y - 1.0, "'%s' is fully drawn inside the scroller: %s of %s" % [name, vr.size, full.size])
		# ... and inside the WINDOW's own width: a body wider than the screen keeps the scroller wide
		# too, so the intersection above passes while the control hangs off the glass.
		var content_w: float = float(_win.size.x) / maxf(_win.content_scale_factor, 0.001)
		_ok(full.position.x >= -0.5 and full.end.x <= content_w + 0.5, "'%s' x-extent %.0f..%.0f is inside the window's %.0f px width" % [name, full.position.x, full.end.x, content_w])
		if _mode != "desktop":
			_ok(full.size.y >= floor_px - 0.5, "'%s' is at least %.0f tall (touch floor): %.1f" % [name, floor_px, full.size.y])
			if c is BaseButton or c is OptionButton:
				# The width floor: 44 on a phone (the LLM-2 bar). On a tablet the LLM-0 mock draws "40+ px
				# targets" and the shell's own switch is 40 wide by the 44 height floor, so 40 is the figure.
				var wfloor := floor_px if _mode == "phone" else 40.0
				_ok(full.size.x >= wfloor - 0.5, "'%s' is at least %.0f wide: %.1f" % [name, wfloor, full.size.x])
	_ok(checked >= 14, "the sweep covered %d controls (silently-empty guard)" % checked)
	# Real presses at the EDGES (not the centre: a centre press cannot see a narrow hit area) of the
	# master switch and of the token-counter switch, each re-fetched because a rebuild replaces them.
	var edge_hits := 0
	var counter_hits := 0
	for fx in [0.04, 0.96]:
		_reset_llm()
		await _reopen()
		var master := _c("enabled") as CheckBox
		var mr := await _visible_rect(master)
		await _click(master, Vector2(mr.position.x + mr.size.x * fx, mr.position.y + mr.size.y * 0.5))
		var fired: bool = DccSettings.llm_value("enabled") == true
		print("  note  master switch real press at x=%.0f%% of its rect fired=%s" % [fx * 100.0, str(fired)])
		if fired:
			edge_hits += 1
		var tc := _c("token_counter") as CheckBox
		var tr := await _visible_rect(tc)
		var was: bool = DccSettings.llm_value("token_counter")
		await _click(tc, Vector2(tr.position.x + tr.size.x * fx, tr.position.y + tr.size.y * 0.5))
		if DccSettings.llm_value("token_counter") != was:
			counter_hits += 1
	_ok(edge_hits == 2, "a real press at BOTH edges of the master switch fires it: %d of 2" % edge_hits)
	_ok(counter_hits == 2, "a real press at BOTH edges of the token-counter switch flips it: %d of 2" % counter_hits)
	await _shot("10_touch")

	# ------------------------------------------------------------------ 11b. zero outbound with the feature off
	print("== 11b. zero outbound connections, feature off, full session ==")
	_reset_llm()
	_ok(DccSettings.llm_value("enabled") == false, "the feature is off for the session")
	var before := _netstat_tcp(pid)
	var udp_before := _netstat_udp(pid)
	if _mode == "desktop":
		_bridge.generate({"seed": 9137, "width_km": 1200.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true})
		while _bridge.generating:
			await get_tree().create_timer(0.25).timeout
		await _settle(10)
		if _app.open_project_dialog:
			_app.open_project_dialog.hide()
		var sdir := ProjectSettings.globalize_path("user://_llmset_probe")
		DirAccess.make_dir_recursive_absolute(sdir)
		_ok(_bridge.save_project(sdir.path_join("session.zip")), "session: generate then SAVE a project")
		var d: AcceptDialog = _app.open_export_dialog()
		await _settle(4)
		d.set_destination(sdir.path_join("session.png"))
		d.start_export()
		var res: Dictionary = await d.export_finished
		_ok(bool(res.get("ok", false)), "session: EXPORT a 2048 PNG: %s" % str(res.get("ok", res)))
		d.hide()
		_app.open_vault_browse()
		await _settle(12)
		_ok(_app.vault_window.visible, "session: open the Vault")
		_app.vault_window.hide()
		_win.open()
		await _settle(6)
		_win.hide()
		await get_tree().create_timer(1.0).timeout
	else:
		print("  note  session (generate/save/export/Vault) runs in the desktop pass; this pass opens only the window")
		_win.open()
		await _settle(6)
		_win.hide()
	var after := _netstat_tcp(pid)
	var udp_after := _netstat_udp(pid)
	_ok(int(before["other"]) == 0 and int(after["other"]) == 0,
		"0 non-listening TCP connections for this process before and after the session (before %d, after %d) %s" % [int(before["other"]), int(after["other"]), str(after["rows"])])
	_ok(udp_after == udp_before, "no UDP socket appeared (before %d, after %d)" % [udp_before, udp_after])

	_finish()


## True when no Label, LinkButton, Button or LineEdit under `n` holds `secret`.
func _all_text_free_of(n: Node, secret: String) -> bool:
	if n is Label and (n as Label).text.contains(secret):
		return false
	if n is LineEdit and ((n as LineEdit).text.contains(secret) or (n as LineEdit).placeholder_text.contains(secret)):
		return false
	if n is Button and (n as Button).text.contains(secret):
		return false
	if n is Control and (n as Control).tooltip_text.contains(secret):
		return false
	for c in n.get_children(true):
		if not _all_text_free_of(c, secret):
			return false
	return true


## The leak greps over a saved project (every zip entry), an exported PNG and godot.log. Needs a world;
## on tablet and phone passes (no world) the greps over the cfg and the log still run.
func _leak_checks(secret: String) -> void:
	var sdir := ProjectSettings.globalize_path("user://_llmset_probe")
	DirAccess.make_dir_recursive_absolute(sdir)
	if _bridge.has_world:
		var zpath := sdir.path_join("leak.zip")
		_ok(_bridge.save_project(zpath), "a project saved while the key is stored")
		var zr := ZIPReader.new()
		_ok(zr.open(zpath) == OK, "the saved project opens as a zip")
		var entries := zr.get_files()
		var leaked := false
		for e in entries:
			if _contains(zr.read_file(e), secret):
				leaked = true
		_ok(entries.size() >= 5 and not leaked, "%d zip entries searched, key absent" % entries.size())
		zr.close()
		_ok(not _contains(_file_bytes(zpath), secret), "the raw zip bytes hold no key")
	else:
		print("  note  no world in this pass; the save and export greps run in the desktop pass")
	var png := sdir.path_join("session.png")
	if FileAccess.file_exists(png):
		_ok(not _contains(_file_bytes(png), secret), "an exported PNG holds no key")
	var log_path := ProjectSettings.globalize_path("user://logs/godot.log")
	var lb := _file_bytes(log_path)
	_ok(lb.size() > 100, "godot.log was readable (%d bytes), so the grep is not over nothing" % lb.size())
	_ok(not _contains(lb, secret), "godot.log holds no key")
