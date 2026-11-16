extends Node
## Ruling FL item 2 probe: the quick bar's three deferred extras (`CUSTOM_QUICK_BAR_SCOPE.md` CQ-8).
##   desktop: godot --path . --resolution 1600x900  _fl2_probe.tscn
##   tablet:  godot --path . --resolution 1700x1000 _fl2_probe.tscn -- --force-touch --density=1.0
##   phone:   godot --path . --resolution 1080x2340 _fl2_probe.tscn -- --force-touch --density 2.625
## **Windowed, never `--headless`; a WIPED user dir (set APPDATA to a scratch folder); under the lock.** Arguments read:
## `--force-touch`, `--density D`, `--shots DIR`; anything else aborts with exit 2. Every expectation is a LITERAL.
##   A. THE STORE (no shell): a replacement is one pin per diagonal, refused for an unknown domain, put back by
##      `quick_bar_replace_clear` / `_reset`; the exported text is the literal layout; eleven invalid files are each
##      refused with their reason and write not one byte; a valid file installs all five parts or none.
##   B. THE RING (desktop and tablet; a world is generated): replacing World NW makes that slot a plain slot with the
##      pin's label and no sub-ring, the other seven slots are the built-in ones untouched, clearing it brings the
##      built-in slot (with its children) back, and a single-tool diagonal (Cartography NW, Label) can be replaced.
##   C. THE WINDOW (real clicks): the Replace slot tab lists twelve diagonals, Add replaces, Remove puts it back, the
##      Reset button puts every one back after a confirmation; a domain tool (Point of interest) pins limited to
##      Cartography and its chip arms the tool there; Export and Import buttons reach the picker / the share sheet,
##      an import confirms before it replaces, and an invalid file is refused with the window's message.
##   D. PHONE: the Export and Import buttons are 44 dp; export goes to the share sheet; the ring tabs stay hidden.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument / watchdog.
const SEED := 20260316

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _app: Node
var _bridge
var _win: QuickBarWindow


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(n: int = 4) -> void:
	for _i in n:
		await get_tree().process_frame


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _cfg_path() -> String:
	return ProjectSettings.globalize_path(DccSettings.CONFIG_PATH)


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


func _move_to(at: Vector2) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 2)
	Input.parse_input_event(mm)


func _button(at: Vector2, down: bool) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = down
	ev.position = at
	ev.global_position = at
	ev.button_mask = 1 if down else 0
	Input.parse_input_event(ev)


func _click_at(at: Vector2) -> void:
	_move_to(at)
	await _settle(2)
	_button(at, true)
	await _settle(2)
	_button(at, false)
	await _settle(4)


func _to_root(p: Vector2, c: Control) -> Vector2:
	var w := c.get_window()
	return p if w == null or w == get_tree().root else p * maxf(w.content_scale_factor, 0.01) + Vector2(w.position)


func _click(c: Control) -> void:
	await _click_at(_to_root(c.get_global_rect().get_center(), c))


func _click_item(list: ItemList, i: int) -> void:
	list.ensure_current_is_visible()
	var r := list.get_item_rect(i, true)
	await _click_at(_to_root(list.get_global_position() + r.get_center(), list))


func _click_tab(i: int) -> void:
	var r := _win._tabs.get_tab_rect(i)
	await _click_at(_to_root(_win._tabs.get_global_position() + r.get_center(), _win._tabs))


func _idx() -> CommandIndex:
	var idx := CommandIndex.new()
	idx.build(_app, _bridge)
	return idx


func _row_key(idx: CommandIndex, title: String) -> String:
	for r in idx.all():
		if String(r.get("title", "")) == title and bool(r.get("pinnable", false)):
			return String(r["pin_key"])
	return ""


## Filters the catalogue to `title` through the real search box's own handler, so the row is on screen to be clicked.
func _find_row(title: String) -> void:
	_win._search.text = title
	_win._render_catalogue()
	await _settle(2)


func _cat_index(key: String) -> int:
	for i in _win._cat_rows.size():
		if String(_win._cat_rows[i].get("pin_key", "")) == key:
			return i
	return -1


func _diag_index(key: String) -> int:
	for i in _win._diag_rows.size():
		if String(_win._diag_rows[i]["key"]) == key:
			return i
	return -1


