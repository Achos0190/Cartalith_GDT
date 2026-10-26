extends Node
## Probe for the ASSISTANT PROFILES (`shell/llm_profiles.gd`, `shell/llm_profile_switch.gd`, their homes in
## the settings window, the panel header and the Assistant chip's menu) -- `LLM_ASSISTANT_SCOPE.md` s.5g,
## Ruling DQ, inside the real shell. Two phases, because "persists across a restart" needs two processes:
##
##   desktop: godot --path . --resolution 1600x1000 _llmprofiles_probe.tscn -- --shots <dir>
##            godot --path . --resolution 1600x1000 _llmprofiles_probe.tscn -- --phase 2     (SAME user dir)
##   tablet:  godot --path . --resolution 1280x800  _llmprofiles_probe.tscn -- --force-touch --shots <dir>
##   phone:   godot --path . --resolution 393x852   _llmprofiles_probe.tscn -- --force-touch --shots <dir>
##
## **Windowed, never `--headless`**, from a scratch copy with its own `config/name` and a wiped user dir
## before phase 1 (`MISTAKES.md`); phase 2 must NOT wipe it. Arguments read: `--force-touch`, `--shots <dir>`
## and `--phase <1|2>`; anything else aborts with exit 2. **No mock model and no model server**: the shell is
## driven through its real controls, and what would reach the wire is read from `LlmStream.prepare`, which
## builds the request bytes without sending them. It dials nothing: the settings window stays hidden
## while the master switch is on, so its model listing never runs.
##
## **Key safety.** The probe never writes under the built-in profiles' accounts (`lm_studio`, `anthropic`):
## those may hold the owner's real key. Every key it saves is under a `pf_*` account of a profile it created,
## and it forgets each on every exit path. It sets `CART_PROFILES_PROBE_KEYVAR` itself (the secret is never
## printed) and clears it on exit.
##
## What it protects (Ruling DQ's rules, each against the real shell):
##   1. FRESH: opening the window and the panel writes NOTHING; the switch reads "Custom settings" and lists
##      Local and External; the list buttons that need an active profile are disabled WITH a reason.
##   2. APPLY AT ONCE, master off: switching in the settings window writes provider, address, model,
##      thinking, ticks, cap, retries and key account in one step; the master switch, Commands, Edit and the
##      provider switches are untouched; no consent card, no socket attempt, a forced Send is refused.
##   3. CONSENT FIRST, master on: a switch to an endpoint with no consent shows the card BEFORE anything can
##      be sent (the gate answers `consent_needed` while it is open), a decline restores every value, an agree
##      records consent for that origin only.
##   4. SEPARATE PER PROFILE: ticks, reply cap and model are each profile's own; a switch swaps them all and a
##      switch back restores them. The request `LlmStream.prepare` builds carries the profile's model, cap
##      and (Local) `reasoning_effort: none`, and External's carries neither.
##   5. KEYS: one credential entry per profile; a profile's text holds no key; a duplicate gets its own
##      account and no key; the key is in neither the settings file, a saved world nor the log.
##   6. DELETE: a card offers "Delete, forget key" and "Delete, keep key"; Cancel and Esc change
##      nothing; neither answer removes the key unless it says so; the built-ins cannot be deleted.
##   7. CAPS AND NAMES: 16 profiles at most, 40-character names, no duplicate names, a duplicate is a copy.
##   8. HTTPS: a remote profile over http:// is switched to with its reason and the gate refuses it.
##   9. DEFAULT AND RESTORE: the default is used when the last used profile is gone; restore at startup
##      applies the last used (phase 2 proves it across a real process restart).
##  10. CHIP MENU: right-clicking the Assistant chip lists the profiles, the active one checked, and a pick
##      applies (desktop and tablet; the phone has no chip menu).
##  11. A WORLD `.zip` holds none of it (every entry and the raw bytes), with a positive control.
##  12. DENSITY: every control meets the density floor and sits inside its window; on a phone Send and Close
##      stay on screen with the header switch added.
## Result: `RESULT: PASS|FAIL (<n> failed)`. Exit 0 pass, 1 fail, 2 bad argument or watchdog.

const SECRET := "sk-PROFILES-PROBE-SECRET-0123456789wxyz"
const KEYVAR := "CART_PROFILES_PROBE_KEYVAR"
const LOCAL_URL := "http://127.0.0.1:1234/v1"
const EXTERNAL_URL := "https://api.anthropic.com/v1"
## A profile name nothing else in a world would contain, so the zip grep is meaningful.
const WORK_NAME := "ZZ Probe Work Profile"

var _fails := 0
var _mode := "desktop"
var _shots := ""
var _phase := 1
var _app: Node
var _bridge
var _set
var _panel
var _accounts: Array = []


func _ok(cond: bool, what: String) -> void:
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _c(name: String) -> Control:
	return _set.controls.get(name) as Control


func _p(name: String) -> Control:
	return _panel.controls.get(name) as Control


