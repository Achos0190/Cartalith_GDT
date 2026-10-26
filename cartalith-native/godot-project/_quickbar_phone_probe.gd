extends Node
## Probe for the PHONE form of the custom quick bar -- milestone CQ-6 of `CUSTOM_QUICK_BAR_SCOPE.md`
## (Ruling DH Q9), inside the real shell: the pinned-chip row in the MAP sheet, the `Quick` section in MORE,
## the `All...` list sheet, the Customise window's phone layout, the search-result star and the Assistant chip
## on the phone.
##
##   portrait: godot --path . --resolution 393x852   _quickbar_phone_probe.tscn -- --force-touch --shots <dir>
##             godot --path . --resolution 720x1600  _quickbar_phone_probe.tscn -- --force-touch
##             godot --path . --resolution 1080x2340 _quickbar_phone_probe.tscn -- --force-touch
##   landscape: godot --path . --resolution 852x393  _quickbar_phone_probe.tscn -- --force-touch
##
## **Windowed, never `--headless`** (it reads the framebuffer's layout and clicks through the input system).
## The arguments read are `--force-touch` and `--shots <dir>`; anything else aborts with exit 2. **Run it from a
## scratch worktree with its own `config/name` and a wiped `app_userdata/<name>`**: section 1 fails if the pin
## store was not empty or the assistant was on at launch. It refuses to run off the phone form.
##
## ## What is real here and what is not (stated, not implied)
##
## Clicks are real MOUSE events through `Input.parse_input_event` at the control's laid-out centre, which the
## engine's `emulate_mouse_from_touch` turns the other way round on a handset. **They are not touch events**:
## a finger's contact patch, a drag that scrolls the chip row, the on-screen keyboard and the OS gesture
## insets cannot be made here (a OnePlus 6T is the device, and it is not connected). Sizes are the laid-out
## rectangles, converted to dp by `phone_scale()` for the main viewport (the Customise window lays out in dp
## already), and every size check has a positive control that must fail on a shrunk copy.
##
## ## What it protects
##   1. THE EMPTY BAR: nothing is mounted, MORE has no Quick section, Preferences has the `Quick bar...` row;
##   2. FIRST PIN, BY HAND: MORE > Preferences > Quick bar... opens the Customise window in its phone layout
##      (no ring tabs, a Commands / Pinned strip, every control at least 44 dp on both axes, list rows tall
##      enough to touch), and three adds by real clicks, a reorder, a removal and a reload all store the
##      literal expected order;
##   3. THE MAP-SHEET ROW: hidden at the 66 dp peek and on other tabs, shown from half up; the peek, half cap
##      and full heights keep the budget; chips are 48 dp (floor 44) on both axes; a press runs the menu row's
##      own handler (vault window, planner view); a pin whose command is gone is dim and says why;
##   4. 24 PINS: the row scrolls, can never widen the shell, and `All...` lists all 24 plus Customise;
##   5. MORE > QUICK: first band, one 52 dp row per pin, a tap closes MORE and runs it, Customise opens the window;
##   6. THE ASSISTANT: hidden / greyed with the gate's reason / live, in the row and in MORE, the chip appears
##      within a frame of the write, and a live tap opens the panel;
##   7. THE STAR: a pinnable command in the `.` search shows a star; tapping pins and unpins through the store;
##   8. EMPTY AGAIN: the row is unmounted and the sheet is exactly as tall as before any pin.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument or watchdog.

const FAKE_PORT := 59871
const SEED := 483920
const FLOOR_DP := 44.0
const CHIP_DP := 48.0

var _fails := 0
var _checks := 0
var _shots := ""
var _app: Node
var _bridge
var _landscape := false
var _tag := "p"


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _wait(seconds: float) -> void:
	await get_tree().create_timer(seconds).timeout
	await _settle(3)


func _shot(name_: String) -> void:
	if _shots == "":
		return
	await _settle(3)
	var path := _shots.path_join("cq6_%s_%s.png" % [_tag, name_])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


## dp from a physical-px length in the MAIN viewport.
func _dp(px: float) -> float:
	return px / _app.phone_scale()


## A point in `c`'s own window as a point in the main viewport. A control inside an embedded dialog reports
## its rect in that dialog's LOGICAL units, so the dialog's content scale and position are applied.
func _to_root(p: Vector2, c: Control) -> Vector2:
	var w := c.get_window()
	if w == null or w == get_tree().root:
		return p
	return w.get_final_transform() * p + Vector2(w.position)


func _click_at(at: Vector2) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 2)
	Input.parse_input_event(mm)
	await _settle(2)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(3)


## Scrolls `c` into view in every `ScrollContainer` above it (a finger would), then clicks its centre.
func _reveal(c: Control) -> void:
	var p: Node = c.get_parent()
	while p != null:
		if p is ScrollContainer:
			(p as ScrollContainer).ensure_control_visible(c)
		p = p.get_parent()
	await _settle(3)


func _click(c: Control) -> void:
	await _reveal(c)
	await _click_at(_to_root(c.get_global_rect().get_center(), c))


func _click_item(list: ItemList, i: int) -> void:
	await _reveal(list)
	list.ensure_current_is_visible()
	var r := list.get_item_rect(i, true)
	await _click_at(_to_root(list.get_global_position() + r.get_center(), list))


func _stored_keys() -> Array:
	var out: Array = []
	for p in DccSettings.quick_bar_value("bar"):
		out.append(String(p["k"]))
	return out


