extends Node
## Probe for the ring dock and the shared sub-rings -- milestone CQ-4 of `CUSTOM_QUICK_BAR_SCOPE.md`
## (Ruling DH) -- inside the real shell: `shell/radial_ring.gd`, `shell/quick_bar_ring.gd`,
## `shell/context_broker.gd` and the Customise window's Ring dock and Ring slots tabs.
##
##   desktop: godot --path . --resolution 1600x1000 _quickbar_ring_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1280x800  _quickbar_ring_probe.tscn -- --force-touch --shots <dir>
##   phone:   godot --path . --resolution 393x852   _quickbar_ring_probe.tscn -- --force-touch --shots <dir>
##
## **Windowed, never `--headless`** (it generates a world, reads the framebuffer and the real cursor
## path). The arguments read are `--force-touch` and `--shots <dir>`; anything else aborts with exit 2.
## **Run it from a scratch worktree with its own `config/name` and a wiped `app_userdata/<name>`**
## (`MISTAKES.md`: a probe that reads `user://` is green for reasons unrelated to the code); section
## 0 fails if the pin store was not empty at launch.
##
## The pixel-identity bar (an empty dock and no shared pins draw exactly what the ring drew before)
## is `_quickbar_ringid_probe.gd`'s: it must run against the PRE-change tree to write its baseline, so
## it cannot share this file.
##
## What this protects (CQ-4's bars, each against the real shell):
##   0. EMPTY: no dock rects, no dock count, a compass of exactly 8 slots with the N/E/S/W cardinals;
##   1. WINDOW TABS: the Ring dock tab adds four pins by real clicks, refuses a fifth with the store's
##      sentence, reorders and removes by real button clicks, and survives a reload from disk; the Ring
##      slots tab lists all 12 diagonals, disables the two single-tool ones with their reason, and
##      refuses a pin past a diagonal's allowance with a sentence naming it (World SE: 8 built in, 1
##      more; Cartography SW: 7 built in, 2 more);
##   2. DOCK GEOMETRY: chips are drawn only when docked, are at least 44 px both ways on touch
##      (measured off the live rects), sit below the disc, and never overlap one another;
##   3. DOCK RUNS: a slow RMB drag onto a chip and a release runs that chip's command (the vault
##      window opens); the hovered chip's full title is in the caption;
##   4. NO FLICK: the same drag as a no-wait flick (released inside 150 ms, ring never drawn) selects
##      no chip and runs nothing -- with the slow drag above as its positive control;
##   5. DISABLED WITH REASON: a docked pin whose command is gone is drawn dimmed, says "Not in this
##      build" in the caption, and a release over it runs nothing;
##   6. EDGES: a ring opened within 20 px of each of the four edges and two corners keeps every dock
##      chip on screen; the clamp is doing the work (the dock-less ring sits lower at the bottom);
##   7. CARD: the context card opened beside a docked ring overlaps no chip;
##   8. SHARED SUB-RINGS: at the maximum (built-in + pinned = 9) on every one of the 10 shareable
##      diagonals the compass is still 8 slots, built-ins stay first and untouched, every pairwise
##      slot distance is at least the live slot size, and a 10th pin is refused; a positive control
##      (a ring built with 10 children) falls below that bar; a leaf diagonal (CARTO Label) takes no
##      pin even from a hand-edited file; a real click on a shared child runs it; an unavailable one
##      runs nothing and says why.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument/watchdog.

