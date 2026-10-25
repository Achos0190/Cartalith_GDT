extends Node
## Probe for the Customise window (`shell/quick_bar_window.gd`) and the right-click-a-menu-row pin
## gesture (`shell/quick_bar_menu_pin.gd`) -- milestone CQ-3 of `CUSTOM_QUICK_BAR_SCOPE.md`
## (Ruling DH), inside the real shell, with real mouse clicks and real key events.
##
##   desktop: godot --path . --resolution 1600x1000 _quickbar_window_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1280x800  _quickbar_window_probe.tscn -- --force-touch --shots <dir>
##   phone:   godot --path . --resolution 393x852   _quickbar_window_probe.tscn -- --force-touch --shots <dir>
##
## **Windowed, never `--headless`** (it moves the real cursor for section 9, generates a world and
## reads the framebuffer). The arguments read are `--force-touch` and `--shots <dir>`; anything
## else aborts with exit 2. **Run it from a scratch worktree with its own `config/name` and a
## wiped `app_userdata/<name>`** (`MISTAKES.md`: a probe reading `user://` is green for reasons
## unrelated to the code); section 0 fails if the pin store was not empty at launch.
##
## What it protects (CQ-3's bars, each against the real shell):
##   0. ENTRY: `Preferences ▸ Quick bar…` is a menu row `CommandIndex` finds BY TITLE (not by search
##      rank), absent on the phone; the `＋` chip (real click), `⋯ ▸ Customise…`, a pin's
##      right-click `Customise…` and the Preferences row all open the SAME window instance;
##   1. CATALOGUE: the default list is pinnable rows only, none with an empty group (colour-picker
##      preset menus), each carries its group and title; the footer counts what was left out;
##   2. ADD / REMOVE / REORDER BY MOUSE: three adds through real clicks on a row and on Add, a
##      reorder by the Down and Up buttons, a removal by the Remove button; the stored order is
##      the literal expected one and survives a reload of the settings file from disk;
##   3. REFUSALS: a 25th pin is refused with a stated reason and stores nothing; adding a pinned
##      command again is refused ("Already pinned."); adding an unpinnable row is refused with
##      that row's own `why_not`;
##   4. UNPINNABLE ROWS: with the toggle on, EVERY unpinnable row listed is disabled and carries a
##      non-empty reason in its text; a pinnable row is never disabled; a real click on a disabled
##      row selects nothing and leaves Add disabled;
##   5. RESET: Reset opens a confirmation; Cancel keeps every pin; OK writes the empty list, which
##      persists (`has_quick_bar_value("bar")` true, value `[]`, also after a reload);
##   6. KEYBOARD ONLY: Tab visits search, the toggle, the catalogue, Add, the pins and their
##      buttons in reading order; Enter adds the highlighted row; Alt+Down / Alt+Up reorder;
##      Delete removes -- all as real key events, no pointer;
##   7. CLUSTER FOLLOWS: every change reaches the cluster in the same call (chip count and order
##      equal the stored list);
##   8. PHONE: no Preferences row, no cluster (CQ-6 owns the phone);
##   9. Q11 (the menu-row right-click, WINDOWED): with the real cursor over a row of a real menu
##      popup, a right-click reaches `window_input`, names that row via `get_focused_item()`,
##      emits NO `id_pressed` (nothing runs) and leaves the menu open; the gesture then offers
##      `Pin to quick bar`, which pins it, and `Remove from quick bar` once pinned; a check row and
##      a parameter row offer nothing; a pin made this way is the pin the window lists.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument or watchdog.

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _app: Node
var _bridge
var _qb: DccQuickBar
var _win: QuickBarWindow


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
	var path := _shots.path_join("cq3_%s_%s.png" % [_mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


## A REAL mouse motion, press and release at the viewport point `at`, through the input system.
func _click_at(at: Vector2, button: int = MOUSE_BUTTON_LEFT) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 2)
	Input.parse_input_event(mm)
	await _settle(2)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = button
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(3)


func _click(c: Control, button: int = MOUSE_BUTTON_LEFT) -> void:
	await _click_at(_to_root(c.get_global_rect().get_center(), c), button)


## A point in `c`'s own window as a point in the main viewport. A control inside an EMBEDDED
## dialog reports `get_global_rect()` in that dialog's coordinates, so a click aimed at it from
## the main viewport must add the dialog's position (the first run of this probe clicked the
## wrong place and every in-window click did nothing).
func _to_root(p: Vector2, c: Control) -> Vector2:
	var w := c.get_window()
	return p if w == null or w == get_tree().root else p + Vector2(w.position)


## A real click on row `i` of `list` (its rect is local to the list, scroll already applied).
func _click_item(list: ItemList, i: int) -> void:
	var r := list.get_item_rect(i, true)
	await _click_at(_to_root(list.get_global_position() + r.get_center(), list))


## A real key press and release delivered to the focused control.
func _key(code: Key, alt: bool = false) -> void:
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = code
		ev.physical_keycode = code
		ev.pressed = down
		ev.alt_pressed = alt
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(2)


func _idx() -> CommandIndex:
	var idx := CommandIndex.new()
	idx.build(_app, _bridge)
	return idx


func _stored_keys() -> Array:
	var out: Array = []
	for p in DccSettings.quick_bar_value("bar"):
		out.append(String(p["k"]))
	return out


func _reload_from_disk() -> void:
	DccSettings._loaded = false
	DccSettings._cfg = null


func _chip_keys() -> Array:
	var out: Array = []
	for c in _qb.chips():
		out.append(String(c.get_meta("pin_key", "")))
	return out


## The catalogue row index whose row has `pin_key`, or -1.
func _cat_index(key: String) -> int:
	for i in _win._cat_rows.size():
		if String(_win._cat_rows[i].get("pin_key", "")) == key:
			return i
	return -1


## Filters the catalogue to `title` (what typing does) and returns the index of that row.
func _search_for(title: String, key: String) -> int:
	_win._search.text = title
	_win._search.text_changed.emit(title)
	await _settle(2)
	return _cat_index(key)


func _pin_titles() -> Array:
	var out: Array = []
	for i in _win._pins_list.item_count:
		out.append(_win._pins_list.get_item_text(i))
	return out


func _hide_windows() -> void:
	if _win and _win.visible:
		_win.hide()
	if _app.vault_window:
		_app.vault_window.hide()
	if _app.journey_planner_view and _app.journey_planner_view.is_visible_in_tree():
		_app.arm_tool("inspect")


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

	_ok(DccSettings.quick_bar_value("bar") == [] and not DccSettings.has_quick_bar_value("bar"), "the pin store is empty at launch (the user dir was wiped)")
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_mode = "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s, viewport %s ==" % [_mode, get_viewport().get_visible_rect().size])
	_bridge = _app.bridge
	_win = _app.quick_bar_window

	if _mode == "phone":
		await _phone()
		return

	_qb = _app._quick_bar
	await _entry()
	await _catalogue()
	await _mouse_edit()
	await _refusals()
	await _unpinnable()
	await _ghost()
	await _reset()
	await _keyboard()
	await _menu_row_gesture()
	_finish()


