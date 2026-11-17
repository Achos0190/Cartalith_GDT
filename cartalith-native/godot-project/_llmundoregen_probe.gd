extends Node
## **The Assistant's "Undo and regenerate" offer, in the real shell** (`LLM_ASSISTANT_SCOPE.md` s.5b.4
## item 6; Ruling DS follow-up). No mock server and no model: the setting proposal is handed to the real
## card builder (`llm_assistant_window.gd::_on_proposal`) as the dictionary Rust's validator produces, and
## every step after that is a REAL mouse click on the real buttons, a real generate and the real
## grouped undo (`undo_group.gd`).
##   desktop: godot --path . --resolution 1600x900 res://_llmundoregen_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1700x1000 res://_llmundoregen_probe.tscn -- --force-touch --density=1.0 --shots <dir>
##   phone:   godot --path . --resolution 1080x2340 res://_llmundoregen_probe.tscn -- --force-touch --density 2.625 --shots <dir>
## **Windowed, never `--headless`**, with a wiped user dir. Arguments: `--shots <dir>`, `--force-touch`,
## `--density D` (read by the shell); anything else exits 2. `--failfast` ends the run at the first
## failed check (the mutation harness).
##
## What it protects (each check names it):
##   1. NO OFFER before Apply, and none when Apply did not rebuild (the switch off, or live-only keys).
##   2. OFFER after "Apply and regenerate": a button reading "Undo and regenerate", enabled once the
##      rebuild finished.
##   3. ONE PRESS: starts EXACTLY one generate, restores all five settings, and the world digest
##      equals the pre-Apply digest (generation is deterministic in settings and seed); the card and its
##      group read undone; the grouped stack did not grow (no second undo stack); Redo is offered.
##   4. V13: the offer is disabled while a world generates, and once used.
##   5. NEWEST ONLY: with a newer group on top the older card's offer is disabled with its reason, and
##      pressing it changes nothing and starts no generate.
##   6. TOUCH: on tablet and phone the button meets the 44 px floor and lies inside the screen.
const SEED := 4242
const WANT := {"climate.ocean": 0.7, "climate.evap": 0.09, "climate.rain_dep": 0.5, "peak_m": 2800.0, "tect.alpha": 0.65}

