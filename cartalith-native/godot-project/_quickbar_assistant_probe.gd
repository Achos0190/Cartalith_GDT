extends Node
## Probe for the conditional Assistant chip -- milestone CQ-7 of `CUSTOM_QUICK_BAR_SCOPE.md` (Ruling DH
## Q6, section 5) -- inside the real shell: `shell/assistant_chip.gd` (the decision), `shell/quick_bar.gd`
## (the chip at the cluster's start), `shell/quick_bar_ring.gd` + `shell/radial_ring.gd` (the chip under
## the cartwheel), the Customise window's switch, and `DccSettings.assistant_hub` (the change notice).
##
##   desktop: godot --path . --resolution 1600x1000 _quickbar_assistant_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1280x800  _quickbar_assistant_probe.tscn -- --force-touch --shots <dir>
##   phone:   godot --path . --resolution 393x852   _quickbar_assistant_probe.tscn -- --force-touch --shots <dir>
##
## **Windowed, never `--headless`** (a world is generated and the real input path is driven). Reads only
## `--force-touch` and `--shots <dir>`; anything else aborts with exit 2. **Run from a scratch worktree
## with its own `config/name` and a wiped `app_userdata/<name>`** (`MISTAKES.md`: a probe that reads
## `user://` is green for reasons unrelated to the code): section 0 fails if the assistant or quick-bar
## stores were not at their defaults at launch.
##
## **No assistant call is ever made.** The configured endpoint is a made-up port nobody listens on; the
## chip only reads settings and asks the local gate. The zero-socket section proves it over a session that
## shows, hides, greys, toggles and presses the chip.
##
## What this protects:
##   0. DEFAULT: assistant off, nothing configured -> the chip is hidden in the cluster, `dock_items` is
##      `[]` and no dock is drawn (the shipped state costs nothing);
##   1. TRUTH TABLE: `AssistantChip.decide` over all 2^7 = 128 combinations of (api, panel, enabled,
##      provider, model, switch, gate) against a table written out as literals, plus the six one-flip rows
##      by name so dropping any one conjunct turns a named check red; `provider_configured` literals;
##   2. LIVE SETTINGS: the real store and the real gate, one setting at a time -- hidden until enabled,
##      provider and model are all set; then greyed with the gate's OWN reason (equal to
##      `bridge.llm_authorize`'s string) until local is allowed and consent is given; then live; the chip
##      follows each write inside ONE process frame, and inside the same call by state; the switch hides it;
##   3. CLICKS: a real click on the live chip opens the assistant panel; a real click on the greyed chip
##      opens nothing; the right-click menu hides it; the Customise window's checkbox hides and restores it;
##   4. KEY: a secret in the key environment variable appears in no label, tooltip or `[quick_bar]` text;
##   5. WIDTH: the shell's minimum width is the same hidden, greyed and live; in a bar too narrow for it the
##      chip moves into the `⋯` menu's first row and the pin counts are unchanged;
##   6. RING: the chip is the dock's first chip, is NOT one of the four dock pins (4 pins + chip = 5 chips,
##      a 5th pin is still refused), sits inside the window at all edges, runs on a slow drag and release,
##      and is dimmed with the gate's reason in the caption when greyed; hidden, the dock is gone again;
##   7. SOCKETS: 0 non-listening TCP connections and no new UDP socket over the whole session.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument/watchdog.

