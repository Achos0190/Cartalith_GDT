extends Node
## Windowed verification for `CultureProfilesWindow` (`GUI_GAP_REGISTER.md`
## CV-02). Opens the real app, generates a small world, opens the window
## through `app.open_culture_profiles()` exactly as the Factions category
## button does, asserts the culture list is real and non-empty, then drives
## the real per-faction culture `OptionButton` in the roster column (not the
## private setter directly) to change one faction's culture and re-reads
## `bridge.get_factions()` fresh afterward to confirm the write actually
## reached the engine, not just this window's own cached rows.
##
## Run: godot4 --path . _cultureprofiles_probe.tscn   (WINDOWED -- no
## `--headless`. A screenshot is taken for visual evidence and is not
## committed; delete it after review.)

var _app: Node

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		push_error("_cultureprofiles_probe: run WINDOWED. ImageTexture.update() is a "
			+ "no-op under --headless, so the screenshot would pass vacuously.")
		print("PROBE REFUSED: headless")
		get_tree().quit(2)
		return

	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	## The cold-start welcome modal (`app.gd::_ready()` -> `open_welcome()`) is
	## its own exclusive child window; leaving it open collides with this
	## probe's own dialog a moment later (`_confsix_probe.gd`'s own note: not
	## a shell flag, only the probe itself reads `--nowelcome` -- here it is
	## just always closed, since this probe drives the engine directly rather
	## than through the New World form).
	_app.open_project_dialog.hide()
	await get_tree().process_frame

	var bridge = _app.bridge
	bridge.generate({
		"seed": 77021, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout

	var fails := 0

	# -- window opens, real cultures list --------------------------------------
	_app.open_culture_profiles()
	await get_tree().process_frame
	await get_tree().process_frame
	var win: CultureProfilesWindow = _app.culture_profiles_window
	print("PROBE window visible=%s cultures=%d list_rows=%d" % [
		str(win.visible), win._cultures.size(), win._list_body.get_child_count()])
	if not win.visible:
		print("FAIL: window did not open")
		fails += 1
	if win._cultures.size() != 7:
		print("FAIL: expected 7 cultures, got %d" % win._cultures.size())
		fails += 1
	if win._list_body.get_child_count() != win._cultures.size():
		print("FAIL: list body child count %d != culture count %d" % [
			win._list_body.get_child_count(), win._cultures.size()])
		fails += 1
	for c in win._cultures:
		var d: Dictionary = c
		print("  culture id=%d key=%s name=%s affinity=%s factions=%d settlements=%d pop=%d" % [
			int(d.get("id", -1)), String(d.get("key", "")), String(d.get("name", "")),
			String(d.get("terrain_affinity", "")), int(d.get("faction_count", 0)),
			int(d.get("settlement_count", 0)), int(d.get("population", 0))])

	# -- real faction data reaches the roster column ---------------------------
	var factions_before: Array = bridge.get_factions()
	print("PROBE factions_before=%d roster_cards=%d" % [
		factions_before.size(), win._roster_body.get_child_count()])
	if factions_before.is_empty():
		print("FAIL: no factions -- villages:true should have produced some")
		fails += 1
		_finish(fails)
		return

	var target: Dictionary = factions_before[0]
	var fid := int(target.get("id", -1))
	var original_key := String(target.get("culture", "common"))
	var vocab := win.bridge.civ_culture_vocabulary()
	var new_key := ""
	for k in vocab:
		if String(k) != original_key:
			new_key = String(k)
			break
	print("PROBE target faction id=%d name=%s original_culture=%s -> new_culture=%s" % [
		fid, String(target.get("name", "?")), original_key, new_key])

	# -- drive the REAL OptionButton in the roster column, not the setter -----
	var ob := _find_option_button_for_faction(win, fid)
	if ob == null:
		print("FAIL: no OptionButton found for faction %d in the roster column" % fid)
		fails += 1
		_finish(fails)
		return
	var idx := -1
	for i in ob.item_count:
		if ob.get_item_text(i).to_lower() == new_key:
			idx = i
			break
	if idx < 0:
		print("FAIL: '%s' not offered in the culture OptionButton" % new_key)
		fails += 1
		_finish(fails)
		return
	ob.select(idx)
	ob.item_selected.emit(idx)
	await get_tree().process_frame
	await get_tree().process_frame

	# -- re-read fresh from the engine, not from any cached row ---------------
	var factions_after: Array = bridge.get_factions()
	var after_key := ""
	for f in factions_after:
		var fd: Dictionary = f
		if int(fd.get("id", -1)) == fid:
			after_key = String(fd.get("culture", ""))
			break
	print("PROBE re-read: bridge.get_factions() now reports faction %d culture=%s" % [fid, after_key])
	if after_key != new_key:
		print("FAIL: culture did not persist -- expected %s, engine reports %s" % [new_key, after_key])
		fails += 1
	else:
		print("PASS: faction %d's culture change reached the engine and re-reads back as %s" % [fid, new_key])

	# The window's own panels should also reflect it without reopening.
	if win._roster_body.get_child_count() > 0:
		print("PROBE window rebuilt roster_cards=%d (post-change)" % win._roster_body.get_child_count())

	# -- visual evidence, windowed only ----------------------------------------
	await get_tree().process_frame
	var img := get_viewport().get_texture().get_image()
	img.save_png("res://_cultureprofiles_probe.png")
	print("PROBE screenshot: res://_cultureprofiles_probe.png (%dx%d)" % [img.get_width(), img.get_height()])

	_finish(fails)


func _find_option_button_for_faction(win: CultureProfilesWindow, fid: int) -> OptionButton:
	## The roster column builds one `card` VBox per faction in
	## `bridge.get_factions()`'s own order, each holding exactly one
	## OptionButton (`DccWidgets.choice()`), so the Nth card's button belongs
	## to the Nth faction row -- found by position, not by re-deriving a name
	## match against a control tree text is a copy of.
	var factions := win.bridge.get_factions()
	var target_pos := -1
	for i in factions.size():
		if int((factions[i] as Dictionary).get("id", -1)) == fid:
			target_pos = i
			break
	if target_pos < 0:
		return null
	var obs := _collect_option_buttons(win._roster_body)
	if target_pos >= obs.size():
		return null
	return obs[target_pos]


func _collect_option_buttons(root: Node) -> Array:
	var out: Array = []
	for n in root.get_children():
		if n is OptionButton:
			out.append(n)
		out.append_array(_collect_option_buttons(n))
	return out


func _finish(fails: int) -> void:
	if fails == 0:
		print("PROBE PASS")
		get_tree().quit(0)
	else:
		print("PROBE FAIL: %d assertion(s)" % fails)
		get_tree().quit(1)
