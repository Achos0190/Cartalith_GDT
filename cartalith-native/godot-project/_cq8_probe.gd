extends Node
## CQ-8 probe: the quick bar's three extensions (Ruling EH item 3, `CUSTOM_QUICK_BAR_SCOPE.md` Q1, Q8, Q10).
##
##   desktop: godot --path . --resolution 1600x1000 _cq8_probe.tscn
##   touch:   godot --path . --resolution 1600x1000 _cq8_probe.tscn -- --force-touch
##   phone:   godot --path . --resolution 1080x2340 _cq8_probe.tscn -- --force-touch --density 2.8125
##
## **Windowed, never `--headless`; a WIPED user dir; under the lock** (`MISTAKES.md`: a probe that reads `user://`
## can be green for reasons that have nothing to do with the code). The arguments read are `--force-touch`,
## `--density D` (the shell reads both) and `--shots DIR`; anything else aborts with exit 2.
##
## What it protects. Every expectation is a LITERAL (a title, a mark, a domain list, a sentence fragment), never
## read back from the code under test:
##   A. THE STORE (no shell; the config file is written by hand and reloaded, as a relaunch does):
##      a pin stored before CQ-8 (no `d`) is shown in all three domains and its file text gains no `d`; a pin
##      added with domains stores them sorted; all three domains stores NO `d`; an empty or unknown selection is
##      REFUSED and writes not one byte; `quick_bar_move_visible` hops over hidden pins; a corrupt `d` (a string,
##      `[]`, an unknown id, a non-string id) keeps the pin, unrestricted, and `quick_bar_notices()` NAMES it, and
##      counts an unreadable entry; a clean file has no notice; a newer file's notice says so.
##   B. PER-DOMAIN (desktop and tablet, the real domain rail clicked): a pin limited to World and Cartography
##      has a chip there and none in Civilization; the notices are spoken once and the Customise window carries
##      them; a ring dock pin limited to Cartography is on the ring only there.
##   C. TOGGLE PINS: a chip's mark follows the command's own state when it changes from the CHIP, from the
##      palette cell, from the File menu (real keys), from the Theme popup's own `id_pressed`, and from a real
##      Shift+D accelerator; a toggle row nobody registered a state source for stays unpinnable with its reason.
##   D. TOOL PINS: the three global tools are catalogued once each; pressing a Measure chip fires `tool_armed`
##      exactly as the palette's cell does and lights the chip; arming another tool from the palette unlights it.
##   E. THE CUSTOMISE WINDOW (real clicks): the pin list says where a restricted pin shows; ticking the missing
##      domain stores no `d`; unticking a pin's last domain is refused with its reason and changes nothing.
##   F. PHONE (touch at phone density): the phone row's entries follow the active domain and carry the toggle mark;
##      the window's `Shown in` boxes clear 44 dp.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument or no extension.

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _app: Node
var _armed_events: Array = []


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(n: int = 4) -> void:
	for _i in n:
		await get_tree().process_frame


func _wait(sec: float) -> void:
	await get_tree().create_timer(sec).timeout


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


# -- file helpers (part A) ------------------------------------------------------------------------

func _cfg_path() -> String:
	return ProjectSettings.globalize_path(DccSettings.CONFIG_PATH)


## Forgets the in-memory ConfigFile so the next accessor re-reads the file, exactly what a relaunch does.
func _reload() -> void:
	DccSettings._loaded = false
	DccSettings._cfg = null


func _text(path: String) -> String:
	var f := FileAccess.open(path, FileAccess.READ)
	return f.get_as_text().replace("\r\n", "\n") if f != null else ""


func _write_text(path: String, text: String) -> void:
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _keys(pins: Array) -> Array:
	var out: Array = []
	for e in pins:
		out.append(String(e["k"]))
	return out


# -- input helpers --------------------------------------------------------------------------------

func _move(at: Vector2) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 2)
	Input.parse_input_event(mm)
	await _settle(2)


## A REAL motion, press and release at `at` through the input system (touch builds emulate the mouse).
func _click_at(at: Vector2) -> void:
	await _move(at)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(3)


func _key(code: Key, shift := false) -> void:
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = code
		ev.physical_keycode = code
		ev.pressed = down
		ev.shift_pressed = shift
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(2)


## The centre of `ctl` in MAIN-viewport coordinates when `ctl` lives in embedded window `w`.
func _centre_in(w: Window, ctl: Control) -> Vector2:
	return ctl.get_global_rect().get_center() + Vector2(w.position)


