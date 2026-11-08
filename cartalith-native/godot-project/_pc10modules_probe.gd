extends Node
## PC-10 probe: the Context panel's ten modules and the user's arrangement of them
## (`PC_SHELL_PROPOSAL.md` §3.3 "Modules: tabs you arrange", milestone PC-10; Ruling DY), inside the
## real shell, with real mouse presses, drags and keys through `Input.parse_input_event()`.
##
##   desktop: godot --path . --resolution 1600x1000 _pc10modules_probe.tscn -- --phase main
##   tablet:  godot --path . --resolution 1600x1000 _pc10modules_probe.tscn -- --force-touch --phase main
##   then, with the SAME user dir:      -- [--force-touch] --phase restart --expect k1,k2,...
##   corrupt store, on a WIPED user dir: -- --phase corrupt --variant malformed|newer|unreadable
##
## **Windowed, never `--headless`**, with a wiped user dir (the harness points `APPDATA` at a fresh
## folder; `MISTAKES.md`, "Isolate a probe's user dir"). Reads `--force-touch`, `--phase`, `--expect`,
## `--variant` and `--shots <dir>`; anything else exits 2.
##
## What it protects (PC-10's bar):
##   A. DEFAULTS: nothing is stored on a wiped dir; the strip is INSPECT · NOTE · ASK with `+` beside it; the
##      registry lists exactly the ten keys (a literal list); Window ▸ Context panel ▸ MODULES' ten rows
##      are live check rows with tooltips, all unticked.
##   B. ADD + FIT: with a settlement selected by a real map click, each module is added through `+`
##      (a real click, then real arrow and Enter keys in its menu); each lands at the end of the
##      arrangement, and the strip shows exactly the modules §3.3's table fits to a settlement, then to a
##      faction (reached by a real click on the settlement page's holder link), then to nothing selected
##      (literal lists, so a wrong `kinds` entry fails here). Every shown module's page draws content.
##   C. ONE BUILDER, TWO HOSTS: for each module whose content ships in a window, the § sections that
##      window draws on its own tab for the same subject all appear in the module's page.
##   D. FOLLOW / PIN per module: with each settlement-fitting module shown, a real map click on another
##      place moves the module to it; FOLLOWING pressed pins it, and the next click leaves it alone.
##   E. ARRANGE: a real drag of one tab onto another moves it there; a tab's menu (right press on the
##      desktop, a held press on touch) Move left / Move right / Remove act (real keys in the menu);
##      Inspect's menu has no Remove; the MODULES row's tick follows; Reset panel returns the defaults.
##   F. KEYS: Ask removed, Ctrl+3 re-adds and shows it; Inspect dragged to the end, Ctrl+1 shows it.
##   G. WINDOW (desktop): a tab's menu ▸ Open in a window shows the module in a window that follows a
##      real map click until its chip pins it; on touch the menu offers no Open in a window.
##   H. WORLD FILE: a bookmark made with a real click in Recent & bookmarks is saved inside the project
##      (`annotations/bookmarks.json` listed), the arrangement is NOT in it, and a reopen restores the
##      bookmark while the session's recent list is empty.
##   R. RESTART (`--phase restart`): the arrangement left by `main` is read back, in order, and drawn.
##   X. CORRUPT (`--phase corrupt`): the probe writes a bad store BEFORE the shell loads it; the panel
##      shows the defaults, a notice names the backup, the backup holds the original bytes and the file
##      is untouched until the user arranges the panel, which then writes a readable store.
## Exit 0 pass, 1 fail, 2 bad argument, watchdog or no world, 3 a leg could not run.

const CONFIG := "user://cartalith_settings.cfg"
## §3.3's table, "Shown for", as literals: the kinds each module fits, read against the registry.
const ALL_KEYS := ["history", "economy", "military", "faith", "territory", "city", "here", "routes", "links", "recent"]
const FIT_SETTLEMENT := ["history", "economy", "military", "city", "here", "routes", "links", "recent"]
const FIT_FACTION := ["history", "economy", "military", "faith", "territory", "links", "recent"]
const FIT_NONE := ["here", "recent"]

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _phase := "main"
var _variant := ""
var _expect := ""
var _shots := ""
var _app: Node
var _bridge


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
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img != null:
		var path := _shots.path_join("pc10_%s_%s.png" % [_mode, tag])
		img.save_png(path)
		print("SHOT %s" % path)


## The pointer's last position, so a motion carries its real `relative`: the viewport starts a drag
## from the accumulated `relative` past 10 px (`Viewport::_gui_input_event`), not from the positions.
var _last_at := Vector2.ZERO


func _motion(at: Vector2, mask: int = 0) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = at - _last_at if at != _last_at else Vector2(0, 2)
	_last_at = at
	mm.button_mask = mask
	Input.parse_input_event(mm)
	await _settle(2)


func _button(at: Vector2, down: bool, idx: MouseButton = MOUSE_BUTTON_LEFT) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = idx
	ev.pressed = down
	ev.position = at
	ev.global_position = at
	if down:
		ev.button_mask = MOUSE_BUTTON_MASK_LEFT if idx == MOUSE_BUTTON_LEFT else MOUSE_BUTTON_MASK_RIGHT
	Input.parse_input_event(ev)
	await _settle(2)