func _shot(name: String) -> void:
	if _shots == "":
		return
	await _settle(4)
	get_viewport().get_texture().get_image().save_png("%s/%s_%s.png" % [_shots, name, _mode])


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
			print("ABORT: unrecognised argument %s -- reads --force-touch, --density D and --shots <dir> only" % a)
			get_tree().quit(2)
			return
		i += 1
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
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
	_bridge = _app.bridge
	_win = _app.quick_bar_window
	if _mode == "phone":
		await _phone()
	else:
		await _shell()
	_finish()


# -- A. the store ---------------------------------------------------------------------------------------

func _part_a_store() -> void:
	print("== A. the store ==")
	var path := _cfg_path()
	_write_text(path, "")
	_reload()
	_ok(DccSettings.quick_bar_value("replace") == {} and not DccSettings.has_quick_bar_value("replace"), "nothing is replaced at launch")
	_ok(DccSettings.quick_bar_replace_set("world.NW", "menu:1", "Pin one") == "" and DccSettings.quick_bar_replace_pin("world.NW") == {"k": "menu:1", "t": "Pin one"}, "a pin replaces World NW")
	_ok(DccSettings.quick_bar_replace_set("world.NW", "menu:2", "Pin two") == "" and DccSettings.quick_bar_replace_pin("world.NW") == {"k": "menu:2", "t": "Pin two"}, "a second pin REPLACES the first (one per diagonal)")
	_ok(DccSettings.quick_bar_replace_set("moon.NW", "menu:1", "x") == "That is not a ring diagonal.", "an unknown domain is refused with the diagonal sentence, not a store failure")
	_ok(DccSettings.quick_bar_replace_set("moon.NW", "menu:1", "x") != "" and DccSettings.quick_bar_replace_set("world.N", "menu:1", "x") != "" and DccSettings.quick_bar_replace_set("world.NW", "", "x") != "",
		"an unknown domain, a cardinal and an empty key are refused")
	_ok((DccSettings.quick_bar_value("replace") as Dictionary).keys() == ["world.NW"], "the refusals stored nothing")
	_reload()
	_ok(DccSettings.quick_bar_replace_pin("world.NW") == {"k": "menu:2", "t": "Pin two"}, "the replacement survives a reload of the settings file")
	_ok(DccSettings.quick_bar_replace_clear("world.NW") and DccSettings.quick_bar_replace_pin("world.NW") == {} and not DccSettings.quick_bar_replace_clear("world.NW"), "clearing puts the built-in back and clearing twice says false")
	DccSettings.quick_bar_replace_set("world.NW", "menu:1", "A")
	DccSettings.quick_bar_replace_set("cartography.NW", "menu:2", "B")
	_ok(DccSettings.quick_bar_replace_reset() and (DccSettings.quick_bar_value("replace") as Dictionary).is_empty() and not DccSettings.quick_bar_replace_reset(), "reset puts every one back")
	## a hand-written bad entry is dropped on read, never repaired
	_write_text(path, "[quick_bar]\n\nversion=1\nreplace={\"world.NW\": {\"k\": \"menu:1\", \"t\": \"ok\"}, \"moon.NE\": {\"k\": \"x\", \"t\": \"x\"}, \"world.N\": {\"k\": \"x\", \"t\": \"x\"}, \"world.SE\": \"junk\"}\n")
	_reload()
	_ok((DccSettings.quick_bar_value("replace") as Dictionary).keys() == ["world.NW"], "a hand-edited file keeps only the one good replacement")

	print("== A2. export text ==")
	_write_text(path, "")
	_reload()
	DccSettings.set_quick_bar_value("bar", [{"k": "menu:1", "t": "One", "d": ["world"]}])
	DccSettings.set_quick_bar_value("dock", [{"k": "menu:2", "t": "Two"}])
	DccSettings.set_quick_bar_value("share", {"civilization.NW": [{"k": "menu:3", "t": "Three"}]})
	DccSettings.quick_bar_replace_set("cartography.SW", "menu:4", "Four")
	DccSettings.set_quick_bar_value("assistant_button", false)
	var got = JSON.parse_string(DccSettings.quick_bar_export_text())
	var want := {"format": "cartalith-quick-bar", "version": 1.0, "bar": [{"k": "menu:1", "t": "One", "d": ["world"]}], "dock": [{"k": "menu:2", "t": "Two"}],
		"share": {"civilization.NW": [{"k": "menu:3", "t": "Three"}]}, "replace": {"cartography.SW": {"k": "menu:4", "t": "Four"}}, "assistant_button": false}
	_ok(got == want, "the exported file is exactly the literal layout (%s)" % str(got))
	_ok(not DccSettings.quick_bar_export_text().contains("seed") and not DccSettings.quick_bar_export_text().contains("world_"), "and carries no world data")
	var good := DccSettings.quick_bar_export_text()

	print("== A3. import: round trip and refusals ==")
	_write_text(path, "")
	_reload()
	_ok(DccSettings.quick_bar_import_text(good) == "" and JSON.parse_string(DccSettings.quick_bar_export_text()) == want, "a valid file installs the layout and exports back identically")
	_ok(DccSettings.quick_bar_value("assistant_button") == false and DccSettings.quick_bar_value("replace").has("cartography.SW"), "all five parts were installed")
	var before := _text(path)
	var bad25 := []
	for n in 25:
		bad25.append({"k": "menu:%d" % n, "t": "p"})
	var cases := [
		["not json", "{nope", "not valid JSON"],
		["a list", "[1,2]", "not a Cartalith quick bar file"],
		["another json file", "{\"hello\": 1}", "not a Cartalith quick bar file"],
		["no version", JSON.stringify({"format": "cartalith-quick-bar"}), "version"],
		["newer version", JSON.stringify({"format": "cartalith-quick-bar", "version": 2}), "newer build"],
		["unknown key", JSON.stringify({"format": "cartalith-quick-bar", "version": 1, "world": {}}), "unknown entry \"world\""],
		["25 bar pins", JSON.stringify({"format": "cartalith-quick-bar", "version": 1, "bar": bad25}), "at most 24"],
		["pin without a key", JSON.stringify({"format": "cartalith-quick-bar", "version": 1, "bar": [{"t": "x"}]}), "bar list"],
		["bad share key", JSON.stringify({"format": "cartalith-quick-bar", "version": 1, "share": {"nowhere": []}}), "\"nowhere\""],
		["replace in unknown domain", JSON.stringify({"format": "cartalith-quick-bar", "version": 1, "replace": {"moon.NW": {"k": "a", "t": "b"}}}), "replaced diagonals"],
		["switch not a bool", JSON.stringify({"format": "cartalith-quick-bar", "version": 1, "assistant_button": "yes"}), "true or false"],
	]
	for c in cases:
		var why := DccSettings.quick_bar_import_text(String(c[1]))
		_ok(why.contains(String(c[2])) and _text(path) == before, "%s is refused (%s) and nothing is written" % [c[0], why])
	## a file with one bad part among good ones writes none of them
	var half := JSON.stringify({"format": "cartalith-quick-bar", "version": 1, "bar": [{"k": "menu:9", "t": "Nine"}], "assistant_button": "x"})
	_ok(DccSettings.quick_bar_import_text(half) != "" and _text(path) == before and DccSettings.quick_bar_value("bar")[0]["k"] == "menu:1", "a half-valid file installs none of its parts")
	## the shipped layout exports as an empty one
	_write_text(path, "")
	_reload()
	var empty = JSON.parse_string(DccSettings.quick_bar_export_text())
	_ok(empty["bar"] == [] and empty["dock"] == [] and empty["share"] == {} and empty["replace"] == {} and empty["assistant_button"] == true, "an untouched store exports the empty layout")
	_write_text(path, "")
	_reload()


