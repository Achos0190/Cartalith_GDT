extends Node
## Probe: the asset library inspector's `Move to…` verb (`asset_library_window.gd::_on_move_to_pressed` /
## `_on_move_target_picked`, `as_move_targets`, `as_move_item`), the way a folder import's unmatched ground
## candidates leave the `Ground candidates` set.
##
##   desktop: godot --path . --resolution 1600x900  _assetmove_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1700x1000 _assetmove_probe.tscn -- --force-touch --density=1.0 --shots <dir>
##   phone:   godot --path . --resolution 450x975   _assetmove_probe.tscn -- --force-touch --density 1.094 --shots <dir>
##
## **Windowed, never `--headless`.** Run with a scratch user dir (APPDATA) so the owner's asset library is never read
## or written; it only uses `as_*` calls on the session it clears first.
##
## Protects: 1. a candidate slot's inspector offers `Move to…` enabled, a real click on it pops a menu listing exactly
## the ground slots (labelled `Family · Slot`) and no sprite slot; 2. choosing `Splat channels · Grass` moves the
## variant (the grass slot gains it with the same content hash, the candidate slot is gone, the status line says so,
## the inspector no longer points at the removed slot); 3. a sprite slot's menu lists no ground slot; 4. a pin slot's
## button is disabled with the reason in its tooltip; 5. an empty slot's button is disabled.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument/watchdog.

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _app: Node


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _shot(tag: String) -> void:
	if _shots == "":
		return
	await _settle(3)
	var path := _shots.path_join("am_%s_%s.png" % [_mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _png(c: Color) -> PackedByteArray:
	var img := Image.create(40, 40, false, Image.FORMAT_RGBA8)
	img.fill(c)
	return img.save_png_to_buffer()


func _click(c: Control) -> void:
	## The phone's inspector scrolls: bring the control into view first, as a user's finger would.
	var p := c.get_parent()
	while p != null:
		if p is ScrollContainer:
			(p as ScrollContainer).ensure_control_visible(c)
		p = p.get_parent()
	await _settle(3)
	## The screen transform includes the owning window's position and content scale (the phone scales its windows).
	var at: Vector2 = c.get_screen_transform() * (c.size * 0.5)
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
		ev.button_mask = 1 if down else 0
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(3)


## Focuses a slot; on the phone the inspector is its own pane, so it is shown too (the user's route is the SLOT tab).
func _focus(win, uid: String) -> void:
	win.focus_slot(uid)
	if _mode == "phone":
		win._show_phone_pane("slot")
	await _settle(4)


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := args[i]
		if a == "--force-touch" or a.begins_with("--density="):
			pass
		elif a == "--density" and i + 1 < args.size():
			i += 1
		elif a == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
		i += 1
	get_tree().create_timer(400.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	if _app.phone_project_picker != null and _app.phone_project_picker.visible:
		_app.phone_project_picker.hide()
	await _settle(6)
	_mode = "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s, viewport %s ==" % [_mode, get_viewport().get_visible_rect().size])
	var br = _app.bridge
	br.as_clear_library()
	_app.open_asset_library()
	await _settle(10)
	var win = _app.asset_library_window
	_ok(win != null and win.visible, "the asset library window is open")

	var cand: String = br.as_add_custom_slot("moss", "Ground candidates")["uid"]
	_ok(bool(br.as_import_item(cand, "moss.png", _png(Color(0.2, 0.5, 0.2)))["ok"]), "a ground candidate is imported into the Ground candidates set")
	var cand_hash: String = br.as_item_summary(cand, 0)["hash"]
	win._refresh_grid()
	await _focus(win, cand)
	await _settle(6)
	var btn: Button = win._insp_move_btn
	_ok(btn != null and btn.visible and not btn.disabled, "Move to… is enabled for the candidate (tooltip: %s)" % btn.tooltip_text)
	await _click(btn)
	var menu: PopupMenu = win._insp_move_menu
	print("    after click: items=%d visible=%s btn_rect=%s menu_pos=%s size=%s" % [menu.item_count, str(menu.visible), str(btn.get_global_rect()), str(menu.position), str(menu.size)])
	var labels: Array = []
	for k in menu.item_count:
		labels.append(menu.get_item_text(k))
	_ok(menu.visible, "a real click on it pops the menu")
	var want := 0
	for f in ["textures", "biomes", "terrains"]:
		want += (br.as_family_slots(f) as Array).size()
	_ok(labels.size() == want and labels.has("Splat channels · Grass"), "the menu lists exactly the %d ground slots, labelled Family · Slot (%d: first %s)" % [want, labels.size(), str(labels.slice(0, 2))])
	_ok(not labels.any(func(l): return String(l).begins_with("Feature icons")), "and lists no sprite slot")
	await _shot("menu_open")
	var gi := labels.find("Splat channels · Grass")
	menu.hide()
	await _settle(2)
	menu.id_pressed.emit(menu.get_item_id(gi))
	await _settle(8)
	_ok(int(br.as_slot_summary("textures:grass").get("item_count", 0)) == 1, "the grass slot gained the variant")
	_ok(String(br.as_item_summary("textures:grass", 0).get("hash", "")) == cand_hash and cand_hash != "", "with the same content hash")
	_ok(not bool(br.as_slot_summary(cand).get("ok", true)), "the emptied candidate slot is gone")
	_ok(win._focused_uid == "", "the inspector no longer points at the removed slot")
	await _shot("after_move")

	# A sprite slot: no ground slot offered.
	br.as_import_item("icons:hill", "h.png", _png(Color(0.4, 0.3, 0.2)))
	win._refresh_grid()
	await _focus(win, "icons:hill")
	await _settle(6)
	await _click(win._insp_move_btn)
	var sl: Array = []
	for k in win._insp_move_menu.item_count:
		sl.append(win._insp_move_menu.get_item_text(k))
	win._insp_move_menu.hide()
	_ok(sl.size() > 0 and not sl.any(func(l): return String(l).begins_with("Splat") or String(l).begins_with("Biome") or String(l).begins_with("Terrain")), "a sprite's menu (%d rows) has no ground slot" % sl.size())

	# A pin slot: disabled with the reason.
	var pin: Dictionary = br.as_pin_add("Flag", "#336699", _png(Color(0.8, 0.1, 0.1)))
	_ok(bool(pin.get("ok", false)), "a pin is added")
	win._refresh_grid()
	await _focus(win, String(pin.get("uid", "")))
	await _settle(6)
	_ok(win._insp_move_btn.disabled and win._insp_move_btn.tooltip_text.contains("Pin section"), "a pin's Move to… is disabled and says why: %s" % win._insp_move_btn.tooltip_text)
	# An empty slot.
	await _focus(win, "icons:mountain")
	await _settle(6)
	_ok(win._insp_move_btn.disabled and win._insp_move_btn.tooltip_text.begins_with("Nothing to move"), "an empty slot's Move to… is disabled")
	_finish()