func _click_at(at: Vector2, idx: MouseButton = MOUSE_BUTTON_LEFT) -> void:
	await _motion(at)
	await _button(at, true, idx)
	await _button(at, false, idx)
	await _settle(4)


func _key(code: Key, ctrl := false, alt := false) -> void:
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = code
		ev.physical_keycode = code
		ev.pressed = down
		ev.ctrl_pressed = ctrl
		ev.alt_pressed = alt
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(4)


func _dock():
	return _app.right_dock_ctrl


func _body() -> Control:
	return _app.right_dock_body


func _hdr(node_name: String) -> Control:
	return _body().find_child(node_name, true, false) as Control


## The keys of the tabs the strip draws, in order (each tab button carries `ctx_tab`).
func _strip() -> Array:
	var out: Array = []
	var tabs := _hdr("CtxTabs")
	if tabs == null:
		return out
	for b in tabs.get_children():
		if (b as Node).has_meta("ctx_tab"):
			out.append(String((b as Node).get_meta("ctx_tab")))
	return out


func _tab(key: String) -> Button:
	for b in _hdr("CtxTabs").get_children() if _hdr("CtxTabs") != null else []:
		if (b as Node).has_meta("ctx_tab") and String((b as Node).get_meta("ctx_tab")) == key:
			return b as Button
	return null


## A real press on `c`'s centre after scrolling it into the dock's view; fails when it is not drawn
## inside the dock's visible rect.
func _press(c: Control, what: String, idx: MouseButton = MOUSE_BUTTON_LEFT) -> bool:
	if c == null:
		_ok(false, "%s exists to be pressed" % what)
		return false
	var sc: ScrollContainer = _app._right_dock_scroll
	if sc != null and sc.is_ancestor_of(c):
		sc.ensure_control_visible(c)
		await _settle(4)
	var at := c.get_global_rect().get_center()
	var vis := sc.get_global_rect() if (sc != null and sc.is_ancestor_of(c)) else Rect2(Vector2.ZERO, get_viewport().get_visible_rect().size)
	var inside := vis.has_point(at) and c.is_visible_in_tree()
	if not inside:
		_ok(false, "%s is drawn inside the dock's visible rect (centre %s in %s)" % [what, str(at), str(vis)])
		return false
	await _click_at(at, idx)
	await _settle(4)
	return true


## The open popup menu named `nm` (a child of the app), or null.
func _popup(nm: String) -> PopupMenu:
	for c in _app.get_children():
		if c is PopupMenu and String(c.name).begins_with(nm) and (c as PopupMenu).visible:
			return c
	return null


## Picks the item with text `text` in the open popup by real keys: Down to it (the popup skips disabled
## items and separators), then Enter. False when the item is absent or disabled.
func _pick(pm: PopupMenu, text: String) -> bool:
	if pm == null:
		_ok(false, "a menu is open to pick '%s' from" % text)
		return false
	var target := -1
	var steps := 0
	for i in pm.item_count:
		if pm.is_item_separator(i) or pm.is_item_disabled(i):
			continue
		steps += 1
		if pm.get_item_text(i) == text:
			target = i
			break
	if target < 0:
		return false
	for _s in steps:
		await _key(KEY_DOWN)
	await _key(KEY_ENTER)
	await _settle(6)
	return true


## The open tab menu's item texts and their disabled flags, `[[text, disabled], ...]`.
func _items(pm: PopupMenu) -> Array:
	var out: Array = []
	if pm == null:
		return out
	for i in pm.item_count:
		if not pm.is_item_separator(i):
			out.append([pm.get_item_text(i), pm.is_item_disabled(i)])
	return out


## Opens tab `key`'s menu the way the form factor does it: a right press on the desktop, a held left
## press (past `TAB_HOLD_MS`) on touch. Returns the menu, or null.
func _open_tab_menu(key: String) -> PopupMenu:
	var b := _tab(key)
	if b == null:
		_ok(false, "the %s tab is drawn to open its menu" % key)
		return null
	await _into_view(b)
	var at := b.get_global_rect().get_center()
	await _motion(at)
	if _mode == "desktop":
		await _button(at, true, MOUSE_BUTTON_RIGHT)
		await _button(at, false, MOUSE_BUTTON_RIGHT)
	else:
		await _button(at, true)
		await get_tree().create_timer(0.8).timeout
		await _settle(4)
		await _button(at, false)
	await _settle(6)
	return _popup("CtxTabMenu")


## Drags tab `from` onto tab `onto` with a real left press, motion in steps with the button held, and a
## release over the target.
func _drag_tab(from: String, onto: String) -> void:
	var a := _tab(from)
	var b := _tab(onto)
	if a == null or b == null:
		_ok(false, "both tabs are drawn to drag %s onto %s" % [from, onto])
		return
	await _into_view(a)
	var p0 := a.get_global_rect().get_center()
	var p1 := b.get_global_rect().get_center()
	await _motion(p0)
	await _button(p0, true)
	for t in [0.1, 0.25, 0.5, 0.75, 1.0]:
		await _motion(p0.lerp(p1, t), MOUSE_BUTTON_MASK_LEFT)
	await _settle(4)
	await _button(p1, false)
	await _settle(8)


## Scrolls the dock so `c` (a tab, in the header at the top of the page) is in view.
func _into_view(c: Control) -> void:
	var sc: ScrollContainer = _app._right_dock_scroll
	if sc != null and sc.is_ancestor_of(c):
		sc.ensure_control_visible(c)
		await _settle(4)