# -- 0 ------------------------------------------------------------------------------------

func _entry() -> void:
	print("== 0. entry points ==")
	_ok(_win != null and _win is QuickBarWindow, "the app owns one Customise window")
	var found := _app.find_children("*", "AcceptDialog", true, false).filter(func(n): return n is QuickBarWindow)
	_ok(found.size() == 1, "exactly one QuickBarWindow in the tree (%d)" % found.size())
	var idx := _idx()
	var row := {}
	for r in idx.all():
		if String(r["title"]) == "Quick bar":
			row = r
	_ok(not row.is_empty(), "CommandIndex finds the Preferences row BY TITLE ('Quick bar'; the index drops a title's ellipsis)")
	## Desktop names the menu; a tablet folds the whole menu bar into one button, so its group is that
	## button's glyph (CQ-2's probe notes the same fold).
	_ok(String(row.get("group", "")) == ("☰" if DccTheme.is_tablet() else "Preferences") and bool(row.get("pinnable", false)), "in the expected menu group and pinnable (a plain action row): group '%s'" % String(row.get("group", "")))
	## 1. the Preferences row, run as the menu click runs it
	_ok(idx.run_row(row), "the row runs through its own popup's id_pressed")
	await _settle(4)
	_ok(_win.visible, "the Preferences row opens the window")
	await _shot("00_window_empty")
	_win.hide()
	await _settle(2)
	## 2. the ＋ chip, a real click
	await _click(_qb.plus_chip())
	_ok(_win.visible, "a real click on ＋ opens the same window")
	_win.hide()
	await _settle(2)
	## 3. the chip's own menu and the overflow menu route to the same method
	_qb._chip_menu.id_pressed.emit(DccQuickBar.ID_CUSTOMISE)
	await _settle(3)
	_ok(_win.visible, "a pin's right-click menu 'Customise…' opens it")
	_win.hide()
	await _settle(2)
	_qb._more_menu.id_pressed.emit(DccQuickBar.ID_CUSTOMISE)
	await _settle(3)
	_ok(_win.visible, "the ⋯ menu's 'Customise…' opens it")
	_win.hide()
	await _settle(2)
	## the right-click menu actually lists the row, on a real pin
	DccSettings.quick_bar_add("bar", "menu:%d" % DccMenus.ID_VAULT, "Markdown vault")
	_qb.refresh()
	await _settle(3)
	var chip: Button = _qb.chips()[0]
	await _click(chip, MOUSE_BUTTON_RIGHT)
	var cm: PopupMenu = _qb._chip_menu
	_ok(cm.visible and cm.item_count == 5 and cm.get_item_text(4) == "Customise…", "a real right-click on a pin lists Customise… (%d rows)" % cm.item_count)
	cm.hide()
	DccSettings.quick_bar_remove("bar", "menu:%d" % DccMenus.ID_VAULT)
	_qb.refresh()
	await _settle(3)
	_ok(DccSettings.quick_bar_value("bar") == [], "back to an empty bar")