func _idx() -> CommandIndex:
	var idx := CommandIndex.new()
	idx.build(_app, _bridge)
	return idx


func _reload_from_disk() -> void:
	DccSettings._cfg = null
	DccSettings._loaded = false


func _labels_text(root: Node) -> Array:
	var out: Array = []
	for n in root.find_children("*", "Label", true, false):
		out.append((n as Label).text)
	return out


## The widest `get_combined_minimum_size().x` among visible Controls under `n` (internal children included), with
## its path. The phone has no single root container to ask (`_app` itself is a plain Control and reads 0), so the
## whole tree is walked, as `_quickbar_probe.gd` does.
func _widest_visible_minimum(n: Node) -> Array:
	var best := [0.0, ""]
	if n is Control and (n as Control).is_visible_in_tree():
		best = [(n as Control).get_combined_minimum_size().x, str(n.get_path())]
	## A scroller that scrolls sideways does not pass its content's width up to its parent: the content is
	## what scrolls. (One that does NOT scroll sideways is descended into, which is the folding trap.)
	if n is ScrollContainer and (n as ScrollContainer).horizontal_scroll_mode != ScrollContainer.SCROLL_MODE_DISABLED:
		return best
	for c in n.get_children(true):
		var r := _widest_visible_minimum(c)
		if float(r[0]) > float(best[0]):
			best = r
	return best


func _sheet_h() -> float:
	return _app._phone_tool_sheet.size.y


func _show(tab: String, det: String) -> void:
	if _app._phone_menu != null and _app._phone_menu.is_open():
		_app._phone_menu.close()
	_app._pick_phone_tab(tab)
	_app.set_phone_detent(det)
	await _wait(0.6)