# -- the map --------------------------------------------------------------------------------------

func _screen_of(cell: Vector2) -> Vector2:
	var ov = _app.viewport.overlay
	return (ov as Control).get_global_transform() * ov._cell_to_screen(cell, ov._displayed_rect())


func _visible_settlements() -> Array:
	var ov = _app.viewport.overlay
	var r: Rect2 = (ov as Control).get_global_rect().grow(-40.0)
	var all: Array = _bridge.settlements()
	var out: Array = []
	for i in all.size():
		var s: Dictionary = all[i]
		if String(s.get("name", "")).strip_edges() == "":
			continue
		if r.has_point(_screen_of(Vector2(float(s.get("x", 0)), float(s.get("y", 0))))):
			out.append(i)
	return out


## Clicks place `i` on the map for real, after the overlay's own hit test confirms the marker is that
## place's (`map_overlay.gd::_pick_mark`). True when the click selected it.
func _click_place(i: int) -> bool:
	var s: Dictionary = _bridge.settlements()[i]
	var p := _screen_of(Vector2(float(s.get("x", 0)), float(s.get("y", 0))))
	var ov = _app.viewport.overlay
	var rect: Rect2 = ov._displayed_rect()
	var local: Vector2 = (ov as Control).get_global_transform().affine_inverse() * p
	var pk: Dictionary = ov._pick_mark(local, ov._interior_rect(rect), rect)
	if int(pk.get("s", -1)) != i or int(pk.get("l", -1)) != -1:
		print("  info place %d's marker is not its own to click (%s)" % [i, str(pk)])
		return false
	## The hit test is the overlay's own maths and cannot see a control drawn over the map (a dock, a
	## toolbar button): the point must be the map's under the pointer, or the press would land on that
	## control instead (measured on the touch tablet: a press meant for a place ran Generate world).
	await _motion(p)
	var under := get_viewport().gui_get_hovered_control()
	if under == null or not (under == ov or (_app.viewport as Node).is_ancestor_of(under) or under == _app.viewport):
		print("  info place %d at %s is covered by %s" % [i, str(p), str(under.get_path()) if under != null else "nothing"])
		return false
	await _click_at(p)
	await _settle(4)
	var hit: bool = _app.selection == {"kind": "settlement", "id": i}
	if not hit:
		print("  info click on place %d at %s missed: hovered %s, selection %s, mouse pressed %s, windows %s" % [i, str(p),
			str(get_viewport().gui_get_hovered_control()), str(_app.selection), str(Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)),
			str(_app.find_children("*", "Window", true, false).filter(func(w): return (w as Window).visible).map(func(w): return w.name))])
	return hit


## Two places a real click selects, at least 60 px apart: `[a, b]`, or fewer when none are found.
func _two_places() -> Array:
	var got: Array = []
	for i in _visible_settlements():
		if got.size() >= 2:
			break
		if got.size() == 1:
			var pa := _screen_of(Vector2(float(_bridge.settlements()[got[0]].get("x", 0)), float(_bridge.settlements()[got[0]].get("y", 0))))
			var pb := _screen_of(Vector2(float(_bridge.settlements()[i].get("x", 0)), float(_bridge.settlements()[i].get("y", 0))))
			if pa.distance_to(pb) < 60.0:
				continue
		if await _click_place(int(i)):
			got.append(int(i))
	return got


## Clicks, for real, a place other than `not_i` that the overlay's hit test gives to it, trying
## `first` before the rest; its index, or -1 when none takes the click. (A place clickable once can be
## covered later, by a label drawn for the selection, so a leg never assumes the same one still is.)
func _click_other(not_i: int, first: int) -> int:
	## A covered or decluttered place is skipped without a press (`_click_place()`), so the cap is on
	## candidates looked at, not on presses.
	var order: Array = [first] if first >= 0 else []
	for i in _visible_settlements():
		if not order.has(i):
			order.append(i)
	var tried := 0
	for i in order:
		if int(i) == not_i or tried >= 40:
			continue
		tried += 1
		if await _click_place(int(i)):
			return int(i)
	return -1


## The § section titles drawn under `n`.
func _sections(n: Node) -> Array:
	var out: Array = []
	if n == null:
		return out
	for l in n.find_children("*", "Label", true, false):
		var t := (l as Label).text
		if t.begins_with("§ "):
			out.append(t)
	return out


# -- boot -----------------------------------------------------------------------------------------

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var k := 0
	while k < args.size():
		var a := args[k]
		if a == "--force-touch":
			pass
		elif a == "--phase" and k + 1 < args.size():
			k += 1
			_phase = args[k]
		elif a == "--variant" and k + 1 < args.size():
			k += 1
			_variant = args[k]
		elif a == "--expect" and k + 1 < args.size():
			k += 1
			_expect = args[k]
		elif a == "--shots" and k + 1 < args.size():
			k += 1
			_shots = args[k]
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
		k += 1
	if not (_phase in ["main", "restart", "corrupt"]) or (_phase == "corrupt" and not (_variant in ["malformed", "newer", "unreadable"])):
		print("ABORT: bad --phase %s / --variant %s" % [_phase, _variant])
		get_tree().quit(2)
		return
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(2)
		return
	## The corrupt store is written before the shell (and so `DccSettings`) reads the file.
	var before := PackedByteArray()
	if _phase == "corrupt":
		before = _write_corrupt_store()
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_mode = "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s, phase %s %s ==" % [_mode, _phase, _variant])
	if _mode == "phone":
		print("[FATAL] this probe is for the desktop and the tablet")
		get_tree().quit(3)
		return
	_bridge = _app.bridge
	match _phase:
		"main":
			await _gen()
			if not bool(_bridge.has_world):
				return
			await _leg_main()
		"restart":
			await _leg_restart()
		"corrupt":
			await _leg_corrupt(before)
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _gen() -> void:
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await _settle(2)
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _settle(10)
	await _settle(30)
	if not bool(_bridge.has_world) or _bridge.settlements().size() < 3 or _bridge.get_factions().size() < 2:
		print("[FATAL] no world with three settlements and two factions")
		get_tree().quit(2)