# -- 1 ------------------------------------------------------------------------------------

func _catalogue() -> void:
	print("== 1. catalogue ==")
	_win.open()
	await _settle(4)
	var rows: Array = _win._cat_rows
	var pinnable := 0
	var bad_group := 0
	var unpinnable_listed := 0
	for r in rows:
		if r.is_empty():
			continue
		if String(r["group"]) == "":
			bad_group += 1
		if bool(r["pinnable"]):
			pinnable += 1
		else:
			unpinnable_listed += 1
	_ok(pinnable >= 60, "the default list holds %d pinnable commands (CQ-0 measured 65-66)" % pinnable)
	_ok(unpinnable_listed == 0, "and no unpinnable row by default (%d)" % unpinnable_listed)
	_ok(bad_group == 0, "and no row with an empty group (colour-picker presets): %d" % bad_group)
	_ok(_win.hidden_picker_rows > 100, "the footer counts the %d colour-picker rows it left out" % _win.hidden_picker_rows)
	_ok(_win._foot.text.contains("%d colour-picker" % _win.hidden_picker_rows), "footer reads: %s" % _win._foot.text)
	_ok(_win._catalogue.get_item_text(0).contains("›"), "a row reads Group › Title: '%s'" % _win._catalogue.get_item_text(0))
	_ok(_win._count.text == "ON THE BAR  0 / 24", "the count starts at 0 / 24: '%s'" % _win._count.text)
	_ok(_win._add_btn.disabled and _win._up_btn.disabled and _win._down_btn.disabled and _win._remove_btn.disabled and _win._reset_btn.disabled, "with nothing selected and no pins, Add / Up / Down / Remove / Reset are all disabled")
	## the search narrows the list, by title, group or blurb
	_win._search.text = "vault"
	_win._search.text_changed.emit("vault")
	await _settle(2)
	_ok(_win._catalogue.item_count >= 1 and _win._catalogue.item_count < pinnable, "searching 'vault' narrows %d to %d" % [pinnable, _win._catalogue.item_count])
	_win._search.text = "zzzz-no-such-command"
	_win._search.text_changed.emit("zzzz")
	await _settle(2)
	_ok(_win._catalogue.item_count == 1 and _win._catalogue.is_item_disabled(0) and _win._catalogue.get_item_text(0).contains("No command matches"), "a search that matches nothing says so in a disabled line")
	_win._search.text = ""
	_win._search.text_changed.emit("")
	await _settle(2)
	_hide_windows()
	await _settle(2)


# -- 2 ------------------------------------------------------------------------------------

