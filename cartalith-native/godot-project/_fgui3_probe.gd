extends Node
## Owner findings of 2026-11-21, batch 3 (F2, F5, F8, F9, F10, F15, F30, F34): one windowed probe, one leg per finding.
##
##   godot --path . --resolution 1600x900 _fgui3_probe.tscn -- --leg f2 --out DIR
##   tablet: --resolution 1700x1000 ... -- --force-touch --density=1.0       phone: --resolution 450x975 ... --density 1.094
##
## **WINDOWED, never --headless**, scratch APPDATA. Mounted directly under the root window (a native pop-out needs the real
## root). Real mouse events are pushed through the root viewport (`_click`). Result lines `F3 ok|FAIL <check>`;
## `F3 RESULT n checks, f failed`. Exit 0 pass, 1 fail, 2 abort. Never reads or prints a key: the key leg only ever
## sets a throwaway environment variable named CART_PROBE_KEY with a made-up value and reads the app's masked status.

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
	print("F3 %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1


func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	get_viewport().get_texture().get_image().save_png(_out.path_join("f3_%s_%s.png" % [_form, name]))
	print("F3 SHOT f3_%s_%s.png" % [_form, name])


## A frame of one native window's OWN viewport (never the screen: a screen grab would capture whatever else is on the desktop).
func _win_shot(w: Window, name: String) -> void:
	if _out == "":
		return
	await _frames(4)
	await RenderingServer.frame_post_draw
	w.get_texture().get_image().save_png(_out.path_join("f3_%s_%s.png" % [_form, name]))
	print("F3 SHOT f3_%s_%s.png (window)" % [_form, name])


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
		print("F3 DBG click %s at %s rect %s scale %s winscale %s hovered %s win %s" % [c.name, pos, c.get_global_rect(), get_tree().root.content_scale_factor, win.content_scale_factor, hv.name if hv != null else "none", win.get_class()])
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


func _ready() -> void:
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("F3 WATCHDOG")
		get_tree().quit(2))
	if not ClassDB.class_exists("WorldGen"):
		print("F3 FATAL extension did not load")
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
	print("F3 form=%s" % _form)
	var leg := _arg("--leg", "all")
	await _generate()
	for l in ["f34", "f2", "f5", "f8", "f15", "f10", "f9", "f30"]:
		if leg == "all" or leg == l:
			print("F3 --- leg %s" % l)
			await call("_leg_" + l)
	print("F3 RESULT %d checks, %d failed (%s)" % [_checks, _fails, _form])
	get_tree().quit(1 if _fails > 0 else 0)


# -- F34 ------------------------------------------------------------------------------------------------------------
func _leg_f34() -> void:
	app.layers_popover.open()
	await _frames(8)
	var all := _texts(app.layers_popover)
	_check("the Layers popover opened and lists Tectonics rows", all.contains("Plate boundaries") and all.contains("Crust age"), all.left(120))
	_check("no Orogeny row is listed", not all.contains("Orogeny"))
	_check("Velocity erosion and the other genuine gaps still show their reason", all.contains("Site profile") or all.contains("Site-profile") or all.contains("Population density"))
	await _shot("layers")
	app.layers_popover.hide()
	await _frames(3)


# -- F2 -------------------------------------------------------------------------------------------------------------
## On the phone exactly one of the roster's two panes is on screen; a real tap on the folded bar opens the list.
func _open_roster_list(w: Node) -> void:
	if DccTheme.is_phone() and not w._phone_list_pane.is_visible_in_tree():
		await _click(w._phone_list_bar)
		await _frames(4)