func _ctx_menu() -> PopupMenu:
	var wp: PopupMenu = null
	var burger: PopupMenu = null
	for ch in _app.menu_bar_row.get_children():
		if ch is MenuButton and (ch as MenuButton).text == "Window":
			wp = (ch as MenuButton).get_popup()
		if ch is MenuButton and (ch as MenuButton).text == "☰":
			burger = (ch as MenuButton).get_popup()
	if wp == null and burger != null:
		for i in burger.item_count:
			if burger.get_item_text(i) == "Window" and burger.get_item_submenu(i) != "":
				wp = burger.get_node_or_null(NodePath(burger.get_item_submenu(i))) as PopupMenu
	return wp.find_child("ContextPanelMenu", true, false) as PopupMenu if wp != null else null


## Each MODULES row's state: `{key: [present, checkable, enabled, tooltip non-empty, checked]}`, the ticks
## read after the submenu's own about-to-popup refresh (`menus.gd::_refresh_context_modules`).
func _module_rows() -> Dictionary:
	var sub := _ctx_menu()
	var out := {}
	if sub == null:
		return out
	sub.about_to_popup.emit()
	for m in DccMenus.CTX_MODULES:
		var i := sub.get_item_index(int(m[2]))
		if i < 0:
			out[String(m[0])] = [false, false, false, false, false]
		else:
			out[String(m[0])] = [true, sub.is_item_checkable(i), not sub.is_item_disabled(i),
				sub.get_item_tooltip(i).strip_edges() != "", sub.is_item_checked(i)]
	return out


func _leg_main() -> void:
	print("== A. defaults ==")
	var cfg := ConfigFile.new()
	var loaded := cfg.load(CONFIG)
	_ok(loaded != OK or not cfg.has_section("context_panel"), "a wiped user dir stores no [context_panel] (load %d)" % loaded)
	_ok(_app.context_panel_modules() == PackedStringArray(["inspect", "note", "ask"]), "the arrangement is Inspect · Note · Ask (got %s)" % str(_app.context_panel_modules()))
	_ok(_strip() == ["inspect", "note", "ask"], "the strip draws INSPECT · NOTE · ASK (got %s)" % str(_strip()))
	_ok(_hdr("CtxTabAdd") != null and _hdr("CtxTabAdd").is_visible_in_tree(), "+ is drawn beside the strip")
	var reg_keys: Array = []
	for e in _app.context_module_registry():
		reg_keys.append(String(e["key"]))
		_ok((e["builder"] as Callable).is_valid() and String(e["empty"]) != "", "the %s module has a builder and an empty sentence" % String(e["key"]))
	_ok(reg_keys == ALL_KEYS, "the registry lists the ten modules (got %s)" % str(reg_keys))
	var rows := _module_rows()
	for key in ALL_KEYS:
		var r: Array = rows.get(key, [false, false, false, false, false])
		_ok(r[0] and r[1] and r[2] and r[3] and not r[4], "MODULES ▸ %s is a live, unticked check row with a tooltip (%s)" % [key, str(r)])

	print("== B. add every module through + ==")
	var pair := await _two_places()
	if pair.size() < 2:
		_ok(false, "two places can be selected by a real map click")
		return
	var a: int = pair[0]
	var b: int = pair[1]
	await _click_place(a)
	for m in ALL_KEYS.size():
		var key: String = ALL_KEYS[m]
		await _press(_hdr("CtxTabAdd"), "+")
		var pm := _popup("CtxAddMenu")
		var title := ""
		for e in DccMenus.CTX_MODULES:
			if String(e[0]) == key:
				title = String(e[1])
		_ok(await _pick(pm, title), "+ lists %s and it can be picked" % title)
		var cur: PackedStringArray = _app.context_panel_modules()
		_ok(cur.size() > 0 and cur[cur.size() - 1] == key, "%s is added at the end of the arrangement (got %s)" % [key, str(cur)])
		var want_tab: String = key if FIT_SETTLEMENT.has(key) else "inspect"
		_ok(_dock().current_tab() == want_tab, "after adding %s with a settlement selected the panel shows %s (got %s)" % [key, want_tab, _dock().current_tab()])
	var cfg2 := ConfigFile.new()
	cfg2.load(CONFIG)
	_ok(cfg2.get_value("context_panel", "tabs", []) == ["inspect", "note", "ask"] + ALL_KEYS, "the store holds the arrangement in order (got %s)" % str(cfg2.get_value("context_panel", "tabs", [])))
	rows = _module_rows()
	for key in ALL_KEYS:
		_ok(bool((rows.get(key, [0, 0, 0, 0, false]) as Array)[4]), "MODULES ▸ %s is ticked once it is in the panel" % key)
	_ok(_strip() == ["inspect", "note", "ask"] + FIT_SETTLEMENT, "a settlement shows exactly §3.3's settlement modules (got %s)" % str(_strip()))
	await _shot("settlement_strip")
	await _leg_pages("settlement", FIT_SETTLEMENT)
	await _leg_same_builder_settlement(a)
	await _leg_follow_pin(a, b)

	## A faction, through a real click on the settlement page's holder link.
	await _press(_tab("inspect"), "INSPECT")
	await _click_place(a)
	var fl: Button = null
	for btn in _body().find_children("*", "Button", true, false):
		if (btn as Node).has_meta("ctx_target") and String((btn as Node).get_meta("ctx_target").get("kind", "")) == "faction" and (btn as Control).is_visible_in_tree():
			fl = btn
			break
	await _press(fl, "the holder link")
	_ok(String(_app.selection.get("kind", "")) == "faction", "the holder link selects a faction (got %s)" % str(_app.selection))
	_ok(_strip() == ["inspect", "note", "ask"] + FIT_FACTION, "a faction shows exactly §3.3's faction modules (got %s)" % str(_strip()))
	await _leg_pages("faction", FIT_FACTION)
	await _leg_same_builder_faction(int(_app.selection.get("id", -1)))

	## Nothing selected: a real Deselect (Ctrl+D is Edit ▸ Deselect's accelerator; a click on empty sea
	## would do the same but depends on the world's coast).
	await _press(_tab("inspect"), "INSPECT")
	_app.clear_selection()
	await _settle(6)
	_ok(_strip() == ["inspect", "note", "ask"] + FIT_NONE, "nothing selected shows exactly Here and Recent & bookmarks (got %s)" % str(_strip()))
	await _leg_pages("nothing", FIT_NONE)
	_ok(_app.context_panel_modules().size() == 13, "hidden modules stay in the arrangement (Inspect, Note, Ask and the ten modules) (got %d)" % _app.context_panel_modules().size())

	await _leg_arrange(a)
	await _leg_keys()
	if _mode == "desktop":
		await _leg_window(a, b)
	else:
		await _click_place(a)
		await _press(_tab("history"), "HISTORY")
		var pm := await _open_tab_menu("history")
		var names: Array = _items(pm).map(func(e): return e[0])
		_ok(pm != null and not names.has("Open in a window") and names.has("Remove"), "on touch a held tab opens its menu, without Open in a window (got %s)" % str(names))
		if pm != null:
			pm.hide()
			await _settle(4)
	await _leg_world_file(a)
	var final: PackedStringArray = _app.context_panel_modules()
	print("ARRANGEMENT %s" % ",".join(final))