const SEED := 24601
## A port nothing listens on: the chip never connects, so nothing needs to.
const FAKE_PORT := 59871
const SECRET := "sk-CQ7CHIPSECRET-9f3a1c"
const ENV_VAR := "CART_LLM_KEY"

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _app: Node
var _bridge
var _bar: DccQuickBar
var _win: QuickBarWindow
var _vault_key := "menu:%d" % DccMenus.ID_VAULT
var _planner_key := "menu:%d" % DccMenus.ID_STORY_JOURNEY_PLANNER
var _quickbar_key := ""
var _filler: Array = []


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1
		## Mutation runs (`CQ7_FAILFAST=1`) stop at the first failure so a killed mutant costs seconds.
		if OS.get_environment("CQ7_FAILFAST") != "":
			print("RESULT: FAIL (failfast after %d checks)" % _checks)
			get_tree().quit(1)


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _shot(tag: String) -> void:
	if _shots == "":
		return
	await _settle(3)
	var path := _shots.path_join("cq7_%s_%s.png" % [_mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


# -- real input ---------------------------------------------------------------------------

func _move_to(at: Vector2, mask: int = 0) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.button_mask = mask
	mm.relative = Vector2(0, 2)
	Input.parse_input_event(mm)


func _button(at: Vector2, button: int, down: bool) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = button
	ev.pressed = down
	ev.position = at
	ev.global_position = at
	ev.button_mask = (1 << (button - 1)) if down else 0
	Input.parse_input_event(ev)


func _click_at(at: Vector2, button: int = MOUSE_BUTTON_LEFT) -> void:
	_move_to(at)
	await _settle(2)
	_button(at, button, true)
	await _settle(2)
	_button(at, button, false)
	await _settle(4)


func _to_root(p: Vector2, c: Control) -> Vector2:
	var w := c.get_window()
	return p if w == null or w == get_tree().root else p + Vector2(w.position)


func _click(c: Control, button: int = MOUSE_BUTTON_LEFT) -> void:
	await _click_at(_to_root(c.get_global_rect().get_center(), c), button)


# -- helpers ------------------------------------------------------------------------------

func _idx() -> CommandIndex:
	var idx := CommandIndex.new()
	idx.build(_app, _bridge)
	return idx


func _broker():
	return _app.context_broker


func _ring_state() -> Dictionary:
	var r = _broker().ring
	return r.debug_state() if r != null else {}


func _ring_reset() -> void:
	if _broker().ring != null:
		_broker().ring_close()
	if _broker().card != null and _broker().card.visible:
		_broker().card.hide()
	await _settle(3)


func _local(global: Vector2) -> Vector2:
	return _app.viewport.overlay.get_global_transform_with_canvas().affine_inverse() * global


func _open_sticky(global: Vector2) -> Dictionary:
	await _ring_reset()
	_broker().ring_key_press(_local(global))
	await _settle(2)
	_broker().ring_key_release()
	await _settle(3)
	return _ring_state()


func _hide_windows() -> void:
	if _win and _win.visible:
		_win.hide()
	if _app.assistant_window and _app.assistant_window.visible:
		_app.assistant_window.hide()
	await _settle(2)


func _panel_open() -> bool:
	return _app.assistant_window != null and _app.assistant_window.visible


## Wipes the `[llm]` section and the chip switch, then announces it (writing the cfg directly does not
## go through the setters, which is what emits the change notice).
func _reset_llm() -> void:
	DccSettings._ensure_loaded()
	if DccSettings._cfg.has_section("llm"):
		DccSettings._cfg.erase_section("llm")
	if DccSettings._cfg.has_section_key("quick_bar", "assistant_button"):
		DccSettings._cfg.erase_section_key("quick_bar", "assistant_button")
	DccSettings._save()
	OS.set_environment(ENV_VAR, "")
	DccSettings.notify_assistant_state()


func _configure_all() -> void:
	DccSettings.set_llm_value("enabled", true)
	DccSettings.set_llm_value("local_allowed", true)
	DccSettings.set_llm_value("prose", true)
	DccSettings.set_llm_value("preset_id", "ollama")
	DccSettings.set_llm_value("base_url", "http://127.0.0.1:%d/v1" % FAKE_PORT)
	DccSettings.set_llm_value("model", "mock-model")
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % FAKE_PORT)


func _gate() -> Dictionary:
	return _bridge.llm_authorize(DccSettings.llm_gate_dict(), "", DccSettings.llm_consented_origins())


## Parses `netstat -ano -p TCP` and `-p TCPv6` rows for `pid` (both families: a connection to `::1`
## is invisible to the IPv4 table alone, `_llmset_probe.gd`'s LLM-3 lesson).
func _netstat_tcp(pid: int) -> Dictionary:
	var listening := 0
	var other := 0
	var rows: Array = []
	for proto in ["TCP", "TCPv6"]:
		var out: Array = []
		OS.execute("netstat", ["-ano", "-p", proto], out)
		for line in String("".join(out)).split("\n"):
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


func _texts_free_of(n: Node, secret: String) -> bool:
	if n is Label and (n as Label).text.contains(secret):
		return false
	if n is BaseButton and ((n as BaseButton).text.contains(secret) or (n as Button).tooltip_text.contains(secret)):
		return false
	if n is Control and (n as Control).tooltip_text.contains(secret):
		return false
	for c in n.get_children():
		if not _texts_free_of(c, secret):
			return false
	return true


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

	print("== 0. shipped default ==")
	_ok(not bool(DccSettings.llm_value("enabled")) and String(DccSettings.llm_value("model")) == "" and not DccSettings.has_quick_bar_value("assistant_button"),
		"the assistant is off, nothing is configured and the switch was never written (the user dir was wiped)")
	_ok(DccSettings.quick_bar_value("assistant_button") == true, "the chip's switch defaults ON (Ruling DH Q6: hidden only by the user)")
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_mode = "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s, viewport %s ==" % [_mode, get_viewport().get_visible_rect().size])
	_bridge = _app.bridge
	_win = _app.quick_bar_window
	_bar = _app._quick_bar
	var pid := OS.get_process_id()
	var tcp_before := _netstat_tcp(pid)
	var udp_before := _netstat_udp(pid)
	_ok(_bridge.llm_api, "this build has the assistant bindings (`llm_api`)")
	_ok(AssistantChip.state_for_app(_app)["state"] == AssistantChip.HIDDEN, "default state: hidden")

	await _truth_table()
	if OS.get_environment("CQ7_TABLE_ONLY") != "":
		_finish()
		return
	if _mode == "phone":
		await _phone()
		await _sockets(pid, tcp_before, udp_before)
		_finish()
		return

	_bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(8)
	for r in _idx().all():
		if bool(r["pinnable"]):
			if String(r["title"]) == "Quick bar":
				_quickbar_key = String(r["pin_key"])
			elif not [_vault_key, _planner_key].has(String(r["pin_key"])) and not String(r["pin_key"]).begins_with("extra:") and _filler.size() < 6:
				_filler.append(String(r["pin_key"]))

	await _default_state()
	await _live_settings()
	await _clicks()
	await _key_secret()
	await _ring()
	await _width()
	await _sockets(pid, tcp_before, udp_before)
	_finish()


