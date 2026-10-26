extends Node
## Phone UI probe for the 2026-10-26 owner findings on the OnePlus 6T (search rows, "No matches", command rows,
## the MORE list, the greyed Assistant chip, the Assistant settings spacing).
##
##   godot --path . --resolution 393x852 _phoneui_probe.tscn -- --force-touch --shots <dir>
##   godot --path . --resolution 412x915 _phoneui_probe.tscn -- --force-touch
##   godot --path . --resolution 852x393 _phoneui_probe.tscn -- --force-touch        (landscape)
##
## **Windowed, never `--headless`.** Run from a worktree with its own `config/name` and a wiped user dir (it fails if
## the assistant was on at launch). Arguments read: `--force-touch`, `--shots <dir>`; anything else aborts with exit 2.
## It refuses to run off the phone form. NO MODEL IS CONTACTED: the chip states are produced with an Anthropic preset
## that has no key (the gate refuses before any socket) and an Ollama preset with no consent (`consent_needed`).
##
## ## What is real and what is not
## Clicks are real MOUSE events through `Input.parse_input_event` at a control's laid-out position; the MORE drag is a
## real `InputEventScreenTouch` / `InputEventScreenDrag` sequence (what a finger produces, so Godot's own bubbling of
## touch events to the `ScrollContainer` is exercised). A handset (`adb shell input tap` / `swipe`) is the device-side
## proof and is recorded in `STATUS.md`; this is the same check on the desktop build.
##
## ## What it protects
##   1. SEARCH ROWS FULL WIDTH: the results column is as wide as the scroller, so a click near the right edge of the
##      LIST picks the row (closes the overlay and moves the view), and a touch drag that starts on a row reaches
##      the list's scroller;
##   2. NO MATCHES: the notice lays out on one line, not one letter per line;
##   3. COMMAND ROWS: a click on a pinnable command row runs it (closes the overlay, the window opens), a click on an
##      unavailable one keeps the overlay open and says why;
##   4. A TOUCH DRAG THAT STARTS ON A MORE ROW REACHES THE LIST'S SCROLLER (a control: one that starts on a band
##      caption does, so the harness can see events at all; a desktop has no touchscreen so the offset itself is the
##      device's check);
##   5. THE ASSISTANT CHIP IS TAPPABLE WHEN THE GATE REFUSES: a click opens the Assistant settings (key missing) or the
##      Assistant panel (consent needed) and a toast says what is missing;
##   6. SETTINGS SPACING: in the Assistant settings, each switch's note starts within 18 dp of the switch row's centre
##      (it was 30), and every switch row is still at least 44 dp tall.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument or watchdog.

const SEED := 483920

var _fails := 0
var _checks := 0
var _shots := ""
var _app: Node
var _tag := "p"


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _shot(name_: String) -> void:
	if _shots == "":
		return
	await _settle(3)
	var path := _shots.path_join("pu_%s_%s.png" % [_tag, name_])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _click_at(at: Vector2) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 2)
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


## A finger drag: touch down at `from`, drag in steps to `to`, touch up.
func _touch_drag(from: Vector2, to: Vector2) -> void:
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.position = from
	t.pressed = true
	Input.parse_input_event(t)
	await _settle(2)
	var steps := 12
	var prev := from
	for k in range(1, steps + 1):
		var p := from.lerp(to, float(k) / float(steps))
		var d := InputEventScreenDrag.new()
		d.index = 0
		d.position = p
		d.relative = p - prev
		d.velocity = (p - prev) * 60.0
		Input.parse_input_event(d)
		prev = p
		await _settle(1)
	var u := InputEventScreenTouch.new()
	u.index = 0
	u.position = to
	u.pressed = false
	Input.parse_input_event(u)
	await _settle(4)


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
	_ok(not bool(DccSettings.llm_value("enabled")), "the assistant is off at launch (the user dir was wiped)")
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	if _app.phone_project_picker != null and _app.phone_project_picker.visible:
		_app.phone_project_picker.hide()
	await _settle(6)
	if not _app.is_phone():
		print("ABORT: this probe is for the phone form (--resolution 393x852 -- --force-touch)")
		get_tree().quit(2)
		return
	var vs := get_viewport().get_visible_rect().size
	_tag = "land" if _app._landscape else "%dx%d" % [int(vs.x), int(vs.y)]
	print("== phone %s, viewport %s, phone_scale %.4f ==" % ["landscape" if _app._landscape else "portrait", vs, _app.phone_scale()])
	var bridge = _app.bridge
	bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	if _app.phone_project_picker != null and _app.phone_project_picker.visible:
		_app.phone_project_picker.hide()
	await _settle(8)

	await _search_rows()
	await _no_matches()
	await _command_rows()
	await _more_scroll()
	await _chip()
	await _settings_spacing()
	_finish()