func _menu_button(title: String) -> MenuButton:
	for ch in _app.menu_bar_row.get_children():
		if ch is MenuButton and (ch as MenuButton).text == title:
			return ch
	return null


func _index_of(p: PopupMenu, text: String) -> int:
	for i in p.item_count:
		if not p.is_item_separator(i) and p.get_item_text(i) == text:
			return i
	return -1


## Presses a popup row with REAL keys (Down until focused, then Enter); a synthetic pointer motion does not
## move a popup's hover focus (measured in `_lmtypes_probe.gd`).
func _press_row(p: PopupMenu, text: String) -> bool:
	var idx := _index_of(p, text)
	if idx < 0 or not p.visible:
		return false
	var guard := 0
	while p.get_focused_item() != idx and guard < 80:
		await _key(KEY_DOWN)
		guard += 1
	if p.get_focused_item() != idx:
		return false
	await _key(KEY_ENTER)
	return true


## The chips the desktop bar draws now, as `[title, label]` pairs in bar order.
func _chips() -> Array:
	var out: Array = []
	for c in _app._quick_bar.chips():
		if c.visible:
			out.append(String(c.text))
	return out


func _chip(pin_key: String) -> Button:
	for c in _app._quick_bar.chips():
		if String(c.get_meta("pin_key", "")) == pin_key:
			return c
	return null


func _row_key(idx: CommandIndex, title: String) -> String:
	for r in idx.all():
		if String(r.get("title", "")) == title and bool(r.get("pinnable", false)):
			return String(r["pin_key"])
	return ""


func _palette_button(tool_id: String) -> Button:
	for b in _app._tool_strip_row.find_children("*", "Button", true, false):
		if String((b as Button).get_meta(DccWidgets.TOOL_ID_META, "")) == tool_id:
			return b
	return null


## A REAL click on item `i` of an `ItemList` inside embedded window `w`.
func _click_item(w: Window, list: ItemList, i: int) -> void:
	list.ensure_current_is_visible()
	var r := list.get_item_rect(i, true)
	await _click_at(list.get_global_rect().position + r.position + r.size * 0.5 + Vector2(w.position))
	await _settle(4)


func _rail(domain: String) -> void:
	var b: Button = _app._domain_buttons[domain]
	await _click_at(b.get_global_rect().get_center())
	await _settle(6)


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := args[i]
		if a == "--force-touch":
			pass
		elif a == "--density" and i + 1 < args.size():
			i += 1
		elif a == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		else:
			print("ABORT: unrecognised argument %s -- reads --force-touch, --density D and --shots <dir> only" % a)
			get_tree().quit(2)
			return
		i += 1
	get_tree().create_timer(560.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(2)
		return
	_part_a_store()
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	var picker := _app.find_child("PhoneProjectPicker", true, false)
	if picker != null and picker is Window:
		(picker as Window).hide()
	await _settle(6)
	_mode = "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s, viewport %s ==" % [_mode, get_viewport().get_visible_rect().size])
	if _mode == "phone":
		await _phone()
	else:
		await _shell_legs()
	_finish()


# -- A. the store -----------------------------------------------------------------------------------