# -- 1 ------------------------------------------------------------------------------------

## The literal table. A row is [api, panel, enabled, provider, model, switch, gate] -> state. Written out
## by hand for the interesting rows; the 128-row sweep below states the same rule a second, independent
## way (by counting the conjuncts that hold) so neither side can pass by being the other.
func _truth_table() -> void:
	print("== 1. the decision, 2^7 = 128 combinations ==")
	var H := AssistantChip.HIDDEN
	var D := AssistantChip.DISABLED
	var L := AssistantChip.LIVE
	var named := [
		["everything true, gate allows", [true, true, true, true, true, true, true], L],
		["everything true, gate refuses", [true, true, true, true, true, true, false], D],
		["no assistant bindings in the build", [false, true, true, true, true, true, true], H],
		["no panel to open", [true, false, true, true, true, true, true], H],
		["assistant disabled", [true, true, false, true, true, true, true], H],
		["no provider configured", [true, true, true, false, true, true, true], H],
		["no model set", [true, true, true, true, false, true, true], H],
		["user switched the chip off", [true, true, true, true, true, false, true], H],
		["a gate that would allow cannot show a chip that is switched off", [true, true, true, true, true, false, false], H],
		["nothing configured, gate irrelevant", [true, true, false, false, false, true, true], H],
	]
	for row in named:
		var a: Array = row[1]
		_ok(AssistantChip.decide(a[0], a[1], a[2], a[3], a[4], a[5], a[6]) == String(row[2]), "%s -> %s" % [String(row[0]), String(row[2])])
	var mismatches := 0
	var seen := {"hidden": 0, "disabled": 0, "live": 0}
	for n in 128:
		var bits: Array = []
		var held := 0
		for b in 7:
			var v := (n >> b) & 1 == 1
			bits.append(v)
			if b < 6 and v:
				held += 1
		## Independent statement: the chip is shown iff all SIX non-gate conditions hold (held == 6); then
		## the gate picks live or greyed.
		var want := H
		if held == 6:
			want = L if bool(bits[6]) else D
		var got := AssistantChip.decide(bits[0], bits[1], bits[2], bits[3], bits[4], bits[5], bits[6])
		if got != want:
			mismatches += 1
		seen[want] += 1
	_ok(mismatches == 0, "all 128 combinations agree with the independent statement (%d mismatches)" % mismatches)
	_ok(int(seen["live"]) == 1 and int(seen["disabled"]) == 1 and int(seen["hidden"]) == 126, "exactly 1 live, 1 greyed and 126 hidden rows (%s)" % str(seen))
	_ok(AssistantChip.provider_configured("ollama", "", "m") and AssistantChip.provider_configured("", "http://x/v1", "m")
		and not AssistantChip.provider_configured("", "", "m") and not AssistantChip.provider_configured("ollama", "", "")
		and not AssistantChip.provider_configured("", "", ""), "provider_configured: a preset OR an address, AND a model")


# -- 0b -----------------------------------------------------------------------------------

func _default_state() -> void:
	print("== 0b. default: nothing shown, nothing resolved ==")
	await _reset_llm()
	await _settle(3)
	_ok(not _bar.assistant_chip().visible and _bar.assistant_state() == "hidden", "the cluster's Assistant chip is hidden")
	_ok(QuickBarRing.dock_items(_app).is_empty(), "`dock_items` is [] (no index built, nothing drawn)")
	var st := await _open_sticky(_app.viewport.overlay.get_global_rect().get_center())
	_ok(int(st["dock_count"]) == 0 and (st["dock_rects"] as Array).is_empty(), "the ring has no dock")
	await _shot("00_default_ring")
	await _ring_reset()
	_ok(_bar.pin_count() == 0, "no pin chip either")


# -- 2 ------------------------------------------------------------------------------------