func _mouse_edit() -> void:
	print("== 2. add three, reorder, remove, reload (real clicks) ==")
	var kv := "menu:%d" % DccMenus.ID_VAULT
	var kj := "menu:%d" % DccMenus.ID_STORY_JOURNEY_PLANNER
	var third := ""
	for r in _idx().all():
		var k := String(r["pin_key"])
		if bool(r["pinnable"]) and String(r["group"]) != "" and k != kv and k != kj and not String(r["title"]).begins_with("Quick bar"):
			third = k
			break
	_ok(third != "", "a third pinnable command exists: %s" % third)
	_win.open()
	await _settle(4)
	for k in [kv, kj, third]:
		var row := _idx().find_pin(k)
		var at := await _search_for(String(row["title"]), k)
		_ok(at >= 0, "searching '%s' lists it" % String(row["title"]))
		await _click_item(_win._catalogue, at)
		_ok(_win._selected_row().get("pin_key", "") == k, "a real click selects that row")
		_ok(not _win._add_btn.disabled, "Add is enabled for a pinnable row")
		await _click(_win._add_btn)
	_ok(_stored_keys() == [kv, kj, third], "stored in click order: %s" % str(_stored_keys()))
	_ok(_chip_keys() == [kv, kj, third], "the cluster's chips equal the stored list at once: %s" % str(_chip_keys()))
	_ok(_win._count.text == "ON THE BAR  3 / 24", "count reads 3 / 24: '%s'" % _win._count.text)
	_ok(_win._catalogue.get_item_text(_cat_index(third)).contains("on the bar"), "the added row is marked 'on the bar' in the re-rendered catalogue: '%s'" % _win._catalogue.get_item_text(_cat_index(third)))
	await _shot("01_three_pins")
	## reorder by the real buttons: select the LAST pin and move it up twice, then down once
	await _click_item(_win._pins_list, 2)
	_ok(_win._selected_pin_index() == 2 and _win._down_btn.disabled and not _win._up_btn.disabled, "the last pin: Down disabled, Up enabled")
	await _click(_win._up_btn)
	await _click(_win._up_btn)
	_ok(_stored_keys() == [third, kv, kj], "Up twice: %s" % str(_stored_keys()))
	_ok(_win._up_btn.disabled and not _win._down_btn.disabled, "now first: Up disabled")
	await _click(_win._down_btn)
	_ok(_stored_keys() == [kv, third, kj], "Down once: %s" % str(_stored_keys()))
	_ok(_chip_keys() == [kv, third, kj], "the chips followed: %s" % str(_chip_keys()))
	## remove the middle one with the real Remove button
	await _click_item(_win._pins_list, 1)
	await _click(_win._remove_btn)
	_ok(_stored_keys() == [kv, kj], "Remove drops the selected pin: %s" % str(_stored_keys()))
	_ok(_chip_keys() == [kv, kj], "and its chip")
	## quit and reload: literal order from disk
	_reload_from_disk()
	_ok(_stored_keys() == [kv, kj], "after a reload from disk the order is [vault, planner]: %s" % str(_stored_keys()))
	_win.hide()
	await _settle(2)
	_win.open()
	await _settle(4)
	_ok(_pin_titles().size() == 2 and _win._count.text == "ON THE BAR  2 / 24", "reopening the window lists the two pins: %s" % str(_pin_titles()))
	await _shot("02_reordered")
	_hide_windows()
	await _settle(2)


# -- 3 ------------------------------------------------------------------------------------