## Clicks every tab in `keys` for real and checks its page: the module box exists, holds content, and
## the tab is the lit one.
func _leg_pages(label: String, keys: Array) -> void:
	await _press(_tab("inspect"), "INSPECT")
	await _settle(6)
	var dock_w: float = _app._right_dock_scroll.size.x
	for key in keys:
		await _press(_tab(key), "the %s tab (%s)" % [key, label])
		await _settle(6)
		_ok(absf(_app._right_dock_scroll.size.x - dock_w) <= 1.0, "%s: the %s page keeps the dock's width (%.0f, Inspect %.0f)" % [label, key, _app._right_dock_scroll.size.x, dock_w])
		var box := _hdr("CtxModule_" + String(key))
		_ok(_dock().current_tab() == key and box != null and box.get_child_count() > 0,
			"%s: the %s tab shows its module page (children %d)" % [label, key, box.get_child_count() if box != null else -1])
		var texts := 0
		if box != null:
			for l in box.find_children("*", "Label", true, false):
				if (l as Label).text.strip_edges() != "" and (l as Control).is_visible_in_tree():
					texts += 1
			for btn in box.find_children("*", "Button", true, false):
				if (btn as Button).text.strip_edges() != "" and (btn as Control).is_visible_in_tree():
					texts += 1
		_ok(texts > 0, "%s: the %s page draws text (%d labels and buttons)" % [label, key, texts])
		await _shot("%s_%s" % [label, key])


## C, for a settlement: the Place editor's own tab and the module draw the same § sections.
func _leg_same_builder_settlement(a: int) -> void:
	print("== C. one builder, two hosts (settlement) ==")
	for spec in [["history", "political"], ["economy", "economy"], ["city", "layout"]]:
		await _press(_tab(spec[0]), spec[0])
		var mod := _sections(_hdr("CtxModule_" + String(spec[0])))
		var pe = _app.place_editor_window
		pe.open_for_tab(a, String(spec[1]))
		await _settle(8)
		var win := _sections(pe._body.get_child(1) if pe._body.get_child_count() > 1 else null)
		pe.hide()
		await _settle(4)
		var missing := win.filter(func(t): return not mod.has(t))
		_ok(not win.is_empty() and missing.is_empty(), "%s: every section of the Place editor's %s tab is in the module (window %s, missing %s)" % [spec[0], spec[1], str(win), str(missing)])


