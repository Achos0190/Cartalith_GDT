extends Node
## TEMPORARY, untracked probe -- opens every shell window against a real
## generated world and dumps what each one actually renders: the visible text
## of every Label/Button/Check/Option, plus geometry, plus a screenshot.
##
##   Godot_v4.7.1-stable_win64.exe --path . _winsweep_probe.tscn
##
## The point is to read the SHIPPED strings, not the source strings: a stale
## sentence, an unsubstituted format specifier or a control that renders empty
## only shows up here.

var _app: Node
var _bridge


func _p(s: String) -> void:
	print("WINSWEEP  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _text_of(n: Node) -> String:
	if n is Label:
		return (n as Label).text
	if n is RichTextLabel:
		return (n as RichTextLabel).get_parsed_text()
	if n is Button:
		var b := n as Button
		var extra := ""
		if b.disabled:
			extra += " [DISABLED]"
		if b is CheckBox or b is CheckButton:
			extra += " [%s]" % ("x" if b.button_pressed else " ")
		return b.text + extra
	if n is OptionButton:
		var o := n as OptionButton
		return "%s (%d items)%s" % [o.text, o.item_count, " [DISABLED]" if o.disabled else ""]
	if n is LineEdit:
		var le := n as LineEdit
		return "<LineEdit '%s' ph='%s'>" % [le.text, le.placeholder_text]
	if n is SpinBox:
		return "<SpinBox %s>" % str((n as SpinBox).value)
	if n is TextEdit:
		return "<TextEdit %d chars>" % (n as TextEdit).text.length()
	return ""


func _dump(n: Node, depth: int, out: Array) -> void:
	if n is Control and not (n as Control).visible:
		return
	var t := _text_of(n)
	if t != "":
		var c := n as Control
		out.append("%s%s: %s   @%s size%s" % [
			"  ".repeat(depth), n.get_class(), t,
			str(c.global_position.round()) if c else "?",
			str(c.size.round()) if c else "?"])
		depth += 1
	for ch in n.get_children():
		_dump(ch, depth, out)


func _shoot(name: String) -> void:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	img.save_png("res://_sweep_%s.png" % name)
	_p("shot _sweep_%s.png  %dx%d" % [name, img.get_width(), img.get_height()])


func _report(label: String, w: Node) -> void:
	await _frames(6)
	var out: Array = []
	_dump(w, 0, out)
	_p("================ %s ================" % label)
	if w is Window:
		_p("  window size=%s title='%s'" % [str((w as Window).size), (w as Window).title])
	for line in out:
		_p("  " + line)
	await _shoot(label)


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 900.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		_p("WATCHDOG")
		get_tree().quit(2))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	_app.open_project_dialog.hide()
	await _frames(2)

	_bridge.generate({
		"seed": 483920, "width_km": 1600.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(6)
	_p("world %s, %d settlements, %d factions" % [
		str(_bridge.grid_size()), _bridge.settlements().size(), _bridge.get_factions().size()])

	# ---- Performance -------------------------------------------------------
	_app.performance_window.open()
	await _report("performance", _app.performance_window)
	_app.performance_window.hide()

	# ---- World data --------------------------------------------------------
	_app.open_world_data()
	await _report("worlddata", _app.world_data_window)
	_app.world_data_window.hide()

	# ---- City viewer -------------------------------------------------------
	_app.open_city_viewer(0)
	await _report("cityviewer", _app.city_viewer_window)
	_app.city_viewer_window.hide()

	# ---- Place editor ------------------------------------------------------
	_app.open_place_editor(0)
	await _report("placeeditor", _app.place_editor_window)
	_app.place_editor_window.hide()

	# ---- Faction roster ----------------------------------------------------
	_app.open_faction_roster()
	await _report("factionroster", _app.faction_roster_window)
	_app.faction_roster_window.hide()

	# ---- Travel library ----------------------------------------------------
	_app.travel_library_window.open()
	await _report("travellib", _app.travel_library_window)
	_app.travel_library_window.hide()

	# ---- Vault overview ----------------------------------------------------
	_app.open_vault_overview()
	await _report("vault", _app.vault_window)
	_app.vault_window.hide()

	_p("DONE")
	get_tree().quit(0)