func _refusals() -> void:
	print("== 3. refusals state their reason ==")
	var kv := "menu:%d" % DccMenus.ID_VAULT
	_win.open()
	await _settle(4)
	## already pinned
	var at := await _search_for("Markdown vault", kv)
	await _click_item(_win._catalogue, at)
	await _click(_win._add_btn)
	_ok(_win._msg.text == "Already pinned.", "adding a pinned command says why: '%s'" % _win._msg.text)
	_ok(_stored_keys().size() == 2, "and stores nothing more")
	## fill to 24, then the 25th
	var keys: Array = []
	var pins: Array = []
	for r in _idx().all():
		if bool(r["pinnable"]) and String(r["group"]) != "":
			keys.append(String(r["pin_key"]))
			pins.append({"k": String(r["pin_key"]), "t": String(r["title"])})
			if keys.size() == 24:
				break
	_ok(DccSettings.set_quick_bar_value("bar", pins), "filled the bar with 24 pins")
	_qb.refresh()
	_win._rebuild_all()
	await _settle(3)
	_ok(_win._count.text == "ON THE BAR  24 / 24", "count reads 24 / 24")
	var spare := ""
	for r in _idx().all():
		var k := String(r["pin_key"])
		if bool(r["pinnable"]) and String(r["group"]) != "" and not keys.has(k):
			spare = k
			break
	_ok(spare != "", "a pinnable command beyond the 24 exists")
	var srow := _idx().find_pin(spare)
	var sat := await _search_for(String(srow["title"]), spare)
	await _click_item(_win._catalogue, sat)
	_ok(not _win._add_btn.disabled, "Add stays enabled at the cap, so pressing it can SAY why")
	await _click(_win._add_btn)
	_ok(_win._msg.text.contains("at most 24") and _win._msg.text.contains("remove one first"), "the 25th is refused with the reason: '%s'" % _win._msg.text)
	_ok(_stored_keys().size() == 24 and not _stored_keys().has(spare), "and nothing was stored")
	_ok(_qb.pin_count() == 24, "the cluster still holds 24")
	await _shot("03_full_refusal")
	## an unpinnable row, offered directly (the window never lets a click reach one; this is the guard)
	var bad := {}
	for r in _idx().all():
		if not bool(r["pinnable"]) and String(r["group"]) != "":
			bad = r
			break
	var why: String = _win.add_row(bad)
	_ok(why.begins_with("Cannot be pinned: ") and why.length() > 20, "an unpinnable row is refused with its own reason: '%s'" % why)
	_ok(_win.add_row({}) == "Select a command first.", "an empty selection is refused")
	_ok(DccSettings.set_quick_bar_value("bar", []), "cleared")
	_qb.refresh()
	_hide_windows()
	await _settle(2)


# -- 4 ------------------------------------------------------------------------------------

func _unpinnable() -> void:
	print("== 4. unpinnable rows are disabled with their reason ==")
	_win.open()
	await _settle(4)
	_win._rebuild_all()
	await _click(_win._show_unpinnable)
	_ok(_win._show_unpinnable.button_pressed, "a real click turns 'Show commands that cannot be pinned' on")
	var n := _win._catalogue.item_count
	var pinnable_ok := 0
	var disabled_ok := 0
	var wrong_state := 0
	var no_reason := 0
	var unpinnable := 0
	for i in n:
		var r: Dictionary = _win._cat_rows[i]
		if r.is_empty():
			continue
		var dis := _win._catalogue.is_item_disabled(i)
		if bool(r["pinnable"]):
			pinnable_ok += 1
			if dis:
				wrong_state += 1
		else:
			unpinnable += 1
			if dis:
				disabled_ok += 1
			else:
				wrong_state += 1
			if String(r.get("why_not", "")).strip_edges() == "" or not _win._catalogue.get_item_text(i).contains("  --  "):
				no_reason += 1
	_ok(unpinnable > 100 and disabled_ok == unpinnable, "all %d listed unpinnable rows are disabled (%d)" % [unpinnable, disabled_ok])
	_ok(wrong_state == 0, "and no pinnable row is disabled (%d wrong)" % wrong_state)
	_ok(no_reason == 0, "and every one carries a non-empty reason in its own text (%d without)" % no_reason)
	_ok(_win.hidden_picker_rows > 100 and not _win._cat_rows.any(func(r): return not r.is_empty() and String(r["group"]) == ""), "colour-picker rows stay out even with the toggle on")
	## a real click on a disabled row selects nothing
	var drow := {}
	for i in n:
		if _win._catalogue.is_item_disabled(i) and not _win._cat_rows[i].is_empty():
			drow = _win._cat_rows[i]
			break
	## Filter to that row's title so it is on screen without scrolling.
	var di := await _search_for(String(drow["title"]), String(drow["pin_key"]))
	_win._catalogue.deselect_all()
	await _settle(2)
	await _shot("04_unpinnable_shown")
	var cr := _win._catalogue.get_item_rect(di, true)
	if cr.position.y >= 0 and cr.end.y <= _win._catalogue.size.y:
		await _click_item(_win._catalogue, di)
		_ok(_win._catalogue.get_selected_items().size() == 0 and _win._add_btn.disabled, "a real click on a disabled row selects nothing; Add stays disabled")
	else:
		_ok(false, "the first disabled row was not on screen to click (rect %s of %s)" % [str(cr), str(_win._catalogue.size)])
	await _click(_win._show_unpinnable)
	_ok(not _win._show_unpinnable.button_pressed and _win._catalogue.item_count < n, "turning it off hides them again (%d -> %d)" % [n, _win._catalogue.item_count])
	_hide_windows()
	await _settle(2)


