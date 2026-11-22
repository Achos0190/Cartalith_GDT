extends Node
## Ruling GI (owner 2026-11-21): every faction may be deleted; Ruling GK (2026-11-22): unclaimed settlements of a
## faction world keep their own garrison, and stale chronos faction filters are pruned. One windowed run through the real shell:
## delete all six factions with the Factions window's own "− Remove last" button and its confirm card,
## read the military model (no faction army; every settlement keeps its own garrison), run the
## collapse timeline, save the project, reopen it, and add a faction again.
##
##   godot --path . --resolution 1600x900 _gi_probe.tscn -- --out DIR
##
## **WINDOWED, never --headless**, scratch APPDATA. Result lines `GI ok|FAIL <check>`; `GI RESULT n checks,
## f failed`. Exit 0 pass, 1 fail, 2 abort. Desktop only: the roster's buttons are the same controls on
## every form (the phone only folds the list pane, which `_fgui3_probe.gd`'s f2 leg already walks).

var app: Node
var bridge
var _checks := 0
var _fails := 0
var _out := ""
var _form := "desktop"


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	return String(args[i + 1]) if i >= 0 and i + 1 < args.size() else dflt


func _check(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	print("GI %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1


func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	get_viewport().get_texture().get_image().save_png(_out.path_join("gi_%s_%s.png" % [_form, name]))
	print("GI SHOT f3_%s_%s.png" % [_form, name])


## A frame of one native window's OWN viewport (never the screen: a screen grab would capture whatever else is on the desktop).
func _win_shot(w: Window, name: String) -> void:
	if _out == "":
		return
	await _frames(4)
	await RenderingServer.frame_post_draw
	w.get_texture().get_image().save_png(_out.path_join("gi_%s_%s.png" % [_form, name]))
	print("GI SHOT f3_%s_%s.png (window)" % [_form, name])


## Scrolls every ScrollContainer above `c` so that `c` is on screen (a card can sit inside two nested scrolls).
func _reveal(c: Control) -> void:
	var n: Node = c.get_parent()
	while n != null:
		if n is ScrollContainer:
			(n as ScrollContainer).ensure_control_visible(c)
		n = n.get_parent()
	await _frames(4)


## A real left click on control `c`: motion, press, release, delivered as the engine delivers a mouse press.
func _click(c: Control) -> void:
	await _reveal(c)
	var win := c.get_window()
	var pos := c.get_global_rect().get_center()
	if win != get_tree().root and not win.is_embedded():
		## A native window has its own viewport: the event goes in there, in its own coordinates.
		for t in ["motion", "down", "up"]:
			var ev: InputEvent
			if t == "motion":
				ev = InputEventMouseMotion.new()
			else:
				ev = InputEventMouseButton.new()
				ev.button_index = MOUSE_BUTTON_LEFT
				ev.pressed = t == "down"
			ev.position = pos
			ev.global_position = pos
			win.push_input(ev, true)
			await _frames(2)
		await _frames(3)
		return
	if win != get_tree().root:
		## An embedded window has its own content scale (the phone's is not 1): window pixels, then its place in the root.
		pos = win.get_final_transform() * pos + Vector2(win.position)
	var mv := InputEventMouseMotion.new()
	mv.position = pos
	mv.global_position = pos
	get_viewport().push_input(mv, true)
	await _frames(2)
	if OS.get_cmdline_user_args().has("--debug-click"):
		var hv := get_viewport().gui_get_hovered_control()
		print("GI DBG click %s at %s rect %s scale %s winscale %s hovered %s win %s" % [c.name, pos, c.get_global_rect(), get_tree().root.content_scale_factor, win.content_scale_factor, hv.name if hv != null else "none", win.get_class()])
	var dn := InputEventMouseButton.new()
	dn.button_index = MOUSE_BUTTON_LEFT
	dn.pressed = true
	dn.position = pos
	dn.global_position = pos
	get_viewport().push_input(dn, true)
	await _frames(2)
	var up := dn.duplicate() as InputEventMouseButton
	up.pressed = false
	get_viewport().push_input(up, true)
	await _frames(3)


## The first node under `root` whose class is `cls` and whose text (Button/Label) or name equals/contains `what`.
func _find(root: Node, cls: String, what: String, exact: bool = true) -> Control:
	for n in root.find_children("*", cls, true, false):
		var t := ""
		if n is Button:
			t = (n as Button).text
		elif n is Label:
			t = (n as Label).text
		## The phone's buttons are drawn upper case: compare without case.
		var tl := t.to_lower()
		var wl := what.to_lower()
		if (tl == wl if exact else tl.contains(wl)) or String(n.name) == what:
			if (n as Control).is_visible_in_tree():
				return n
	return null


func _texts(n: Node) -> String:
	var out: PackedStringArray = []
	for c in n.find_children("*", "Control", true, false):
		if c is Label:
			out.append((c as Label).text)
		elif c is Button:
			out.append((c as Button).text)
	return " | ".join(out)


## The visible ConfirmationDialog (a prompt or a confirm card), or null.
func _dialog() -> ConfirmationDialog:
	for d in get_tree().root.find_children("*", "ConfirmationDialog", true, false):
		if (d as Window).visible:
			return d
	return null


func _generate(seed_v: int = 4242) -> void:
	bridge.generate({"seed": seed_v, "width_km": 2400.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await _frames(2)
	while bool(bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _frames(10)
	await _frames(30)


## Every settlement row of `civ_military_summary_at(year)` carries its own garrison, no faction row
## exists, and `civ_settlement_garrison` reads `loose` for the first settlement. `what` names the reading.
func _check_loose(year: int, what: String) -> void:
	var m: Dictionary = bridge.civ_military_summary_at(year)
	var facs: Array = m.get("factions", [])
	var places: Array = m.get("settlements", [])
	_check("%s: no faction army" % what, facs.is_empty(), str(facs.size()))
	var with_g := places.filter(func(p): return (p as Dictionary).has("garrison"))
	_check("%s: every settlement keeps its own garrison" % what, not places.is_empty() and with_g.size() == places.size(), "%d of %d" % [with_g.size(), places.size()])
	var men := 0
	for p in with_g:
		men += int((p as Dictionary)["garrison"])
	_check("%s: the garrisons hold men" % what, men > 0, str(men))
	if not places.is_empty():
		var g: Dictionary = bridge.civ_settlement_garrison(int((places[0] as Dictionary).get("tid", 0)), year)
		_check("%s: the settlement's own garrison reads loose" % what, bool(g.get("loose", false)) and g.has("garrison") and not g.has("absent"), str(g))


func _remove_all(w: Node) -> void:
	var guard := 0
	while int(bridge.civ_faction_count()) > 0 and guard < 12:
		guard += 1
		var n := int(bridge.civ_faction_count())
		var rm := _find(w, "Button", "− Remove last")
		_check("Remove last is enabled with %d factions" % n, rm != null and not (rm as Button).disabled)
		if rm == null:
			return
		await _click(rm)
		var d := _dialog()
		_check("the confirm card appeared (%d)" % n, d != null)
		if d == null:
			return
		if n == 1:
			_check("the last one's prompt says what the world becomes", d.dialog_text.contains("the last one") and d.dialog_text.contains("own garrison"), d.dialog_text)
			if d.is_embedded():
				await _shot("confirm_last")
			else:
				await _win_shot(d, "confirm_last")
		await _click(d.get_ok_button())
		await _frames(4)
		_check("one fewer (%d)" % n, int(bridge.civ_faction_count()) == n - 1, str(bridge.civ_faction_count()))


func _ready() -> void:
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("GI WATCHDOG")
		get_tree().quit(2))
	if not ClassDB.class_exists("WorldGen"):
		print("GI FATAL extension did not load")
		get_tree().quit(2)
		return
	_out = _arg("--out", "")
	if _out != "":
		DirAccess.make_dir_recursive_absolute(_out)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(6)
	bridge = app.bridge
	var vs := get_tree().root.size
	_form = ("phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")) + "_%dx%d" % [vs.x, vs.y]
	print("GI form=%s" % _form)
	await _generate()
	var n_settle := (bridge.settlements() as Array).size()
	_check("the world has six factions and settlements", int(bridge.civ_faction_count()) == 6 and n_settle >= 4, "%d / %d" % [bridge.civ_faction_count(), n_settle])

	## Ruling GK item 1: with factions, an unclaimed settlement keeps its own garrison (the app has the switch on).
	var m0: Dictionary = bridge.civ_military_summary_at(int(bridge.get_civ_year()))
	var unclaimed: Array = (m0.get("settlements", []) as Array).filter(func(p): return int((p as Dictionary).get("faction", -1)) == 0)
	if unclaimed.is_empty():
		## Generated worlds often claim every settlement: removing the last faction releases its places
		## (the button path is walked below for the other five).
		_check("removing one faction leaves unclaimed settlements beside five factions", bridge.civ_remove_faction() and int(bridge.civ_faction_count()) == 5)
		m0 = bridge.civ_military_summary_at(int(bridge.get_civ_year()))
		unclaimed = (m0.get("settlements", []) as Array).filter(func(p): return int((p as Dictionary).get("faction", -1)) == 0)
	if unclaimed.is_empty():
		_check("an unclaimed settlement exists for Ruling GK", false)
	else:
		var g0: Dictionary = bridge.civ_settlement_garrison(int((unclaimed[0] as Dictionary).get("tid", 0)), int(bridge.get_civ_year()))
		_check("GK: an unclaimed settlement of a faction world keeps its own garrison", bool(g0.get("loose", false)) and g0.has("garrison"), str(g0))
	## Ruling GK item 3: a saved chronos faction filter naming a faction that does not exist is pruned.
	## The strip lives in `timeline_row`, filled when CIVIL's timeline is expanded (`_ct_probe.gd`'s own route).
	app.select_domain("civilization")
	await _frames(8)
	app._tl_expanded = true
	app._fill_timeline_strip()
	await _frames(30)
	var chron: Node = app.timeline_row.find_child("ChronosItems", true, false) if app.timeline_row != null else null
	_check("the chronos strip exists", chron != null)
	if chron != null:
		chron._factions = [3, 99]
		chron._save_prefs()
		var fd: Dictionary = chron.filter_dict()
		_check("GK: the chronos filter drops an id the roster lacks and keeps a real one", Array(fd.get("factions", PackedInt32Array())) == [3], str(fd))

	app.open_faction_roster()
	await get_tree().create_timer(2.0).timeout
	var w: Node = app.faction_roster_window
	_check("the Factions window opened", w != null and w.visible)
	await _remove_all(w)
	if chron != null:
		var fd2: Dictionary = chron.filter_dict()
		var cfg := ConfigFile.new()
		cfg.load(DccSettings.CONFIG_PATH)
		_check("GK: with every faction deleted the chronos filter names none, saved too", not fd2.has("factions") and String(cfg.get_value("timeline_items", "factions", "x")) == "", "%s / %s" % [fd2, cfg.get_value("timeline_items", "factions", "x")])
	_check("the rest deleted through the button", int(bridge.civ_faction_count()) == 0)
	var rm := _find(w, "Button", "− Remove last") as Button
	_check("with none left Remove last is disabled, and says why", rm != null and rm.disabled and rm.tooltip_text.contains("No faction is left"), rm.tooltip_text if rm != null else "")
	var txt := _texts(w)
	_check("the list says there are no factions, not that there is no world", txt.contains("No factions: every settlement is loose") and not txt.contains("No world generated"), txt.left(200))
	await _shot("roster_none")
	_check("every settlement is Unclaimed", (bridge.settlements() as Array).all(func(s): return int((s as Dictionary).get("faction", -1)) == 0))
	_check_loose(int(bridge.get_civ_year()), "live")

	## The timeline: a 100-year collapse run, then the military reading at its last year.
	var r: Dictionary = bridge.civ_run_collapse_simulation({"mode": "collapse", "start_year": 0, "duration": 100, "step_years": 10, "confirm_overwrite": true})
	_check("the collapse run succeeds on loose settlements", bool(r.get("ok", false)), str(r))
	var years: PackedInt64Array = bridge.get_civ_timeline_years()
	_check("it recorded its years", years.size() >= 10, str(years))
	var end_year := int(years[years.size() - 1]) if years.size() > 0 else 0
	bridge.civ_goto_year(end_year)
	await _frames(6)
	_check_loose(end_year, "year %d" % end_year)

	## Save, reopen, re-add.
	var path := OS.get_environment("APPDATA").path_join("gi_probe_world.cartalith")
	_check("the project saves", bridge.save_project(path), path)
	_check("the project reopens", bridge.load_save(path))
	await _frames(30)
	_check("the reopened world still has no faction", int(bridge.civ_faction_count()) == 0, str(bridge.civ_faction_count()))
	_check("its settlements came back", (bridge.settlements() as Array).size() > 0)
	_check("its timeline came back", (bridge.get_civ_timeline_years() as PackedInt64Array).size() == years.size(), str(bridge.get_civ_timeline_years()))
	_check_loose(int(bridge.get_civ_year()), "reopened")
	app.open_faction_roster()
	await get_tree().create_timer(1.5).timeout
	var add := _find(w, "Button", "+ Add faction")
	if add != null:
		await _click(add)
		await _frames(4)
	_check("Add faction works after the reopen", int(bridge.civ_faction_count()) == 1, str(bridge.civ_faction_count()))
	var rm3 := _find(w, "Button", "− Remove last") as Button
	_check("and Remove last is enabled again", rm3 != null and not rm3.disabled)
	await _shot("roster_readded")
	print("GI RESULT %d checks, %d failed (%s)" % [_checks, _fails, _form])
	get_tree().quit(1 if _fails > 0 else 0)