func _leg_f2() -> void:
	app.open_faction_roster()
	await get_tree().create_timer(2.0).timeout
	var w: Node = app.faction_roster_window if "faction_roster_window" in app else null
	_check("the Factions window opened", w != null and w.visible)
	var n0 := int(bridge.civ_faction_count())
	_check("the world has several factions", n0 >= 3, str(n0))
	var guard := 0
	while int(bridge.civ_faction_count()) > 1 and guard < 12:
		guard += 1
		await _open_roster_list(w)
		var rm := _find(w, "Button", "− Remove last")
		_check("Remove last is enabled above the floor (%d factions)" % int(bridge.civ_faction_count()), rm != null and not (rm as Button).disabled, ", ".join(Array(w.find_children("*", "Button", true, false).filter(func(b): return b.text.to_lower().contains("remove") or b.text.to_lower().contains("add fa")).map(func(b): return "%s/%s/%s" % [b.text, b.is_visible_in_tree(), b.disabled]))))
		if rm == null:
			return
		await _click(rm)
		var ok_btn: Button = null
		for d in get_tree().root.find_children("*", "ConfirmationDialog", true, false):
			if (d as Window).visible:
				ok_btn = (d as ConfirmationDialog).get_ok_button()
		_check("the confirm card appeared", ok_btn != null)
		if ok_btn == null:
			return
		await _click(ok_btn)
		await _frames(4)
	_check("down to one faction (plus Unclaimed)", int(bridge.civ_faction_count()) == 1, str(bridge.civ_faction_count()))
	await _open_roster_list(w)
	var rm2 := _find(w, "Button", "− Remove last")
	_check("at the floor the button is disabled", rm2 != null and (rm2 as Button).disabled)
	_check("its tooltip says why", rm2 != null and (rm2 as Button).tooltip_text.contains("cannot be removed") and (rm2 as Button).tooltip_text.contains("Unclaimed"), rm2.tooltip_text if rm2 != null else "")
	_check("the engine refuses the last one too", not bridge.civ_remove_faction())
	await _shot("factions_floor")
	var add := _find(w, "Button", "+ Add faction")
	await _click(add)
	await _frames(4)
	_check("Add faction still works from the floor", int(bridge.civ_faction_count()) == 2)
	var rm3 := _find(w, "Button", "− Remove last")
	_check("and Remove last is enabled again", rm3 != null and not (rm3 as Button).disabled)
	w.hide()
	await _frames(3)


# -- F5 -------------------------------------------------------------------------------------------------------------
func _records() -> PopupMenu:
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton and (ch as MenuButton).text == "Records":
			return (ch as MenuButton).get_popup()
	return null