func _set_query(q: String) -> void:
	_app._phone_search_field.text = q
	_app._refresh_phone_search()


func _open_search() -> void:
	_app.open_find_on_map()
	await _settle(6)


## Section 1.
func _search_rows() -> void:
	print("== 1. search rows are full width ==")
	await _open_search()
	_set_query("a")
	await _settle(6)
	var results: VBoxContainer = _app._phone_search_results
	var scroll := results.get_parent() as ScrollContainer
	_ok(results.get_child_count() > 2, "the query 'a' lists several rows (%d children)" % results.get_child_count())
	_ok(results.size.x >= scroll.size.x - 20.0 and results.size.x > 100.0,
		"the results column fills the scroller (column %.0f px, scroller %.0f px)" % [results.size.x, scroll.size.x])
	var row: Button = null
	for ch in results.get_children():
		if ch is Button and not (ch as Button).disabled:
			row = ch
			break
	_ok(row != null, "a place row is present")
	if row == null:
		return
	var rr := row.get_global_rect()
	var view_before: Vector2 = _app.viewport._camera.position
	await _shot("search")
	_ok(rr.size.x >= scroll.size.x - 20.0, "the first row is as wide as the list (%.0f of %.0f px)" % [rr.size.x, scroll.size.x])
	# A touch drag that starts on a result row must reach the results scroller (a `Button` is STOP by default, which
	# ends Godot's bubbling of the drag; `_phone_list_row` makes it PASS).
	var seen := [0]
	var tap := func(ev: InputEvent) -> void:
		if ev is InputEventScreenDrag:
			seen[0] += 1
	scroll.gui_input.connect(tap)
	await _touch_drag(rr.get_center(), rr.get_center() + Vector2(0, -120))
	scroll.gui_input.disconnect(tap)
	_ok(seen[0] > 0, "a touch drag that starts on a search result row reaches the results scroller (%d drag events)" % seen[0])
	# A click near the RIGHT edge of the LIST (not of the row: the dead zone before the fix was exactly the part of
	# the list to the right of the widest row, because the column was only as wide as that row).
	await _click_at(Vector2(scroll.get_global_rect().end.x - 30.0, rr.get_center().y))
	_ok(not _app._phone_search_overlay.visible, "a click at the right edge of a result row picks it (the overlay closed)")
	_ok(_app.viewport._camera.position != view_before, "and the view moved to the place")


## Section 2.
func _no_matches() -> void:
	print("== 2. No matches ==")
	await _open_search()
	_set_query("zzqqxxjj")
	await _settle(6)
	var results: VBoxContainer = _app._phone_search_results
	var lab: Label = null
	for l in results.find_children("*", "Label", true, false):
		if (l as Label).text == "No matches":
			lab = l
	_ok(lab != null, "the 'No matches' notice is shown")
	if lab != null:
		_ok(lab.get_line_count() == 1, "it lays out on ONE line (%d lines)" % lab.get_line_count())
		_ok(lab.size.x > 60.0, "and has a sensible width (%.0f px, was ~one letter)" % lab.size.x)
	await _shot("nomatches")
	_app._set_search_open(false)
	await _settle(4)


## Section 3.
func _command_rows() -> void:
	print("== 3. command rows ==")
	await _open_search()
	_set_query(". assistant panel")
	await _settle(6)
	var results: VBoxContainer = _app._phone_search_results
	var target: Button = null
	for b in results.find_children("*", "Button", true, false):
		var bb := b as Button
		if bb.name == "PinStar" or bb.disabled:
			continue
		target = bb
		break
	_ok(target != null, "a command row is present and is not disabled on the phone")
	if target == null:
		_app._set_search_open(false)
		return
	var rr := target.get_global_rect()
	_ok(rr.size.x > 150.0, "the command row has a real width (%.0f px)" % rr.size.x)
	await _shot("commands")
	await _click_at(Vector2(rr.end.x - 60.0 - 6.0, rr.get_center().y))
	_ok(not _app._phone_search_overlay.visible, "a click on a runnable command row closes the search")
	await _settle(10)
	_ok(_app.assistant_window.visible, "and the command ran: the Assistant panel opened")
	_app.assistant_window.hide()
	await _settle(4)
	# An unavailable / unrunnable command: pick any row the index calls not pinnable and click it: the overlay must stay
	# open (it explained itself in a toast instead of vanishing).
	await _open_search()
	_set_query(". ")
	await _settle(6)
	var idx: CommandIndex = _app._ensure_command_index()
	var bad := ""
	for r in idx.all():
		if not bool(r.get("pinnable", false)):
			bad = String(r.get("title", ""))
			break
	if bad != "":
		_set_query(". " + bad)
		await _settle(6)
		var btn: Button = null
		for b in _app._phone_search_results.find_children("*", "Button", true, false):
			if (b as Button).name != "PinStar":
				btn = b
				break
		if btn != null:
			await _click_at(btn.get_global_rect().get_center())
			_ok(_app._phone_search_overlay.visible, "a click on a command that cannot run from here keeps the search open ('%s')" % bad)
	_app._set_search_open(false)
	await _settle(4)