func _live_settings() -> void:
	print("== 2. live settings: the real store and the real gate ==")
	await _reset_llm()
	DccSettings.set_llm_value("enabled", true)
	await _settle(1)
	_ok(_bar.assistant_state() == "hidden" and not _bar.assistant_chip().visible, "enabled alone: still hidden (no provider)")
	DccSettings.set_llm_value("preset_id", "ollama")
	DccSettings.set_llm_value("base_url", "http://127.0.0.1:%d/v1" % FAKE_PORT)
	await _settle(1)
	_ok(_bar.assistant_state() == "hidden", "a provider but no model: still hidden")
	## The appearance bar: ONE process frame after the write.
	DccSettings.set_llm_value("model", "mock-model")
	_ok(_bar.assistant_state() == "disabled", "the state follows the write inside the same call (no frame yet)")
	await get_tree().process_frame
	_ok(_bar.assistant_chip().visible and _bar.assistant_chip().disabled, "one process frame after the model is set the chip is drawn, greyed")
	var gate := _gate()
	_ok(not bool(gate.get("ok", true)) and String(gate.get("reason", "")) != "", "the real gate refuses, with a reason: '%s'" % String(gate.get("reason", "")).left(80))
	_ok(_bar.assistant_chip().tooltip_text == "Assistant unavailable: " + String(gate["reason"]), "the tooltip is the gate's own reason, word for word")
	_ok(AssistantChip.state_for_app(_app)["reason"] == String(gate["reason"]), "and so is the state's `reason`")
	var first_reason := String(gate["reason"])
	await _shot("01_chip_greyed")
	DccSettings.set_llm_value("local_allowed", true)
	await get_tree().process_frame
	var g2 := _gate()
	_ok(not bool(g2.get("ok", true)) and _bar.assistant_chip().disabled, "local allowed but no consent yet: still greyed")
	_ok(String(g2["reason"]) != first_reason and _bar.assistant_chip().tooltip_text.ends_with(String(g2["reason"])), "the reason changed to the next obstacle and the tooltip followed it: '%s'" % String(g2["reason"]).left(80))
	DccSettings.set_llm_value("prose", true)
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % FAKE_PORT)
	await get_tree().process_frame
	_ok(bool(_gate().get("ok", false)), "after consent the real gate allows")
	_ok(_bar.assistant_state() == "live" and _bar.assistant_chip().visible and not _bar.assistant_chip().disabled, "one frame later the chip is live")
	_ok(_bar.assistant_chip().tooltip_text == "Open the assistant panel", "its tooltip says what a click does")
	await _shot("02_chip_live")
	## withdrawing consent greys it again
	DccSettings.llm_withdraw_consent()
	await get_tree().process_frame
	_ok(_bar.assistant_state() == "disabled", "withdrawing consent greys the chip within a frame")
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % FAKE_PORT)
	## the user switch
	_ok(DccSettings.set_quick_bar_value("assistant_button", false), "the switch writes")
	await get_tree().process_frame
	_ok(_bar.assistant_state() == "hidden" and not _bar.assistant_chip().visible, "switched off: hidden within a frame although the gate allows")
	_ok(QuickBarRing.dock_items(_app).is_empty(), "and the dock no longer carries it")
	_ok(DccSettings.set_quick_bar_value("assistant_button", true), "switched back on")
	await get_tree().process_frame
	_ok(_bar.assistant_chip().visible and not _bar.assistant_chip().disabled, "back within a frame, live")
	DccSettings.set_llm_value("enabled", false)
	await get_tree().process_frame
	_ok(not _bar.assistant_chip().visible, "assistant disabled again: hidden within a frame, no restart")
	_ok(not DccSettings.set_quick_bar_value("assistant_button", "yes") and DccSettings.quick_bar_value("assistant_button") == true, "a wrong-typed switch value is refused and changes nothing")
	_ok(DccSettings.quick_bar_value("bar") == [] and DccSettings.quick_bar_value("dock") == [], "the chip never entered either pin list")
	_ok(_bar.pin_count() == 0, "pin_count() is 0 with the chip shown or hidden")


# -- 3 ------------------------------------------------------------------------------------