func _leg_f5() -> void:
	var rec := _records()
	if DccTheme.is_phone():
		_check("(phone: the menu is the MENU tab; checked by the story-menus probe)", true)
	else:
		_check("Records has the Name styles row on this form", rec != null and rec.get_item_index(DccMenus.ID_NAME_STYLES) >= 0)
		var idx := rec.get_item_index(DccMenus.ID_NAME_STYLES)
		_check("the row is enabled", idx >= 0 and not rec.is_item_disabled(idx))
		rec.id_pressed.emit(DccMenus.ID_NAME_STYLES)
	if DccTheme.is_phone():
		app.open_name_styles()
	await _frames(8)
	var w = app.name_styles_window
	_check("Name styles opens", w != null and w.visible)
	var rr := _find(w, "Button", "Rename all…")
	_check("Rename all is reachable in the window", rr != null and not rr.disabled)
	var undo_rr := _find(w, "Button", "Undo rename all")
	_check("and so is its Undo", undo_rr != null)
	await _shot("name_styles")
	var sets: Array = bridge.settlements()
	var before := String((sets[0] as Dictionary)["name"])
	var nn := String(bridge.civ_reroll_settlement_name(0))
	_check("re-roll gives a name from the engine", nn != "")
	app.open_place_editor(0)
	await _frames(8)
	var pe = app.place_editor_window if "place_editor_window" in app else null
	var dice: Control = null
	if pe != null:
		for b in pe.find_children("*", "Button", true, false):
			if (b as Button).tooltip_text.begins_with("Re-roll a name"):
				dice = b
	_check("the Place editor shows the name re-roll button", dice != null and dice.is_visible_in_tree())
	if dice != null:
		var name_before := String((bridge.settlements()[0] as Dictionary)["name"])
		var field: LineEdit = null
		for le in pe.find_children("*", "LineEdit", true, false):
			if (le as LineEdit).text == name_before and field == null:
				field = le
		_check("the name field holds the settlement's name", field != null)
		await _click(dice)
		await _frames(4)
		var engine_name := String((bridge.settlements()[0] as Dictionary)["name"])
		_check("a real click on the dice re-rolls the settlement's name in the engine", engine_name != name_before and engine_name != "", "%s -> %s" % [name_before, engine_name])
		await _shot("place_editor_reroll")
	if pe != null:
		pe.hide()
	await _frames(3)
	var s0: Dictionary = bridge.settlements()[0]
	var n_before: int = bridge.settlements().size()
	var id := -1
	for gx in range(10, 250, 5):
		for gy in range(10, 185, 5):
			if bridge.settlements().size() == n_before:
				id = bridge.civ_drop_settlement(float(gx), float(gy), "village", 1, "", false)
	var nm: String = String((bridge.settlements()[id] as Dictionary)["name"]) if id >= 0 else ""
	_check("a new settlement dropped with no name receives a generated one", id >= 0 and nm != "" and bridge.settlements().size() == n_before + 1, "%d '%s'" % [id, nm])
	# Rename all through its real buttons: Rename all… then the confirm card
	var before_names: Array = []
	for st in bridge.settlements():
		before_names.append(String(st["name"]))
	bridge.civ_edit_settlement(0, {"name": "Handy Rest"})
	await _click(rr)
	await _frames(4)
	var okc: Button = null
	for d in get_tree().root.find_children("*", "ConfirmationDialog", true, false):
		if (d as Window).visible:
			okc = (d as ConfirmationDialog).get_ok_button()
	_check("Rename all asks first", okc != null)
	if okc != null:
		await _click(okc)
		await _frames(8)
	var changed := 0
	var after: Array = bridge.settlements()
	for i in mini(before_names.size(), after.size()):
		if i > 0 and String(after[i]["name"]) != before_names[i]:
			changed += 1
	_check("Rename all renamed the generated places (%d changed)" % changed, changed > 10 and String(after[0]["name"]) == "Handy Rest")
	await _shot("rename_all_done")
	w.hide()
	await _frames(3)


# -- F8 -------------------------------------------------------------------------------------------------------------
func _two_routes() -> void:
	var towns: Array = bridge.settlements()
	var picks: Array = []
	for t in towns:
		if picks.size() < 5:
			picks.append(t)
	for k in 2:
		bridge.route_begin("mixed")
		bridge.route_append_stop(float(picks[k]["x"]), float(picks[k]["y"]))
		bridge.route_append_stop(float(picks[k + 2]["x"]), float(picks[k + 2]["y"]))
		bridge.route_commit()