func _part_a_store() -> void:
	print("== A. the store ==")
	var path := _cfg_path()
	## A pin stored BEFORE CQ-8 has no `d`: it must read as shown everywhere.
	_write_text(path, "[quick_bar]\n\nversion=1\nbar=[{\"k\": \"menu:1\", \"t\": \"Old pin\"}]\n")
	_reload()
	var old_pins := DccSettings.quick_bar_value("bar") as Array
	_ok(old_pins == [{"k": "menu:1", "t": "Old pin"}], "a pre-CQ-8 pin reads back unchanged, with no d key invented (%s)" % str(old_pins))
	for dom in ["world", "civilization", "cartography"]:
		_ok(_keys(DccSettings.quick_bar_pins_for("bar", dom)) == ["menu:1"], "...and is shown in %s" % dom)
	_ok(DccSettings.quick_bar_notices() == [], "a clean file has no notice")
	## Adding with domains stores them sorted; all three stores none.
	_ok(DccSettings.quick_bar_add("bar", "menu:2", "Two", ["cartography", "world"]) == "", "a pin can be added for two domains")
	_reload()
	_ok((DccSettings.quick_bar_value("bar") as Array)[1] == {"k": "menu:2", "t": "Two", "d": ["world", "cartography"]}, "stored with its domains in the shell's order")
	_ok(_keys(DccSettings.quick_bar_pins_for("bar", "civilization")) == ["menu:1"], "...and absent from civilization")
	_ok(_keys(DccSettings.quick_bar_pins_for("bar", "cartography")) == ["menu:1", "menu:2"], "...and present in cartography")
	_ok(DccSettings.quick_bar_pins_for("bar", "").size() == 2, "no known domain filters nothing")
	_ok(DccSettings.quick_bar_set_domains("bar", "menu:2", ["civilization", "world", "cartography"]) == "", "ticking every domain is accepted")
	_reload()
	_ok((DccSettings.quick_bar_value("bar") as Array)[1] == {"k": "menu:2", "t": "Two"} and not _text(path).contains("\"d\""), "...and stores NO d (absent, not a full list)")
	## Refusals write nothing.
	DccSettings.quick_bar_set_domains("bar", "menu:2", ["world"])
	var before := _text(path)
	var why := DccSettings.quick_bar_set_domains("bar", "menu:2", [])
	_ok(why.contains("at least one domain") and _text(path) == before, "an empty selection is refused with its reason and writes nothing")
	_ok(DccSettings.quick_bar_set_domains("bar", "menu:2", ["moon"]) != "" and _text(path) == before, "an unknown domain id is refused and writes nothing")
	_ok(DccSettings.quick_bar_set_domains("bar", "menu:99", ["world"]) == "That command is not pinned." and _text(path) == before, "an unpinned key is refused")
	_ok(DccSettings.quick_bar_add("bar", "menu:3", "Three", ["moon"]) != "" and _text(path) == before, "adding with an unknown domain is refused")
	## Moves hop over hidden pins: [A all, B world only, C all] in civilization.
	_write_text(path, "[quick_bar]\n\nversion=1\nbar=[{\"k\": \"A\", \"t\": \"A\"}, {\"k\": \"B\", \"t\": \"B\", \"d\": [\"world\"]}, {\"k\": \"C\", \"t\": \"C\"}]\n")
	_reload()
	_ok(DccSettings.quick_bar_move_visible("bar", "C", -1, "civilization"), "C moves left in civilization")
	_ok(_keys(DccSettings.quick_bar_value("bar")) == ["C", "A", "B"], "...past the hidden B, to A's place (got %s)" % str(_keys(DccSettings.quick_bar_value("bar"))))
	_ok(not DccSettings.quick_bar_move_visible("bar", "C", -1, "civilization"), "...and has no visible neighbour further left")
	## Corrupt d: kept, unrestricted, named. A junk entry is counted.
	_write_text(path, "[quick_bar]\n\nversion=1\nbar=[{\"k\": \"m:1\", \"t\": \"Str\", \"d\": \"yes\"}, {\"k\": \"m:2\", \"t\": \"Empty\", \"d\": []}, {\"k\": \"m:3\", \"t\": \"Moon\", \"d\": [\"moon\"]}, {\"k\": \"m:4\", \"t\": \"Num\", \"d\": [1]}, {\"k\": \"m:5\", \"t\": \"Good\", \"d\": [\"world\"]}, 7]\n")
	_reload()
	var cp := DccSettings.quick_bar_value("bar") as Array
	_ok(_keys(cp) == ["m:1", "m:2", "m:3", "m:4", "m:5"], "every pin with a corrupt d is KEPT (%s)" % str(_keys(cp)))
	_ok(not cp[0].has("d") and not cp[1].has("d") and not cp[2].has("d") and not cp[3].has("d") and cp[4]["d"] == ["world"], "...unrestricted, while the good one keeps its d")
	_ok(_keys(DccSettings.quick_bar_pins_for("bar", "civilization")) == ["m:1", "m:2", "m:3", "m:4"], "...so civilization shows the four, not the world-only one")
	var notes := DccSettings.quick_bar_notices()
	var blob := " ".join(PackedStringArray(notes))
	_ok(notes.size() == 2 and blob.contains("1 unreadable entry in the quick bar was skipped") and blob.contains("Str, Empty, Moon, Num"),
		"the notices name the four repaired pins and count the one skipped entry (%s)" % blob)
	## The first edit after it persists the repair (no corrupt d left), the notice then clears.
	_ok(DccSettings.quick_bar_add("bar", "m:6", "Six") == "", "a user edit is accepted")
	_reload()
	_ok(DccSettings.quick_bar_notices() == [] and not _text(path).contains("yes"), "...and the repair is what is stored: no notice, no corrupt d left")
	## A newer file.
	_write_text(path, "[quick_bar]\n\nversion=2\nbar=[{\"k\": \"m:1\", \"t\": \"Future\"}]\n")
	_reload()
	_ok((DccSettings.quick_bar_notices() as Array).size() == 1 and String(DccSettings.quick_bar_notices()[0]).contains("newer version"), "a newer file's notice says so")
	_ok(_text(path).contains("Future"), "...and reading it did not rewrite it")
	## Seed the shell run with an empty store.
	_write_text(path, "")
	_reload()