func _clicks() -> void:
	print("== 3. clicks ==")
	await _reset_llm()
	_configure_all()
	await _settle(3)
	await _hide_windows()
	_ok(_bar.assistant_chip().visible, "the chip is shown")
	_ok(_bar.assistant_chip().get_index() == 0, "and it is the cluster's FIRST child (index %d)" % _bar.assistant_chip().get_index())
	_ok(_bar.assistant_chip().get_global_rect().position.x < _bar.plus_chip().get_global_rect().position.x, "left of the `＋` chip")
	if DccTheme.is_tablet():
		var sz := _bar.assistant_chip().get_global_rect().size
		_ok(sz.x >= 44.0 and sz.y >= 44.0, "tablet: the chip is at least 44 x 44 as laid (%.0f x %.0f)" % [sz.x, sz.y])
	else:
		_ok(is_equal_approx(_bar.assistant_chip().get_global_rect().size.y, float(DccQuickBar.cell_px())), "desktop: the chip is the palette cell's height (%.0f)" % _bar.assistant_chip().get_global_rect().size.y)
	await _click(_bar.assistant_chip())
	_ok(_panel_open(), "a real click on the live chip opened the assistant panel")
	await _shot("03_panel_opened")
	await _hide_windows()
	## greyed: nothing opens
	DccSettings.llm_withdraw_consent()
	await _settle(3)
	_ok(_bar.assistant_chip().disabled, "consent withdrawn: the chip is greyed")
	await _click(_bar.assistant_chip())
	_ok(not _panel_open(), "a real click on the greyed chip opened nothing")
	_ok(not AssistantChip.open_panel(_app), "and `open_panel` refuses too (a second line of defence)")
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % FAKE_PORT)
	await _settle(3)
	## right-click menu
	await _click(_bar.assistant_chip(), MOUSE_BUTTON_RIGHT)
	var m: PopupMenu = _bar._assistant_menu
	## Ruling DQ: the menu now leads with one "Profile: <name>" row per assistant profile (the two built-ins on a
	## fresh settings file) and a separator, then the two rows it always had.
	var mt: Array = []
	for k in m.item_count:
		mt.append(m.get_item_text(k))
	_ok(m.visible and m.item_count == 6 and mt[0] == "Profile: Local" and mt[1] == "Profile: External" and mt[3] == "Hide the Assistant chip" and mt[5] == "Customise…", "a real right-click pops its menu: the profiles, then Hide the Assistant chip / Customise…: %s" % str(mt))
	await _shot("04_chip_menu")
	m.id_pressed.emit(DccQuickBar.ID_ASSISTANT_HIDE)   ## the row is chosen through its signal, not a pointer
	m.hide()
	await get_tree().process_frame
	_ok(DccSettings.quick_bar_value("assistant_button") == false and not _bar.assistant_chip().visible, "choosing it stores the switch off and the chip is gone within a frame")
	## the Customise window brings it back
	_app.open_quick_bar_window()
	await _settle(6)
	var chk: CheckBox = _win._assistant_chk
	_ok(chk != null and not chk.button_pressed, "the Customise window's checkbox shows OFF")
	_ok(_win._assistant_note.text.begins_with("The Assistant chip is hidden"), "its note says hidden: '%s'" % _win._assistant_note.text.left(70))
	var win_rect := Rect2(Vector2.ZERO, Vector2(_win.size))
	_ok(win_rect.encloses(_win._add_btn.get_global_rect()) and win_rect.encloses(_win._foot.get_global_rect()), "the Add button and the footer are still inside the window (add %s, window %s)" % [str(_win._add_btn.get_global_rect()), str(win_rect)])
	await _shot("05_window_switch_off")
	await _click(chk)
	_ok(DccSettings.quick_bar_value("assistant_button") == true and chk.button_pressed, "a real click on the checkbox stores the switch ON")
	await _settle(3)
	_ok(_bar.assistant_chip().visible and not _bar.assistant_chip().disabled, "and the chip is back, live")
	_ok(_win._assistant_note.text.begins_with("The Assistant chip is shown and opens"), "the note follows while the window is open: '%s'" % _win._assistant_note.text.left(70))
	await _click(chk)
	_ok(DccSettings.quick_bar_value("assistant_button") == false and not _bar.assistant_chip().visible, "a second click hides it again")
	await _click(chk)
	await _hide_windows()
	_ok(_bar.assistant_chip().visible, "restored for the sections after")
	## persisted
	DccSettings._loaded = false
	DccSettings._cfg = null
	_ok(DccSettings.quick_bar_value("assistant_button") == true, "the switch survives a reload of the settings file")


# -- 4 ------------------------------------------------------------------------------------

func _key_secret() -> void:
	print("== 4. the key never appears ==")
	await _reset_llm()
	_configure_all()
	OS.set_environment(ENV_VAR, SECRET)
	DccSettings.set_llm_value("env_var", ENV_VAR)
	await _settle(3)
	DccSettings.llm_withdraw_consent()   ## a greyed chip puts the gate's sentence in its tooltip
	await _settle(2)
	_app.open_quick_bar_window()
	await _settle(4)
	_ok(_texts_free_of(get_tree().root, SECRET), "no label, button text or tooltip anywhere in the tree contains the key (greyed chip, window open)")
	_ok(_bar.assistant_chip().tooltip_text != "" and not _bar.assistant_chip().tooltip_text.contains(SECRET), "the greyed tooltip is non-empty and key-free")
	var f := FileAccess.open("user://cartalith_settings.cfg", FileAccess.READ)
	var cfg_text := f.get_as_text() if f != null else ""
	_ok(f != null and cfg_text.contains("[llm]"), "the settings file was read back and has its [llm] section (%d bytes)" % cfg_text.length())
	_ok(not cfg_text.contains(SECRET), "and holds no copy of the key")
	var qb := ""
	if cfg_text.contains("[quick_bar]"):
		qb = cfg_text.substr(cfg_text.find("[quick_bar]") + 11)
		var nxt := qb.find("\n[")
		qb = qb if nxt < 0 else qb.substr(0, nxt)
	print("  note  quick_bar section: '%s'" % qb.replace("\n", " | "))
	_ok(not qb.contains("env_var") and not qb.contains("CART_LLM") and not qb.contains("model"), "the `[quick_bar]` section holds no provider setting")
	await _hide_windows()
	OS.set_environment(ENV_VAR, "")
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % FAKE_PORT)


# -- 5 ------------------------------------------------------------------------------------

func _shell_min_width() -> float:
	## The shell's minimum width is its root VBoxContainer's combined minimum (`_quickbar_probe.gd`).
	for c in _app.get_children():
		if c is VBoxContainer:
			return (c as VBoxContainer).get_combined_minimum_size().x
	return -1.0