func _leg_f8() -> void:
	_two_routes()
	_check("two committed routes exist", int(bridge.route_count()) >= 2, str(bridge.route_count()))
	app.open_journey_planner()
	await _frames(12)
	var pv = app.journey_planner_view
	var sec: Node = pv._left_route_section
	pv._refresh_route_choice()
	await _frames(4)
	_check("the planner lists every plotted route as a row", sec.find_children("RouteRow*", "HBoxContainer", true, false).size() == int(bridge.route_count()))
	var n0 := int(bridge.route_count())
	# a saved journey on route 1, to see its index follow a delete of route 0
	pv._route_index = 1
	pv._commit_saved_journey("Probe journey")
	await _frames(4)
	_check("a saved journey is listed with Rename and Forget", sec.find_children("JourneyRename", "Button", true, false).size() == 1 and sec.find_children("JourneyDelete", "Button", true, false).size() == 1)
	await _shot("planner_list")
	# rename route 0 through the real pencil + prompt
	var rn := sec.find_children("RouteRename", "Button", true, false)[0] as Button
	await _click(rn)
	await _frames(4)
	var dlg := _dialog()
	_check("the rename prompt opens with a name field", dlg != null and not dlg.find_children("*", "LineEdit", true, false).is_empty())
	if dlg != null:
		(dlg.find_children("*", "LineEdit", true, false)[0] as LineEdit).text = "Salt road"
		await _click(dlg.get_ok_button())
		await _frames(4)
	_check("the engine route now carries the typed name", String((bridge.route_get(0) as Dictionary).get("name", "")) == "Salt road", String((bridge.route_get(0) as Dictionary).get("name", "")))
	_check("the list shows it", _texts(sec).contains("Salt road"))
	# rename the saved journey
	var jr := sec.find_children("JourneyRename", "Button", true, false)[0] as Button
	await _click(jr)
	await _frames(4)
	dlg = _dialog()
	if dlg != null:
		(dlg.find_children("*", "LineEdit", true, false)[0] as LineEdit).text = "Grain run"
		await _click(dlg.get_ok_button())
		await _frames(4)
	var jl: Array = bridge.journey_list()
	_check("the engine's own journey is renamed (Story > Saved journeys reads it)", jl.size() == 1 and String(jl[0]["name"]) == "Grain run", str(jl))
	# delete route 0 through the real x + confirm
	var dl := sec.find_children("RouteDelete", "Button", true, false)[0] as Button
	await _click(dl)
	await _frames(4)
	var okc: Button = null
	for d in get_tree().root.find_children("*", "ConfirmationDialog", true, false):
		if (d as Window).visible:
			okc = (d as ConfirmationDialog).get_ok_button()
	_check("delete asks first", okc != null)
	if okc != null:
		await _click(okc)
		await _frames(6)
	_check("the route is gone from the engine", int(bridge.route_count()) == n0 - 1, str(bridge.route_count()))
	_check("the saved journey's route index followed the renumbering", int((pv._journeys[0] as Dictionary)["route"]) == 0, str(pv._journeys[0]["route"]))
	_check("the saved journey survives", bridge.journey_list().size() == 1)
	await _shot("planner_after_delete")
	# forget the journey, then the last route: list stays usable
	await _click(sec.find_children("JourneyDelete", "Button", true, false)[0] as Button)
	await _frames(4)
	_check("Forget removes the journey from the engine", bridge.journey_list().size() == 0)
	if app.journey_planner_view.has_method("_hide"):
		app.journey_planner_view._hide()
	await _frames(4)