# -- B..E. the shell on desktop and tablet ---------------------------------------------------------------

func _shell_legs() -> void:
	var idx := CommandIndex.new()
	idx.build(_app, _app.bridge)
	var k_measure := "tool:measure"
	var k_auto := _row_key(idx, "Autosave")
	var k_dark := _row_key(idx, "Dark")
	var k_diag := _row_key(idx, "Diagnostics overlay")
	## The Dark chip's expected mark below assumes the dark theme; a wiped user dir may boot on the system theme.
	_app.menus._theme_popup.id_pressed.emit(DccMenus.ID_PREF_THEME_DARK)
	await _wait(0.6)
	print("== D0. tools and toggles are in the catalogue ==")
	var counts := {"Inspect": 0, "Measure": 0, "Region select": 0}
	for r in idx.all():
		if String(r.get("kind", "")) == "tool" and counts.has(String(r["title"])):
			counts[String(r["title"])] = int(counts[String(r["title"])]) + 1
	_ok(counts == {"Inspect": 1, "Measure": 1, "Region select": 1}, "Inspect, Measure and Region select are catalogued once each as tool rows (%s)" % str(counts))
	_ok(k_auto != "" and k_dark != "" and k_diag != "", "Autosave, Dark and Diagnostics overlay are pinnable toggle rows (%s, %s, %s)" % [k_auto, k_dark, k_diag])
	var unreg := idx.find_pin("menu:%d" % DccMenus.ID_PREF_UNITS_MI)
	_ok(unreg.is_empty() or (not bool(unreg.get("pinnable", false)) and String(unreg.get("why_not", "")).contains("no live state source is registered")),
		"a radio row nobody registered stays unpinnable and says why (%s)" % str(unreg.get("why_not", "-")))
	## Seed: a restricted tool, a plain toggle, a corrupt-d toggle, a civ-only toggle, and a junk entry.
	var path := _cfg_path()
	_write_text(path, "[quick_bar]\n\nversion=1\nbar=[{\"k\": \"%s\", \"t\": \"Measure\", \"d\": [\"world\", \"cartography\"]}, {\"k\": \"%s\", \"t\": \"Autosave\"}, {\"k\": \"%s\", \"t\": \"Dark\", \"d\": \"yes\"}, {\"k\": \"%s\", \"t\": \"Diagnostics overlay\", \"d\": [\"civilization\"]}, 7]\ndock=[{\"k\": \"%s\", \"t\": \"Region select\", \"d\": [\"cartography\"]}]\n" % [k_measure, k_auto, k_dark, k_diag, "tool:region"])
	_reload()
	_app._quick_bar._notices_said = false
	_app._quick_bar.refresh()
	await _settle(6)
	_app.select_domain("world")
	await _settle(6)
	print("== B. per-domain pins ==")
	_ok(_chips() == ["○ Measure", "● Autosave", "● Dark"], "world: the Measure, Autosave and Dark chips, with their live marks (got %s)" % str(_chips()))
	_ok(_chip(k_diag) == null, "...and no Diagnostics chip (it is civilization-only)")
	var spoke := false
	for l in _app.find_children("*", "Label", true, false):
		if (l as Label).text.begins_with("Quick bar:") and (l as Label).text.contains("Dark"):
			spoke = true
	_ok(spoke, "the store's notice was spoken in the status line and names Dark")
	await _rail("civilization")
	_ok(_chips() == ["● Autosave", "● Dark", "○ Diagnostics ove…"], "civilization: Autosave, Dark (the corrupt-d pin, kept everywhere) and Diagnostics (got %s)" % str(_chips()))
	await _rail("cartography")
	_ok(_chips() == ["○ Measure", "● Autosave", "● Dark"], "cartography: Measure is back (got %s)" % str(_chips()))
	var di: Array = []
	_app.select_domain("world")
	await _settle(4)
	di = QuickBarRing.dock_items(_app)
	var dock_labels: Array = []
	for it in di:
		dock_labels.append(String((it as Dictionary).get("label", "")))
	_ok(not dock_labels.has("○ Region select") and not dock_labels.has("● Region select"), "ring dock, world: the Cartography-only Region pin is not on the ring (%s)" % str(dock_labels))
	await _rail("cartography")
	dock_labels.clear()
	for it in QuickBarRing.dock_items(_app):
		dock_labels.append(String((it as Dictionary).get("label", "")))
	_ok(dock_labels.has("○ Region select"), "ring dock, cartography: the Region pin is on the ring with its mark (%s)" % str(dock_labels))
	await _rail("world")

	print("== C/D. toggle and tool chips follow the command's own state ==")
	var measure := _chip(k_measure)
	_armed_events.clear()
	_app.tool_armed.connect(func(id: String) -> void: _armed_events.append(id))
	await _click_at(measure.get_global_rect().get_center())
	await _wait(0.5)
	_ok(_app.armed_tool == "measure" and _armed_events.has("measure"), "a real click on the Measure chip armed Measure and fired tool_armed (armed %s, events %s)" % [_app.armed_tool, str(_armed_events)])
	_ok(measure.text == "● Measure", "...and the chip is lit (%s)" % measure.text)
	var insp := _palette_button("inspect")
	await _click_at(insp.get_global_rect().get_center())
	await _wait(0.5)
	_ok(_app.armed_tool == "inspect" and measure.text == "○ Measure", "arming Inspect from the PALETTE unlit the chip (armed %s, chip %s)" % [_app.armed_tool, measure.text])
	var pm := _palette_button("measure")
	await _click_at(pm.get_global_rect().get_center())
	await _wait(0.5)
	_ok(measure.text == "● Measure", "arming Measure from the palette lit it (%s)" % measure.text)
	await _click_at(insp.get_global_rect().get_center())
	await _wait(0.4)
	## Autosave: the chip, then the File menu.
	var auto := _chip(k_auto)
	var was := DccSettings.autosave_enabled()
	await _click_at(auto.get_global_rect().get_center())
	await _wait(0.5)
	_ok(DccSettings.autosave_enabled() == not was and auto.text == ("● Autosave" if not was else "○ Autosave"),
		"a real click on the Autosave chip flipped the setting (%s -> %s) and its mark (%s)" % [str(was), str(DccSettings.autosave_enabled()), auto.text])
	var fb := _menu_button("File")
	await _click_at(fb.get_global_rect().get_center())
	await _settle(3)
	var fp := fb.get_popup()
	var found := await _press_row(fp, "Autosave")
	await _wait(0.5)
	_ok(found and DccSettings.autosave_enabled() == was and auto.text == ("● Autosave" if was else "○ Autosave"),
		"File > Autosave pressed with real keys flipped it back and the chip followed (%s)" % auto.text)
	## Theme from the popup's own id_pressed (the menu route).
	var dark := _chip(k_dark)
	_app.menus._theme_popup.id_pressed.emit(DccMenus.ID_PREF_THEME_LIGHT)
	await _wait(0.6)
	_ok(dark.text == "○ Dark", "choosing Light in Preferences > Theme unlit the Dark chip (%s)" % dark.text)
	_app.menus._theme_popup.id_pressed.emit(DccMenus.ID_PREF_THEME_DARK)
	await _wait(0.6)
	_ok(dark.text == "● Dark", "choosing Dark lit it again (%s)" % dark.text)
	## Diagnostics via the real Shift+D accelerator, in civilization where the chip lives.
	await _rail("civilization")
	var diag := _chip(k_diag)
	await _key(KEY_D, true)
	await _wait(0.6)
	_ok(diag.text == "● Diagnostics ove…", "a real Shift+D accelerator turned the overlay on and the chip followed (%s)" % diag.text)
	await _key(KEY_D, true)
	await _wait(0.6)
	_ok(diag.text == "○ Diagnostics ove…", "...and off (%s)" % diag.text)
	await _rail("world")
	_ok(CommandIndex.new().find_pin("x").is_empty(), "(control) an absent key finds nothing")
	## run_pin_on arms a tool the same way the chip does.
	_ok(DccQuickBar.run_pin_on(_app, "tool:region") == "" and _app.armed_tool == "region", "a pin on the Region tool arms Region through run_pin_on (armed %s)" % _app.armed_tool)
	_app.arm_tool("inspect")

	print("== E. the Customise window, by real clicks ==")
	_app.open_quick_bar_window()
	await _settle(10)
	var w: QuickBarWindow = _app.quick_bar_window
	_ok(w.visible, "the Customise window opened")
	var notice := w.find_child("StoreNotice", true, false) as Label
	_ok(notice != null and notice.visible and notice.text.contains("Dark"), "its banner repeats the store's notice (%s)" % (notice.text if notice else "-"))
	var pins_list: ItemList = w.find_child("Pins", true, false)
	var texts: Array = []
	for n in pins_list.item_count:
		texts.append(pins_list.get_item_text(n))
	_ok(texts.size() == 4 and String(texts[0]).ends_with("only in World, Cartography") and String(texts[3]).ends_with("only in Civilization") and not String(texts[1]).contains("only in"),
		"the pin list says where a restricted pin shows and says nothing for an unrestricted one (%s)" % str(texts))
	await _click_item(w, pins_list, 0)
	var civ_box := w.find_child("Dom_civilization", true, false) as CheckBox
	var world_box := w.find_child("Dom_world", true, false) as CheckBox
	_ok(not civ_box.button_pressed and world_box.button_pressed and not civ_box.disabled, "selecting Measure shows World and Cartography ticked, Civilization not")
	await _click_at(_centre_in(w, civ_box))
	await _settle(6)
	_ok(not (DccSettings.quick_bar_value("bar")[0] as Dictionary).has("d") and _keys(DccSettings.quick_bar_pins_for("bar", "civilization")).has(k_measure),
		"ticking Civilization with a real click stored all three, which is no d at all (the Measure pin is now shown in civilization)")
	await _click_item(w, pins_list, 3)
	var civ_box2 := w.find_child("Dom_civilization", true, false) as CheckBox
	var store_before := _text(path)
	await _click_at(_centre_in(w, civ_box2))
	await _settle(6)
	var msg := w.find_child("Message", true, false) as Label
	_ok(msg.text.contains("at least one domain") and civ_box2.button_pressed and _text(path) == store_before,
		"unticking a pin's last domain is refused with its reason, the box goes back and nothing is stored (%s)" % msg.text)
	w.hide()
	await _shot("desktop")