func _width() -> void:
	print("== 5. width ==")
	await _reset_llm()
	await _settle(3)
	var hidden_w := _shell_min_width()
	var hidden_bar := _bar.get_combined_minimum_size().x
	_configure_all()
	await _settle(3)
	var live_w := _shell_min_width()
	var live_bar := _bar.get_combined_minimum_size().x
	DccSettings.llm_withdraw_consent()
	await _settle(3)
	var grey_w := _shell_min_width()
	var grey_bar := _bar.get_combined_minimum_size().x
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % FAKE_PORT)
	_ok(hidden_w > 0.0 and hidden_w == live_w and live_w == grey_w, "the shell's minimum width is identical hidden, live and greyed (%.0f / %.0f / %.0f)" % [hidden_w, live_w, grey_w])
	_ok(hidden_bar == live_bar and live_bar == grey_bar, "and so is the cluster's own minimum (%.0f / %.0f / %.0f)" % [hidden_bar, live_bar, grey_bar])
	_ok(live_bar == _bar.plus_chip().get_combined_minimum_size().x, "and the cluster's minimum is exactly the `＋` chip's, as before the Assistant chip existed (%.0f)" % live_bar)
	## 24 pins plus the chip: the pin counts are unchanged
	var pins: Array = []
	for k in _filler:
		if pins.size() < 6:
			pins.append({"k": String(k), "t": "p"})
	DccSettings.set_quick_bar_value("bar", pins)
	_bar.refresh()
	await _settle(4)
	_ok(_bar.pin_count() == pins.size() and _bar.assistant_chip().visible, "the chip is shown beside %d pins and `pin_count()` is still %d" % [pins.size(), pins.size()])
	var shown_with := _bar.shown_count() + _bar.overflow_titles().size()
	_ok(shown_with == pins.size(), "shown + overflow = %d pins with the chip present" % pins.size())
	var left_of_pins := true
	for c in _bar.chips():
		if c.visible and c.get_global_rect().position.x < _bar.assistant_chip().get_global_rect().end.x:
			left_of_pins = false
	_ok(_bar.assistant_chip().get_index() == 0 and left_of_pins, "with pins present the Assistant chip is still the FIRST child and lies left of every shown pin")
	## a bar too narrow for the chip: a standalone cluster in a 90 px box
	var holder := Control.new()
	holder.custom_minimum_size = Vector2(90, 60)
	holder.size = Vector2(90, 60)
	add_child(holder)
	var narrow := DccQuickBar.new()
	holder.add_child(narrow)
	narrow.size = Vector2(90, 60)
	narrow.setup(_app)
	await _settle(4)
	_ok(narrow.assistant_state() == "live" and not narrow.assistant_chip().visible, "in a 90 px bar the live chip does not fit and is not drawn")
	_ok(narrow.more_chip().visible, "the `⋯` chip stands in for it")
	narrow._on_more_pressed()
	var first_text := narrow._more_menu.get_item_text(0) if narrow._more_menu.item_count > 0 else ""
	_ok(first_text == "Assistant" and not narrow._more_menu.is_item_disabled(0), "and its menu's FIRST row is the live Assistant row: '%s'" % first_text)
	narrow._more_menu.hide()
	await _hide_windows()
	narrow._more_menu.id_pressed.emit(DccQuickBar.ID_ASSISTANT)
	await _settle(3)
	_ok(_panel_open(), "choosing that row opens the panel")
	await _hide_windows()
	DccSettings.llm_withdraw_consent()
	await _settle(2)
	narrow._on_more_pressed()
	print("  note  narrow: state=%s menu0='%s' disabled=%s tip='%s'" % [narrow.assistant_state(), narrow._more_menu.get_item_text(0), narrow._more_menu.is_item_disabled(0), narrow._more_menu.get_item_tooltip(0).left(70)])
	_ok(narrow._more_menu.get_item_text(0) == "Assistant" and narrow._more_menu.is_item_disabled(0) and narrow._more_menu.get_item_tooltip(0).contains(String(_gate()["reason"])), "greyed, the row is disabled and its tooltip carries the gate's reason")
	narrow._more_menu.hide()
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % FAKE_PORT)
	holder.queue_free()
	DccSettings.set_quick_bar_value("bar", [])
	_bar.refresh()
	await _settle(3)


# -- 6 ------------------------------------------------------------------------------------

func _store_dock(keys: Array) -> void:
	var idx := _idx()
	var pins: Array = []
	for k in keys:
		pins.append({"k": String(k), "t": String(idx.find_pin(String(k)).get("title", "ghost"))})
	_ok(DccSettings.set_quick_bar_value("dock", pins), "stored %d dock pins" % pins.size())