# -- B/C. the shell ----------------------------------------------------------------------------------

func _slot_sig(slots: Dictionary) -> Array:
	var out: Array = []
	for d in ["N", "E", "S", "W", "NW", "NE", "SE", "SW"]:
		var r: Dictionary = slots.get(d, {})
		out.append("%s:%s:%d" % [d, String(r.get("label", "")), (r["children"] as Array).size() if r.has("children") else -1])
	return out


func _broker():
	return _app.context_broker


func _shell() -> void:
	_bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384, "archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(8)
	var idx := _idx()
	var k_vault := _row_key(idx, "Markdown vault")
	var k_plan := _row_key(idx, "Journey planner")
	_ok(k_vault != "" and k_plan != "", "two pinnable commands found (%s, %s)" % [k_vault, k_plan])

	print("== B. the ring ==")
	var req_w := {"domain": "world", "armed_tool": "", "finalized": false}
	var req_c := {"domain": "cartography", "armed_tool": "", "finalized": false}
	var base_w := _slot_sig(_broker().ring_collect(req_w))
	var base_c := _slot_sig(_broker().ring_collect(req_c))
	_ok(base_w[4].begins_with("NW:") and not base_w[4].ends_with(":-1") and base_c[4].ends_with(":-1"), "built-in: World NW opens a sub-ring, Cartography NW is a single tool (%s, %s)" % [base_w[4], base_c[4]])
	DccSettings.quick_bar_replace_set("world.NW", k_vault, "Markdown vault")
	DccSettings.quick_bar_replace_set("cartography.NW", k_plan, "Journey planner")
	var rw: Dictionary = _broker().ring_collect(req_w)
	var sw := _slot_sig(rw)
	_ok(sw[4] == "NW:Vault:-1" or (sw[4].begins_with("NW:") and sw[4].ends_with(":-1") and bool(rw["NW"].get("replaced", false))), "World NW is now a plain slot showing the pin (%s)" % sw[4])
	_ok(rw["NW"].has("callable") and not rw["NW"].has("children") and String(rw["NW"]["builtin_label"]) == base_w[4].split(":")[1], "...it runs the pin, opens no sub-ring and remembers the built-in label")
	_ok(sw.slice(0, 4) == base_w.slice(0, 4) and sw.slice(5) == base_w.slice(5), "the four cardinals and the other three diagonals are exactly the built-in ones")
	var rc: Dictionary = _broker().ring_collect(req_c)
	_ok(bool(rc["NW"].get("replaced", false)) and _slot_sig(rc).slice(5) == base_c.slice(5), "a single-tool diagonal (Cartography NW, Label) can be replaced too")
	_ok(rw.size() == 8, "the compass still has exactly eight slots")
	_ok(_slot_sig(_broker().ring_collect({"domain": "civilization", "armed_tool": "", "finalized": false})).size() == 8, "Civilization is untouched")
	DccSettings.quick_bar_replace_reset()
	_ok(_slot_sig(_broker().ring_collect(req_w)) == base_w and _slot_sig(_broker().ring_collect(req_c)) == base_c, "putting the built-in slots back restores the exact original rings")

	print("== C. the Customise window, real clicks ==")
	_app.open_quick_bar_window()
	await _settle(10)
	_win = _app.quick_bar_window
	_ok(_win.visible and _win._tabs.tab_count == 4 and _win._tabs.get_tab_title(3) == "Replace slot", "the window has a fourth tab, Replace slot")
	await _click_tab(3)
	_ok(_win._target == "replace" and _win._diag.visible, "its tab shows the diagonal chooser")
	var enabled := 0
	for n in _win._diag.item_count:
		if not _win._diag.is_item_disabled(n):
			enabled += 1
	_ok(_win._diag.item_count == 12 and enabled == 12, "twelve diagonals are listed and every one can be replaced (%d of %d)" % [enabled, _win._diag.item_count])
	var cn := _diag_index("cartography.NW")
	_win._diag.select(cn)
	_win._on_diagonal_selected()
	await _settle(4)
	_ok(String(_win._diag.get_item_text(cn)).contains("built in") and _win._builtin_lbl.text.contains("a single tool"), "Cartography NW reads as a built-in single tool")
	await _shot("fl2_replace_tab")
	await _find_row("Journey planner")
	var ci := _cat_index(k_plan)
	_ok(ci >= 0, "the planner is in the catalogue")
	await _click_item(_win._catalogue, ci)
	await _click(_win._add_btn)
	await _settle(6)
	_ok(DccSettings.quick_bar_replace_pin("cartography.NW") == {"k": k_plan, "t": "Journey planner"}, "a real click on Replace the slot stores the pin")
	_ok(String(_win._diag.get_item_text(cn)).contains("replaced by Journey planner") and _win._pins_list.item_count == 1, "the chooser and the list say it")
	await _click(_win._add_btn)
	_ok(_win._msg.text.contains("already shows"), "adding the same command again says so (%s)" % _win._msg.text)
	_ok(bool(_broker().ring_collect(req_c)["NW"].get("replaced", false)), "the live ring shows it")
	await _click_item(_win._pins_list, 0)
	await _click(_win._remove_btn)
	await _settle(6)
	_ok(DccSettings.quick_bar_replace_pin("cartography.NW").is_empty() and _slot_sig(_broker().ring_collect(req_c)) == base_c, "Remove puts the built-in slot back (the ring is the original again)")
	## reset
	DccSettings.quick_bar_replace_set("world.NE", k_vault, "Markdown vault")
	DccSettings.quick_bar_replace_set("world.SW", k_plan, "Journey planner")
	_win._render_diagonals()
	_win._sync_buttons()
	await _click(_win._reset_btn)
	await _settle(6)
	_ok(_win._reset_dlg.visible and _win._reset_dlg.dialog_text.contains("2"), "Reset asks first and names the count (%s)" % _win._reset_dlg.dialog_text)
	_win._reset_dlg.get_cancel_button().pressed.emit()
	_win._reset_dlg.hide()
	await _settle(4)
	_ok((DccSettings.quick_bar_value("replace") as Dictionary).size() == 2, "Cancel keeps both replacements")
	await _click(_win._reset_btn)
	await _settle(6)
	_win._reset_dlg.get_ok_button().pressed.emit()
	await _settle(6)
	_ok((DccSettings.quick_bar_value("replace") as Dictionary).is_empty(), "OK puts every built-in slot back")

	print("== C2. a domain's own tool ==")
	await _click_tab(0)
	var k_poi := _row_key(_idx(), "Point of interest")
	_ok(k_poi == "tool:poi", "Point of interest is pinnable under the global tools' own key scheme (%s)" % k_poi)
	var sculpt := _row_key(_idx(), "Sculpt")
	_ok(sculpt == "tool:sculpt", "so is World's Sculpt (%s)" % sculpt)
	await _find_row("Point of interest")
	var pi := _cat_index(k_poi)
	await _click_item(_win._catalogue, pi)
	_ok(String(_win._detail.text).contains("Cartography tool"), "its detail line says it is a Cartography tool (%s)" % _win._detail.text)
	await _click(_win._add_btn)
	await _settle(6)
	var bar: Array = DccSettings.quick_bar_value("bar")
	_ok(bar.size() == 1 and bar[0] == {"k": "tool:poi", "t": "Point of interest", "d": ["cartography"]}, "the pin is stored limited to Cartography (%s)" % str(bar))
	_win.hide()
	await _settle(4)
	_app._select_domain("cartography")
	await _settle(8)
	var chip: Button = null
	for c in _app._quick_bar.chips():
		if String(c.get_meta("pin_key", "")) == "tool:poi" and c.visible:
			chip = c
	_ok(chip != null, "the chip is on the bar in Cartography")
	if chip != null:
		await _click(chip)
		await _settle(6)
		_ok(String(_app.armed_tool) == "poi", "a real click on it arms the Pins tool through arm_tool (armed %s)" % _app.armed_tool)
	_app.arm_tool("inspect")
	_app._select_domain("world")
	await _settle(8)
	var seen := false
	for c in _app._quick_bar.chips():
		if String(c.get_meta("pin_key", "")) == "tool:poi" and c.visible:
			seen = true
	_ok(not seen, "and it is not shown in World")
	DccSettings.set_quick_bar_value("bar", [])

	print("== C3. export and import ==")
	DccSettings.set_quick_bar_value("bar", [{"k": k_vault, "t": "Markdown vault"}])
	DccSettings.quick_bar_replace_set("civilization.NE", k_plan, "Journey planner")
	_app.open_quick_bar_window()
	await _settle(8)
	var asks: Array = []
	_win._picker_spy = func(what: String): asks.append(what)
	var shared: Array = []
	_app._share_spy = func(p: String): shared.append(p)
	await _click(_win._export_btn)
	await _click(_win._import_btn)
	if DccTheme.is_touch():
		_ok(shared.size() == 1 and String(shared[0]).ends_with("cartalith-quick-bar.json") and asks == ["import"], "touch: Export hands a cache file to the share sheet, Import raises the picker (%s %s)" % [str(shared), str(asks)])
		_ok(JSON.parse_string(FileAccess.get_file_as_string("user://cartalith-quick-bar.json"))["replace"].has("civilization.NE"), "...and the file is the layout")
	else:
		_ok(asks == ["export", "import"], "desktop: the buttons raise the export and import pickers (%s)" % str(asks))
	var tmp := "user://fl2_layout.json"
	_ok(_win.write_layout_file(tmp) == "", "the layout writes to a chosen path")
	DccSettings.set_quick_bar_value("bar", [])
	DccSettings.quick_bar_replace_reset()
	_ok(_win.import_from_path(tmp) == "" and is_instance_valid(_win._import_dlg) and _win._import_dlg.visible, "importing a valid file asks before replacing")
	_ok((DccSettings.quick_bar_value("bar") as Array).is_empty(), "...and has changed nothing yet")
	await _shot("fl2_import_confirm")
	_win._import_dlg.get_ok_button().pressed.emit()
	await _settle(6)
	_ok(DccSettings.quick_bar_value("bar")[0]["k"] == k_vault and DccSettings.quick_bar_replace_pin("civilization.NE")["k"] == k_plan and _win._msg.text == "Imported the layout.", "confirming installs the bar and the replacement")
	var junk := "user://fl2_junk.json"
	var jf := FileAccess.open(junk, FileAccess.WRITE)
	jf.store_string("{\"hello\": 1}")
	jf.close()
	var kept := _text(_cfg_path())
	var why := _win.import_from_path(junk)
	_ok(why.contains("not a Cartalith quick bar file") and _win._msg.text == why and not is_instance_valid(_win._import_dlg) and _text(_cfg_path()) == kept, "an invalid file is refused with a message and the settings file is byte-identical (%s)" % why)
	_ok(_win.import_from_path("user://does_not_exist.json").contains("Could not read"), "a missing file is refused")
	await _shot("fl2_refused")
	_win.hide()
	DccSettings.set_quick_bar_value("bar", [])
	DccSettings.quick_bar_replace_reset()