# -- F. phone ----------------------------------------------------------------------------------------

func _phone() -> void:
	print("== F. phone ==")
	var path := _cfg_path()
	var idx := CommandIndex.new()
	idx.build(_app, _app.bridge)
	var k_auto := _row_key(idx, "Autosave")
	_write_text(path, "[quick_bar]\n\nversion=1\nbar=[{\"k\": \"%s\", \"t\": \"Autosave\"}, {\"k\": \"tool:measure\", \"t\": \"Measure\", \"d\": [\"civilization\"]}]\n" % k_auto)
	_reload()
	_app.select_domain("world")
	await _settle(8)
	var es := QuickBarPhone.entries(_app)
	var labels: Array = []
	for e in es:
		labels.append(String((e as Dictionary)["label"]))
	_ok(labels.has("● Autosave") and not labels.has("○ Measure") and not labels.has("● Measure"),
		"world: the phone's entries hold the Autosave toggle with its mark and not the civilization-only Measure (%s)" % str(labels))
	_app.select_domain("civilization")
	await _settle(8)
	labels.clear()
	for e in QuickBarPhone.entries(_app):
		labels.append(String((e as Dictionary)["label"]))
	_ok(labels.has("● Autosave") and labels.has("○ Measure"), "civilization: Measure is there too (%s)" % str(labels))
	_app.open_quick_bar_window()
	await _settle(12)
	var w: QuickBarWindow = _app.quick_bar_window
	_ok(w.visible, "the Customise window opened on the phone")
	var small: Array = []
	var n := 0
	for dom in DccSettings.QUICK_BAR_DOMAINS:
		var c := w.find_child("Dom_" + String(dom), true, false) as CheckBox
		if c == null:
			continue
		n += 1
		if c.size.y < 43.0:
			small.append("%s %.0f" % [dom, c.size.y])
	_ok(n == 3 and small.is_empty(), "the three Shown in boxes clear 44 dp (%d found; under: %s)" % [n, str(small)])
	await _shot("phone")


func _shot(tag: String) -> void:
	if _shots == "":
		return
	await _settle(3)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img != null:
		DirAccess.make_dir_recursive_absolute(_shots)
		img.save_png(_shots.path_join("cq8_%s_%s.png" % [_mode, tag]))