## C, for a faction: the Factions window's own tab and the module draw the same § sections.
func _leg_same_builder_faction(fid: int) -> void:
	print("== C. one builder, two hosts (faction) ==")
	for spec in [["history", "history"], ["economy", "economy"], ["military", "military"], ["faith", "relations"], ["territory", "territory"]]:
		await _press(_tab(spec[0]), spec[0])
		await _settle(6)
		var mod := _sections(_hdr("CtxModule_" + String(spec[0])))
		var fr = _app.faction_roster_window
		fr.open(fid, String(spec[1]))
		await _settle(8)
		var win := _sections(fr._tab_panes.get(String(spec[1])))
		fr.hide()
		await _settle(4)
		var missing := win.filter(func(t): return not mod.has(t))
		_ok(not win.is_empty() and missing.is_empty(), "%s: every section of the Factions window's %s tab is in the module (window %s, missing %s)" % [spec[0], spec[1], str(win), str(missing)])


## D: per settlement-fitting module, follow moves it to the newly clicked place; a pin holds it.
func _leg_follow_pin(a: int, b: int) -> void:
	print("== D. follow and pin, per module ==")
	## Setup, not navigation: the Place editor opened in leg C can move the camera onto one place, and on
	## the touch tablet that left a single named place in view, under the left dock. The view is put back
	## to the whole map (View ▸ Reset view's function) so the leg has places to click.
	_app.viewport.reset_view()
	await _settle(10)
	for key in FIT_SETTLEMENT:
		## Each step clicks whichever place takes a real click now (`_click_other`): a place clickable
		## once can be covered later (a window that moved the camera, a label for the selection).
		var ai := await _click_other(-1, a)
		_ok(ai >= 0, "%s: a place takes a real click (%d)" % [key, ai])
		var sa := {"kind": "settlement", "id": ai}
		await _press(_tab(key), key)
		_ok(_dock()._module_subject() == sa, "%s shows place A" % key)
		var bi := await _click_other(ai, b)
		_ok(bi >= 0, "%s: another place takes a real click (%d)" % [key, bi])
		var sb := {"kind": "settlement", "id": bi}
		_ok(_dock().current_tab() == key and _dock()._module_subject() == sb, "%s follows a real click to place B (subject %s)" % [key, str(_dock()._module_subject())])
		await _press(_hdr("CtxPin"), "FOLLOWING")
		_ok(_dock().is_pinned(), "%s: FOLLOWING pressed pins the panel" % key)
		var ci := await _click_other(bi, ai)
		_ok(ci >= 0, "%s: a third click selects a place other than B while pinned (%d)" % [key, ci])
		var sc := {"kind": "settlement", "id": ci}
		_ok(_dock().current_tab() == key and _dock()._module_subject() == sb and _app.selection == sc,
			"%s stays on B while pinned and another place is selected (subject %s)" % [key, str(_dock()._module_subject())])
		await _press(_hdr("CtxFollow"), "FOLLOW")
		_ok(not _dock().is_pinned() and _dock()._module_subject() == sc, "%s: FOLLOW shows the selection again" % key)


## E: drag, the tab menu's Move left / Move right / Remove, Inspect's menu, the MODULES tick, Reset.
func _leg_arrange(a: int) -> void:
	print("== E. arrange ==")
	await _click_place(a)
	var before: PackedStringArray = _app.context_panel_modules()
	## Drag Routes onto History: Routes takes History's place.
	await _drag_tab("routes", "history")
	var after: PackedStringArray = _app.context_panel_modules()
	var want := before.duplicate()
	want.remove_at(want.find("routes"))
	want.insert(before.find("history"), "routes")
	_ok(after == want, "a real drag of ROUTES onto HISTORY moves it there (got %s, want %s)" % [str(after), str(want)])
	var strip := _strip()
	_ok(strip.find("routes") >= 0 and strip.find("routes") < strip.find("history"), "the strip draws ROUTES before HISTORY (got %s)" % str(strip))
	## Move right / Move left through the tab's menu.
	var at := after.find("economy")
	var pm := await _open_tab_menu("economy")
	_ok(await _pick(pm, "Move right"), "economy's menu has Move right")
	_ok(_app.context_panel_modules().find("economy") == at + 1, "Move right moves economy one place right (got %d, want %d)" % [_app.context_panel_modules().find("economy"), at + 1])
	pm = await _open_tab_menu("economy")
	_ok(await _pick(pm, "Move left"), "economy's menu has Move left")
	_ok(_app.context_panel_modules().find("economy") == at, "Move left moves it back (got %d)" % _app.context_panel_modules().find("economy"))
	## Inspect's menu: no Remove, and Move left disabled at the front.
	pm = await _open_tab_menu("inspect")
	var items := _items(pm)
	var names: Array = items.map(func(e): return e[0])
	_ok(pm != null and not names.has("Remove") and items.size() > 0 and items[0] == ["Move left", true], "Inspect's menu has no Remove and Move left is off at the front (got %s)" % str(items))
	if pm != null:
		pm.hide()
		await _settle(4)
	## Remove each settlement-fitting module through its menu, then add it back through the MODULES row.
	for key in ["here", "city"]:
		pm = await _open_tab_menu(key)
		_ok(await _pick(pm, "Remove"), "%s's menu has Remove" % key)
		_ok(not _app.context_panel_modules().has(key) and _tab(key) == null, "Remove takes %s out of the arrangement and the strip" % key)
		_ok(not bool((_module_rows().get(key, [0, 0, 0, 0, true]) as Array)[4]), "MODULES ▸ %s is unticked after Remove" % key)
		var sub := _ctx_menu()
		var id := -1
		for m in DccMenus.CTX_MODULES:
			if String(m[0]) == key:
				id = int(m[2])
		sub.id_pressed.emit(id)
		await _settle(6)
		_ok(_app.context_panel_modules().has(key), "MODULES ▸ %s adds it back" % key)
	## Reset panel.
	var keep: PackedStringArray = _app.context_panel_modules()
	_ctx_menu().id_pressed.emit(DccMenus.ID_WIN_CTX_RESET)
	await _settle(6)
	_ok(_app.context_panel_modules() == PackedStringArray(["inspect", "note", "ask"]) and _strip() == ["inspect", "note", "ask"], "Reset panel returns Inspect · Note · Ask (got %s)" % str(_app.context_panel_modules()))
	## Put the arrangement back for the legs after this one.
	for key in keep:
		if not _app.context_panel_modules().has(key):
			_app.set_context_module_shown(key, true)
	_ok(_app.context_panel_modules() == keep, "the arrangement is restored for the next legs")