func _ring() -> void:
	print("== 6. the cartwheel ==")
	await _reset_llm()
	_configure_all()
	await _settle(3)
	await _hide_windows()
	var mid: Vector2 = _app.viewport.overlay.get_global_rect().get_center()
	var items := QuickBarRing.dock_items(_app)
	_ok(items.size() == 1 and bool(items[0].get("assistant", false)) and String(items[0]["title"]) == "Assistant" and bool(items[0]["enabled"]) and not items[0].has("pin_key"), "with nothing docked, `dock_items` is the Assistant chip alone, enabled, with no pin_key")
	var st := await _open_sticky(mid)
	_ok(int(st["dock_count"]) == 1 and (st["dock_rects"] as Array).size() == 1, "the ring draws a dock of one chip with no pin stored")
	var r0: Rect2 = st["dock_rects"][0]
	_ok(r0.position.y >= (st["centre"] as Vector2).y + float(st["disc_radius"]), "it sits under the disc (top %.0f px below the centre, disc %.0f)" % [r0.position.y - (st["centre"] as Vector2).y, float(st["disc_radius"])])
	if DccTheme.is_tablet():
		_ok(r0.size.x >= 44.0 and r0.size.y >= 44.0, "tablet: at least 44 x 44 as drawn (%.0f x %.0f)" % [r0.size.x, r0.size.y])
	await _shot("06_ring_assistant_only")
	await _ring_reset()
	_ok(DccSettings.quick_bar_value("dock") == [], "the dock store is still empty: the chip is not a pin")
	## a slow drag onto chip 0 runs it
	var res := await _rmb_to_chip(0)
	_ok(bool(res["visible"]) and int(res["dock_hover"]) == 0 and String(res["caption"]) == "Dock · Assistant", "a slow drag hovers chip 0 and the caption reads: '%s'" % String(res["caption"]))
	_ok(_panel_open(), "releasing over it opened the assistant panel")
	await _hide_windows()
	## flick: never selects
	var fl := await _rmb_to_chip(0, false)
	_ok(not bool(fl["visible"]) and int(fl["dock_hover"]) == -1 and not _panel_open(), "a flick (ring never drawn) selects nothing and opens nothing")
	await _hide_windows()
	## greyed
	DccSettings.llm_withdraw_consent()
	await _settle(2)
	var gres := await _rmb_to_chip(0)
	var reason := String(_gate()["reason"])
	_ok(int(gres["dock_hover"]) == 0 and String(gres["caption"]).begins_with("Dock · Assistant") and String(gres["caption"]).ends_with(reason), "greyed: the caption carries the gate's own reason: '%s'" % String(gres["caption"]).left(100))
	_ok(not _panel_open(), "and releasing over the greyed chip opens nothing")
	await _hide_windows()
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % FAKE_PORT)
	## four pins plus the chip
	var four := [_vault_key, _planner_key, _quickbar_key]
	for k in _filler:
		if four.size() < 4 and not four.has(k):
			four.append(k)
	_store_dock(four)
	items = QuickBarRing.dock_items(_app)
	_ok(items.size() == 5 and bool(items[0].get("assistant", false)) and not bool(items[1].get("assistant", false)), "four dock pins plus the chip are FIVE items, the chip first")
	var spare := ""
	for k in _filler:
		if not four.has(k):
			spare = String(k)
	st = await _open_sticky(mid)
	_ok(int(st["dock_count"]) == 5, "the ring draws five chips (%d)" % int(st["dock_count"]))
	var rects: Array = st["dock_rects"]
	var gaps_ok := true
	for k in range(1, rects.size()):
		if (rects[k] as Rect2).position.x < (rects[k - 1] as Rect2).end.x:
			gaps_ok = false
	_ok(gaps_ok, "five chips do not overlap")
	var row_w: float = (rects[rects.size() - 1] as Rect2).end.x - (rects[0] as Rect2).position.x
	var disc_w := 2.0 * float(st["disc_radius"])
	print("  note  dock row %.0f px wide against a disc %.0f px wide (%s)" % [row_w, disc_w, "wider" if row_w > disc_w else "inside"])
	await _shot("07_ring_assistant_and_four")
	await _ring_reset()
	_ok(DccSettings.quick_bar_value("dock").size() == 4, "the store holds exactly the four pins")
	var again := DccSettings.quick_bar_add("dock", spare, "spare")
	_ok(again == "The ring dock holds at most 4 pins; remove one first." and DccSettings.quick_bar_value("dock").size() == 4, "a 5th pin is still refused with the chip shown: '%s'" % again)
	## edges
	var ov: Rect2 = _app.viewport.overlay.get_global_rect()
	var view := Rect2(Vector2.ZERO, get_viewport().get_visible_rect().size)
	var pts := {
		"top": Vector2(ov.get_center().x, ov.position.y + 12.0),
		"bottom": Vector2(ov.get_center().x, ov.end.y - 12.0),
		"left": Vector2(ov.position.x + 12.0, ov.get_center().y),
		"right": Vector2(ov.end.x - 12.0, ov.get_center().y),
		"top-left": ov.position + Vector2(12, 12),
		"bottom-right": ov.end - Vector2(12, 12),
	}
	for nm in pts:
		var s2 := await _open_sticky(pts[nm])
		var rr: Array = s2["dock_rects"]
		var all_in := rr.size() == 5
		for r in rr:
			if not view.encloses(r):
				all_in = false
		_ok(all_in, "%s edge: all 5 chips inside the window (centre %s)" % [nm, str(s2["centre"])])
	await _ring_reset()
	## hidden again: the dock is gone
	_ok(DccSettings.set_quick_bar_value("dock", []), "the dock emptied (the four pins were only for the geometry above)")
	DccSettings.set_quick_bar_value("assistant_button", false)
	var st3 := await _open_sticky(mid)
	_ok(int(st3["dock_count"]) == 0 and (st3["dock_rects"] as Array).is_empty() and QuickBarRing.dock_items(_app).is_empty(), "switched off with nothing docked: no dock rects and no dock items")
	await _ring_reset()
	DccSettings.set_quick_bar_value("assistant_button", true)