var _fails := 0
var _checks := 0
var _failfast := false
var _mode := "desktop"
var _shots := ""
var _app: Node
var _bridge
var _win
var _gen_started := 0


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1
		if _failfast:
			_finish()


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _shot(tag: String, card: Dictionary = {}) -> void:
	if _shots == "":
		return
	await _settle(4)
	if not card.is_empty():
		## The card the frame is about, scrolled into the transcript's small viewport.
		var tgt = card.get("undo_regen")
		await _reveal(tgt if tgt != null and is_instance_valid(tgt) else card["box"])
	var path := _shots.path_join("undoregen_%s_%s.png" % [_mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


func _finish() -> void:
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


## Scrolls EVERY scroll container above `c` (the dock's, then the transcript's) so `c` is on screen.
func _reveal(c: Control) -> void:
	var p := c.get_parent()
	while p != null:
		if p is ScrollContainer:
			(p as ScrollContainer).ensure_control_visible(c)
			await _settle(2)
		p = p.get_parent()


## A REAL mouse press and release at the centre of `c`, scrolled into view first.
func _click(c: Control) -> void:
	await _reveal(c)
	var vp := c.get_viewport()
	var at: Vector2 = vp.get_final_transform() * c.get_global_rect().get_center()
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


func _digest() -> Dictionary:
	return (_bridge.edit_state_digest() as Dictionary).duplicate()


func _generate() -> void:
	## Through the New World dialog's own request, because "Apply and regenerate" rebuilds from THAT dialog's
	## seed and size (`NewWorldDialog.generate_checked`): the world a press rebuilds is the previous world
	## only when the dialog still holds the values it was generated with, which is the real flow.
	var dlg = _app.new_world_dialog
	dlg.seed_input.value = SEED
	dlg.width_input.value = 1200.0
	dlg.grid_w_input.value = 256
	dlg.grid_h_input.value = 164
	dlg.generate_checked()
	await _settle(2)
	await _wait_idle()


func _wait_idle() -> void:
	while _bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _settle(45)  ## the busy overlay fades out over about half a second
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()


## The proposal the validator would hand the card: `rows` is `{key: new}`; `regen` the engine's class.
func _info(rows: Dictionary, regen: String, summary: String) -> Dictionary:
	var valid: Array = []
	for k in rows:
		valid.append({"key": k, "label": k, "old": _bridge.param_get(k), "new": rows[k], "unit": "", "regen": regen, "reason": "probe"})
	return {"summary": summary, "valid": valid, "cost": PackedStringArray(), "rejected": []}


func _propose(rows: Dictionary, regen: String, summary: String) -> Dictionary:
	_win._on_proposal(_info(rows, regen, summary))
	return _win.cards[_win.cards.size() - 1]


func _settings_are(want: Dictionary) -> bool:
	for k in want:
		if absf(float(_bridge.param_get(k)) - float(want[k])) > 1e-9:
			return false
	return true


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--force-touch":
			pass
		elif args[i] == "--density" and i + 1 < args.size():
			i += 1
		elif args[i].begins_with("--density="):
			pass
		elif args[i] == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		elif args[i] == "--failfast":
			_failfast = true
		else:
			print("ABORT: unrecognised argument %s -- this probe reads --force-touch, --density, --shots <dir>, --failfast" % args[i])
			get_tree().quit(2)
			return
		i += 1
	if DisplayServer.get_name() == "headless":
		print("ABORT: run windowed; this probe clicks real buttons and saves frames")
		get_tree().quit(2)
		return
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
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
	_mode = "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s, viewport %s ==" % [_mode, get_viewport().get_visible_rect().size])
	_bridge = _app.bridge
	_win = _app.assistant_window
	_ok(_bridge.llm_api and _win != null and _win.has_method("_undo_regen_card"), "the library has LlmBridge; the panel has the Undo and regenerate path")
	if not _bridge.llm_api or _win == null:
		_finish()
		return
	_bridge.generation_started.connect(func(): _gen_started += 1)
	DccSettings.set_llm_value("enabled", true)
	DccSettings.set_llm_value("regen_on_apply", true)
	await _generate()
	_win.open()
	await _settle(10)
	await _shot("1_open")
	print("  (the Assistant window: size %s, contents minimum %s)" % [str(_win.size), str(_win.get_contents_minimum_size())])
	## The phone's sheet is the whole window, so its contents must fit the window's width: 407 px against 393 was the old
	## measurement (the Place facts row's 160 px label beside two 44 px buttons), which pushed Apply and Send past the edge.
	if _mode == "phone":
		_ok(float(_win.get_contents_minimum_size().x) <= float(_win.size.x) + 0.5, "the phone Assistant's contents minimum %.0f fits its %d px window" % [_win.get_contents_minimum_size().x, _win.size.x])

	print("== 1. no offer before Apply, none without a rebuild ==")
	var peak0 := float(_bridge.param_get("peak_m"))
	var d0 := _digest()
	var card := _propose(WANT, "generate", "Less ocean moisture and a lower relief ceiling.")
	_ok(card.get("undo_regen") == null, "no offer before Apply")
	DccSettings.set_llm_value("regen_on_apply", false)
	var c_off := _propose({"peak_m": 3100.0}, "generate", "taller, switch off")
	await _click(c_off["apply"])
	_ok(c_off["state"] == "applied" and c_off.get("undo_regen") == null, "with 'Regenerate when I apply' off, Apply leaves no offer (nothing was rebuilt)")
	_app.undo_last()
	await _settle(4)
	_ok(absf(float(_bridge.param_get("peak_m")) - peak0) < 1e-9, "(that Apply was undone: peak_m %.0f)" % float(_bridge.param_get("peak_m")))
	DccSettings.set_llm_value("regen_on_apply", true)
	var c_live := _propose({"peak_m": 3100.0}, "live", "a live-class change")
	await _click(c_live["apply"])
	_ok(c_live["state"] == "applied" and c_live.get("undo_regen") == null, "a change that needs no rebuild leaves no offer")
	_app.undo_last()
	await _settle(4)

	print("== 2. the offer after Apply and regenerate ==")
	_ok(card["state"] == "new" and (card["apply"] as Button).text == "Apply and regenerate", "the card's button reads 'Apply and regenerate'")
	var g0 := _gen_started
	await _click(card["apply"])
	_ok(card["state"] == "applied", "the click applied the card: %s" % (card["reason"] as Label).text.left(60))
	await _wait_idle()
	_ok(_gen_started == g0 + 1, "Apply started exactly one generate (%d)" % (_gen_started - g0))
	var ub = card.get("undo_regen")
	_ok(ub != null and is_instance_valid(ub) and (ub as Button).text == "Undo and regenerate", "the applied-and-rebuilt card offers 'Undo and regenerate'")
	if ub == null or not is_instance_valid(ub):
		_finish()
		return
	_ok(not (ub as Button).disabled, "the offer is enabled once the rebuild finished ('%s')" % (ub as Button).tooltip_text)
	_ok(_settings_are(WANT), "the world was built WITH the applied settings")
	var d1 := _digest()
	_ok(d1 != d0, "the applied world differs from the first")
	await _shot("2_offer", card)
	if _mode != "desktop":
		await _reveal(ub)
		var r: Rect2 = (ub as Control).get_global_rect()
		_ok(r.size.y >= 43.5, "touch: the offer meets the 44 px floor (%s)" % str(r.size))
		_ok(get_viewport().get_visible_rect().encloses(r), "touch: the offer lies on screen (%s)" % str(r))

	print("== 4. V13: a generate started by anything else disables the offer, and its end re-enables it ==")
	_app.new_world_dialog.generate_checked()
	await _settle(3)
	_ok(_bridge.generating and (ub as Button).disabled, "while a world generates the offer is disabled (reason '%s')" % (ub as Button).tooltip_text.left(50))
	await _wait_idle()
	_ok(not (ub as Button).disabled, "when it ends the offer is enabled again (the group is still the newest step)")

	print("== 3. one press: settings, world, one group ==")
	var n_groups: int = _app.undo_groups.groups.size()
	g0 = _gen_started
	await _click(ub)
	_ok(_bridge.generating or _gen_started == g0 + 1, "the press started a generate")
	_ok((ub as Button).disabled, "the offer is disabled while the rebuild runs")
	await _wait_idle()
	_ok(_gen_started == g0 + 1, "EXACTLY one generate for the press (%d)" % (_gen_started - g0))
	_ok(absf(float(_bridge.param_get("peak_m")) - peak0) < 1e-9 and absf(float(_bridge.param_get("tect.alpha")) - 0.85) < 1e-9, "the previous settings are back (peak_m %.0f, tect.alpha %.2f)" % [float(_bridge.param_get("peak_m")), float(_bridge.param_get("tect.alpha"))])
	_ok(_digest() == d0, "the world is the previous world again (digest equals the pre-Apply digest)")
	_ok(card["state"] == "undone" and String(_app.undo_groups.state_of(int(card["group_id"]))) == "undone", "the card and its group read undone")
	_ok(_app.undo_groups.groups.size() == n_groups, "through the existing grouped stack, no second one (%d groups)" % _app.undo_groups.groups.size())
	_ok(_app.undo_groups.wants_redo(), "Edit > Redo can re-apply the settings")
	_ok((ub as Button).disabled, "the used offer is disabled")
	_ok((card["reason"] as Label).text.contains("not restored") and (card["reason"] as Label).text.contains("Rebuilding"), "the card says hand edits are not restored and that a rebuild runs")
	await _shot("3_undone", card)

	print("== 5. a newer group on top: the older offer is refused ==")
	var c2 := _propose(WANT, "generate", "again")
	await _click(c2["apply"])
	await _wait_idle()
	var c3 := _propose({"peak_m": 3000.0}, "generate", "taller")
	await _click(c3["apply"])
	await _wait_idle()
	_win._refresh_cards()
	var b2 = c2.get("undo_regen")
	var b3 = c3.get("undo_regen")
	_ok(b2 != null and (b2 as Button).disabled and (b2 as Button).tooltip_text.contains("newer change"), "the older card's offer is disabled with its reason: '%s'" % ((b2 as Button).tooltip_text.left(60) if b2 != null else "missing"))
	_ok(b3 != null and not (b3 as Button).disabled, "the newest card's offer is enabled")
	var g1 := _gen_started
	_win._undo_regen_card(c2)
	await _settle(8)
	_ok(_gen_started == g1 and absf(float(_bridge.param_get("peak_m")) - 3000.0) < 1e-9, "forcing the older offer changes nothing and starts no generate")
	await _shot("5_two_cards", c3)
	_finish()