# -- F15 ------------------------------------------------------------------------------------------------------------
func _leg_f15() -> void:
	var dir := ProjectSettings.globalize_path("user://f3vault")
	DirAccess.make_dir_recursive_absolute(dir.path_join("Notes"))
	var f := FileAccess.open(dir.path_join("Notes/Kelvhold.md"), FileAccess.WRITE)
	f.store_string("# Kelvhold
Intro.
## The Old Quarter
Walled in 1204.
")
	f.close()
	var vr: Dictionary = bridge.vault_connect(dir, "probe")
	_check("a scratch vault connected", bool(vr.get("ok", false)), str(vr))
	app.open_assistant_panel()
	await _frames(8)
	var aw = app.assistant_window
	await _shot("assistant_open")
	var id := "V:Notes/Kelvhold.md#Kelvhold > The Old Quarter"
	aw._reply = "Kelvhold was walled in 1204 [%s]. Nothing else [V:Gone/Away.md]." % id
	aw._reply_label = aw._add_msg("Assistant", aw._reply, "text_secondary", true)
	aw._on_sources({"sources": [], "citations": [{"id": id, "verified": true, "label": "V", "title": "Kelvhold"},
		{"id": "V:Gone/Away.md", "verified": false}]})
	await _frames(6)
	var want := "[[Notes/Kelvhold#The Old Quarter]]"
	_check("the reply text now carries the Obsidian link", aw._reply_label.text.contains(want) and not aw._reply_label.text.contains("[" + id + "]"), aw._reply_label.text)
	_check("the unverified citation is left as the model wrote it", aw._reply_label.text.contains("[V:Gone/Away.md]"))
	_check("the raw reply (what Rust verified) is untouched", aw._reply.contains("[" + id + "]"))
	var box: Node = aw.source_cards[-1]["box"]
	var link := _find(box, "Button", want)
	_check("a followable link button shows the same form", link != null, _texts(box))
	_check("the card states the link form", _texts(box).contains("[[Folder/Note#Heading]]") and _texts(box).contains("never a drive"))
	_check("the link form helper", LlmAssistantWindow.vault_link_form("V:A/B.md") == "[[A/B]]" and LlmAssistantWindow.vault_link_form("V:A/B.md#H1 > H2") == "[[A/B#H2]]")
	await _shot("assistant_links")
	if link != null:
		await _reveal(link)
		await _shot("assistant_link_in_view")
		await _click(link)
		await _frames(10)
		var vw = app.vault_window
		var shown: bool = vw.visible and String(vw._pick_file if "_pick_file" in vw else vw._browse_path).ends_with("Notes/Kelvhold.md")
		_check("pressing it opens that note in the Vault (window %s, file %s)" % [vw.visible, str(vw._pick_file if "_pick_file" in vw else "?")], shown)
		await _shot("assistant_link_followed")
	bridge.vault_disconnect()


# -- F10 / F9 ------------------------------------------------------------------------------------------------------
## The fake provider is a throwaway Python server started by the runner on 127.0.0.1:18765 (`--fake-port`): a local
## OpenAI-style `/v1/models`. No hosted address is ever dialled and no real key is read or made.
func _open_settings() -> Node:
	app.open_assistant_settings()
	await _frames(10)
	return app.assistant_settings_window


func _leg_f10() -> void:
	DccSettings.set_llm_values({"enabled": true, "local_allowed": true, "remote_allowed": true, "preset_id": "openrouter", "base_url": "", "key_source": "env", "env_var": "CART_PROBE_KEY"})
	var w = await _open_settings()
	var src: OptionButton = w.controls["key_source"]
	_check("Key source offers the two sources the scope defines (env var, OS store)", src.item_count == 2 and src.get_item_text(0).contains("Environment") and src.get_item_text(1).contains("credential"), "%s | %s" % [src.get_item_text(0), src.get_item_text(1)])
	_check("the OS store is available on this PC", not src.is_item_disabled(1))
	var st0: Dictionary = bridge.llm_key_status("env", "CART_PROBE_KEY", "probe")
	_check("env source with the variable unset: not present, and the words name the fix", not bool(st0.get("present", true)) and String(st0.get("problem", "")).contains("CART_PROBE_KEY"), str(st0.get("problem", "")))
	OS.set_environment("CART_PROBE_KEY", "probe-not-a-key-0123456789")
	var st1: Dictionary = bridge.llm_key_status("env", "CART_PROBE_KEY", "probe")
	_check("env source with the variable set: present, masked, never the value", bool(st1.get("present", false)) and not str(st1).contains("probe-not-a-key"), str(st1.get("masked", "")))
	w._refresh_key_line()
	await _frames(3)
	_check("the window's key line reads present", _texts(w).contains("Key present"))
	w._scroll.ensure_control_visible(src)
	w._scroll.scroll_vertical += 200
	await _frames(4)
	await _shot("llm_env")
	# pick the OS store then the env var again through the picker's own signal (a popup list is a native-less widget
	# here: its item_selected is the signal a click on a row emits)
	src.select(1)
	src.item_selected.emit(1)
	await _frames(6)
	_check("picking the OS store sticks", String(DccSettings.llm_value("key_source")) == "store")
	_check("and shows the Key field with Save key / Forget key", w.controls.has("key_field") and w.controls.has("key_save"))
	w._scroll.ensure_control_visible(w.controls["key_source"])
	w._scroll.scroll_vertical += 200
	await _frames(4)
	await _shot("llm_store")
	src = w.controls["key_source"]
	src.select(0)
	src.item_selected.emit(0)
	await _frames(6)
	_check("picking the env var again sticks", String(DccSettings.llm_value("key_source")) == "env" and w.controls.has("env_var"))
	OS.unset_environment("CART_PROBE_KEY")
	w._refresh_key_line()
	_check("with the variable unset the line offers the run-only key", _texts(w).contains("paste a key for this run"))
	# the run-only key through the real controls: type into the masked field, press the button
	var sk: LineEdit = w.controls["session_key_field"]
	_check("the field is masked", sk.secret)
	await _click(sk)
	sk.text = "probe-not-a-key-ABCDEF123456"
	await _click(w.controls["session_key_use"])
	await _frames(6)
	_check("the field was cleared at once", sk.text == "")
	var st2: Dictionary = bridge.llm_key_status("env", "CART_PROBE_KEY", "probe")
	_check("the engine now sees a key under the variable, masked", bool(st2.get("present", false)) and String(st2.get("masked", "")) == "****3456" and not str(st2).contains("ABCDEF"), str(st2.get("masked", "")))
	_check("the window says so, without the key", _texts(w).contains("Key present") and not _texts(w).contains("ABCDEF") and not _texts(w).contains("probe-not"))
	w._scroll.ensure_control_visible(w.controls["session_key_use"])
	await _frames(4)
	await _shot("llm_session_key")
	await _click(w.controls["session_key_clear"])
	await _frames(4)
	_check("Forget removes it from the process", not OS.has_environment("CART_PROBE_KEY") and not bool((bridge.llm_key_status("env", "CART_PROBE_KEY", "probe")).get("present", true)))
	var leaked := false
	for f in DirAccess.get_files_at("user://"):
		if FileAccess.get_file_as_string("user://" + f).contains("ABCDEF123456"):
			leaked = true
	_check("no file in the user directory holds the key", not leaked)
	var cfg := FileAccess.get_file_as_string(DccSettings.CONFIG_PATH)
	_check("(settings file checked: %d bytes)" % cfg.length(), not cfg.contains("ABCDEF123456"))
	w.hide()
	await _frames(3)


func _leg_f9() -> void:
	var port := int(_arg("--fake-port", "18765"))
	var base := "http://127.0.0.1:%d/v1" % port
	DccSettings.set_llm_values({"enabled": true, "local_allowed": true, "remote_allowed": false, "preset_id": "custom_openai", "base_url": base, "model": "", "key_source": "env", "env_var": "CART_PROBE_KEY"})
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % port)
	LlmModelList.clear_cache()
	var w = await _open_settings()
	var rf := w.controls["model_refresh"] as Button
	await _click(rf)
	var t0 := Time.get_ticks_msec()
	while w._ml_state == "loading" and Time.get_ticks_msec() - t0 < 15000:
		await _frames(5)
	await _frames(5)
	_check("the model list loaded from the fake server (state=%s msg=%s)" % [w._ml_state, w._ml_msg], w._ml_state == "ok")
	var pick: OptionButton = w.controls["model_pick"]
	var full := pick.item_count
	_check("the dropdown holds the 12 models the server sent plus Choose and Type a name", full == 14, str(full))
	var row := w.controls["model_search_row"] as Control
	_check("a long list shows the search box", row.visible)
	var box := w.controls["model_search"] as LineEdit
	await _click(box)
	box.text = "mist"
	box.text_changed.emit("mist")
	await _frames(4)
	var names: PackedStringArray = []
	for i in pick.item_count:
		names.append(pick.get_item_text(i))
	var hits := 0
	for n in names:
		if n.contains("mistral"):
			hits += 1
	_check("typing narrows the dropdown to the two matches", hits == 2 and not " ".join(names).contains("llama") and pick.item_count == 4, str(names))
	w._scroll.ensure_control_visible(box)
	w._scroll.scroll_vertical += 120
	await _frames(4)
	await _shot("llm_models_search")
	box.text = "zzz-none"
	box.text_changed.emit("zzz-none")
	await _frames(4)
	_check("no match leaves only Choose and Type a name", pick.item_count == 2, str(pick.item_count))
	box.text = ""
	box.text_changed.emit("")
	await _frames(4)
	_check("clearing restores the whole list", pick.item_count == full)
	pick.select(3)
	pick.item_selected.emit(3)
	await _frames(4)
	var chosen := String(DccSettings.llm_value("model"))
	_check("choosing a row stores that model", chosen != "", chosen)
	box.text = "zzz-none"
	box.text_changed.emit("zzz-none")
	await _frames(4)
	_check("a search that hides the chosen model keeps it as row 0, never silently dropped", pick.get_item_text(0).contains(chosen) and pick.get_item_text(0).contains("hidden by search") and String(DccSettings.llm_value("model")) == chosen, pick.get_item_text(0))
	await _shot("llm_models")
	w.hide()
	await _frames(3)