func _reset_llm() -> void:
	DccSettings.set_llm_value("enabled", false)
	for k in ["preset_id", "base_url", "model"]:
		DccSettings.set_llm_value(k, "")
	DccSettings.set_llm_value("local_allowed", false)
	DccSettings.llm_withdraw_consent()
	DccSettings.set_quick_bar_value("assistant_button", true)
	await _settle(2)


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
	get_tree().create_timer(1200.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	print("== 1. the empty bar ==")
	_ok(DccSettings.quick_bar_value("bar") == [] and not DccSettings.has_quick_bar_value("bar"), "the pin store is empty at launch (the user dir was wiped)")
	_ok(not bool(DccSettings.llm_value("enabled")), "the assistant is off at launch")
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	if not _app.is_phone():
		print("ABORT: this probe is for the phone form (--resolution 393x852 -- --force-touch)")
		get_tree().quit(2)
		return
	_landscape = _app._landscape
	_tag = "land" if _landscape else "%dx%d" % [int(get_viewport().get_visible_rect().size.x), int(get_viewport().get_visible_rect().size.y)]
	print("== phone %s, viewport %s, phone_scale %.4f ==" % ["landscape" if _landscape else "portrait", get_viewport().get_visible_rect().size, _app.phone_scale()])
	_bridge = _app.bridge
	_bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(8)

	await _empty()
	await _first_pin()
	await _map_row()
	await _many_pins()
	await _more_quick()
	await _assistant()
	await _star()
	await _empty_again()
	await _second_boot()
	_finish()


# -- 1 ------------------------------------------------------------------------------------

var _width_two_pins := 0.0
var _ref_peek := 0.0
var _ref_half := 0.0
var _ref_full := 0.0

func _empty() -> void:
	_ok(_app.phone_quick != null, "the app owns a phone quick-bar controller")
	_ok(_app.phone_quick.row() == null and _app.phone_quick_row() == null, "nothing is mounted with an empty bar and the assistant off")
	_ok(QuickBarPhone.entries(_app).is_empty(), "QuickBarPhone.entries() is empty")
	## reference heights, taken on the empty bar, for the budget bars below and for 'empty again' (section 8)
	await _show("map", "peek")
	_ref_peek = _sheet_h()
	await _show("map", "half")
	_ref_half = _sheet_h()
	await _show("map", "full")
	_ref_full = _sheet_h()
	print("  info  empty-bar sheet heights px: peek %.1f half %.1f full %.1f  (%.1f / %.1f / %.1f dp)" % [_ref_peek, _ref_half, _ref_full, _dp(_ref_peek), _dp(_ref_half), _dp(_ref_full)])
	if not _landscape:
		_ok(absf(_dp(_ref_peek) - 66.0) <= 1.0, "the empty peek is 66 dp (%.2f)" % _dp(_ref_peek))
	await _show("more", "peek")
	await _wait(0.4)
	var pm = _app._phone_menu
	_ok(pm.is_open(), "MORE opens")
	var rows: Array = pm.find_children("QuickRow_*", "", true, false)
	_ok(rows.is_empty(), "MORE has no Quick row with an empty bar")
	_ok(not _labels_text(pm).has("QUICK"), "MORE has no QUICK band with an empty bar")
	pm._push_screen("prefs")
	await _settle(6)
	var texts := _labels_text(pm)
	var has_row := false
	for t in texts:
		if String(t).begins_with("Quick bar"):
			has_row = true
	_ok(has_row, "Preferences lists the 'Quick bar…' row on the phone (the way to a first pin)")
	pm.close()
	await _settle(2)


# -- 2 ------------------------------------------------------------------------------------

## Every visible BaseButton, LineEdit and TabBar tab of `root` against the 44 dp floor, in `root`'s own units.
func _floors(root: Node, label: String, scale: float) -> void:
	var small := 0
	var seen := 0
	var worst := ""
	for n in root.find_children("*", "Control", true, false):
		var c := n as Control
		if not c.is_visible_in_tree() or c.size.x <= 0 or c.size.y <= 0:
			continue
		if not (c is BaseButton or c is LineEdit):
			continue
		if c is Button and (c as Button).get_parent() is Window:
			continue
		seen += 1
		var w := c.size.x / scale
		var h := c.size.y / scale
		if w < FLOOR_DP - 0.6 or h < FLOOR_DP - 0.6:
			small += 1
			worst = "%s %.1fx%.1f dp" % [c.name, w, h]
	_ok(seen > 0 and small == 0, "%s: %d tappable controls, every one at least 44 dp on both axes%s" % [label, seen, "" if small == 0 else " (under the floor: %d, e.g. %s)" % [small, worst]])


func _list_hit_height(list: ItemList, i: int) -> float:
	## Rows are hit-tested over the row plus its separation; sample the item's x centre over y and count the
	## contiguous pixels that name item `i` (an edge press, not a centre press: MISTAKES.md "Write a probe that PRESSES").
	list.ensure_current_is_visible()
	var r := list.get_item_rect(i, true)
	var x := r.position.x + r.size.x * 0.5
	var n := 0
	var y := maxf(r.position.y - 40.0, 0.0)
	while y < r.end.y + 40.0:
		if list.get_item_at_position(Vector2(x, y), false) == i:
			n += 1
		y += 0.5
	return float(n) * 0.5


func _first_pin() -> void:
	print("== 2. the first pin, by hand ==")
	var idx := _idx()
	var vault := idx.find_pin("menu:45")
	var planner := idx.find_pin("menu:726")
	_ok(not vault.is_empty() and bool(vault["pinnable"]) and not planner.is_empty() and bool(planner["pinnable"]), "Markdown vault (menu:45) and Journey planner (menu:726) are pinnable on the phone")
	## reach the window the way a user with nothing pinned does: MORE > Preferences > Quick bar…
	await _show("more", "peek")
	var pm = _app._phone_menu
	pm._push_screen("prefs")
	await _settle(6)
	var target: Control = null
	for n in pm.find_children("*", "Label", true, false):
		if (n as Label).text.begins_with("Quick bar") and (n as Label).is_visible_in_tree():
			var p: Node = n
			while p != null and not (p is PanelContainer):
				p = p.get_parent()
			target = p as Control
			break
	_ok(target != null, "the Preferences screen draws a tappable 'Quick bar…' row")
	if target != null:
		## The Preferences list is longer than the screen; bring the row into view like a finger would.
		pm._screen_scroll.ensure_control_visible(target)
		await _settle(4)
		print("  info  Quick bar row rect %s (screen %s)" % [str(target.get_global_rect()), str(get_viewport().get_visible_rect().size)])
		await _click(target)
		await _wait(0.6)
	var win: QuickBarWindow = _app.quick_bar_window
	_ok(win.visible, "tapping the row opens the Customise window")
	var sv: Vector2 = get_viewport().get_visible_rect().size
	_ok(win.size.x >= int(sv.x) - 1, "and it fills the screen's width (%d of %d px)" % [win.size.x, int(sv.x)])
	await _settle(4)
	_ok(not win._tabs.visible and not win._tab_note.visible, "the ring tabs (Ring dock, Ring slots) are not offered: the phone has no cartwheel")
	_ok(win._pages != null and win._pages.tab_count == 2 and win._pages.get_tab_title(0) == "Commands" and win._pages.get_tab_title(1) == "Pinned", "a Commands / Pinned strip replaces them")
	_ok(win._left.visible and not win._right.visible, "the Commands page shows first")
	var f := win.content_scale_factor
	_ok(absf(f - _app.phone_scale()) < 0.01, "the window lays out in dp (content scale %.3f = phone scale %.3f)" % [f, _app.phone_scale()])
	await _shot("02_customise_commands")
	## overflow walk: nothing in the window may be wider than the window
	var wl := Vector2(win.size) / f
	var wide := 0
	var wname := ""
	for n in win.find_children("*", "Control", true, false):
		var c := n as Control
		if c.is_visible_in_tree() and c.get_combined_minimum_size().x > wl.x + 0.5:
			wide += 1
			wname = "%s %.0f" % [c.name, c.get_combined_minimum_size().x]
	_ok(wide == 0, "no control in the window has a minimum wider than the window (%.0f dp)%s" % [wl.x, "" if wide == 0 else " (e.g. %s)" % wname])
	_floors(win, "Customise, Commands page", 1.0)
	var hit := _list_hit_height(win._catalogue, 0) if win._catalogue.item_count > 0 else 0.0
	_ok(hit >= FLOOR_DP - 0.6, "a catalogue row is hit-tested over %.1f dp (floor 44)" % hit)
	## positive control for the size walk: a shrunk copy must be caught
	var probe_btn := Button.new()
	probe_btn.custom_minimum_size = Vector2(20, 20)
	win._left.add_child(probe_btn)
	await _settle(2)
	var small_before := 0
	for n in win._left.find_children("*", "BaseButton", true, false):
		if (n as Control).size.y / f < FLOOR_DP - 0.6:
			small_before += 1
	_ok(small_before >= 1, "positive control: a 20 dp button added to the page IS flagged by the walk")
	win._left.remove_child(probe_btn)
	probe_btn.queue_free()
	## three adds by real clicks
	var picks := ["menu:45", "menu:726"]
	var extra := ""
	for r in idx.all():
		var k := String(r.get("pin_key", ""))
		if bool(r.get("pinnable", false)) and String(r.get("group", "")) != "" and not picks.has(k) and extra == "":
			extra = k
	picks.append(extra)
	for k in picks:
		var row := idx.find_pin(k)
		win._search.text = String(row["title"])
		win._search.text_changed.emit(win._search.text)
		await _settle(3)
		var at := -1
		for j in win._cat_rows.size():
			if String((win._cat_rows[j] as Dictionary).get("pin_key", "")) == k:
				at = j
		_ok(at >= 0, "the catalogue lists '%s' for the search '%s'" % [k, String(row["title"])])
		if at >= 0:
			await _click_item(win._catalogue, at)
			await _click(win._add_btn)
	_ok(_stored_keys() == ["menu:45", "menu:726", extra], "three real adds store [vault, planner, %s] in that order" % extra)
	## Pinned page, via a real tap on the strip's second segment
	await _reveal(win._pages)
	var seg := win._pages.get_tab_rect(1)
	await _click_at(_to_root(win._pages.get_global_position() + seg.get_center(), win._pages))
	_ok(win._right.visible and not win._left.visible, "a tap on 'Pinned' shows the pinned list")
	_ok(win._pins_list.item_count == 3, "it lists the 3 pins")
	_floors(win, "Customise, Pinned page", 1.0)
	var hit2 := _list_hit_height(win._pins_list, 0)
	_ok(hit2 >= FLOOR_DP - 0.6, "a pinned row is hit-tested over %.1f dp (floor 44)" % hit2)
	await _shot("02_customise_pinned")
	await _click_item(win._pins_list, 0)
	await _click(win._down_btn)
	_ok(_stored_keys() == ["menu:726", "menu:45", extra], "Down on the first pin stores [planner, vault, %s]" % extra)
	await _click_item(win._pins_list, 2)
	await _click(win._remove_btn)
	_ok(_stored_keys() == ["menu:726", "menu:45"], "Remove on the third stores [planner, vault]")
	_reload_from_disk()
	_ok(_stored_keys() == ["menu:726", "menu:45"], "and the order survives a reload from disk")
	## the Close button must be reachable and above the gesture inset
	var ok_btn := win.get_ok_button()
	## The window stops above `H_PHONE_GESTURE` dp, the figure `DccWidgets.phone_present` uses for every phone window.
	var gesture := roundf(DccTheme.H_PHONE_GESTURE * _app.phone_scale())
	var ok_bottom := _to_root(ok_btn.get_global_rect().end, ok_btn).y
	_ok(ok_bottom <= sv.y - gesture + 1.0, "the Close button ends at y=%.0f, above the %.0f px gesture inset (screen %.0f)" % [ok_bottom, gesture, sv.y])
	await _click(ok_btn)
	await _wait(0.4)
	_ok(not win.visible, "Close closes the window")


# -- 3 ------------------------------------------------------------------------------------

func _chip_dp_ok(row: QuickBarPhoneRow, label: String) -> void:
	var worst_w := 1.0e9
	var worst_h := 1.0e9
	for b in row.chips():
		worst_w = minf(worst_w, _dp(b.size.x))
		worst_h = minf(worst_h, _dp(b.size.y))
	_ok(worst_w >= FLOOR_DP - 0.6 and worst_h >= FLOOR_DP - 0.6, "%s: every chip is at least 44 dp on both axes (min %.1f x %.1f dp)" % [label, worst_w, worst_h])
	var tall := _dp(row.chips()[0].size.y)
	_ok(tall >= CHIP_DP - 0.6, "%s: chips are 48 dp tall (%.1f)" % [label, tall])


func _map_row() -> void:
	print("== 3. the MAP-sheet chip row ==")
	var ctl: QuickBarPhone = _app.phone_quick
	_ok(ctl.row() != null and _app.phone_quick_row() == ctl.row(), "the pins mounted the row (it was created when the first pin was stored)")
	await _show("map", "peek")
	var row: QuickBarPhoneRow = ctl.row()
	if not _landscape:
		_ok(not row.visible, "hidden at the peek detent")
		_ok(absf(_sheet_h() - _ref_peek) < 0.5, "the peek is exactly as tall as with no pins (%.1f vs %.1f px)" % [_sheet_h(), _ref_peek])
	await _show("map", "half")
	_ok(row.visible and row.is_visible_in_tree(), "shown at the half detent on the MAP tab")
	var sheet: Control = _app._phone_tool_sheet
	var rr := row.get_global_rect()
	var sr := sheet.get_global_rect()
	_ok(sr.encloses(rr), "the row lies inside the sheet")
	_ok(row.get_parent() == _app._phone_sheet_col and row.get_index() == 1, "it is mounted directly under the header, above the tool options")
	var fh: float = get_viewport().get_visible_rect().size.y - _app._phone_nav_reserve()
	var cap := roundf(fh * 0.46)
	if not _landscape:
		_ok(_sheet_h() <= cap + 1.0, "half stays within the 0.46 budget (%.1f of %.1f px)" % [_sheet_h(), cap])
		_ok(_sheet_h() >= _ref_half - 0.5, "half grew by the row's height or stayed (%.1f vs the empty %.1f px)" % [_sheet_h(), _ref_half])
		print("  info  half with the row: %.1f px (%.1f dp), without %.1f px, cap %.1f px, row %.1f px" % [_sheet_h(), _dp(_sheet_h()), _ref_half, cap, row.size.y])
		## When the empty half had room under the cap, the row's height must have been added to it (the content
		## height the half detent holds counts the row); when it was already at the cap there is nothing to add.
		if cap - _ref_half >= row.size.y + 2.0:
			_ok(_sheet_h() >= _ref_half + row.size.y - 1.5, "half grew by the row's height (%.1f -> %.1f px, row %.1f)" % [_ref_half, _sheet_h(), row.size.y])
		else:
			print("  note  the empty half (%.1f px) is already within a row of the cap (%.1f px): the growth bar has nothing to measure on this viewport" % [_ref_half, cap])
	var titles: Array = []
	for b in row.chips():
		titles.append(b.text)
	_ok(titles.size() == 3 and titles[2] == "All…", "the row holds 2 pins and 'All…' last (got %s)" % str(titles))
	_ok(titles.size() == 3 and titles[0] == "Journey planner" and titles[1] == "Markdown vault", "in pin order: Journey planner, Markdown vault")
	_chip_dp_ok(row, "MAP row")
	_floors(row, "MAP row (tappables)", _app.phone_scale())
	var vp: Vector2 = get_viewport().get_visible_rect().size
	var worst := _widest_visible_minimum(_app)
	_ok(float(worst[0]) > 100.0, "the walk reads real minimums (widest visible node: %s at %.1f)" % [worst[1], float(worst[0])])
	_ok(float(worst[0]) <= vp.x + 0.5, "no visible node's minimum width (%.1f, %s) exceeds the %.0f px screen with the row shown" % [float(worst[0]), worst[1], vp.x])
	_width_two_pins = float(worst[0])
	_ok(row.get_combined_minimum_size().x <= 1.0, "the row's own minimum width is %.0f px: it takes its width from the sheet, not from its chips" % row.get_combined_minimum_size().x)
	await _shot("03_map_row_half")
	## a real press runs the menu row's own handler
	await _click(row.chips()[1])
	await _wait(0.5)
	_ok(_app.vault_window.visible, "a real click on the Markdown vault chip opens the vault window (same handler as the menu)")
	_app.vault_window.hide()
	await _show("map", "half")
	await _click(row.chips()[0])
	await _wait(0.5)
	_ok(_app.journey_planner_view != null and _app.journey_planner_view.visible, "a real click on the Journey planner chip opens the planner view")
	_app.journey_planner_view.hide()
	await _show("map", "half")
	## a pin whose command is gone: dim, kept, says why
	_ok(DccSettings.quick_bar_add("bar", "menu:99999", "Gone command") == "", "a pin to a command that does not exist can be stored")
	ctl.refresh()
	await _settle(3)
	row = ctl.row()
	var ghost: Button = null
	for b in row.chips():
		if b.text == "Gone command":
			ghost = b
	_ok(ghost != null and not bool(ghost.get_meta("live")), "its chip is kept and marked not live")
	_ok(ghost != null and String(ghost.get_meta("reason")).begins_with("Not in this build"), "with the reason 'Not in this build...' (%s)" % (String(ghost.get_meta("reason")) if ghost != null else ""))
	if ghost != null:
		var toasts_before := _count_toasts()
		await _click(ghost)
		await _settle(4)
		_ok(_count_toasts() == toasts_before + 1, "tapping it toasts instead of staying silent (%d -> %d toasts)" % [toasts_before, _count_toasts()])
		_ok(_stored_keys().has("menu:99999"), "and the pin is still stored")
	DccSettings.quick_bar_remove("bar", "menu:99999")
	ctl.refresh()
	await _settle(3)
	row = ctl.row()
	## tab switches lose no pin; the row follows the tab
	for t in ["gen", "map", "more", "map", "gen", "map"]:
		await _show(t, "half")
	_ok(_stored_keys() == ["menu:726", "menu:45"], "after cycling the tabs the store still holds both pins")
	_ok(ctl.row() != null and ctl.row().chips().size() == 3, "and the row still has 3 chips")
	await _show("gen", "half")
	_ok(not ctl.row().visible, "the row is hidden on the GENERATE tab")
	await _show("map", "full")
	_ok(ctl.row().visible, "shown at full")
	if not _landscape:
		_ok(absf(_sheet_h() - _ref_full) < 1.5, "full is as tall as with no pins (%.1f vs %.1f px)" % [_sheet_h(), _ref_full])
	await _show("map", "peek")
	if not _landscape:
		_ok(not ctl.row().visible and absf(_sheet_h() - _ref_peek) < 0.5, "back at peek the row is hidden and the sheet is %.1f px again" % _sheet_h())
	var tabs := PackedStringArray()
	for t in _app.PHONE_TABS:
		tabs.append(String(t["caption"]))
	_ok(tabs == PackedStringArray(["MAP", "GENERATE", "PLAN", "MORE"]), "PHONE_TABS is unchanged: %s" % str(tabs))


func _count_toasts() -> int:
	var n := 0
	for c in _app._phone_root.get_children():
		if c is PanelContainer and (c as Control).z_index == 50:
			n += 1
	return n


# -- 4 ------------------------------------------------------------------------------------

func _many_pins() -> void:
	print("== 4. 24 pins ==")
	var idx := _idx()
	var keys: Array = ["menu:726", "menu:45"]
	for r in idx.all():
		var k := String(r.get("pin_key", ""))
		if keys.size() < 24 and bool(r.get("pinnable", false)) and String(r.get("group", "")) != "" and not keys.has(k):
			keys.append(k)
	_ok(keys.size() == 24, "24 distinct pinnable commands found")
	var min_before: float = _app.phone_quick.row().get_combined_minimum_size().x
	for k in keys:
		if not _stored_keys().has(k):
			var why := DccSettings.quick_bar_add("bar", k, String(idx.find_pin(k)["title"]))
			if why != "":
				print("  note  add refused: %s" % why)
	_ok(_stored_keys().size() == 24, "24 pins are stored")
	_ok(DccSettings.quick_bar_add("bar", "menu:99998", "One too many") != "", "a 25th is refused by the store")
	_app.phone_quick.refresh()
	await _show("map", "half")
	var row: QuickBarPhoneRow = _app.phone_quick.row()
	_ok(row.chips().size() == 25, "the row holds 24 pins and 'All…' (%d chips)" % row.chips().size())
	var vp: Vector2 = get_viewport().get_visible_rect().size
	var worst24 := _widest_visible_minimum(_app)
	_ok(float(worst24[0]) <= vp.x + 0.5, "no visible node's minimum width (%.1f, %s) exceeds the screen with 24 pins (%.0f px)" % [float(worst24[0]), worst24[1], vp.x])
	_ok(absf(float(worst24[0]) - _width_two_pins) < 0.5, "and the widest minimum is the same with 24 pins as with 2 (%.1f vs %.1f px): the pin count never reaches the layout" % [float(worst24[0]), _width_two_pins])
	_ok(row.get_combined_minimum_size().x <= maxf(min_before, 1.0) + 0.5, "the row's own minimum width does not grow with the pin count (%.0f -> %.0f px)" % [min_before, row.get_combined_minimum_size().x])
	var hs := row.get_h_scroll_bar()
	_ok(hs.max_value > hs.page + 1.0, "the row scrolls: content %.0f px against %.0f px visible" % [hs.max_value, hs.page])
	_ok(row.get_global_rect().end.x <= vp.x + 0.5, "the row itself ends inside the screen (%.0f px)" % row.get_global_rect().end.x)
	_ok(_dp(row.chips()[0].size.y) >= CHIP_DP - 0.6 and _dp(row.chips()[24].size.y) >= CHIP_DP - 0.6, "chips keep 48 dp at 24 pins")
	var sheet_h_24 := _sheet_h()
	_ok(sheet_h_24 <= roundf((vp.y - _app._phone_nav_reserve()) * 0.46) + 1.0 or _landscape, "half is still within the 0.46 budget with 24 pins (%.1f px)" % sheet_h_24)
	row.scroll_horizontal = int(hs.max_value)
	await _settle(4)
	var all_chip := row.all_chip
	var visible_rect := row.get_global_rect()
	_ok(visible_rect.encloses(all_chip.get_global_rect().grow(-1.0)), "scrolled to the end, 'All…' is fully inside the row's visible area")
	await _shot("04_row_24")
	await _click(all_chip)
	await _wait(0.6)
	var lst: QuickBarPhoneList = _app.phone_quick.list_sheet()
	_ok(lst != null and lst.visible, "a real click on 'All…' opens the list sheet")
	if lst != null:
		_ok(lst.rows.size() == 25, "it lists all 24 pins and the Customise row (%d rows)" % lst.rows.size())
		var small := 0
		for b in lst.rows:
			if b.size.y < FLOOR_DP - 0.6:
				small += 1
		_ok(small == 0, "every list row is at least 44 dp tall (row 0: %.1f dp)" % lst.rows[0].size.y)
		var sv: Vector2 = get_viewport().get_visible_rect().size
		_ok(lst.size.x >= int(sv.x) - 1, "the sheet fills the screen's width")
		await _shot("04_all_sheet")
		lst.hide()
	# restore to two pins for the next sections
	for k in _stored_keys().duplicate():
		if k != "menu:726" and k != "menu:45":
			DccSettings.quick_bar_remove("bar", k)
	_app.phone_quick.refresh()
	await _settle(3)
	_ok(_stored_keys() == ["menu:726", "menu:45"], "back to 2 pins")


# -- 5 ------------------------------------------------------------------------------------

func _more_quick() -> void:
	print("== 5. MORE > Quick ==")
	await _show("more", "half")
	var pm = _app._phone_menu
	var body: VBoxContainer = pm._screen_body
	var first_labels := _labels_text(body)
	var qi: int = first_labels.find("QUICK")
	_ok(qi == 0, "the QUICK band is the first thing in MORE's root (index %d of %d labels)" % [qi, first_labels.size()])
	var rows: Array = pm.find_children("QuickRow_*", "", true, false)
	_ok(rows.size() == 2, "one row per pin (%d)" % rows.size())
	var tall := 1.0e9
	for r in rows:
		tall = minf(tall, _dp((r as Control).size.y))
	_ok(tall >= 46.0, "every Quick row is at least 46 dp tall (min %.1f dp)" % tall)
	var cust: Control = null
	for n in pm.find_children("*", "Label", true, false):
		if (n as Label).text == "Customise…":
			var p: Node = n
			while p != null and not (p is PanelContainer):
				p = p.get_parent()
			cust = p as Control
	_ok(cust != null, "a Customise… row follows them")
	await _shot("05_more_quick")
	if rows.size() == 2:
		_ok(String((rows[0] as Control).name) == "QuickRow_menu_726" and String((rows[1] as Control).name) == "QuickRow_menu_45", "rows are in pin order")
		await _click(rows[1] as Control)
		await _wait(0.5)
		_ok(not pm.is_open(), "a tap on a Quick row closes MORE")
		_ok(_app.vault_window.visible, "and runs the command (the vault window is open)")
		_app.vault_window.hide()
	await _show("more", "half")
	cust = null
	for n in pm.find_children("*", "Label", true, false):
		if (n as Label).text == "Customise…":
			var p2: Node = n
			while p2 != null and not (p2 is PanelContainer):
				p2 = p2.get_parent()
			cust = p2 as Control
	if cust != null:
		await _click(cust)
		await _wait(0.5)
		_ok(_app.quick_bar_window.visible and not pm.is_open(), "Customise… closes MORE and opens the Customise window")
		_app.quick_bar_window.hide()


# -- 6 ------------------------------------------------------------------------------------

func _gate() -> Dictionary:
	return _bridge.llm_authorize(DccSettings.llm_gate_dict(), "", DccSettings.llm_consented_origins())


func _assistant() -> void:
	print("== 6. the Assistant chip on the phone ==")
	await _reset_llm()
	await _show("map", "half")
	var ctl: QuickBarPhone = _app.phone_quick
	_ok(ctl.row().chips().size() == 3 and ctl.row().chips()[0].text != AssistantChip.LABEL, "hidden: no Assistant chip")
	DccSettings.set_llm_value("enabled", true)
	DccSettings.set_llm_value("preset_id", "ollama")
	DccSettings.set_llm_value("base_url", "http://127.0.0.1:%d/v1" % FAKE_PORT)
	DccSettings.set_llm_value("model", "mock-model")
	_ok(ctl.row().entries.size() == 3 and String(ctl.row().entries[0]["kind"]) == "assistant", "the row follows the write inside the same call (3 entries, the Assistant first)")
	await get_tree().process_frame
	var row: QuickBarPhoneRow = ctl.row()
	_ok(row.chips()[0].text == AssistantChip.LABEL and row.chips()[0].is_visible_in_tree(), "one process frame later the Assistant chip is drawn first")
	var gate := _gate()
	_ok(not bool(gate.get("ok", true)) and String(gate.get("reason", "")) != "", "the real gate refuses, with a reason: '%s'" % String(gate.get("reason", "")).left(70))
	_ok(not bool(row.chips()[0].get_meta("live")) and String(row.chips()[0].get_meta("reason")) == "Assistant unavailable: " + String(gate["reason"]), "greyed, and its reason is the gate's own sentence")
	_chip_dp_ok(row, "row with the Assistant")
	await _shot("06_assistant_greyed")
	var before := _count_toasts()
	await _click(row.chips()[0])
	await _settle(4)
	## Owner decision 2026-10-26 (phone pass, OnePlus 6T): the phone's greyed chip is a way in, not a dead button. It still
	## toasts the reason, and now also opens the Assistant settings (`QuickBarPhone.press`); it never opens the panel.
	_ok(_count_toasts() == before + 1 and not _app.assistant_window.visible and _app.assistant_settings_window.visible, "tapping the greyed chip toasts the reason and opens the Assistant settings (not the panel)")
	_app.assistant_settings_window.hide()
	await _settle(4)
	## MORE shows it first, dim, with the reason on its second line
	await _show("more", "half")
	var pm = _app._phone_menu
	var arow: Array = pm.find_children("QuickRow_assistant", "", true, false)
	_ok(arow.size() == 1, "MORE > Quick has the Assistant row")
	if arow.size() == 1:
		_ok(_labels_text(arow[0]).any(func(t): return String(t).contains(String(gate["reason"]).left(30))), "its second line carries the gate's reason")
	pm.close()
	DccSettings.set_llm_value("local_allowed", true)
	DccSettings.set_llm_value("prose", true)
	DccSettings.llm_add_consent("http://127.0.0.1:%d" % FAKE_PORT)
	await get_tree().process_frame
	_ok(bool(_gate().get("ok", false)), "after consent the real gate allows")
	await _show("map", "half")
	row = ctl.row()
	_ok(bool(row.chips()[0].get_meta("live")), "the chip is live")
	await _click(row.chips()[0])
	await _wait(0.6)
	_ok(_app.assistant_window.visible, "a real click on the live chip opens the assistant panel")
	_app.assistant_window.hide()
	_ok(DccSettings.quick_bar_value("bar").size() == 2, "the chip is not a pin: still 2 stored pins")
	DccSettings.set_quick_bar_value("assistant_button", false)
	await get_tree().process_frame
	_ok(ctl.row().chips().size() == 3 and ctl.row().chips()[0].text != AssistantChip.LABEL, "switched off: gone within a frame although the gate allows")
	await _reset_llm()
	await _settle(2)


# -- 7 ------------------------------------------------------------------------------------

func _star() -> void:
	print("== 7. the search-result star ==")
	await _show("map", "peek")
	_app._set_search_open(true)
	await _settle(4)
	_app._set_search_query(". vault")
	await _settle(6)
	var res: VBoxContainer = _app._phone_search_results
	var stars: Array = res.find_children("PinStar", "", true, false)
	_ok(stars.size() >= 1, "a '.' search shows %d star(s) beside pinnable commands" % stars.size())
	if stars.is_empty():
		_app._set_search_open(false)
		return
	var st: Button = null
	for s in stars:
		var line: Node = (s as Node).get_parent()
		for n in line.find_children("*", "Button", true, false):
			if (n as Button).text.contains("Markdown vault") or (n as Button).accessibility_description.contains("Markdown vault"):
				st = s as Button
	if st == null:
		st = stars[0] as Button
	_ok(_dp(st.size.x) >= CHIP_DP - 0.6 and _dp(st.size.y) >= CHIP_DP - 0.6, "a star is 48 dp square (%.1f x %.1f dp)" % [_dp(st.size.x), _dp(st.size.y)])
	var vp: Vector2 = get_viewport().get_visible_rect().size
	_ok(st.get_global_rect().end.x >= vp.x - float(_app.phone_px(30)) and st.get_global_rect().end.x <= vp.x + 0.5, "the star sits at the row's right edge (ends at x=%.0f of %.0f px)" % [st.get_global_rect().end.x, vp.x])
	var keys_before := _stored_keys()
	await _click(st)
	var after := _stored_keys()
	_ok(after.size() == keys_before.size() + 1 or after.size() == keys_before.size() - 1, "a tap toggles one pin through the store (%d -> %d)" % [keys_before.size(), after.size()])
	await _shot("07_search_star")
	await _click(st)
	_ok(_stored_keys() == keys_before, "a second tap restores the store exactly")
	_app._set_search_open(false)
	await _settle(3)


# -- 8 ------------------------------------------------------------------------------------

func _empty_again() -> void:
	print("== 8. empty again ==")
	## The sheet is left at HALF on the MAP tab while the last pins go, so the row's height has to leave the
	## sheet by the unmount itself (`refresh_phone_quick` re-snaps); changing the detent would hide a miss.
	DccSettings.quick_bar_add("bar", "menu:726", "Journey planner")
	DccSettings.quick_bar_add("bar", "menu:45", "Markdown vault")
	_app.phone_quick.refresh()
	await _show("map", "half")
	var with_row := _sheet_h()
	DccSettings.quick_bar_remove("bar", "menu:726")
	DccSettings.quick_bar_remove("bar", "menu:45")
	_app.phone_quick.refresh()
	await _settle(3)
	_ok(absf(_sheet_h() - _ref_half) < 0.5 and (with_row > _ref_half + 1.0 or _landscape), "removing the last pin at half re-snaps the sheet at once (%.1f -> %.1f px; empty reference %.1f)" % [with_row, _sheet_h(), _ref_half])
	_ok(DccSettings.has_quick_bar_value("bar") and _stored_keys().is_empty(), "the empty list is stored (clearing the last pin still writes)")
	_ok(_app.phone_quick.row() == null and _app.phone_quick_row() == null, "the row is unmounted")
	await _show("map", "peek")
	_ok(absf(_sheet_h() - _ref_peek) < 0.5, "peek is %.1f px = the empty reference" % _sheet_h())
	await _show("map", "half")
	_ok(absf(_sheet_h() - _ref_half) < 0.5, "half is %.1f px = the empty reference %.1f (identity by control flow)" % [_sheet_h(), _ref_half])
	await _show("map", "full")
	_ok(absf(_sheet_h() - _ref_full) < 1.5, "full is %.1f px = the empty reference %.1f" % [_sheet_h(), _ref_full])
	await _show("more", "peek")
	_ok(_app._phone_menu.find_children("QuickRow_*", "", true, false).is_empty() and not _labels_text(_app._phone_menu).has("QUICK"), "MORE has no Quick section again")
	_app._phone_menu.close()


# -- 9 ------------------------------------------------------------------------------------

## A bar stored on disk must show on the next launch, not only after a pin is added in the same session.
func _second_boot() -> void:
	print("== 9. a second launch with pins stored ==")
	DccSettings.quick_bar_add("bar", "menu:726", "Journey planner")
	DccSettings.quick_bar_add("bar", "menu:45", "Markdown vault")
	_app.queue_free()
	await _settle(6)
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(8)
	_bridge = _app.bridge
	var ctl: QuickBarPhone = _app.phone_quick
	_ok(ctl.row() != null and _app.phone_quick_row() == ctl.row(), "the row is mounted a few frames after launch, from the stored pins alone")
	_ok(ctl.row() != null and ctl.row().get_parent() == _app._phone_sheet_col, "inside the MAP sheet's column")
	await _show("map", "half")
	_ok(ctl.row() != null and ctl.row().visible and ctl.row().chips().size() == 3, "and it shows at half with 2 pins and 'All…'")
	_ok(ctl.row() != null and ctl.row().chips()[0].text == "Journey planner", "in the stored order")
	await _shot("09_second_boot")
	for k in _stored_keys().duplicate():
		DccSettings.quick_bar_remove("bar", k)
	ctl.refresh()
	await _settle(3)
	_ok(ctl.row() == null, "removing them unmounts it again")