# -- 4b -----------------------------------------------------------------------------------

func _ghost() -> void:
	print("== 4b. a pin whose command is gone is listed, kept, and removable ==")
	var kv := "menu:%d" % DccMenus.ID_VAULT
	DccSettings.set_quick_bar_value("bar", [{"k": "menu:99999", "t": "Gone command"}, {"k": kv, "t": "Markdown vault"}])
	_qb.refresh()
	_win.open()
	await _settle(4)
	var titles := _pin_titles()
	_ok(titles == ["Gone command  --  not in this build", "Markdown vault"], "the window lists the ghost with its stored title and the reason: %s" % str(titles))
	await _click_item(_win._pins_list, 0)
	await _click(_win._remove_btn)
	_ok(_stored_keys() == [kv], "Remove deletes the ghost by hand and keeps the live pin: %s" % str(_stored_keys()))
	_ok(_chip_keys() == [kv], "and the cluster follows")
	DccSettings.set_quick_bar_value("bar", [])
	_qb.refresh()
	_hide_windows()
	await _settle(2)


# -- 5 ------------------------------------------------------------------------------------

func _reset() -> void:
	print("== 5. reset to empty is confirmed and persists ==")
	var kv := "menu:%d" % DccMenus.ID_VAULT
	var kj := "menu:%d" % DccMenus.ID_STORY_JOURNEY_PLANNER
	DccSettings.set_quick_bar_value("bar", [{"k": kv, "t": "Markdown vault"}, {"k": kj, "t": "Journey planner"}])
	_qb.refresh()
	_win.open()
	await _settle(4)
	_ok(not _win._reset_btn.disabled, "Reset is enabled with two pins")
	await _click(_win._reset_btn)
	var dlg: ConfirmationDialog = _win._reset_dlg
	_ok(dlg.visible, "Reset opens a confirmation")
	_ok(_stored_keys().size() == 2, "and nothing is removed yet")
	_ok(dlg.dialog_text.contains("2 pins"), "which names the count: '%s'" % dlg.dialog_text)
	await _shot("05_reset_confirm")
	await _click(dlg.get_cancel_button())
	_ok(not dlg.visible and _stored_keys().size() == 2, "Cancel keeps both pins")
	await _click(_win._reset_btn)
	await _click(dlg.get_ok_button())
	_ok(_stored_keys() == [] and DccSettings.has_quick_bar_value("bar"), "OK writes the EMPTY list (the key exists, value [])")
	_reload_from_disk()
	_ok(_stored_keys() == [] and DccSettings.has_quick_bar_value("bar"), "and it persists across a reload")
	_ok(_qb.pin_count() == 0 and _chip_keys() == [], "the cluster is empty")
	_ok(_win._reset_btn.disabled and _win._count.text == "ON THE BAR  0 / 24", "and the window shows it")
	_hide_windows()
	await _settle(2)


# -- 6 ------------------------------------------------------------------------------------