## F: Ctrl+3 re-adds a removed Ask; Ctrl+1 reaches Inspect moved to the end.
func _leg_keys() -> void:
	print("== F. keys ==")
	var pm := await _open_tab_menu("ask")
	_ok(await _pick(pm, "Remove"), "Ask's menu has Remove")
	_ok(not _app.context_panel_modules().has("ask"), "Remove takes Ask out")
	await _key(KEY_3, true)
	_ok(_app.context_panel_modules().has("ask") and _dock().current_tab() == "ask", "Ctrl+3 re-adds Ask and shows it (tab %s)" % _dock().current_tab())
	await _key(KEY_1, true)
	var last: String = _strip()[_strip().size() - 1]
	await _drag_tab("inspect", last)
	var cur: PackedStringArray = _app.context_panel_modules()
	_ok(cur.find("inspect") > 0, "Inspect dragged along the strip leaves the front (at %d of %s)" % [cur.find("inspect"), str(cur)])
	await _press(_tab("history"), "HISTORY")
	await _key(KEY_1, true)
	_ok(_dock().current_tab() == "inspect", "Ctrl+1 shows Inspect wherever it sits (tab %s)" % _dock().current_tab())
	## Back to the front, by its menu's Move left, until it is first.
	for _i in 12:
		if _app.context_panel_modules().find("inspect") == 0:
			break
		var p := await _open_tab_menu("inspect")
		await _pick(p, "Move left")
	_ok(_app.context_panel_modules().find("inspect") == 0, "Move left walks Inspect back to the front")


## G (desktop): Open in a window, following then pinned.
func _leg_window(a: int, b: int) -> void:
	print("== G. open in a window ==")
	await _click_place(a)
	await _press(_tab("history"), "HISTORY")
	var pm := await _open_tab_menu("history")
	_ok(await _pick(pm, "Open in a window"), "history's menu has Open in a window")
	await _settle(8)
	var w: AcceptDialog = _app._module_windows.get("history")
	_ok(w != null and w.visible, "a History window is open")
	if w == null:
		return
	var wb := w.find_child("ModuleWindowBody", true, false) as Control
	_ok(wb != null and wb.get_child_count() > 0 and not w.exclusive, "the window draws the module and does not block the map (exclusive %s)" % str(w.exclusive))
	_ok(w._subject == {"kind": "settlement", "id": a}, "the window opens on the panel's subject")
	## Move the window off the map's middle so the click on a place reaches the map.
	w.position = Vector2i(get_viewport().get_visible_rect().size.x - w.size.x - 330, 60)
	await _settle(4)
	var bi := await _click_other(a, b)
	_ok(bi >= 0 and w._subject == {"kind": "settlement", "id": bi}, "the window follows a real click to place B (got %s)" % str(w._subject))
	var chip := w.find_child("FollowChip", true, false) as Button
	if chip != null:
		await _click_at(Vector2(w.position) + chip.get_global_rect().get_center())
	_ok(chip != null and w.has_meta("_ctxwin_pinned"), "the window's chip pins it")
	await _click_place(a)
	_ok(w._subject == {"kind": "settlement", "id": bi}, "the pinned window stays on B (got %s)" % str(w._subject))
	w.hide()
	await _settle(4)