const SEED := 483920

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _app: Node
var _bridge
var _win: QuickBarWindow
var _keys: Array = []   ## pinnable pin_keys in index order, for filler pins
var _vault_key := "menu:%d" % DccMenus.ID_VAULT
var _planner_key := "menu:%d" % DccMenus.ID_STORY_JOURNEY_PLANNER
var _ghost_key := "menu:999999"
## The four pins the Ring dock tab stored by clicks; later sections restore it by the store so one
## failure there cannot cascade into every ring section.
var _dock_want: Array = []
## The key of the `Preferences > Quick bar` row: a dock pin whose effect (the Customise window) is
## observable AND closable, so the gestures can run it without leaving a modal dialog open under later
## clicks (the first run pinned `File > New world`, whose dialog swallowed every later click).
var _quickbar_key := ""


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
	var path := _shots.path_join("cq4_%s_%s.png" % [_mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


# -- real input ---------------------------------------------------------------------------

func _move_to(at: Vector2, mask: int = 0) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.button_mask = mask
	mm.relative = Vector2(0, 2)
	Input.parse_input_event(mm)


func _button(at: Vector2, button: int, down: bool) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = button
	ev.pressed = down
	ev.position = at
	ev.global_position = at
	ev.button_mask = (1 << (button - 1)) if down else 0
	Input.parse_input_event(ev)


func _click_at(at: Vector2, button: int = MOUSE_BUTTON_LEFT) -> void:
	_move_to(at)
	await _settle(2)
	_button(at, button, true)
	await _settle(2)
	_button(at, button, false)
	await _settle(4)


func _to_root(p: Vector2, c: Control) -> Vector2:
	var w := c.get_window()
	return p if w == null or w == get_tree().root else p + Vector2(w.position)


func _click(c: Control) -> void:
	await _click_at(_to_root(c.get_global_rect().get_center(), c))


func _click_item(list: ItemList, i: int) -> void:
	var r := list.get_item_rect(i, true)
	await _click_at(_to_root(list.get_global_position() + r.get_center(), list))


func _click_tab(i: int) -> void:
	var r := _win._tabs.get_tab_rect(i)
	await _click_at(_to_root(_win._tabs.get_global_position() + r.get_center(), _win._tabs))


# -- helpers ------------------------------------------------------------------------------

func _idx() -> CommandIndex:
	var idx := CommandIndex.new()
	idx.build(_app, _bridge)
	return idx


func _reload_from_disk() -> void:
	DccSettings._loaded = false
	DccSettings._cfg = null


func _broker():
	return _app.context_broker


func _ring_state() -> Dictionary:
	var r = _broker().ring
	return r.debug_state() if r != null else {}


func _ring_reset() -> void:
	if _broker().ring != null:
		_broker().ring_close()
	if _broker().card != null and _broker().card.visible:
		_broker().card.hide()
	await _settle(3)


func _hide_windows() -> void:
	if _win and _win.visible:
		_win.hide()
	if _app.vault_window:
		_app.vault_window.hide()
	await _settle(2)


func _vault_open() -> bool:
	return _app.vault_window != null and _app.vault_window.visible


func _local(global: Vector2) -> Vector2:
	return _app.viewport.overlay.get_global_transform_with_canvas().affine_inverse() * global


func _slot_global(st: Dictionary, dir: String) -> Vector2:
	var deg := {"N": -90.0, "NE": -45.0, "E": 0.0, "SE": 45.0, "S": 90.0, "SW": 135.0, "W": 180.0, "NW": -135.0}
	var a := deg_to_rad(float(deg[dir]))
	return (st["centre"] as Vector2) + Vector2(cos(a), sin(a)) * float(st["radius"])


## Opens a sticky ring at `global` through the broker's Q path (visible at once; a tap leaves it open).
func _open_sticky(global: Vector2) -> Dictionary:
	await _ring_reset()
	_broker().ring_key_press(_local(global))
	await _settle(2)
	_broker().ring_key_release()
	await _settle(3)
	return _ring_state()


func _store_dock(keys: Array) -> void:
	var idx := _idx()
	var pins: Array = []
	for k in keys:
		var row := idx.find_pin(String(k))
		pins.append({"k": String(k), "t": String(row.get("title", "ghost"))})
	_ok(DccSettings.set_quick_bar_value("dock", pins), "stored %d dock pins" % pins.size())


func _dock_keys() -> Array:
	var out: Array = []
	for p in DccSettings.quick_bar_value("dock"):
		out.append(String(p["k"]))
	return out


func _min_pair_distance(pts: Array) -> float:
	var m := 1.0e9
	for i in pts.size():
		for j in range(i + 1, pts.size()):
			m = minf(m, (pts[i] as Vector2).distance_to(pts[j]))
	return m


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
	get_tree().create_timer(1100.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)

	print("== 0. shipped default ==")
	_ok(DccSettings.quick_bar_value("dock") == [] and DccSettings.quick_bar_value("share") == {} and not DccSettings.has_quick_bar_value("dock"), "the dock and share stores are empty at launch (the user dir was wiped)")
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

	_bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(8)
	for r in _idx().all():
		if bool(r["pinnable"]):
			_keys.append(String(r["pin_key"]))
			if String(r["title"]) == "Quick bar":
				_quickbar_key = String(r["pin_key"])
	_dock_want = [_vault_key, _planner_key, _quickbar_key]
	for k in _keys:
		if _dock_want.size() >= 4:
			break
		if not _dock_want.has(k) and not String(k).begins_with("extra:"):
			_dock_want.append(k)
	_ok(_quickbar_key != "" and _dock_want.size() == 4, "four distinct pinnable dock pins found: %s" % str(_dock_want))

	await _empty()
	await _window_tabs()
	await _dock_geometry()
	await _dock_runs()
	await _dock_disabled()
	await _edges()
	await _card()
	await _shared()
	_finish()


# -- 0 ------------------------------------------------------------------------------------

func _empty() -> void:
	print("== 0b. empty: the ring is the compass alone ==")
	var mid: Vector2 = _app.viewport.overlay.get_global_rect().get_center()
	var st := await _open_sticky(mid)
	_ok(bool(st.get("active", false)), "a ring opens")
	_ok(int(st["dock_count"]) == 0 and (st["dock_rects"] as Array).is_empty(), "no dock count and no dock rects with nothing docked")
	var slots: Dictionary = _broker().ring_collect(_broker().ring_domain_req())
	var keys: Array = slots.keys()
	keys.sort()
	_ok(keys == ["E", "N", "NE", "NW", "S", "SE", "SW", "W"], "exactly the 8 compass slots: %s" % str(keys))
	_ok(String(slots["N"]["label"]) == "Inspect" and String(slots["E"]["label"]) == "Measure" and String(slots["W"]["label"]) == "Region" and String(slots["S"]["label"]).begins_with("Undo"), "with the four fixed cardinals Inspect / Measure / Undo / Region")
	await _shot("00_empty_ring")
	await _ring_reset()


# -- 1 ------------------------------------------------------------------------------------

func _search_for(title: String, key: String) -> int:
	_win._search.text = title
	_win._search.text_changed.emit(title)
	await _settle(2)
	for i in _win._cat_rows.size():
		if String(_win._cat_rows[i].get("pin_key", "")) == key:
			return i
	return -1


func _add_by_clicks(key: String) -> void:
	var title := String(_idx().find_pin(key).get("title", ""))
	var at := await _search_for(title, key)
	if at < 0:
		_ok(false, "the catalogue lists %s" % key)
		return
	await _click_item(_win._catalogue, at)
	await _click(_win._add_btn)


func _window_tabs() -> void:
	print("== 1. the window's Ring dock and Ring slots tabs ==")
	_app.open_quick_bar_window()
	await _settle(6)
	_ok(_win.visible and _win._tabs != null and _win._tabs.tab_count == 3, "the window has three tabs")
	_ok(_win._tabs.get_tab_title(0) == "Bar" and _win._tabs.get_tab_title(1) == "Ring dock" and _win._tabs.get_tab_title(2) == "Ring slots", "named Bar, Ring dock, Ring slots")
	if DccTheme.is_tablet():
		_ok(_win._tabs.get_combined_minimum_size().y >= 44.0, "tablet: the tab strip is at least 44 px tall (%.0f)" % _win._tabs.get_combined_minimum_size().y)
	## The Ring dock tab
	await _click_tab(1)
	var win_rect := Rect2(Vector2.ZERO, Vector2(_win.size))
	_ok(win_rect.encloses(_win._add_btn.get_global_rect()) and win_rect.encloses(_win._foot.get_global_rect()), "the Add button and the footer are inside the window on the Ring dock tab (add %s, window %s)" % [str(_win._add_btn.get_global_rect()), str(win_rect)])
	_ok(_win._target == "dock" and _win._count.text == "ON THE RING DOCK  0 / 4", "a real click on the Ring dock tab switches to it: '%s'" % _win._count.text)
	_ok(not _win._diag.visible, "the diagonal chooser is hidden on this tab")
	var want: Array = _dock_want.duplicate()
	for k in want:
		await _add_by_clicks(String(k))
	_ok(_dock_keys() == want, "four real add clicks stored the dock in order: %s" % str(_dock_keys()))
	_ok(_win._count.text == "ON THE RING DOCK  4 / 4", "count reads 4 / 4: '%s'" % _win._count.text)
	var spare := ""
	for k in _keys:
		if not want.has(k):
			spare = String(k)
	await _add_by_clicks(spare)
	_ok(_dock_keys() == want and _win._msg.text == "The ring dock holds at most 4 pins; remove one first.", "a 5th dock pin is refused with the store's sentence and stores nothing: '%s'" % _win._msg.text)
	await _shot("01_window_dock_tab")
	## reorder by the real buttons, remove by the real button
	await _click_item(_win._pins_list, 2)
	await _click(_win._up_btn)
	var after_up: Array = want.duplicate()
	var t = after_up[2]
	after_up[2] = after_up[1]
	after_up[1] = t
	_ok(_dock_keys() == after_up, "the Up button moved pin 3 above pin 2: %s" % str(_dock_keys()))
	await _click(_win._down_btn)
	_ok(_dock_keys() == want, "the Down button moved it back")
	await _click_item(_win._pins_list, 3)
	await _click(_win._remove_btn)
	_ok(_dock_keys() == want.slice(0, 3), "the Remove button removed the last pin: %s" % str(_dock_keys()))
	_reload_from_disk()
	_ok(_dock_keys() == want.slice(0, 3), "the dock survives a reload of the settings file from disk")
	await _add_by_clicks(String(want[3]))
	_ok(_dock_keys() == want, "and adds back to four")

	_ok(DccSettings.quick_bar_value("bar") == [], "nothing leaked into the bar list")

	## The Ring slots tab
	await _click_tab(2)
	_ok(win_rect.encloses(_win._add_btn.get_global_rect()) and win_rect.encloses(_win._foot.get_global_rect()), "and on the Ring slots tab, with the chooser showing (add %s)" % str(_win._add_btn.get_global_rect()))
	_ok(_win._target == "share" and _win._diag.visible, "the Ring slots tab shows the diagonal chooser")
	_ok(_win._diag.item_count == 12, "12 diagonals are listed (3 domains x 4): %d" % _win._diag.item_count)
	var leaf := 0
	var usable := 0
	for n in _win._diag.item_count:
		var txt := _win._diag.get_item_text(n)
		if _win._diag.is_item_disabled(n):
			leaf += 1
			_ok(txt.contains("cannot share") or txt.contains("load a world"), "a disabled entry says why: %s" % txt)
		else:
			usable += 1
	_ok(leaf == 2 and usable == 10, "two single-tool diagonals disabled, ten shareable: %d / %d" % [leaf, usable])
	await _shot("02_window_slots_tab")
	## World SE: Freehand, 8 built in, 1 more fits
	var se := -1
	var sw := -1
	for n in _win._diag_rows.size():
		if String(_win._diag_rows[n]["key"]) == "world.SE":
			se = n
		if String(_win._diag_rows[n]["key"]) == "cartography.SW":
			sw = n
	_ok(se >= 0 and int(_win._diag_rows[se]["children"]) == 8, "World SE reports 8 built-in children (%d)" % (int(_win._diag_rows[se]["children"]) if se >= 0 else -1))
	_ok(sw >= 0 and int(_win._diag_rows[sw]["children"]) == 7, "Cartography SW reports 7 built-in children (%d)" % (int(_win._diag_rows[sw]["children"]) if sw >= 0 else -1))
	_win._diag.select(se)
	_win._diag.item_selected.emit(se)
	await _settle(2)
	_ok(_win._share_key == "world.SE" and _win._count.text == "ON WORLD.SE  0 / 1", "World SE chosen: '%s'" % _win._count.text)
	_ok(_win._builtin_lbl.text.begins_with("Built in, always first (8)"), "its built-in items are listed read-only: %s" % _win._builtin_lbl.text.left(60))
	await _add_by_clicks(_vault_key)
	_ok(DccSettings.quick_bar_share_pins("world.SE").size() == 1, "one pin added to World SE")
	await _add_by_clicks(_planner_key)
	_ok(DccSettings.quick_bar_share_pins("world.SE").size() == 1 and _win._msg.text.contains("at most 1 more fit"), "a second is refused, naming the allowance: '%s'" % _win._msg.text)
	_win._diag.select(sw)
	_win._diag.item_selected.emit(sw)
	await _settle(2)
	await _add_by_clicks(_vault_key)
	await _add_by_clicks(_planner_key)
	var third_key := ""
	for k in _keys:
		if k != _vault_key and k != _planner_key and not String(k).begins_with("extra:"):
			third_key = String(k)
			break
	await _add_by_clicks(third_key)
	_ok(DccSettings.quick_bar_share_pins("cartography.SW").size() == 2 and _win._msg.text.contains("at most 2 more fit"), "Cartography SW takes 2 and refuses the 3rd: '%s'" % _win._msg.text)
	## a leaf: the store refuses it with a reason
	var leaf_msg := DccSettings.quick_bar_share_add("cartography.NW", _vault_key, "Markdown vault", -1)
	_ok(leaf_msg.contains("no sub-ring") and DccSettings.quick_bar_share_pins("cartography.NW").is_empty(), "the store refuses a pin on the single-tool Label slot: '%s'" % leaf_msg)
	await _shot("03_window_slots_filled")
	_win.hide()
	await _settle(2)
	## clear the experiments; later sections set their own state
	_ok(DccSettings.set_quick_bar_value("share", {}), "share cleared for the ring sections")


# -- 2 ------------------------------------------------------------------------------------

func _dock_geometry() -> void:
	print("== 2. dock geometry ==")
	if _dock_keys() != _dock_want and not _dock_want.is_empty():
		_store_dock(_dock_want)
	var mid: Vector2 = _app.viewport.overlay.get_global_rect().get_center()
	var st := await _open_sticky(mid)
	_ok(int(st["dock_count"]) == 4, "four chips with four docked pins (%d)" % int(st["dock_count"]))
	var rects: Array = st["dock_rects"]
	_ok(rects.size() == 4, "four rects")
	var disc: float = float(st["disc_radius"])
	var centre: Vector2 = st["centre"]
	var below := true
	var tall_ok := true
	for r in rects:
		var rr: Rect2 = r
		if rr.position.y < centre.y + disc:
			below = false
		if DccTheme.is_tablet() and (rr.size.x < 44.0 or rr.size.y < 44.0):
			tall_ok = false
	_ok(below, "every chip sits below the disc's lower edge (disc %.0f px, first chip top %.0f px below the centre)" % [disc, (rects[0] as Rect2).position.y - centre.y])
	var gaps_ok := true
	for k in range(1, rects.size()):
		if (rects[k] as Rect2).position.x < (rects[k - 1] as Rect2).end.x:
			gaps_ok = false
	_ok(gaps_ok, "chips do not overlap one another")
	if DccTheme.is_tablet():
		_ok(tall_ok, "tablet: every chip is at least 44 x 44 as drawn (%.0f x %.0f)" % [(rects[0] as Rect2).size.x, (rects[0] as Rect2).size.y])
	else:
		_ok(is_equal_approx((rects[0] as Rect2).size.y, 40.0), "desktop: chips are the scope's 40 px tall (%.0f)" % (rects[0] as Rect2).size.y)
	await _shot("04_ring_with_dock")
	await _ring_reset()


# -- 3 / 4 --------------------------------------------------------------------------------

func _rmb_gesture_to_chip(chip_index: int, slow: bool) -> Dictionary:
	await _ring_reset()
	await _hide_windows()
	var mid: Vector2 = _app.viewport.overlay.get_global_rect().get_center()
	_move_to(mid)
	await _settle(2)
	var t_press := Time.get_ticks_msec()
	_button(mid, MOUSE_BUTTON_RIGHT, true)
	var st := _ring_state()
	if (st["dock_rects"] as Array).is_empty():
		await _settle(1)
		st = _ring_state()
	var rects: Array = st["dock_rects"]
	var target: Vector2 = (rects[chip_index] as Rect2).get_center() if not rects.is_empty() else mid + Vector2(0, 200)
	var out := {}
	if slow:
		_move_to(mid + Vector2(0, 30), MOUSE_BUTTON_MASK_RIGHT)
		await get_tree().create_timer(0.30).timeout   ## past FLICK_MS: the ring is drawn
		await _settle(2)
		var steps := 6
		var from := mid + Vector2(0, 30)
		for k in range(1, steps + 1):
			_move_to(from.lerp(target, float(k) / float(steps)), MOUSE_BUTTON_MASK_RIGHT)
			await _settle(2)
		var ms := _ring_state()
		out = {"visible": bool(ms.get("visible", false)), "dock_hover": int(ms.get("dock_hover", -1)),
			"caption": String(ms.get("caption", "")), "target": target, "seen_dock_hover": int(ms.get("dock_hover", -1)) >= 0}
		await _shot("05_ring_hover_dock_chip")
		_button(target, MOUSE_BUTTON_RIGHT, false)
	else:
		## The flick: press, one big move and the release are delivered with NO frame between them (an
		## input event is dispatched at once), so the ring is never drawn. The elapsed time is measured
		## and reported; the bar is 150 ms.
		_move_to(target, MOUSE_BUTTON_MASK_RIGHT)
		var ms := _ring_state()
		out = {"visible": bool(ms.get("visible", false)), "dock_hover": int(ms.get("dock_hover", -1)),
			"caption": String(ms.get("caption", "")), "target": target, "seen_dock_hover": int(ms.get("dock_hover", -1)) >= 0}
		_button(target, MOUSE_BUTTON_RIGHT, false)
		out["elapsed_ms"] = Time.get_ticks_msec() - t_press
	await _settle(6)
	out["after"] = _ring_state()
	return out


func _dock_runs() -> void:
	print("== 3. a slow drag onto a dock chip runs it; 4. a flick never does ==")
	_ok(not _vault_open(), "the vault window is closed before the gestures")
	## 4 first: the FLICK at chip 0 (the vault)
	var fl := await _rmb_gesture_to_chip(0, false)
	_ok(int(fl["elapsed_ms"]) <= 150, "the flick lasted %d ms from press to release (bar: 150)" % int(fl["elapsed_ms"]))
	_ok(not bool(fl["visible"]), "the flick never drew the ring (visible=false)")
	_ok(int(fl["dock_hover"]) == -1 and not bool(fl["seen_dock_hover"]), "no dock chip was hovered during the flick")
	_ok(not _vault_open(), "and the vault did not open: a flick never selects a dock chip")
	## Chip 0 lies in the SW sector, so a flick that far is the compass flick toward SW: it opens that
	## slot's sub-ring exactly as it does with no dock. That is the point -- the flick reads the
	## compass and nothing else.
	_ok(bool((fl["after"] as Dictionary).get("sub_open", false)) and int((fl["after"] as Dictionary).get("dock_hover", -1)) == -1, "the flick was read as the compass flick toward SW (its sub-ring is open), not as a dock pick")
	## 3: the SLOW drag, the positive control
	var sl := await _rmb_gesture_to_chip(0, true)
	_ok(bool(sl["visible"]), "the slow drag drew the ring")
	_ok(int(sl["dock_hover"]) == 0, "the pointer is over dock chip 0 (hover %d)" % int(sl["dock_hover"]))
	_ok(String(sl["caption"]).begins_with("Dock · ") and String(sl["caption"]).contains("Markdown vault"), "the caption names the chip's full title: '%s'" % String(sl["caption"]))
	_ok(_vault_open(), "releasing over it ran the pinned command: the vault window opened")
	_ok(not bool((sl["after"] as Dictionary).get("active", true)), "and closed the ring")
	await _hide_windows()
	## the third chip opens the Customise window (a second observable effect)
	var third := String(_dock_keys()[2])
	var sl2 := await _rmb_gesture_to_chip(2, true)
	var row := _idx().find_pin(third)
	_ok(int(sl2["dock_hover"]) == 2 and String(sl2["caption"]).contains(String(row.get("title", "?"))), "chip 3 hovered with its own title: '%s'" % String(sl2["caption"]))
	await _hide_windows()


# -- 5 ------------------------------------------------------------------------------------

func _dock_disabled() -> void:
	print("== 5. a docked pin whose command is gone ==")
	_store_dock([_vault_key, _ghost_key])
	await _ring_reset()
	var mid: Vector2 = _app.viewport.overlay.get_global_rect().get_center()
	var st := await _open_sticky(mid)
	_ok(int(st["dock_count"]) == 2, "two chips")
	var rects: Array = st["dock_rects"]
	_broker().ring_pointer(_local((rects[1] as Rect2).get_center()))
	await _settle(3)
	var s2 := _ring_state()
	_ok(int(s2["dock_hover"]) == 1 and String(s2["caption"]).contains("Not in this build"), "hovering the ghost chip says why: '%s'" % String(s2["caption"]))
	await _shot("06_ring_dock_ghost_hover")
	await _click_at((rects[1] as Rect2).get_center())
	await _settle(3)
	_ok(not _vault_open() and not bool(_ring_state().get("active", true)), "a click on it runs nothing and closes the ring")
	_ok(_dock_keys() == [_vault_key, _ghost_key], "the pin is kept, not deleted")
	await _ring_reset()


# -- 6 ------------------------------------------------------------------------------------

func _edges() -> void:
	print("== 6. the dock is never clipped at an edge ==")
	_store_dock(_dock_want)
	var ov: Rect2 = _app.viewport.overlay.get_global_rect()
	var view := Rect2(Vector2.ZERO, get_viewport().get_visible_rect().size)
	var pts := {
		"top": Vector2(ov.get_center().x, ov.position.y + 12.0),
		"bottom": Vector2(ov.get_center().x, ov.end.y - 12.0),
		"left": Vector2(ov.position.x + 12.0, ov.get_center().y),
		"right": Vector2(ov.end.x - 12.0, ov.get_center().y),
		"top-left": ov.position + Vector2(12, 12),
		"bottom-right": ov.end - Vector2(12, 12),
	}
	for name in pts:
		var st := await _open_sticky(pts[name])
		var rects: Array = st["dock_rects"]
		var all_in := rects.size() == 4
		for r in rects:
			if not view.encloses(r):
				all_in = false
		_ok(all_in, "%s edge: all 4 chips inside the window %s (centre %s)" % [name, str(view.size), str(st["centre"])])
		var disc_in := view.encloses(Rect2((st["centre"] as Vector2) - Vector2.ONE * float(st["disc_radius"]), Vector2.ONE * 2.0 * float(st["disc_radius"])))
		_ok(disc_in, "%s edge: the disc is inside too" % name)
	## the clamp is doing the work: with the dock the ring sits higher at the bottom than without
	var with_dock := await _open_sticky(pts["bottom"])
	var y_with: float = (with_dock["centre"] as Vector2).y
	_ok(DccSettings.set_quick_bar_value("dock", []), "dock emptied")
	var without := await _open_sticky(pts["bottom"])
	var y_without: float = (without["centre"] as Vector2).y
	_ok(y_with < y_without - 20.0, "positive control: at the bottom edge the docked ring sits %.0f px higher than the dock-less one" % (y_without - y_with))
	_ok(int(without["dock_count"]) == 0 and (without["dock_rects"] as Array).is_empty(), "and the emptied dock leaves no rects")
	var top_with: Dictionary = {}
	_store_dock(_dock_want)
	top_with = await _open_sticky(pts["top"])
	_ok(DccSettings.set_quick_bar_value("dock", []), "dock emptied again")
	var top_without := await _open_sticky(pts["top"])
	_ok((top_with["centre"] as Vector2).y == (top_without["centre"] as Vector2).y, "the top edge is unaffected by the dock (same centre y %.1f)" % (top_with["centre"] as Vector2).y)
	await _ring_reset()


# -- 7 ------------------------------------------------------------------------------------

func _card() -> void:
	print("== 7. the context card clears the dock ==")
	_store_dock(_dock_want)
	await _ring_reset()
	await _hide_windows()
	var mid: Vector2 = _app.viewport.overlay.get_global_rect().get_center()
	if DccTheme.is_tablet():
		_broker().ring_touch_open(_local(mid))
		await _settle(6)
	else:
		_move_to(mid)
		await _settle(2)
		_button(mid, MOUSE_BUTTON_RIGHT, true)
		await get_tree().create_timer(0.55).timeout   ## still hold: ring and card open together
		await _settle(4)
	var st := _ring_state()
	var card = _broker().card
	_ok(card != null and card.visible and bool(st.get("active", false)), "ring and card are both open")
	if card != null and card.visible:
		var cr: Rect2 = card.panel_rect()
		var hit := false
		for r in st["dock_rects"]:
			if cr.intersects(r):
				hit = true
		for k in ["N", "NE", "E", "SE", "S", "SW", "W", "NW"]:
			if cr.intersects(Rect2(_slot_global(st, k) - Vector2.ONE * float(st["slot"]) * 0.5, Vector2.ONE * float(st["slot"]))):
				hit = true
		_ok(not hit, "the card %s overlaps no dock chip and no slot" % str(cr))
		await _shot("07_ring_dock_with_card")
	if not DccTheme.is_tablet():
		_button(mid, MOUSE_BUTTON_RIGHT, false)
		await _settle(3)
	await _ring_reset()


# -- 8 ------------------------------------------------------------------------------------

func _shared() -> void:
	print("== 8. shared sub-rings ==")
	_ok(DccSettings.set_quick_bar_value("dock", []), "dock emptied")
	var mid: Vector2 = _app.viewport.overlay.get_global_rect().get_center()
	## which diagonals can share, and with how many built-in children, as the live rings report them
	var shareable: Array = []   ## [{domain, dir, builtin}]
	var leaves: Array = []
	for d in QuickBarRing.DOMAINS:
		var built: Dictionary = QuickBarRing.builtin_diagonals(_app, d)
		for dir in QuickBarRing.DIAGONALS:
			if not built.has(dir):
				_ok(false, "%s %s is read from the live ring" % [d, dir])
				continue
			if int(built[dir]["children"]) < 0:
				leaves.append("%s.%s" % [d, dir])
			else:
				shareable.append({"domain": d, "dir": dir, "builtin": int(built[dir]["children"])})
	_ok(shareable.size() == 10 and leaves.size() == 2, "10 shareable diagonals and 2 single-tool ones: %d / %s" % [shareable.size(), str(leaves)])
	_ok(leaves.has("cartography.NW") and leaves.has("civilization.SW"), "the single-tool ones are Cartography Label and Civilization Route")
	## fill each to its maximum (built-in + pins = 9) through the store, then one more
	var distinct: Array = []
	for k in _keys:
		if not String(k).begins_with("extra:") and distinct.size() < 12:
			distinct.append(k)
	var builtin_labels := {}
	for sd in shareable:
		var key := QuickBarRing.share_key(sd["domain"], sd["dir"])
		var room := DccSettings.quick_bar_share_allowance(int(sd["builtin"]))
		builtin_labels[key] = QuickBarRing.builtin_child_labels(_app, sd["domain"], sd["dir"])
		var added := 0
		for n in room:
			if DccSettings.quick_bar_share_add(key, String(distinct[n]), "t", int(sd["builtin"])) == "":
				added += 1
		_ok(added == room and int(sd["builtin"]) + added <= 9, "%s: %d built in + %d pinned = %d" % [key, int(sd["builtin"]), added, int(sd["builtin"]) + added])
		var extra := DccSettings.quick_bar_share_add(key, String(distinct[room]), "t", int(sd["builtin"]))
		_ok(extra != "" and DccSettings.quick_bar_share_pins(key).size() == room, "%s: pin number %d is refused: '%s'" % [key, int(sd["builtin"]) + room + 1, extra])
	## A hand-edited file cannot overfill: write more than the allowance straight into the store's
	## cap (9 for any diagonal), and the ring still places built-in + allowance.
	var over: Dictionary = DccSettings.quick_bar_value("share")
	var wkey := "world.SE"
	var big: Array = []
	for n in 9:
		big.append({"k": String(distinct[n]), "t": "t"})
	over[wkey] = big
	_ok(DccSettings.set_quick_bar_value("share", over), "a 9-pin list for World SE is storable (the store only knows the 9 cap)")
	for sd in shareable:
		var d: String = sd["domain"]
		var slots: Dictionary = _broker().ring_collect({"domain": d, "armed_tool": "", "finalized": false})
		var ks: Array = slots.keys()
		ks.sort()
		_ok(ks == ["E", "N", "NE", "NW", "S", "SE", "SW", "W"], "%s: still exactly 8 compass slots with shares stored" % d)
		var dir: String = sd["dir"]
		var kids: Array = slots[dir]["children"]
		var key2 := QuickBarRing.share_key(d, dir)
		_ok(kids.size() == 9, "%s: the ring places exactly 9 children (%d), the hand-edited overfill was cut" % [key2, kids.size()])
		var first_ok := true
		var bl: Array = builtin_labels[key2]
		for n in bl.size():
			if String(kids[n].get("label", "")) != String(bl[n]) or bool(kids[n].get("shared", false)):
				first_ok = false
		_ok(first_ok, "%s: the %d built-in children are first, in order and untouched" % [key2, bl.size()])
		var last_ok := true
		for n in range(bl.size(), kids.size()):
			if not bool(kids[n].get("shared", false)):
				last_ok = false
		_ok(last_ok, "%s: everything after them is a pinned child" % key2)
	## Measure the drawn sub-rings of every shareable diagonal at 9
	var minima: Array = []
	var shown_once := false
	for sd in shareable:
		var d2: String = sd["domain"]
		_app.select_domain(d2)
		await _settle(8)
		var st := await _open_sticky(mid)
		var dir2: String = sd["dir"]
		_broker().ring_pointer(_local(_slot_global(st, dir2)))
		await _settle(2)
		_broker().ring_click(_local(_slot_global(st, dir2)))
		await _settle(3)
		var st2 := _ring_state()
		var cs: Array = st2["sub_slot_centres"]
		var md := _min_pair_distance(cs)
		minima.append(md)
		_ok(cs.size() == 9 and md >= float(st2["sub_slot_size"]) - 0.01, "%s.%s: 9 slots open and the closest pair is %.1f px apart (slot %.1f)" % [d2, dir2, md, float(st2["sub_slot_size"])])
		if not shown_once and d2 == "cartography":
			shown_once = true
			await _shot("08_shared_subring_9")
		await _ring_reset()
	## positive control: ten children DO fall below the bar, so the check above can fail
	await _ring_reset()
	var ten: Array = []
	for n in 10:
		ten.append({"label": "c%d" % n, "callable": func() -> void: pass})
	var fake := {"N": {"label": "Ten", "children": ten, "enabled": true}}
	_broker()._ensure_ring().arm(mid, fake, true)
	await _settle(2)
	var fs := _ring_state()
	_broker().ring.pointer(_slot_global(fs, "N"))
	_broker().ring.release()
	await _settle(2)
	var fst := _ring_state()
	var tmin := _min_pair_distance(fst["sub_slot_centres"])
	_ok((fst["sub_slot_centres"] as Array).size() == 10 and tmin < float(fst["sub_slot_size"]), "positive control: a ten-child ring packs its closest pair to %.1f px, under the %.1f slot size" % [tmin, float(fst["sub_slot_size"])])
	await _ring_reset()
	## a leaf diagonal takes nothing, even from a hand-edited file
	var hand: Dictionary = DccSettings.quick_bar_value("share")
	hand["cartography.NW"] = [{"k": _vault_key, "t": "Markdown vault"}]
	_ok(DccSettings.set_quick_bar_value("share", hand), "a hand-written pin for CARTO Label is storable")
	var cs2: Dictionary = _broker().ring_collect({"domain": "cartography", "armed_tool": "", "finalized": false})
	_ok(String(cs2["NW"]["label"]) == "Label" and not cs2["NW"].has("children"), "CARTO Label is still a single tool with no sub-ring")
	## a shared child runs through a real click; an unavailable one runs nothing
	_ok(DccSettings.set_quick_bar_value("share", {"world.SW": [{"k": _vault_key, "t": "Markdown vault"}, {"k": _ghost_key, "t": "Gone command"}]}), "World SW: vault + a ghost pinned (3 built in + 2 = 5, under the ceiling)")
	_app.select_domain("world")
	await _settle(8)
	await _hide_windows()
	## Opened right of centre: on a tablet the map is a narrow strip between two docks, and a real click
	## on a ring part that hangs over a dock panel reaches the PANEL, not the map overlay that owns the ring
	## (found by this probe's first tablet run; an existing limit of the ring, not changed here).
	var mid2 := mid + Vector2(150, 0)
	var st3 := await _open_sticky(mid2)
	_broker().ring_pointer(_local(_slot_global(st3, "SW")))
	await _settle(2)
	_broker().ring_click(_local(_slot_global(st3, "SW")))
	await _settle(3)
	var st4 := _ring_state()
	var cs4: Array = st4["sub_slot_centres"]
	_ok(cs4.size() == 5, "World SW now has 3 + 2 children (%d)" % cs4.size())
	## hover the ghost (last) and read the caption
	_broker().ring_pointer(_local(cs4[4]))
	await _settle(3)
	var cap := String(_ring_state()["caption"])
	_ok(cap.contains("Gone command") and cap.contains("Not in this build"), "the unavailable child's caption says why: '%s'" % cap)
	await _shot("09_shared_unavailable")
	await _click_at(cs4[4])
	_ok(not _vault_open() and not bool(_ring_state().get("active", true)), "a real click on it runs nothing and closes the ring (vault open: %s, ring active: %s)" % [str(_vault_open()), str(_ring_state().get("active", "?"))])
	var st5 := await _open_sticky(mid2)
	_broker().ring_pointer(_local(_slot_global(st5, "SW")))
	await _settle(2)
	_broker().ring_click(_local(_slot_global(st5, "SW")))
	await _settle(3)
	var cs5: Array = _ring_state()["sub_slot_centres"]
	_broker().ring_pointer(_local(cs5[3]))
	await _settle(3)
	_ok(String(_ring_state()["caption"]).contains("Markdown vault") and String(_ring_state()["caption"]).contains("(pinned)"), "the pinned child's caption: '%s'" % String(_ring_state()["caption"]))
	await _click_at(cs5[3])
	await _settle(4)
	_ok(_vault_open(), "a real click on the shared child ran its command: the vault window opened")
	await _hide_windows()
	_ok(DccSettings.set_quick_bar_value("share", {}), "share cleared")
	var back: Dictionary = _broker().ring_collect({"domain": "world", "armed_tool": "", "finalized": false})
	_ok((back["SW"]["children"] as Array).size() == 3, "and World SW is back to its 3 built-in children")
	await _ring_reset()


# -- phone --------------------------------------------------------------------------------

func _phone() -> void:
	print("== phone: no cartwheel, so no dock (the Quick bar row exists since CQ-6, its window offers no ring tab) ==")
	var idx := _idx()
	var row := {}
	for r in idx.all():
		if String(r["title"]) == "Quick bar":
			row = r
	_ok(not row.is_empty(), "the phone has the `Quick bar` menu row since CQ-6 (the way to a first pin)")
	_app.open_quick_bar_window()
	await _settle(6)
	_ok(_app.quick_bar_window.visible and not _app.quick_bar_window._tabs.visible, "and its window offers no Ring dock or Ring slots tab: the phone has no cartwheel")
	_app.quick_bar_window.hide()
	_ok(DccSettings.quick_bar_value("dock") == [] and DccSettings.quick_bar_value("share") == {}, "the dock and share stores stay empty")
	_ok(QuickBarRing.dock_items(_app).is_empty(), "no dock items are resolved for an empty store")
	_finish()