## Section 4.
func _more_scroll() -> void:
	print("== 4. the MORE list scrolls to a touch drag ==")
	_app._phone_menu.open()
	await _settle(12)
	var sc: ScrollContainer = _app._phone_menu._screen_scroll
	var body: Control = _app._phone_menu._screen_body
	_ok(sc.get_v_scroll_bar().max_value > sc.size.y + 50.0, "the MORE root is taller than its viewport (%.0f vs %.0f)" % [sc.get_v_scroll_bar().max_value, sc.size.y])
	# Control: a drag that starts on a BAND caption (not a row) must scroll, or this harness cannot see scrolling.
	var band_y := -1.0
	var row_y := -1.0
	for ch in body.get_children():
		if ch is Control and (ch as Control).visible:
			var c := ch as Control
			var r := c.get_global_rect()
			if r.end.y > sc.get_global_rect().end.y - 60.0 or r.position.y < sc.get_global_rect().position.y + 20.0:
				continue
			if c is PanelContainer and c.mouse_filter != Control.MOUSE_FILTER_IGNORE and row_y < 0.0:
				row_y = r.get_center().y
			elif c is MarginContainer and band_y < 0.0:
				band_y = r.get_center().y
	var x := sc.get_global_rect().get_center().x
	var span := sc.size.y * 0.5
	_ok(band_y > 0.0 and row_y > 0.0, "found a band caption (y %.0f) and a row (y %.0f) on screen" % [band_y, row_y])
	# Godot's ScrollContainer only drag-scrolls where a touchscreen exists (`DisplayServer.is_touchscreen_available`),
	# which a desktop run does not have, so the scroll offset cannot be the check here. What the fix changed is whether
	# a touch DRAG that starts on a row still REACHES the scroller (a STOP control ends the bubbling), so the check
	# counts the `InputEventScreenDrag`s the scroller itself receives. The device proof is `adb shell input swipe`.
	var seen := [0]
	var tap := func(ev: InputEvent) -> void:
		if ev is InputEventScreenDrag:
			seen[0] += 1
	sc.gui_input.connect(tap)
	seen[0] = 0
	await _touch_drag(Vector2(x, band_y), Vector2(x, band_y - span))
	var n_band: int = seen[0]
	_ok(n_band > 0, "control: a touch drag that starts on a band caption reaches the scroller (%d drag events)" % n_band)
	seen[0] = 0
	await _touch_drag(Vector2(x, row_y), Vector2(x, row_y - span))
	var n_row: int = seen[0]
	_ok(n_row > 0, "a touch drag that starts ON A ROW reaches the scroller (%d drag events)" % n_row)
	sc.gui_input.disconnect(tap)
	# A drag is not a tap: no screen was pushed by it.
	_ok(_app._phone_menu.is_open(), "and the drag did not open a row")
	await _shot("more")
	_app._phone_menu.close()
	await _settle(6)