func _rmb_to_chip(chip_index: int, slow: bool = true) -> Dictionary:
	await _ring_reset()
	await _hide_windows()
	var mid: Vector2 = _app.viewport.overlay.get_global_rect().get_center()
	_move_to(mid)
	await _settle(2)
	_button(mid, MOUSE_BUTTON_RIGHT, true)
	var st := _ring_state()
	var waited := 0
	## `debug_state()` of a CLOSED ring still reports its last rects (found by the first run: the stale
	## rects of the previous bottom-right ring were read as this ring's dock and the drag aimed at them).
	## `arm()` sets the centre and the dock synchronously at the press, so the new ring's rects are there
	## at once: take them only when the centre is near the press point. **Do not wait for `active`**: a
	## ring is active only after travel or the still-hold timer, and waiting 20 frames here let the
	## still-hold fire, after which the overlay stopped forwarding the drag (found by the same run).
	while waited < 3 and ((st.get("dock_rects", []) as Array).is_empty() or (st.get("centre", Vector2(-9999, -9999)) as Vector2).distance_to(mid) > 90.0):
		await get_tree().process_frame
		waited += 1
		st = _ring_state()
	var rects: Array = st.get("dock_rects", [])
	_ok(not rects.is_empty(), "the ring armed with a dock after %d frame(s)" % waited)
	var target: Vector2 = (rects[chip_index] as Rect2).get_center() if not rects.is_empty() else mid + Vector2(0, 200)
	var out := {}
	if slow:
		_move_to(mid + Vector2(0, 30), MOUSE_BUTTON_MASK_RIGHT)
		await get_tree().create_timer(0.30).timeout
		await _settle(2)
		var from := mid + Vector2(0, 30)
		for k in range(1, 7):
			_move_to(from.lerp(target, float(k) / 6.0), MOUSE_BUTTON_MASK_RIGHT)
			await _settle(2)
		var ms := _ring_state()
		out = {"visible": bool(ms.get("visible", false)), "dock_hover": int(ms.get("dock_hover", -1)), "caption": String(ms.get("caption", ""))}
		_button(target, MOUSE_BUTTON_RIGHT, false)
	else:
		_move_to(target, MOUSE_BUTTON_MASK_RIGHT)
		var ms := _ring_state()
		out = {"visible": bool(ms.get("visible", false)), "dock_hover": int(ms.get("dock_hover", -1)), "caption": String(ms.get("caption", ""))}
		_button(target, MOUSE_BUTTON_RIGHT, false)
	await _settle(6)
	return out


# -- phone --------------------------------------------------------------------------------

func _phone() -> void:
	print("== phone: no desktop cluster and no cartwheel; the phone's chip row and MORE section are checked in _quickbar_phone_probe.gd (CQ-6) ==")
	_ok(_bar == null, "the phone has no quick cluster")
	await _reset_llm()
	_configure_all()
	await _settle(3)
	_ok(String(AssistantChip.state_for_app(_app)["state"]) == "live", "the decision still answers on a phone (live), reading the same settings")
	DccSettings.set_quick_bar_value("assistant_button", false)
	_ok(String(AssistantChip.state_for_app(_app)["state"]) == "hidden", "and follows the switch")
	DccSettings.set_quick_bar_value("assistant_button", true)
	DccSettings.set_llm_value("enabled", false)
	await _settle(2)
	_ok(String(AssistantChip.state_for_app(_app)["state"]) == "hidden", "and the master switch")
	await _reset_llm()


# -- 7 ------------------------------------------------------------------------------------

func _sockets(pid: int, before: Dictionary, udp_before: int) -> void:
	print("== 7. sockets ==")
	## positive control: the detector sees a listening socket when one exists
	var server := TCPServer.new()
	var perr := server.listen(0, "127.0.0.1")
	var with_server := _netstat_tcp(pid)
	_ok(perr == OK and int(with_server["listening"]) > int(before["listening"]), "positive control: a listening socket is seen (%d -> %d)" % [int(before["listening"]), int(with_server["listening"])])
	server.stop()
	## one more full show / grey / hide / toggle cycle with the panel opened and closed
	if _mode != "phone":
		await _reset_llm()
		_configure_all()
		await _settle(2)
		DccSettings.llm_withdraw_consent()
		await _settle(2)
		DccSettings.llm_add_consent("http://127.0.0.1:%d" % FAKE_PORT)
		await _click(_bar.assistant_chip())
		await _hide_windows()
		DccSettings.set_quick_bar_value("assistant_button", false)
		await _settle(2)
		DccSettings.set_quick_bar_value("assistant_button", true)
		await _settle(2)
		await _open_sticky(_app.viewport.overlay.get_global_rect().get_center())
		await _ring_reset()
		await _reset_llm()
	await get_tree().create_timer(0.5).timeout
	var after := _netstat_tcp(pid)
	var udp_after := _netstat_udp(pid)
	_ok(int(before["other"]) == 0 and int(after["other"]) == 0, "0 non-listening TCP connections before and after the session (before %d, after %d) %s" % [int(before["other"]), int(after["other"]), str(after["rows"])])
	_ok(udp_after == udp_before, "no UDP socket appeared (before %d, after %d)" % [udp_before, udp_after])