# -- F30 -----------------------------------------------------------------------------------------------------------
func _leg_f30() -> void:
	var T := PopoutHost
	var u := Rect2i(1920, 0, 2560, 1400)
	_check("full_rect fills the width and sits under the title strip", T.full_rect(u) == Rect2i(1920, PopoutGeometry.TITLE_ROW, 2560, 1400 - PopoutGeometry.TITLE_ROW), str(T.full_rect(u)))
	_check("needs_fit: wider than the screen", T.needs_fit(Rect2i(0, 40, 2561, 600), u))
	_check("needs_fit: taller than the area under the title strip", T.needs_fit(Rect2i(0, 40, 800, 1400 - PopoutGeometry.TITLE_ROW + 1), u))
	_check("needs_fit: a window that fits is left alone", not T.needs_fit(Rect2i(0, 40, 2560, 1400 - PopoutGeometry.TITLE_ROW), u))
	if not PopoutHost.supported():
		_check("(pop-outs are not supported on this form: %s)" % PopoutHost.unsupported_reason(DisplayServer.has_feature(DisplayServer.FEATURE_SUBWINDOWS), false, DccTheme.is_phone(), DccTheme.is_tablet()), true)
		return
	app.open_faction_roster()
	await _frames(8)
	var w: Window = app.faction_roster_window
	var fb0 := w.find_child(PopoutHost.FIT_BUTTON_NAME, true, false) as Button
	_check("docked: the Fit screen button exists and is hidden", fb0 != null and not fb0.visible)
	var pop := w.find_child(PopoutHost.BUTTON_NAME, true, false) as Button
	await _click(pop)
	await _frames(10)
	_check("the window is native now", w.visible and not w.is_embedded())
	var fb := w.find_child(PopoutHost.FIT_BUTTON_NAME, true, false) as Button
	_check("native: Fit screen is shown", fb != null and fb.visible)
	var usable := DisplayServer.screen_get_usable_rect(w.current_screen)
	var before := Rect2i(w.position, w.size)
	await _win_shot(w, "popout_native")
	await _click(fb)
	await _frames(8)
	var want := PopoutHost.full_rect(usable)
	_check("a real click fills the screen it is on (%s)" % str(want), w.size.x == want.size.x and w.size.y == want.size.y and w.position == want.position, "size=%s pos=%s" % [w.size, w.position])
	_check("the button now offers Restore size", fb.text == "Restore size")
	await _win_shot(w, "popout_fitted")
	await _click(fb)
	await _frames(8)
	_check("pressed again it gives the old rectangle back", Rect2i(w.position, w.size) == before, "%s vs %s" % [Rect2i(w.position, w.size), before])
	# dragged onto a smaller screen: simulate by making it larger than the screen and giving it focus
	w.size = Vector2i(usable.size.x + 300, usable.size.y)
	await _frames(4)
	PopoutHost._on_focus(w)
	await _frames(6)
	_check("a window larger than its screen is brought inside it on focus", not PopoutHost.needs_fit(Rect2i(w.position, w.size), usable), "size=%s usable=%s" % [w.size, usable])
	await _click(pop)
	await _frames(8)
	w.hide()
	await _frames(3)


