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
## Protects: 1. a candidate slot's inspector offers `Move to…` enabled, a real click on it opens the Move-to sheet (2026-11-17: a themed, grouped, searchable list, rows 44 px
## on touch, inside the window) listing exactly the ground slots (labelled `Family · Slot`) and no sprite slot; 2. choosing `Splat channels · Grass` moves the
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
	var sheet: PopupPanel = win._move_sheet
	var vp := get_viewport().get_visible_rect()
	var labels: Array = []
	for row in win._move_rows:
		labels.append(String(row["label"]))
	print("    after click: rows=%d visible=%s sheet=%s viewport=%s" % [labels.size(), str(sheet.visible if sheet != null else false), str(Rect2(sheet.position, sheet.size)) if sheet != null else "-", str(vp)])
	_ok(sheet != null and sheet.visible, "a real click on it opens the Move-to sheet")
	var want := 0
	var fams := {}
	for f in ["textures", "biomes", "terrains"]:
		want += (br.as_family_slots(f) as Array).size()
	_ok(labels.size() == want and labels.has("Splat channels · Grass"), "the sheet lists exactly the %d ground slots, labelled Family · Slot (%d: first %s)" % [want, labels.size(), str(labels.slice(0, 2))])
	_ok(not labels.any(func(l): return String(l).begins_with("Feature icons")), "and lists no sprite slot")
	for l in labels:
		fams[String(l).split(" · ")[0]] = true
	_ok(win._move_heads.size() == fams.size() and fams.size() >= 2, "rows are grouped under one band per family (%d bands for %d families)" % [win._move_heads.size(), fams.size()])
	# The window is the content plus the frame's shadow on every side; the content is what must lie inside the view.
	var shadow: float = float((sheet.get_theme_stylebox("panel") as StyleBoxFlat).shadow_size)
	var content := Rect2(Vector2(sheet.position) + Vector2(shadow, shadow), Vector2(sheet.size) - Vector2(2.0 * shadow, 2.0 * shadow))
	print("    sheet content %s in viewport %s" % [str(content), str(vp)])
	_ok(Rect2(vp.position - Vector2(1, 1), vp.size + Vector2(2, 2)).encloses(content) and content.size.y < vp.size.y, "the sheet lies inside the window (a 35-row list scrolls inside it, it does not run off)")
	# Themed, not stock Godot: the frame is the shell's panel stylebox and a row carries the shell's ink.
	var frame := sheet.get_theme_stylebox("panel") as StyleBoxFlat
	_ok(frame != null and frame.bg_color.is_equal_approx(DccTheme.c("panel")) and frame.border_width_left == 1, "the sheet frame is the shell's bordered panel, not Godot's stock popup")
	_ok(sheet.has_theme_stylebox_override("panel"), "the shell's frame is set on the sheet itself (not inherited from the engine's defaults)")
	var first_btn: Button = win._move_rows[0]["btn"]
	_ok(first_btn.get_theme_color("font_color").is_equal_approx(DccTheme.c("text")) and first_btn.has_theme_stylebox_override("hover"), "a row carries the shell's ink and amber hover")
	# Row height: 44 on a touch composition, the menu pitch (28) otherwise, measured on screen.
	# On the phone 44 DP is device px x the shell's phone scale (48 px at 1.094): a bare 44 px row would be 40 dp.
	var floor_px := (float(_app.phone_tap(44)) if _mode == "phone" else 44.0) if DccTheme.is_touch() else 28.0
	var short := 0
	for row in win._move_rows:
		var rb: Button = row["btn"]
		if rb.size.y * rb.get_screen_transform().get_scale().y < floor_px - 0.5:
			short += 1
	var sc: float = first_btn.get_screen_transform().get_scale().y
	print("    row height %.1f px on screen (floor %.0f), %d row(s) short" % [first_btn.size.y * sc, floor_px, short])
	_ok(short == 0, "every row is at least %.0f px tall on screen" % floor_px)
	var search: LineEdit = win._move_search
	_ok(search.visible and search.is_visible_in_tree(), "the sheet has a search field")
	await _shot("menu_open")
	# Search: "grass" keeps only matching rows and hides the bands with none; nonsense leaves nothing and says so.
	search.text = "grass"
	search.text_changed.emit("grass")
	await _settle(3)
	var shown: Array = []
	for row in win._move_rows:
		if (row["btn"] as Button).visible:
			shown.append(String(row["label"]))
	_ok(shown.size() >= 1 and shown.size() < labels.size() and shown.all(func(l): return String(l).to_lower().contains("grass")), "searching 'grass' leaves only matching rows (%d of %d: %s)" % [shown.size(), labels.size(), str(shown.slice(0, 3))])
	var live_heads := 0
	for h in win._move_heads:
		if (h["head"] as Label).visible:
			live_heads += 1
	_ok(live_heads >= 1 and live_heads < win._move_heads.size(), "bands with no matching row are hidden (%d of %d left)" % [live_heads, win._move_heads.size()])
	await _shot("menu_search")
	search.text = "zzzz"
	search.text_changed.emit("zzzz")
	await _settle(2)
	var none_shown := 0
	for row in win._move_rows:
		if (row["btn"] as Button).visible:
			none_shown += 1
	_ok(none_shown == 0 and win._move_empty.visible, "a search with no match shows no rows and says so")
	search.text = ""
	search.text_changed.emit("")
	await _settle(2)
	var gi := labels.find("Splat channels · Grass")
	# A real click on the row (it is inside the scroller, so it is brought into view first).
	await _click(win._move_rows[gi]["btn"])
	await _settle(8)
	_ok(not sheet.visible, "choosing a row closes the sheet")
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
	for row in win._move_rows:
		sl.append(String(row["label"]))
	win._move_sheet.hide()
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