## H: bookmarks in the world file; the arrangement and the recent list are not.
func _leg_world_file(a: int) -> void:
	print("== H. the world file ==")
	await _click_place(a)
	if _tab("recent") == null:
		_app.set_context_module_shown("recent", true)
		await _settle(4)
	await _press(_tab("recent"), "RECENT & BOOKMARKS")
	var tog := _hdr("ModuleBookmarkToggle") as Button
	await _press(tog, "Bookmark this")
	_ok(_app.context_bookmarks.size() == 1 and _app.is_bookmarked({"kind": "settlement", "id": a}), "the press bookmarks place A (%s)" % str(_app.context_bookmarks))
	_ok(_hdr("BookmarkRow") != null, "the bookmark is listed")
	var path := ProjectSettings.globalize_path("user://pc10_world.ctl")
	_app.current_project_path = path
	_app.save_project()
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < 60000 and (not FileAccess.file_exists(path) or bool(_bridge.get("saving") if _bridge.get("saving") != null else false)):
		await _settle(10)
	await _settle(60)
	var zr := ZIPReader.new()
	var opened := zr.open(path)
	_ok(opened == OK, "the project was saved (%s, %d)" % [path, opened])
	if opened != OK:
		return
	var files := zr.get_files()
	_ok(files.has("annotations/bookmarks.json"), "the saved .zip lists annotations/bookmarks.json")
	var leak := false
	for f in files:
		var txt := zr.read_file(f).get_string_from_utf8()
		if txt.contains("context_panel") or txt.contains("\"territory\",\"city\"") or txt.contains("CtxTab"):
			leak = true
			print("  info arrangement text found in %s" % f)
	zr.close()
	_ok(not leak, "no member of the saved .zip holds the arrangement (%d members)" % files.size())
	print("  info members: %s" % str(files))
	## Reopen.
	_ok(_app._load_project(path), "the project reopens")
	await _settle(30)
	_ok(_app.context_bookmarks.size() == 1 and _app.is_bookmarked({"kind": "settlement", "id": a}), "the bookmark survives the reopen (%s)" % str(_app.context_bookmarks))
	_ok(_app._selection_back.is_empty() and _app.selection.is_empty(), "the recent list is empty after the reopen (session-only)")
	await _press(_tab("recent"), "RECENT & BOOKMARKS")
	_ok(_hdr("BookmarkRow") != null and _hdr("RecentRow") == null, "Recent & bookmarks lists the bookmark and no recent row")


## R: the arrangement `main` left, read back after a restart, and drawn.
func _leg_restart() -> void:
	print("== R. restart ==")
	var want := PackedStringArray(_expect.split(",", false))
	_ok(not want.is_empty(), "--expect names the arrangement (%s)" % _expect)
	_ok(_app.context_panel_modules() == want, "the arrangement survives a restart (got %s, want %s)" % [str(_app.context_panel_modules()), str(want)])
	var drawn := _strip()
	var fits: Array = Array(want).filter(func(k): return k == "inspect" or k == "note" or k == "ask" or FIT_NONE.has(k))
	_ok(drawn == fits, "the strip draws it, in order, for nothing selected (got %s, want %s)" % [str(drawn), str(fits)])


## Writes the bad store the variant names, before the shell loads, and returns the file's bytes.
func _write_corrupt_store() -> PackedByteArray:
	var text := ""
	match _variant:
		"malformed":
			text = "[ui]\n\ntheme=\"dark\"\n\n[context_panel]\n\nversion=1\ntabs=42\n"
		"newer":
			text = "[ui]\n\ntheme=\"dark\"\n\n[context_panel]\n\nversion=99\ntabs=[\"inspect\", \"compare\", \"ask\"]\n"
		"unreadable":
			text = "[context_panel\ntabs=[\"inspect\"\n=== not a config file ===\n"
	var f := FileAccess.open(CONFIG, FileAccess.WRITE)
	f.store_string(text)
	f.close()
	return FileAccess.get_file_as_bytes(CONFIG)


## X: defaults with a notice; the backup holds the original bytes; the file is untouched until an
## arrangement is made, which then writes a readable store.
func _leg_corrupt(before: PackedByteArray) -> void:
	print("== X. corrupt store (%s) ==" % _variant)
	await _settle(10)
	_ok(_app.context_panel_modules() == PackedStringArray(["inspect", "note", "ask"]), "a %s store falls back to Inspect · Note · Ask (got %s)" % [_variant, str(_app.context_panel_modules())])
	_ok(_strip() == ["inspect", "note", "ask"], "the strip draws the defaults (got %s)" % str(_strip()))
	var status: String = _app.context_notice_shown
	var bak := "user://cartalith_settings.unreadable.bak.cfg" if _variant == "unreadable" else "user://cartalith_settings.context_panel.bak.cfg"
	print("  info notice: %s" % status)
	_ok(FileAccess.file_exists(bak) and FileAccess.get_file_as_bytes(bak) == before, "the original file was copied to %s byte for byte" % bak)
	_ok(status.contains(ProjectSettings.globalize_path(bak)), "the status line's notice names the backup (got '%s')" % status)
	## Other settings are written at boot (window size, layouts), so the FILE changes; the bad section
	## must not: it is left as it was until the user arranges the panel (an unreadable file has no section
	## to keep, and its backup above is what keeps it).
	if _variant != "unreadable":
		var c0 := ConfigFile.new()
		var lr := c0.load(CONFIG)
		var kept: bool = lr == OK and (c0.get_value("context_panel", "tabs", null) == 42 if _variant == "malformed"
			else int(c0.get_value("context_panel", "version", 0)) == 99)
		_ok(kept, "the bad [context_panel] section is left as it is until the panel is arranged (load %d)" % lr)
	## Arrange: + then Here (fits nothing selected).
	await _press(_hdr("CtxTabAdd"), "+")
	_ok(await _pick(_popup("CtxAddMenu"), "Here"), "+ adds Here")
	var cfg := ConfigFile.new()
	_ok(cfg.load(CONFIG) == OK and cfg.get_value("context_panel", "tabs", []) == ["inspect", "note", "ask", "here"], "the arrangement is then written readable (got %s)" % str(cfg.get_value("context_panel", "tabs", [])))
	_ok(FileAccess.get_file_as_bytes(bak) == before, "the backup still holds the original bytes")
	if _variant == "malformed" or _variant == "newer":
		_ok(String(cfg.get_value("ui", "theme", "")) == "dark", "the file's other sections survive the rewrite")