func _shot(tag: String) -> void:
	if _shots == "":
		return
	await _settle(3)
	var path := _shots.path_join("llmprof_%s_%s.png" % [_mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


func _cfg_text() -> String:
	var f := FileAccess.open(DccSettings.CONFIG_PATH, FileAccess.READ)
	return f.get_as_text() if f != null else ""


func _file_bytes(path: String) -> PackedByteArray:
	var f := FileAccess.open(path, FileAccess.READ)
	return f.get_buffer(f.get_length()) if f != null else PackedByteArray()


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


## Clears every `[llm]` and `[llm_profiles]` key, so a section starts from the shipped state.
func _reset_all() -> void:
	DccSettings._ensure_loaded()
	for sec in ["llm", "llm_profiles"]:
		if DccSettings._cfg.has_section(sec):
			DccSettings._cfg.erase_section(sec)
	DccSettings._save()


func _live() -> Dictionary:
	return LlmProfiles.live_values()


func _row(id: String) -> Dictionary:
	return LlmProfiles.find(LlmProfiles.load_set(), id)


## The dropdown's visible item texts.
func _items(ob: OptionButton) -> Array:
	var out: Array = []
	for i in ob.item_count:
		out.append(ob.get_item_text(i))
	return out


## Picks the row whose text begins with `prefix` through the real OptionButton, as a click would.
func _pick(ob: OptionButton, prefix: String) -> bool:
	for i in ob.item_count:
		if ob.get_item_text(i).begins_with(prefix):
			ob.select(i)
			ob.item_selected.emit(i)
			await _settle(6)
			return true
	return false


func _card() -> Dictionary:
	return LlmProfiles.last_consent


func _card_open() -> bool:
	var c := _card()
	return not c.is_empty() and is_instance_valid(c["card"]["dialog"]) \
		and not (c["card"]["dialog"] as Node).is_queued_for_deletion() and (c["card"]["dialog"] as Window).visible


func _collect_text(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
	for ch in n.get_children(true):
		_collect_text(ch, out)


func _card_text(card: Dictionary) -> String:
	var parts: Array = []
	if card.is_empty() or not is_instance_valid(card["card"]["dialog"]):
		return ""
	_collect_text(card["card"]["dialog"], parts)
	return " ".join(parts)


func _press_card(card: Dictionary, kind: String) -> void:
	var b := card["choices"].get(kind) as Button
	if b != null:
		b.pressed.emit()
	await _settle(6)


func _press_escape() -> void:
	for pressed in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = KEY_ESCAPE
		ev.physical_keycode = KEY_ESCAPE
		ev.pressed = pressed
		Input.parse_input_event(ev)
		await get_tree().process_frame
	await _settle(6)


func _netstat_other(pid: int) -> int:
	var n := 0
	for proto in ["TCP", "TCPv6"]:
		var out: Array = []
		OS.execute("netstat", ["-ano", "-p", proto], out)
		for line in String("".join(out)).split("\n"):
			var cols := line.strip_edges().split(" ", false)
			if cols.size() >= 5 and cols[0] == "TCP" and int(cols[cols.size() - 1]) == pid and cols[3] != "LISTENING":
				n += 1
	return n


func _forget_all() -> void:
	for a in _accounts:
		_bridge.llm_key_forget(String(a))


func _finish() -> void:
	_forget_all()
	OS.unset_environment(KEYVAR)
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
		elif a == "--phase" and i + 1 < args.size():
			i += 1
			_phase = int(args[i])
		else:
			print("ABORT: unrecognised argument %s -- this probe reads --force-touch, --shots <dir> and --phase <1|2> only" % a)
			get_tree().quit(2)
			return
		i += 1
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	## Unset: the master-off phase must not have a key to find, and the hosted gate is exercised only where
	## a step sets it.
	OS.unset_environment(KEYVAR)
	if _phase == 1:
		_reset_all()
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_mode = "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	var vp := get_viewport().get_visible_rect().size
	print("== mode %s, viewport %s, phase %d ==" % [_mode, vp, _phase])
	_bridge = _app.bridge
	_set = _app.assistant_settings_window
	_panel = _app.assistant_window
	_ok(_bridge.llm_api and _set != null and _panel != null, "the library has LlmBridge and the shell has both windows")
	_ok(LlmProfiles.available(), "LlmProfiles is installed on the bridge at boot")
	if not _bridge.llm_api or _set == null or _panel == null or not LlmProfiles.available():
		_finish()
		return
	if _phase == 2:
		await _phase2()
		return
	await _phase1(vp)


# ---------------------------------------------------------------------------------------------------
# Phase 1
# ---------------------------------------------------------------------------------------------------

func _phase1(vp: Vector2) -> void:
	var pid := OS.get_process_id()

	# ------------------------------------------------------------------ 1. fresh
	print("== 1. fresh: nothing written by opening, the switch, the disabled buttons ==")
	var cfg0 := _cfg_text()
	_ok(not cfg0.contains("[llm_profiles]") and not cfg0.contains("[llm]"), "the user dir starts with no [llm] or [llm_profiles] section")
	_set.open()
	await _settle(8)
	_ok(_set.visible, "the settings window opens")
	_ok(not _cfg_text().contains("[llm_profiles]"), "opening the settings window wrote no [llm_profiles] section")
	var opt := _c("profile_option") as OptionButton
	_ok(opt != null, "the settings window has the profile switch")
	if opt == null:
		_finish()
		return
	_ok(_items(opt) == ["Custom settings", "Local · default", "External"], "the switch lists Custom settings, Local (the default) and External: %s" % str(_items(opt)))
	_ok(opt.selected == 0 and opt.is_item_disabled(0), "it reads 'Custom settings' (not selectable as a target) with no profile active")
	for k in ["profile_duplicate", "profile_rename", "profile_delete", "profile_default", "profile_reset"]:
		var b := _c(k) as Button
		_ok(b.disabled and b.tooltip_text.begins_with("No profile is active"), "'%s' is disabled with its reason: '%s'" % [k, b.tooltip_text.left(40)])
	_ok(not (_c("profile_new") as Button).disabled, "New is enabled")
	_ok(LlmProfiles.active_id() == "", "no profile is active")
	## The shipped profiles, read back across the bridge as LITERALS (not the constants they were built from).
	var shipped: Dictionary = _bridge.llm_profiles_parse("", false)
	var loc: Dictionary = LlmProfiles.find(shipped, "local")
	var ext: Dictionary = LlmProfiles.find(shipped, "external")
	_ok(String(loc["name"]) == "Local" and String(loc["live"]["preset_id"]) == "lm_studio" and String(loc["live"]["base_url"]) == LOCAL_URL \
		and String(loc["live"]["model"]) == "qwen3.8-27b" and String(loc["live"]["thinking"]) == "none", "the shipped Local: LM Studio, %s, qwen3.8-27b, thinking off" % LOCAL_URL)
	_ok(String(ext["name"]) == "External" and String(ext["live"]["preset_id"]) == "anthropic" and String(ext["live"]["base_url"]) == EXTERNAL_URL \
		and String(ext["live"]["model"]) == "claude-sonnet-5-5" and String(ext["live"]["thinking"]) == "default", "the shipped External: Anthropic, %s, claude-sonnet-5-5" % EXTERNAL_URL)
	var phone_local: Dictionary = LlmProfiles.find(_bridge.llm_profiles_parse("", true), "local")
	_ok(String(phone_local["live"]["base_url"]) == "" and String(phone_local["live"]["preset_id"]) == "lm_studio", "on a phone the shipped Local has NO address (the LAN address is the user's)")
	for k in ["ctx_entity", "ctx_selection", "ctx_notes", "ctx_params", "ctx_rag_world", "ctx_rag_vault"]:
		_ok(not bool(loc["live"][k]) and not bool(ext["live"][k]), "both shipped profiles have '%s' OFF" % k)
	await _shot("1_fresh_settings")
	_set.hide()
	_panel.open()
	await _settle(8)
	var popt := _p("profile") as OptionButton
	_ok(popt != null and _items(popt) == _items(opt), "the panel header has the same switch: %s" % str(_items(popt) if popt != null else []))
	_ok(not _cfg_text().contains("[llm_profiles]"), "opening the panel wrote no [llm_profiles] section")
	await _shot("1_fresh_panel")

	# ------------------------------------------------------------------ 2. apply at once, master off
	print("== 2. a switch applies everything at once; nothing global changes; nothing is sent ==")
	_panel.hide()
	var globals0 := {}
	for k in ["enabled", "prose", "commands", "edit", "local_allowed", "remote_allowed", "key_source", "env_var", "token_cap"]:
		globals0[k] = DccSettings.llm_value(k)
	var att0 := LlmWire.connect_attempts
	var net0 := _netstat_other(pid)
	_set.open()
	await _settle(8)
	_ok(await _pick(_c("profile_option") as OptionButton, "Local"), "Local is picked through the real dropdown")
	var live := _live()
	_ok(String(live["preset_id"]) == "lm_studio" and String(live["base_url"]) == LOCAL_URL and String(live["model"]) == "qwen3.8-27b", "provider, address and model are the Local profile's: %s %s %s" % [live["preset_id"], live["base_url"], live["model"]])
	_ok(String(live["thinking"]) == "none" and int(live["max_tokens"]) == 0 and int(live["retry_limit"]) == 1, "thinking off, cap not pinned, one retry")
	_ok(String(DccSettings.llm_value("key_account")) == "lm_studio", "the key account is the profile's: lm_studio")
	var all_off := true
	for k in ["ctx_entity", "ctx_selection", "ctx_notes", "ctx_params", "ctx_rag_world", "ctx_rag_vault"]:
		if bool(live[k]):
			all_off = false
	_ok(all_off, "every context tick is off")
	_ok(LlmProfiles.active_id() == "local", "Local is the active profile")
	var same_globals := true
	for k in globals0:
		if DccSettings.llm_value(k) != globals0[k]:
			same_globals = false
	_ok(same_globals, "the master switch, Commands, Edit, Local and Remote switches, key source and cap are UNCHANGED by the switch")
	_ok(not bool(DccSettings.llm_value("enabled")), "the master switch is still off")
	_ok(not _card_open() and DccSettings.llm_consented_origins().is_empty(), "no consent card appeared and no consent was recorded (master off)")
	_ok(_cfg_text().contains("[llm_profiles]"), "the first switch stored the profiles ([llm_profiles])")
	_ok((_c("base_url") as LineEdit).text == LOCAL_URL and (_c("model") as LineEdit).text == "qwen3.8-27b", "the window's own fields were redrawn from the profile")
	_ok((_c("profile_option") as OptionButton).get_item_text((_c("profile_option") as OptionButton).selected).begins_with("Local"), "the switch shows Local")
	_ok((_c("profile_delete") as Button).disabled and (_c("profile_delete") as Button).tooltip_text.contains("built-in"), "Delete is disabled for a built-in with its reason: '%s'" % (_c("profile_delete") as Button).tooltip_text.left(40))
	_ok(not (_c("profile_reset") as Button).disabled, "Reset is enabled for a built-in")
	await _shot("2_local_applied")
	## Reply, thinking and retry controls are real controls and write through to the active profile.
	var cap := _c("max_tokens") as SpinBox
	cap.value = 300
	await _settle(4)
	var th := _c("thinking") as OptionButton
	th.select(0)
	th.item_selected.emit(0)
	await _settle(4)
	var rt := _c("retry_limit") as OptionButton
	rt.select(0)
	rt.item_selected.emit(0)
	await _settle(4)
	live = _live()
	_ok(int(live["max_tokens"]) == 300 and String(live["thinking"]) == "default" and int(live["retry_limit"]) == 0, "the REPLY controls wrote cap 300, thinking default, no retry")
	var lrow := _row("local")
	_ok(int(lrow["live"]["max_tokens"]) == 300 and String(lrow["live"]["thinking"]) == "default" and int(lrow["live"]["retry_limit"]) == 0, "...and wrote THROUGH to the Local profile")
	## Put Local back to its shipped thinking for the later request check; keep the cap pin.
	th.select(1)
	th.item_selected.emit(1)
	await _settle(4)
	rt.select(1)
	rt.item_selected.emit(1)
	await _settle(4)
	## Now External, through the window.
	_ok(await _pick(_c("profile_option") as OptionButton, "External"), "External is picked through the real dropdown")
	live = _live()
	_ok(String(live["preset_id"]) == "anthropic" and String(live["base_url"]) == EXTERNAL_URL and String(live["model"]) == "claude-sonnet-5-5", "External's provider, address and model applied at once")
	_ok(int(live["max_tokens"]) == 0 and String(live["thinking"]) == "default", "...and Local's cap pin and thinking did NOT carry over")
	_ok(String(DccSettings.llm_value("key_account")) == "anthropic", "the key account is now External's: anthropic")
	await _shot("2_external_applied")
	_ok(await _pick(_c("profile_option") as OptionButton, "Local"), "back to Local")
	_ok(int(_live()["max_tokens"]) == 300 and String(_live()["thinking"]) == "none", "Local's own cap pin (300) and thinking (off) came back")
	## A forced Send with the feature off is refused and opens no connection.
	_set.hide()
	_panel.open()
	await _settle(6)
	(_p("input") as TextEdit).text = "Describe the port."
	_panel._on_send()
	await _settle(6)
	_ok(LlmWire.connect_attempts == att0 and _netstat_other(pid) == net0, "a forced Send with the master switch off opened no connection (attempts %d, sockets %d)" % [LlmWire.connect_attempts - att0, _netstat_other(pid) - net0])
	_panel.hide()

	# ------------------------------------------------------------------ 3. consent first
	print("== 3. consent first, master on ==")
	DccSettings.set_llm_value("enabled", true)
	DccSettings.set_llm_value("local_allowed", true)
	DccSettings.set_llm_value("remote_allowed", true)
	## The capability the gate checks before consent: a user's own global switch, set here, never by a profile.
	DccSettings.set_llm_value("prose", true)
	DccSettings.set_llm_value("key_source", "env")
	DccSettings.set_llm_value("env_var", KEYVAR)
	OS.set_environment(KEYVAR, SECRET)
	_panel.open()
	await _settle(8)
	var consent0 := DccSettings.llm_consented_origins().size()
	_ok(consent0 == 0, "no consent is recorded yet")
	_ok(await _pick(_p("profile") as OptionButton, "External"), "External is picked in the panel header")
	_ok(_card_open(), "the consent card is shown BEFORE the switch completes")
	var ct := _card_text(_card())
	_ok(ct.contains("api.anthropic.com"), "the card names the new endpoint")
	_ok(String(_live()["model"]) == "claude-sonnet-5-5", "the new profile's values are live (tentatively) so the card describes what will be sent")
	var gate: Dictionary = _bridge.llm_authorize(DccSettings.llm_gate_dict(), "prose", DccSettings.llm_consented_origins())
	_ok(String(gate.get("code", "")) == "consent_needed", "while the card is open the gate refuses with consent_needed: %s" % str(gate.get("code", "ok")))
	await _shot("3_consent_card")
	await _press_card(_card(), "cancel")
	_ok(not _card_open(), "Not now closes the card")
	_ok(String(_live()["preset_id"]) == "lm_studio" and String(_live()["model"]) == "qwen3.8-27b" and int(_live()["max_tokens"]) == 300, "a decline put EVERY value back (Local, qwen3.8-27b, cap 300)")
	_ok(LlmProfiles.active_id() == "local" and DccSettings.llm_consented_origins().is_empty(), "...Local is still active and no consent was recorded")
	_ok((_p("profile") as OptionButton).get_item_text((_p("profile") as OptionButton).selected).begins_with("Local"), "...and the switch shows Local again")
	_ok(await _pick(_p("profile") as OptionButton, "External"), "External is picked again")
	_ok(_card_open(), "the card is shown again")
	await _press_escape()
	_ok(not _card_open() and LlmProfiles.active_id() == "local", "Esc is also a decline: Local stays active")
	_ok(await _pick(_p("profile") as OptionButton, "External"), "External is picked a third time")
	await _press_card(_card(), "safe")
	_ok(LlmProfiles.active_id() == "external" and DccSettings.llm_consented_origins() == PackedStringArray([EXTERNAL_URL.trim_suffix("/v1")]), "agreeing applied External and recorded consent for exactly that origin: %s" % str(DccSettings.llm_consented_origins()))
	_ok(_bridge.llm_authorize(DccSettings.llm_gate_dict(), "prose", DccSettings.llm_consented_origins()).get("ok", false), "the gate now says External is ready")
	## Local's endpoint has no consent either: the same card.
	_ok(await _pick(_p("profile") as OptionButton, "Local"), "Local is picked")
	_ok(_card_open() and _card_text(_card()).contains("127.0.0.1:1234"), "Local's card names 127.0.0.1:1234")
	await _press_card(_card(), "safe")
	_ok(LlmProfiles.active_id() == "local" and DccSettings.llm_consented_origins().size() == 2, "Local applied; consent recorded for its origin too (2 origins)")

	# ------------------------------------------------------------------ 4. separate per profile
	print("== 4. ticks, cap and model are each profile's own ==")
	DccSettings.set_llm_value("ctx_notes", true)
	DccSettings.set_llm_value("ctx_params", true)
	DccSettings.set_llm_value("model", "qwen3-14b")
	_ok(await _pick(_p("profile") as OptionButton, "External"), "switch to External (consent exists: no card)")
	_ok(not _card_open() and LlmProfiles.active_id() == "external", "External applied with no card")
	live = _live()
	_ok(not bool(live["ctx_notes"]) and not bool(live["ctx_params"]) and String(live["model"]) == "claude-sonnet-5-5" and int(live["max_tokens"]) == 0, "External has its OWN ticks (off), model and cap (not Local's notes/params, qwen3-14b, 300)")
	DccSettings.set_llm_value("ctx_entity", true)
	DccSettings.set_llm_value("max_tokens", 512)
	_ok(await _pick(_p("profile") as OptionButton, "Local"), "switch back to Local")
	live = _live()
	_ok(bool(live["ctx_notes"]) and bool(live["ctx_params"]) and not bool(live["ctx_entity"]) and String(live["model"]) == "qwen3-14b" and int(live["max_tokens"]) == 300, "Local's ticks (notes, params), model (qwen3-14b) and cap (300) came back; External's entity tick did not leak")
	_ok(bool(_row("external")["live"]["ctx_entity"]) and not bool(_row("external")["live"]["ctx_notes"]) and int(_row("external")["live"]["max_tokens"]) == 512, "External's stored profile holds entity on, notes off, cap 512")
	## What would be sent: build the real request bytes (no socket) for each profile.
	var world_ready := await _make_world()
	_ok(world_ready, "a world with a selected place exists for the request check")
	if world_ready:
		var stream: Object = ClassDB.instantiate("LlmStream")
		await _panel_to_place()
		var args: Dictionary = _panel._args("Describe it.")
		var prep: Dictionary = stream.prepare(DccSettings.llm_gate_dict(), "prose", DccSettings.llm_consented_origins(), args)
		_ok(bool(prep.get("ok", false)), "Local: a request is built: %s" % str(prep.get("reason", "ok")))
		if bool(prep.get("ok", false)):
			var body: Dictionary = JSON.parse_string((prep["body"] as PackedByteArray).get_string_from_utf8())
			_ok(String(body.get("model", "")) == "qwen3-14b", "Local's request names qwen3-14b: %s" % str(body.get("model", "")))
			_ok(int(body.get("max_tokens", -1)) == 300, "Local's request carries the pinned cap 300 (not the task default): %s" % str(body.get("max_tokens", "none")))
			_ok(String(body.get("reasoning_effort", "")) == "none", "Local's request asks for reasoning_effort none (thinking off): %s" % str(body.get("reasoning_effort", "absent")))
		_ok(await _pick(_p("profile") as OptionButton, "External"), "switch to External")
		args = _panel._args("Describe it.")
		prep = stream.prepare(DccSettings.llm_gate_dict(), "prose", DccSettings.llm_consented_origins(), args)
		_ok(bool(prep.get("ok", false)), "External: a request is built: %s" % str(prep.get("reason", "ok")))
		if bool(prep.get("ok", false)):
			var body2: Dictionary = JSON.parse_string((prep["body"] as PackedByteArray).get_string_from_utf8())
			_ok(String(body2.get("model", "")) == "claude-sonnet-5-5" and int(body2.get("max_tokens", -1)) == 512, "External's request names claude-sonnet-5-5 with cap 512: %s %s" % [str(body2.get("model", "")), str(body2.get("max_tokens", ""))])
			_ok(not body2.has("reasoning_effort") and not body2.has("chat_template_kwargs"), "External's request carries no thinking field")
		_ok(await _pick(_p("profile") as OptionButton, "Local"), "back to Local")
	## The retry limit: 0 turns the one automatic retry off (the code path in `LlmRun`), 1 leaves it.
	var run = _panel.run
	DccSettings.set_llm_value("retry_limit", 0)
	var failed := []
	run.failed.connect(func(m: String): failed.append(m))
	run._dispatch([{"type": "error", "message": "The server is busy", "will_retry": true}])
	await _settle(2)
	_ok(not run._retry_pending and failed.size() == 1, "retry limit 0: a transient failure is final (no retry pending, failed emitted)")
	DccSettings.set_llm_value("retry_limit", 1)
	failed.clear()
	run._dispatch([{"type": "error", "message": "The server is busy", "will_retry": true}])
	await _settle(2)
	_ok(run._retry_pending and failed.is_empty(), "retry limit 1: the one retry is pending, no failure shown yet")
	run._retry_pending = false
	run.shutdown()
	await _settle(4)

	# ------------------------------------------------------------------ 5. keys per profile
	print("== 5. one key entry per profile; no key in a profile ==")
	var store_ok: bool = bool(_bridge.llm_key_status("env", "X", "x").get("store_available", false))
	_ok(true, "OS credential store available here: %s" % str(store_ok))
	var work := LlmProfiles.add(WORK_NAME)
	_ok(bool(work.get("ok", false)), "a custom profile is created: %s" % str(work.get("error", "ok")))
	var work_id := String(work.get("id", ""))
	var work_acct := String(_row(work_id).get("live", {}).get("key_account", ""))
	_ok(work_acct == "pf_" + work_id, "its key account is its own: %s" % work_acct)
	_accounts.append(work_acct)
	var sw := LlmProfiles.switch_to(work_id)
	_ok(bool(sw.get("ok", false)) and LlmProfiles.active_id() == work_id, "it is switched to")
	live = _live()
	_ok(String(live["preset_id"]) == "" and String(live["model"]) == "" and String(live["base_url"]) == "", "a NEW profile starts empty: nothing chosen (never a plausible default)")
	var blank := true
	for k in ["ctx_entity", "ctx_selection", "ctx_notes", "ctx_params", "ctx_rag_world", "ctx_rag_vault"]:
		if bool(live[k]):
			blank = false
	_ok(blank, "...and every context tick is OFF")
	_ok(DccSettings.llm_key_account() == work_acct, "the live key account is the new profile's")
	if store_ok:
		_ok(bool(_bridge.llm_key_save(work_acct, SECRET).get("ok", false)), "a key is saved under the profile's own account")
		_ok(LlmProfiles.key_present(work_acct), "it is present for the profile")
		var dup := LlmProfiles.copy_profile(work_id)
		var dup_id := String(dup.get("id", ""))
		var dup_acct := String(_row(dup_id).get("live", {}).get("key_account", ""))
		_accounts.append(dup_acct)
		_ok(bool(dup.get("ok", false)) and dup_acct != work_acct and dup_acct == "pf_" + dup_id, "a duplicate has its own account: %s" % dup_acct)
		_ok(not LlmProfiles.key_present(dup_acct), "...and NO key: a key is never copied")
		_ok(LlmProfiles.key_present(work_acct), "...while the original's key is untouched")
		_ok(String(_row(dup_id)["name"]) == WORK_NAME + " copy", "the copy's name is '<name> copy': %s" % String(_row(dup_id)["name"]))
		LlmProfiles.switch_to("local")
		_ok(DccSettings.llm_key_account() == "lm_studio" and not LlmProfiles.key_present("pf_" + work_id + "x"), "switching to Local points the key account at lm_studio")
	else:
		print("  note  no OS credential store on this machine: the per-profile key saves are not exercised")
		LlmProfiles.switch_to("local")
	## No key anywhere in the profile text or the settings file.
	var data := DccSettings.llm_profiles_data()
	_ok(data.length() > 200 and not data.contains(SECRET) and not _cfg_text().contains(SECRET), "the stored profile text (%d bytes) and cartalith_settings.cfg hold no key" % data.length())
	_ok(not data.contains("api_key") and not data.contains("password") and not data.contains("\"key\""), "the profile text has no key-like field")
	_ok(not DccSettings.set_llm_profiles_data(123) and not DccSettings.set_llm_profiles_data("x".repeat(DccSettings.LLM_PROFILES_MAX_BYTES + 1)), "the profile section refuses a non-string and an oversize write")
	_ok(not DccSettings.set_llm_value("api_key", SECRET) and not _cfg_text().contains("api_key"), "the [llm] allow-list still refuses a key field")

	# ------------------------------------------------------------------ 6. delete
	print("== 6. delete: asked, never silent about the key ==")
	var count0 := (LlmProfiles.load_set()["profiles"] as Array).size()
	_ok(LlmProfiles.request_delete(_app, "local").is_empty() and LlmProfiles.request_delete(_app, "external").is_empty(), "the built-in profiles cannot be deleted (no card)")
	if store_ok:
		var d := LlmProfiles.request_delete(_app, work_id)
		_ok(not d.is_empty() and bool(d["has_key"]), "the card knows a key is stored for it")
		var dt := _card_text(d)
		_ok(dt.contains("Delete, forget key") and dt.contains("Delete, keep key") and dt.contains("Cancel"), "it offers 'Delete, forget key', 'Delete, keep key' and Cancel")
		_ok(dt.contains("does not remove the key unless you choose"), "it says deleting alone does not remove the key")
		await _shot("6_delete_card")
		await _press_card(d, "cancel")
		_ok(not LlmProfiles.find(LlmProfiles.load_set(), work_id).is_empty() and LlmProfiles.key_present(work_acct), "Cancel deleted nothing and kept the key")
		d = LlmProfiles.request_delete(_app, work_id)
		await _press_escape()
		_ok(not LlmProfiles.find(LlmProfiles.load_set(), work_id).is_empty() and LlmProfiles.key_present(work_acct), "Esc deleted nothing and kept the key")
		d = LlmProfiles.request_delete(_app, work_id)
		await _press_card(d, "safe")
		_ok(LlmProfiles.find(LlmProfiles.load_set(), work_id).is_empty(), "'Delete, keep key' deleted the profile")
		_ok(LlmProfiles.key_present(work_acct), "...and the key is STILL in the credential store")
		var again := LlmProfiles.add("ZZ Probe Forget")
		var fid := String(again["id"])
		var facct := String(_row(fid)["live"]["key_account"])
		_accounts.append(facct)
		_bridge.llm_key_save(facct, SECRET)
		d = LlmProfiles.request_delete(_app, fid)
		await _press_card(d, "destructive")
		_ok(LlmProfiles.find(LlmProfiles.load_set(), fid).is_empty() and not LlmProfiles.key_present(facct), "'Delete, forget key' deleted the profile AND removed its key")
		_bridge.llm_key_forget(work_acct)
	## A profile without a key gets a plain Delete.
	var plain := LlmProfiles.add("ZZ Probe Plain")
	var d2 := LlmProfiles.request_delete(_app, String(plain["id"]))
	_ok(not bool(d2["has_key"]) and _card_text(d2).contains("No key is stored"), "with no key stored the card says so and offers a plain Delete")
	await _press_card(d2, "destructive")
	_ok(LlmProfiles.find(LlmProfiles.load_set(), String(plain["id"])).is_empty(), "the plain Delete deleted it")
	_ok((LlmProfiles.load_set()["profiles"] as Array).size() >= count0 - 1, "the profile count is consistent after the deletes")

	# ------------------------------------------------------------------ 7. caps and names
	print("== 7. 16 profiles, 40-character names, no duplicate names ==")
	var cur := (LlmProfiles.load_set()["profiles"] as Array).size()
	var made: Array = []
	for n in range(16 - cur):
		var r := LlmProfiles.add("ZZ Cap %d" % n)
		if bool(r.get("ok", false)):
			made.append(String(r["id"]))
	_ok((LlmProfiles.load_set()["profiles"] as Array).size() == 16, "the set holds exactly 16 profiles")
	var over := LlmProfiles.add("ZZ One Too Many")
	_ok(not bool(over.get("ok", false)) and String(over.get("error", "")).contains("16 profiles"), "the 17th is refused with words: '%s'" % String(over.get("error", "")))
	_ok(not bool(LlmProfiles.copy_profile(made[0]).get("ok", false)), "a duplicate is refused at the cap too")
	_set.open()
	await _settle(8)
	_ok((_c("profile_new") as Button).disabled and (_c("profile_new") as Button).tooltip_text.contains("16"), "New is disabled with its reason at the cap")
	_set.hide()
	for id in made:
		LlmProfiles.delete(String(id))
	var long41 := LlmProfiles.add("n".repeat(41))
	_ok(not bool(long41.get("ok", false)) and String(long41["error"]).contains("40"), "a 41-character name is refused: '%s'" % String(long41.get("error", "")))
	_ok(not bool(LlmProfiles.add("   ").get("ok", false)), "an empty name is refused")
	_ok(not bool(LlmProfiles.add("local").get("ok", false)) and not bool(LlmProfiles.add("  EXTERNAL ").get("ok", false)), "a name another profile has is refused, ignoring case and space")
	_ok(bool(LlmProfiles.add("n".repeat(40)).get("ok", false)), "a 40-character name is accepted")
	var rn := LlmProfiles.add("ZZ Rename Me")
	_ok(bool(LlmProfiles.rename(String(rn["id"]), "ZZ Renamed").get("ok", false)) and String(_row(String(rn["id"]))["name"]) == "ZZ Renamed", "a custom profile can be renamed")
	_ok(not bool(LlmProfiles.rename("local", "Mine").get("ok", false)), "a built-in cannot be renamed")
	for p in LlmProfiles.load_set()["profiles"]:
		if not bool(p["builtin"]):
			LlmProfiles.delete(String(p["id"]))

	# ------------------------------------------------------------------ 8. https
	print("== 8. a remote profile over http:// ==")
	LlmProfiles.switch_to("external")
	DccSettings.set_llm_value("base_url", "http://api.anthropic.com/v1")
	_ok(String(_row("external")["live"]["base_url"]) == "http://api.anthropic.com/v1", "the edit was written through to External")
	_ok(not (_row("external")["problems"] as Array).is_empty(), "External now reports a problem")
	LlmProfiles.switch_to("local")
	var res := LlmProfiles.switch_to("external")
	_ok(bool(res.get("ok", false)) and String(res.get("warn", "")).contains("https"), "switching to it is allowed (so it can be fixed) and says why nothing will be sent: '%s'" % String(res.get("warn", "")).left(90))
	_ok(not _card_open(), "no consent card is offered for an http remote address")
	var g2: Dictionary = _bridge.llm_authorize(DccSettings.llm_gate_dict(), "prose", DccSettings.llm_consented_origins())
	_ok(String(g2.get("code", "")) == "cleartext_remote", "the gate refuses it: %s" % str(g2.get("code", "")))
	DccSettings.set_llm_value("base_url", EXTERNAL_URL)
	_ok((_row("external")["problems"] as Array).is_empty(), "fixing the address clears the problem")

	# ------------------------------------------------------------------ 9. default and restore
	print("== 9. default and restore ==")
	var cust := LlmProfiles.add("ZZ Default Me")
	var cid := String(cust["id"])
	_ok(bool(LlmProfiles.set_default(cid).get("ok", false)) and String(LlmProfiles.load_set()["default"]) == cid, "a custom profile can be made the device's default")
	LlmProfiles.switch_to("local")
	var vals := _live()
	## A restart restores the last used profile even when the live values were changed behind its back.
	DccSettings.set_llm_values({"preset_id": "ollama", "model": "zzz", "ctx_entity": true})
	LlmProfiles.restore()
	_ok(String(_live()["preset_id"]) == "lm_studio" and String(_live()["model"]) == String(vals["model"]) and not bool(_live()["ctx_entity"]), "restore applied the last used profile (Local) over hand-edited values")
	_ok(LlmProfiles.active_id() == "local", "...and Local is the active profile")
	## With the last used profile gone, the default is restored.
	LlmProfiles.delete(cid)
	_ok(String(LlmProfiles.load_set()["default"]) == "local", "deleting the default falls back to Local as the default")
	LlmProfiles.set_default("external")
	LlmProfiles._op("last", {"id": ""}, false)
	DccSettings.set_llm_values({"preset_id": "ollama", "model": "zzz"})
	LlmProfiles.restore()
	_ok(LlmProfiles.active_id() == "external" and String(_live()["model"]) == "claude-sonnet-5-5", "with no last used profile, restore applies the DEFAULT (External)")
	## Final state for phase 2: Local active with its own ticks, a custom default.
	var keep := LlmProfiles.add(WORK_NAME)
	var keep_id := String(keep["id"])
	LlmProfiles.set_default(keep_id)
	LlmProfiles.switch_to("local")
	DccSettings.set_llm_value("ctx_notes", true)
	DccSettings.set_llm_value("max_tokens", 300)
	DccSettings.set_llm_value("model", "qwen3-14b")
	_ok(LlmProfiles.active_id() == "local" and String(LlmProfiles.load_set()["default"]) == keep_id, "phase 1 leaves Local active (notes tick, cap 300, qwen3-14b) and '%s' as the default" % WORK_NAME)

	# ------------------------------------------------------------------ 10. chip menu
	print("== 10. the Assistant chip's menu ==")
	if _mode == "phone":
		print("  note  the phone has no chip menu: its quick bar's Assistant entry opens the panel, which carries the switch")
	else:
		var qb = _app._quick_bar
		DccSettings.set_quick_bar_value("assistant_button", true)
		qb.refresh_assistant()
		await _settle(4)
		var chip: Button = qb.assistant_chip()
		_ok(chip != null and qb.assistant_state() != "hidden", "the Assistant chip is shown (enabled, a provider and a model set): %s" % qb.assistant_state())
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_RIGHT
		ev.pressed = true
		qb._assistant_input(ev)
		await _settle(4)
		var menu: PopupMenu = qb._assistant_menu
		var texts: Array = []
		var checked: Array = []
		for k in menu.item_count:
			texts.append(menu.get_item_text(k))
			if menu.is_item_checked(k):
				checked.append(menu.get_item_text(k))
		_ok(texts.has("Profile: Local") and texts.has("Profile: External") and texts.has("Hide the Assistant chip") and texts.has("Customise…"), "the menu lists the profiles beside its old rows: %s" % str(texts))
		_ok(checked == ["Profile: Local"], "the active profile is checked: %s" % str(checked))
		await _shot("10_chip_menu")
		var ext_idx := texts.find("Profile: External")
		menu.id_pressed.emit(menu.get_item_id(ext_idx))
		await _settle(6)
		_ok(LlmProfiles.active_id() == "external" and String(_live()["model"]) == "claude-sonnet-5-5", "picking External from the chip's menu applied the whole profile")
		qb._assistant_input(ev)
		await _settle(3)
		_ok(menu.is_item_checked(menu.get_item_index(menu.get_item_id(texts.find("Profile: External")))), "the menu now checks External")
		menu.hide()
		LlmProfiles.switch_to("local")

	# ------------------------------------------------------------------ 11. a world zip
	print("== 11. a saved world holds none of it ==")
	if _bridge.has_world:
		var sdir := ProjectSettings.globalize_path("user://_llmprofiles_probe")
		DirAccess.make_dir_recursive_absolute(sdir)
		var zpath := sdir.path_join("world.zip")
		_ok(_bridge.save_project(zpath), "a project is saved while profiles and keys are stored")
		var zr := ZIPReader.new()
		_ok(zr.open(zpath) == OK, "the saved project opens as a zip")
		var needles := ["llm_profiles", "pf_p", WORK_NAME, "qwen3.8-27b", "qwen3-14b", "claude-sonnet-5-5", "lm_studio", "Previous settings", SECRET, "key_account"]
		var leaked: Array = []
		var entries := zr.get_files()
		for e in entries:
			var bytes := zr.read_file(e)
			for n in needles:
				if _contains(bytes, n):
					leaked.append("%s in %s" % [n, e])
		_ok(entries.size() >= 5 and leaked.is_empty(), "%d zip entries searched for %d needles: %s" % [entries.size(), needles.size(), str(leaked)])
		var raw := _file_bytes(zpath)
		var raw_leak: Array = []
		for n in needles:
			if _contains(raw, n):
				raw_leak.append(n)
		_ok(raw.size() > 1000 and raw_leak.is_empty(), "the raw zip (%d bytes) holds none of them: %s" % [raw.size(), str(raw_leak)])
		## Positive control: the same entries plus the profile text MUST trip the detector.
		var cpath := sdir.path_join("control.zip")
		var packer := ZIPPacker.new()
		packer.open(cpath)
		for e in entries:
			packer.start_file(e)
			packer.write_file(zr.read_file(e))
			packer.close_file()
		packer.start_file("smuggled.txt")
		packer.write_file(DccSettings.llm_profiles_data().to_utf8_buffer())
		packer.close_file()
		packer.close()
		var zc := ZIPReader.new()
		zc.open(cpath)
		var hits := 0
		for e in zc.get_files():
			if _contains(zc.read_file(e), "claude-sonnet-5-5") and _contains(zc.read_file(e), WORK_NAME):
				hits += 1
		_ok(hits == 1, "positive control: the profile text smuggled into a zip is detected (%d hit)" % hits)
		zc.close()
		zr.close()
	else:
		print("  note  no world to save")
	var lb := _file_bytes(ProjectSettings.globalize_path("user://logs/godot.log"))
	_ok(lb.size() > 100 and not _contains(lb, SECRET), "godot.log (%d bytes) holds no key" % lb.size())

	# ------------------------------------------------------------------ 12. density
	print("== 12. density and layout ==")
	_panel.open()
	await _settle(10)
	var floor_px := 0.0
	if _mode == "phone":
		floor_px = float(DccTheme.PHONE_TAP_MIN)
	elif _mode == "tablet":
		floor_px = float(DccTheme.role_px("btn_min_h"))
	var wr := Rect2(Vector2(_panel.position), Vector2(_panel.size))
	_ok(wr.position.x >= -0.5 and wr.position.y >= -0.5 and wr.end.x <= vp.x + 0.5 and wr.end.y <= vp.y + 0.5, "the panel %s is inside the viewport %s" % [wr, vp])
	var po := _p("profile") as OptionButton
	_ok(po.is_visible_in_tree() and po.size.y + 0.5 >= maxf(floor_px, 1.0), "the panel's profile switch is at least %d px tall: %.0f" % [int(floor_px), po.size.y])
	var por := po.get_global_rect()
	var pw := Vector2(_panel.size)
	_ok(por.position.x >= -1.0 and por.end.x <= pw.x + 1.0, "...and sits inside the window width (%s of %s)" % [por, pw])
	var send := _p("send") as Button
	var close_btn: Button = _panel.get_ok_button()
	var sr := send.get_global_rect()
	var cr := close_btn.get_global_rect()
	_ok(sr.end.y <= pw.y + 0.5 and sr.position.y >= 0.0 and sr.end.x <= pw.x + 0.5, "Send is inside the window: %s of %s" % [sr, pw])
	_ok(close_btn.is_visible_in_tree() and cr.end.y <= pw.y + 0.5 and cr.position.y >= 0.0, "Close is inside the window: %s of %s" % [cr, pw])
	_ok(not sr.intersects(cr, false), "Send and Close do not overlap")
	if _mode == "phone":
		_ok(send.size.y + 0.5 >= floor_px, "Send is >= 44 px tall: %.0f" % send.size.y)
		## FINDING, not a profile control and not changed here: the phone sheet's stock Close button measures
		## 29 px (the 44 px floor `phone_window` applies runs before `phone_fit` resizes the sheet; two
		## attempts to re-seat it after the resize did not move it). Recorded in the report.
		print("  note  Close is %.0f px tall on the phone sheet (stock dialog button, pre-existing)" % close_btn.size.y)
	await _shot("12_panel_header")
	_panel.hide()
	_set.open()
	await _settle(10)
	var swr := Rect2(Vector2(_set.position), Vector2(_set.size))
	_ok(swr.position.x >= -0.5 and swr.position.y >= -0.5 and swr.end.x <= vp.x + 0.5 and swr.end.y <= vp.y + 0.5, "the settings window %s is inside the viewport %s" % [swr, vp])
	for name in ["profile_option", "profile_new", "profile_duplicate", "profile_rename", "profile_delete", "profile_default", "profile_reset", "thinking", "max_tokens", "retry_limit"]:
		var c := _c(name)
		(_set._scroll as ScrollContainer).ensure_control_visible(c)
		await _settle(2)
		if floor_px > 0.0:
			_ok(c.size.y + 0.5 >= floor_px, "'%s' is at least %d px tall: %.0f" % [name, int(floor_px), c.size.y])
		var rr := c.get_global_rect()
		## A window's controls are laid out in the window's own space (0 .. size), embedded or not.
		_ok(rr.position.x >= -1.0 and rr.end.x <= float(_set.size.x) + 1.0, "'%s' sits inside the window width (%s of %d)" % [name, rr, _set.size.x])
	(_set._scroll as ScrollContainer).ensure_control_visible(_c("profile_switch"))
	await _shot("12_settings_profile")
	(_set._scroll as ScrollContainer).ensure_control_visible(_c("max_tokens"))
	await _shot("12_settings_reply")
	_set.hide()
	_forget_all()
	_finish_phase1_state()


## Ends phase 1 leaving the stored state for phase 2 (which asserts it after a real process restart).
func _finish_phase1_state() -> void:
	## The probe's own key variable and the keys it saved are cleaned; the settings and profiles stay.
	print("RESULT: %s (%d failed)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	OS.unset_environment(KEYVAR)
	get_tree().quit(0 if _fails == 0 else 1)


## A small world with a place selected in the right dock, so a place request can be built.
func _make_world() -> bool:
	_bridge.generate({"seed": 9137, "width_km": 1200.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _settle(10)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	return _bridge.has_world and _bridge.settlements().size() > 0


func _panel_to_place() -> void:
	var places: Array = _bridge.settlements()
	_app.right_dock_ctrl.on_settlement_selected(places[0], 0)
	await _settle(4)
	_panel.open()
	await _settle(8)
	_panel._pick_task("prose")
	await _settle(4)
	_panel._ensure_facts(true)
	await _settle(8)
	_panel.hide()


# ---------------------------------------------------------------------------------------------------
# Phase 2: a new process on the same user dir
# ---------------------------------------------------------------------------------------------------

func _phase2() -> void:
	print("== 13. restart: the last used profile is restored at startup ==")
	var set := LlmProfiles.load_set()
	_ok(bool(set.get("ok", false)) and String(set["last"]) == "local", "the stored last used profile survived the restart: %s" % String(set.get("last", "")))
	_ok(LlmProfiles.active_id() == "local", "Local is the active profile in the new process")
	var live := _live()
	_ok(String(live["preset_id"]) == "lm_studio" and String(live["base_url"]) == LOCAL_URL and String(live["model"]) == "qwen3-14b", "provider, address and the edited model were restored: %s %s %s" % [live["preset_id"], live["base_url"], live["model"]])
	_ok(bool(live["ctx_notes"]) and not bool(live["ctx_entity"]) and int(live["max_tokens"]) == 300, "its OWN ticks (notes on, entity off) and cap (300) were restored")
	var profiles: Array = set["profiles"]
	var names: Array = []
	for p in profiles:
		names.append(String(p["name"]))
	_ok(names == ["Local", "External", WORK_NAME], "the same three profiles exist after the restart: %s" % str(names))
	_ok(String(LlmProfiles.find(set, String(set["default"]))["name"]) == WORK_NAME, "the default profile survived: %s" % String(LlmProfiles.find(set, String(set["default"]))["name"]))
	_ok(DccSettings.llm_consented_origins().size() == 2, "the two consents survived (profiles never stored or removed them)")
	_ok(bool(DccSettings.llm_value("enabled")), "the master switch is exactly as it was left (on): a restart does not touch it")
	## A profile never holds a key, after a restart either.
	_ok(not DccSettings.llm_profiles_data().contains(SECRET) and not _cfg_text().contains(SECRET), "no key in the stored profiles or the settings file")
	_set.open()
	await _settle(8)
	var opt := _c("profile_option") as OptionButton
	_ok(opt.get_item_text(opt.selected).begins_with("Local") and not opt.is_item_disabled(opt.selected), "the settings switch shows Local")
	_set.hide()
	await _shot("13_restart")
	## Clean up everything this probe created.
	_reset_all()
	_ok(not _cfg_text().contains("[llm_profiles]") and not _cfg_text().contains("[llm]"), "cleanup: the probe's sections are removed from the settings file")
	_finish()