# -- D. phone ------------------------------------------------------------------------------------------

func _phone() -> void:
	print("== D. phone ==")
	_app.open_quick_bar_window()
	await _settle(14)
	_win = _app.quick_bar_window
	## The window lays out in logical units (its content scale is the phone scale), so 44 dp is 44 here.
	var fl := 43.5
	_ok(_win.visible and not _win._tabs.visible, "the window is open and the ring tabs stay hidden on a phone")
	for b in [_win._export_btn, _win._import_btn]:
		_ok(b.is_visible_in_tree() and b.get_global_rect().size.y >= fl, "%s is at the 44 dp floor (%.1f logical px, floor %.1f)" % [b.text, b.get_global_rect().size.y, fl])
	var shared: Array = []
	_app._share_spy = func(p: String): shared.append(p)
	DccSettings.set_quick_bar_value("bar", [{"k": "menu:1", "t": "One"}])
	## The phone's column scrolls; bring the row on screen the way a thumb would, then check it IS on screen.
	var sc := _win._root_box.get_parent() as ScrollContainer
	_ok(sc != null, "the phone window's column is a scroller")
	if sc != null:
		sc.ensure_control_visible(_win._export_btn)
		await _settle(4)
		var vis := Rect2(sc.get_global_rect())
		_ok(vis.encloses(_win._export_btn.get_global_rect()) and vis.encloses(_win._import_btn.get_global_rect()), "scrolled to the end, both buttons are inside the window (%s in %s)" % [str(_win._export_btn.get_global_rect()), str(vis)])
	await _shot("fl2_phone")
	await _click(_win._export_btn)
	_ok(shared.size() == 1 and String(shared[0]).ends_with("cartalith-quick-bar.json"), "Export hands the file to the share sheet (%s)" % str(shared))
	_ok(JSON.parse_string(FileAccess.get_file_as_string("user://cartalith-quick-bar.json"))["bar"][0]["k"] == "menu:1", "and the file holds the layout")
	await _shot("fl2_phone")
	DccSettings.set_quick_bar_value("bar", [])