func _keyboard() -> void:
	print("== 6. keyboard only ==")
	var kv := "menu:%d" % DccMenus.ID_VAULT
	var kj := "menu:%d" % DccMenus.ID_STORY_JOURNEY_PLANNER
	_win.open()
	await _settle(4)
	## every control is in the focus chain
	for c in [_win._search, _win._show_unpinnable, _win._catalogue, _win._add_btn, _win._pins_list, _win._up_btn, _win._down_btn, _win._remove_btn, _win._reset_btn]:
		_ok((c as Control).focus_mode == Control.FOCUS_ALL, "%s takes keyboard focus" % c.name)
	## Tab order from the search field: collect names, and check they appear in reading order
	_win._search.grab_focus()
	await _settle(2)
	var order: Array = []
	var cur: Control = _win._search
	for _n in 8:
		order.append(String(cur.name))
		var nxt := cur.find_next_valid_focus()
		if nxt == null or nxt == cur:
			break
		cur = nxt
	print("  info tab order from Search: ", order)
	var want := ["Search", "ShowUnpinnable", "Catalogue", "Add", "Pins", "Up", "Down", "Remove", "Reset"]
	var last := -1
	var monotone := true
	var seen := 0
	for nm in order:
		var wi := want.find(nm)
		if wi >= 0:
			seen += 1
			if wi <= last:
				monotone = false
			last = wi
	_ok(monotone and order.has("Catalogue") and order[1] == "ShowUnpinnable" and order[2] == "Catalogue", "Tab visits search, the toggle, then the catalogue, in reading order")
	## type to filter, Down to highlight, Enter to add
	var at := await _search_for("Markdown vault", kv)
	_ok(at == 0, "the filtered list starts with Markdown vault (index %d)" % at)
	_win._catalogue.grab_focus()
	await _settle(2)
	await _key(KEY_DOWN)
	_ok(_win._selected_row().get("pin_key", "") == kv, "Down highlights the first row")
	await _key(KEY_ENTER)
	_ok(_stored_keys() == [kv], "Enter adds it: %s" % str(_stored_keys()))
	at = await _search_for("Journey planner", kj)
	_win._catalogue.grab_focus()
	await _settle(2)
	await _key(KEY_DOWN)
	await _key(KEY_ENTER)
	_ok(_stored_keys() == [kv, kj], "Enter adds a second: %s" % str(_stored_keys()))
	## reorder with Alt+Up / Alt+Down on the pins list
	_win._pins_list.grab_focus()
	await _settle(2)
	_win._pins_list.select(1)
	_win._sync_buttons()
	await _key(KEY_UP, true)
	_ok(_stored_keys() == [kj, kv], "Alt+Up moves the selected pin up: %s" % str(_stored_keys()))
	await _key(KEY_DOWN, true)
	_ok(_stored_keys() == [kv, kj], "Alt+Down moves it back: %s" % str(_stored_keys()))
	_ok(_chip_keys() == [kv, kj], "the cluster followed both keys")
	## Delete removes the selected pin and selects its neighbour
	_win._pins_list.select(0)
	_win._sync_buttons()
	await _key(KEY_DELETE)
	_ok(_stored_keys() == [kj], "Delete removes the selected pin: %s" % str(_stored_keys()))
	_ok(_win._selected_pin_index() == 0, "and the remaining pin is selected, so Delete can be repeated")
	await _key(KEY_DELETE)
	_ok(_stored_keys() == [] and DccSettings.has_quick_bar_value("bar"), "a second Delete empties the bar and still writes it")
	await _shot("06_keyboard_done")
	_hide_windows()
	await _settle(2)


# -- 9 (Q11) -------------------------------------------------------------------------------

## Moves the REAL cursor: `DisplayServer.warp_mouse` is the only motion that updates a popup's
## hover (a synthetic `Input.parse_input_event` motion read `get_focused_item() == -1` across the
## whole popup when this was first tried; recorded in `quick_bar_menu_pin.gd`).
func _hover(p: Vector2) -> void:
	DisplayServer.warp_mouse(Vector2i(p))
	await _settle(2)


## Opens `popup` at a fixed place and returns the global point over its item `id`, found by
## sweeping the real cursor down the popup's centre until `get_focused_item()` names it.
func _row_point(popup: PopupMenu, id: int) -> Vector2:
	popup.popup(Rect2i(300, 120, 0, 0))
	await _settle(5)
	var want := popup.get_item_index(id)
	var x := float(popup.position.x) + float(popup.size.x) * 0.5
	var y := float(popup.position.y)
	while y < float(popup.position.y + popup.size.y):
		await _hover(Vector2(x, y))
		if popup.get_focused_item() == want:
			return Vector2(x, y)
		y += 3.0
	return Vector2(-1, -1)