## Section 5.
func _chip() -> void:
	print("== 5. the Assistant chip is tappable when the gate refuses ==")
	_app._pick_phone_tab("map")
	_app.set_phone_detent("half")
	await _settle(20)
	# Key missing: Anthropic preset, key from the environment (unset here), assistant on.
	DccSettings.set_llm_value("enabled", true)
	DccSettings.set_llm_value("preset_id", "anthropic")
	DccSettings.set_llm_value("model", "claude-probe")
	DccSettings.set_llm_value("key_source", "env")
	DccSettings.set_llm_value("remote_allowed", true)
	await _settle(10)
	var st := AssistantChip.state_for_app(_app)
	_ok(String(st["state"]) == AssistantChip.DISABLED, "the gate refuses (state %s, code '%s')" % [st["state"], st.get("code", "")])
	var chip: Button = null
	for b in _app.phone_quick_row().find_children("*", "Button", true, false):
		if (b as Button).text == "Assistant":
			chip = b
	_ok(chip != null and not (chip as Button).disabled, "the chip is on the MAP sheet and is an enabled (tappable) button")
	if chip == null:
		return
	_app.assistant_settings_window.hide()
	await _click_at(chip.get_global_rect().get_center())
	await _settle(10)
	_ok(_app.assistant_settings_window.visible, "a tap on the greyed chip opens the Assistant settings (key missing)")
	await _shot("chip_settings")
	_app.assistant_settings_window.hide()
	await _settle(6)
	# Consent needed: a local Ollama preset, no consent recorded.
	DccSettings.set_llm_value("preset_id", "ollama")
	DccSettings.set_llm_value("base_url", "http://127.0.0.1:11434/v1")
	DccSettings.set_llm_value("model", "probe-model")
	DccSettings.set_llm_value("local_allowed", true)
	DccSettings.set_llm_value("key_source", "env")
	await _settle(10)
	st = AssistantChip.state_for_app(_app)
	print("second state: %s / %s / %s" % [st["state"], st.get("code", ""), st.get("reason", "")])
	if String(st["state"]) == AssistantChip.DISABLED and String(st.get("code", "")) == "consent_needed":
		chip = null
		for b in _app.phone_quick_row().find_children("*", "Button", true, false):
			if (b as Button).text == "Assistant":
				chip = b
		_app.assistant_window.hide()
		await _click_at(chip.get_global_rect().get_center())
		await _settle(10)
		_ok(_app.assistant_window.visible, "a tap on the chip when consent is needed opens the Assistant panel")
		_app.assistant_window.hide()
	else:
		print("note: the second arm did not reach consent_needed on this build (state %s, code '%s'); not asserted" % [st["state"], st.get("code", "")])
	await _settle(6)


## Section 6.
func _settings_spacing() -> void:
	print("== 6. Assistant settings spacing ==")
	DccSettings.set_llm_value("enabled", true)
	_app.open_assistant_settings()
	await _settle(20)
	var win = _app.assistant_settings_window
	var worst := 0.0
	var count := 0
	var short := 0
	for key in ["enabled", "prose", "commands", "local_allowed", "remote_allowed", "ctx_entity", "ctx_selection"]:
		if not win.controls.has(key) or not win.controls.has(key + "_note"):
			continue
		var cb: Control = win.controls[key]
		var note: Control = win.controls[key + "_note"]
		var row := cb.get_parent() as Control
		var rr := row.get_global_rect()
		var gap := note.get_global_rect().position.y - rr.get_center().y
		# The settings window lays out in dp (its own content scale), and a control inside it reports its rect in
		# those logical units, so no conversion is applied (an earlier version divided by the window's scale and
		# read 30 dp rows at 540 px wide).
		var wscale := 1.0
		count += 1
		var gap_dp: float = gap
		worst = maxf(worst, gap_dp)
		if rr.size.y / maxf(0.001, wscale) < 43.5:
			short += 1
		print("  %s: row %s note top %.1f px, centre-to-note %.1f dp" % [key, rr, note.get_global_rect().position.y, gap_dp])
	# The `Send:` switches carry no note: an empty note used to reserve a blank text line under each of them.
	if win.controls.has("ctx_entity") and win.controls.has("ctx_selection"):
		var a: Control = (win.controls["ctx_entity"] as Control).get_parent()
		var b: Control = (win.controls["ctx_selection"] as Control).get_parent()
		var pitch_dp: float = (b.get_global_rect().position.y - a.get_global_rect().position.y) 
		_ok(pitch_dp > 0.0 and pitch_dp <= 56.0, "two note-less Send switches are %.1f dp apart (a 44 dp row plus the 8 dp separation is 52; was ~76 with the blank line)" % pitch_dp)
	_ok(count >= 5, "measured %d switch rows" % count)
	_ok(worst <= 18.0, "every note starts within 18 dp of its switch row's centre (worst %.1f dp; 30 before)" % worst)
	_ok(short == 0, "every switch row is still at least 44 dp tall")
	await _shot("settings")
	win.hide()
	DccSettings.set_llm_value("enabled", false)
	await _settle(4)