func _menu_row_gesture() -> void:
	print("== 9. Q11: right-click a menu row, windowed ==")
	var pin: QuickBarMenuPin = _app.quick_bar_menu_pin
	_ok(pin != null, "the app owns the gesture's hook")
	var kv := "menu:%d" % DccMenus.ID_VAULT
	var idx := _idx()
	var row := idx.find_pin(kv)
	var popup: PopupMenu = row["_popup"]
	var pressed: Array = []
	popup.id_pressed.connect(func(id: int): pressed.append(id))
	var at := await _row_point(popup, DccMenus.ID_VAULT)
	_ok(at.x >= 0.0, "the real cursor over the World menu's vault row makes it the focused item (at %s)" % str(at))
	var seen: Array = []
	popup.window_input.connect(func(ev: InputEvent):
		if ev is InputEventMouseButton and (ev as InputEventMouseButton).button_index == MOUSE_BUTTON_RIGHT:
			seen.append([(ev as InputEventMouseButton).pressed, popup.get_focused_item()]))
	await _click_at(at, MOUSE_BUTTON_RIGHT)
	_ok(seen.size() >= 1 and seen[0][0] == true and seen[0][1] == popup.get_item_index(DccMenus.ID_VAULT), "window_input received the right-button press with get_focused_item() naming the vault row: %s" % str(seen))
	_ok(pressed.is_empty(), "no id_pressed was emitted: nothing ran (%s)" % str(pressed))
	var gm: PopupMenu = pin._menu
	_ok(gm.visible and gm.item_count == 1 and gm.get_item_text(0) == "Pin to quick bar", "a one-row menu offers 'Pin to quick bar' (visible=%s, '%s')" % [gm.visible, gm.get_item_text(0) if gm.item_count > 0 else ""])
	await _shot("07_row_context_pin")
	gm.id_pressed.emit(QuickBarMenuPin.ID_PIN)
	await _settle(3)
	_ok(_stored_keys() == [kv], "choosing it pins the command: %s" % str(_stored_keys()))
	_ok(_chip_keys() == [kv], "and the chip is on the bar")
	gm.hide()
	popup.hide()
	await _settle(3)
	## second right-click: now it offers removal
	at = await _row_point(popup, DccMenus.ID_VAULT)
	await _click_at(at, MOUSE_BUTTON_RIGHT)
	_ok(gm.visible and gm.get_item_text(0) == "Remove from quick bar", "a pinned row offers 'Remove from quick bar'")
	gm.id_pressed.emit(QuickBarMenuPin.ID_UNPIN)
	await _settle(3)
	_ok(_stored_keys() == [] and DccSettings.has_quick_bar_value("bar"), "choosing it removes the pin and still writes the empty list")
	gm.hide()
	popup.hide()
	await _settle(3)
	## a row that is not a pinnable command offers nothing
	var checks := 0
	for r in idx.all():
		if not bool(r["pinnable"]) and String(r["group"]) != "" and r.has("_popup") and is_instance_valid(r["_popup"]) and (r["_popup"] as PopupMenu).is_item_checkable((r["_popup"] as PopupMenu).get_item_index(int(r["_id"]))):
			var cp: PopupMenu = r["_popup"]
			var cat := await _row_point(cp, int(r["_id"]))
			if cat.x < 0.0:
				cp.hide()
				continue
			gm.hide()
			await _click_at(cat, MOUSE_BUTTON_RIGHT)
			_ok(not gm.visible, "a check row ('%s') offers no pin menu" % String(r["title"]))
			cp.hide()
			checks += 1
			break
	_ok(checks == 1, "a check row was exercised")
	## the pin made by the gesture is the pin the window lists
	DccSettings.quick_bar_add("bar", kv, "Markdown vault")
	_win.open()
	await _settle(4)
	_ok(_pin_titles() == ["Markdown vault"], "the window lists a pin made elsewhere: %s" % str(_pin_titles()))
	_hide_windows()
	DccSettings.set_quick_bar_value("bar", [])


# -- 8 ------------------------------------------------------------------------------------

func _phone() -> void:
	print("== 8. phone: no Preferences row, no cluster ==")
	var idx := _idx()
	var found := false
	for r in idx.all():
		if String(r["title"]) == "Quick bar":
			found = true
	_ok(not found, "no 'Quick bar' row on the phone (CQ-6 owns its pins)")
	_ok(_app._quick_bar == null, "and no cluster")
	await _shot("08_phone")
	_finish()
