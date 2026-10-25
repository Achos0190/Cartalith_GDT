extends Node
## Probe for the quick cluster in the tool palette bar (`shell/quick_bar.gd`) -- milestone CQ-2
## of `CUSTOM_QUICK_BAR_SCOPE.md` (Ruling DH) -- inside the real shell, with real mouse clicks.
##
##   desktop: godot --path . --resolution 1600x1000 _quickbar_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1280x800  _quickbar_probe.tscn -- --force-touch --shots <dir>
##   phone:   godot --path . --resolution 393x852   _quickbar_probe.tscn -- --force-touch --shots <dir>
##
## **Windowed, never `--headless`** (it generates worlds, resizes the window and reads the
## framebuffer). The arguments read are `--force-touch` and `--shots <dir>`; anything else aborts
## with exit 2 (a usage header that names a flag the body never reads is a recorded mistake).
## **Run it from a scratch worktree with its own `config/name` and a wiped
## `app_userdata/<name>`** (`MISTAKES.md`: a probe that reads `user://` is green for reasons
## unrelated to the code); section 0 fails if the pin store was not empty at launch.
##
## What it protects (CQ-2's bars, each against the real shell):
##   0. SHIPPED DEFAULT: with no pins the cluster is the `＋` chip alone, after a rule, at the
##      palette bar's right end, and the bar's laid height is the palette's;
##   1. ADD: a real click on `＋` opens the Customise window (CQ-3; the interim grouped menu is gone)
##      and the window's add puts a pin on the bar in the same frame (the window itself is
##      `_quickbar_window_probe.gd`'s subject);
##   2. RUN: a real click at each EDGE of the Markdown vault chip opens the vault window, and the
##      Journey planner chip opens `journey_planner_view` -- the same effect as the menu row;
##   3. SAME HANDLER: pressing a chip emits its popup's own `id_pressed` exactly once (counted
##      by a listener on that popup), the signal the menu click emits;
##   4. WIDTH: the shell root's `get_combined_minimum_size().x` is IDENTICAL with 0 and with 24
##      pins at 1280 / 1600 / 1920 px on three generated worlds and all three domains; shown +
##      overflow equals the pin count at every width; the cluster's own minimum is constant;
##   5. OVERFLOW: with 24 pins `⋯` is shown, its real-click menu lists exactly the hidden pins in
##      pin order, and choosing one runs it;
##   6. UNAVAILABLE: a pin whose command is disabled stays, is disabled, with a non-empty reason;
##      a pin whose command is removed from this build shows "not in this build", is NOT deleted
##      (it survives a workspace change and a reload from disk) and can be removed by hand;
##   7. REBUILD: three workspace changes lose no pin and no order;
##   8. TOOLTIP: a pin with an accelerator names it; a chip is as tall as a palette cell and, on a
##      tablet, at least 44 x 44 drawn (measured, both axes, inside the cluster's visible rect);
##   9. CHIP MENU: a real right-click opens Remove / Move left / Move right; each does its job and
##      persists; clearing the last pin persists the empty list;
##  10. PHONE: the phone has no palette bar and so no cluster; nothing is added to its tree;
##  11. COST: the index build behind a refresh, printed with its spread (not a bar).
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument or watchdog.

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _app: Node
var _bridge
var _qb: DccQuickBar
## Counts of `id_pressed` emissions per (popup, id), from listeners this probe connects.
var _press_log: Array = []


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
	var path := _shots.path_join("cq2_%s_%s.png" % [_mode, tag])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


## A REAL mouse press and release at the viewport point `at` (motion first for hover), through
## the input system, not `pressed.emit()`.
func _click_at(at: Vector2, button: int = MOUSE_BUTTON_LEFT) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
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


## A real click on `c` at its horizontal EDGE (`edge` -1 left, +1 right, 0 centre), vertically
## centred: a centre press cannot see a target's width (`MISTAKES.md`).
func _click(c: Control, edge: int = 0, button: int = MOUSE_BUTTON_LEFT) -> void:
	var r := c.get_global_rect()
	var x := r.get_center().x + float(edge) * (r.size.x * 0.5 - 3.0)
	await _click_at(Vector2(x, r.get_center().y), button)


func _idx() -> CommandIndex:
	var idx := CommandIndex.new()
	idx.build(_app, _bridge)
	return idx


func _set_pins(keys: Array) -> void:
	var idx := _idx()
	var pins: Array = []
	for k in keys:
		var row := idx.find_pin(String(k))
		pins.append({"k": String(k), "t": String(row.get("title", "ghost " + String(k)))})
	_ok(DccSettings.set_quick_bar_value("bar", pins), "stored %d pins" % pins.size())
	_qb.refresh()
	await _settle(3)


func _stored_keys() -> Array:
	var out: Array = []
	for p in DccSettings.quick_bar_value("bar"):
		out.append(String(p["k"]))
	return out


func _chip(key: String) -> Button:
	for c in _qb.chips():
		if String(c.get_meta("pin_key", "")) == key:
			return c
	return null


func _hide_windows() -> void:
	if _app.vault_window:
		_app.vault_window.hide()
	## The planner is a tool takeover: it closes by disarming, never by hiding the view node
	## (hiding the node itself made a later open() paint panels under a hidden parent).
	if _app.journey_planner_view and _app.journey_planner_view.is_visible_in_tree():
		_app.arm_tool("inspect")


## Up to `n` pinnable keys, in index order, for the width sweep (never pressed).
func _pinnable_keys(n: int) -> Array:
	var out: Array = []
	for r in _idx().all():
		if bool(r["pinnable"]):
			out.append(String(r["pin_key"]))
			if out.size() == n:
				break
	return out


## The shell's minimum width: the root `VBoxContainer`'s combined minimum, the node that
## folds in every bar and dock. (`_app` itself is a plain Control and reports 0 -- measured --
## which would make every "identical minimum" assertion vacuous, so `_min_width_live()` is
## also checked against a positive control.)
func _min_width() -> float:
	for c in _app.get_children():
		if c is VBoxContainer:
			return (c as Control).get_combined_minimum_size().x
	return -1.0


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

	print("== 0. shipped default ==")
	_ok(DccSettings.quick_bar_value("bar") == [] and not DccSettings.has_quick_bar_value("bar"), "the pin store is empty at launch (the user dir was wiped)")
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_mode = "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	var vp := get_viewport().get_visible_rect().size
	print("== mode %s, viewport %s ==" % [_mode, vp])
	_bridge = _app.bridge

	if _mode == "phone":
		await _phone()
		return

	_qb = _app._quick_bar
	_ok(_qb != null and is_instance_valid(_qb) and _qb.is_inside_tree(), "the cluster exists in the shell")
	var bar: Control = _app.tool_palette_bar
	_ok(bar != null and bar.is_ancestor_of(_qb), "and sits inside the tool palette bar")
	_ok(_qb.pin_count() == 0 and _qb.shown_count() == 0, "no pins, none shown")
	_ok(not _qb.more_chip().visible, "no overflow chip with no pins")
	_ok(_qb.plus_chip().visible and not _qb.plus_chip().disabled, "the ＋ chip is there and live")
	var strip: Control = _app._tool_strip_row
	_ok(_qb.plus_chip().get_global_rect().position.x > strip.get_global_rect().end.x, "the cluster is to the RIGHT of the palette cells (plus at x=%.0f, strip ends %.0f)" % [_qb.plus_chip().get_global_rect().position.x, strip.get_global_rect().end.x])
	var rule: Control = _app._quick_rule
	_ok(rule.get_global_rect().end.x <= _qb.get_global_rect().position.x + 0.5 and rule.get_global_rect().position.x >= strip.get_global_rect().end.x - 0.5, "a rule separates them")
	var cell: Button = null
	for b in strip.find_children("*", "Button", true, false):
		if (b as Button).toggle_mode:
			cell = b
			break
	_ok(cell != null, "a palette cell exists to measure against")
	_ok(is_equal_approx(_qb.plus_chip().size.y, cell.size.y), "chip height %.0f equals the palette cell's %.0f" % [_qb.plus_chip().size.y, cell.size.y])
	var floor_px := float(DccQuickBar.cell_px())
	_ok(floor_px == (44.0 if _mode == "tablet" else 30.0), "the floor for this density is %.0f (44 tablet, 30 desktop)" % floor_px)
	_ok(_qb.plus_chip().size.x >= floor_px - 0.01 and _qb.plus_chip().size.y >= floor_px - 0.01, "the ＋ chip is %.0f x %.0f, at least %.0f on both axes" % [_qb.plus_chip().size.x, _qb.plus_chip().size.y, floor_px])
	var min_empty := _qb.get_combined_minimum_size().x
	await _shot("00_empty")

	print("== 1. add a pin through the real ＋ chip (CQ-3: it opens the Customise window) ==")
	await _click(_qb.plus_chip(), 0)
	var qw: QuickBarWindow = _app.quick_bar_window
	_ok(qw.visible, "a real click on ＋ opens the Customise window (CQ-2's interim grouped menu is gone)")
	await _shot("01_plus_window")
	## The window is `_quickbar_window_probe.gd`'s subject; here it is only the way in. Adding by
	## its own `add_row`, the call its Add button and Enter both reach.
	var idx1 := _idx()
	_ok(qw.add_row(idx1.find_pin("menu:%d" % DccMenus.ID_VAULT)) == "", "the window adds Markdown vault")
	_ok(_stored_keys() == ["menu:%d" % DccMenus.ID_VAULT], "the pin is stored: %s" % str(_stored_keys()))
	_ok(_qb.pin_count() == 1, "and the cluster has its chip at once (the window's change reaches the cluster in the same call)")
	await _settle(3)
	_ok(_qb.shown_count() == 1, "and it is laid out and drawn")
	_ok(qw.add_row(idx1.find_pin("menu:%d" % DccMenus.ID_STORY_JOURNEY_PLANNER)) == "", "the window adds Journey planner")
	_ok(_stored_keys() == ["menu:%d" % DccMenus.ID_VAULT, "menu:%d" % DccMenus.ID_STORY_JOURNEY_PLANNER], "second pin appended in order: %s" % str(_stored_keys()))
	qw.hide()
	await _settle(3)
	await _shot("02_two_pins")

	print("== 2/3. run through real clicks; same handler as the menu ==")
	var vkey := "menu:%d" % DccMenus.ID_VAULT
	var jkey := "menu:%d" % DccMenus.ID_STORY_JOURNEY_PLANNER
	var vchip := _chip(vkey)
	_ok(vchip != null and vchip.text == "Markdown vault", "the vault chip reads 'Markdown vault' (%s)" % (vchip.text if vchip else "?"))
	var vrow := _idx().find_pin(vkey)
	var vpop: PopupMenu = vrow["_popup"]
	var presses := [0]
	var counter := func(id: int): if id == DccMenus.ID_VAULT: presses[0] += 1
	vpop.id_pressed.connect(counter)
	for edge in [-1, 1]:
		_hide_windows()
		await _settle(3)
		await _click(vchip, edge)
		await _settle(6)
		_ok(_app.vault_window != null and _app.vault_window.visible, "a real click at the %s edge of the vault chip opens the vault window" % ("left" if edge < 0 else "right"))
	_ok(presses[0] == 2, "the vault popup's own id_pressed fired once per click (%d of 2)" % presses[0])
	vpop.id_pressed.disconnect(counter)
	_hide_windows()
	await _settle(3)
	var jchip := _chip(jkey)
	await _click(jchip, 0)
	await _settle(14)
	_ok(_app.journey_planner_view != null and _app.journey_planner_view.is_visible_in_tree(), "the Journey planner chip opens journey_planner_view")
	await _shot("03_journey_open")
	_hide_windows()
	await _settle(3)
	_ok(_qb.run_pin("menu:99999") == "Not in this build.", "running a pin the build lacks answers a reason, not silence")

	print("== 8a. tooltip and accelerator ==")
	var acc_row := {}
	for r in _idx().all():
		if bool(r["pinnable"]) and r.has("_popup"):
			var ix: int = (r["_popup"] as PopupMenu).get_item_index(int(r["_id"]))
			if ix >= 0 and (r["_popup"] as PopupMenu).get_item_accelerator(ix) != 0:
				acc_row = r
				break
	_ok(not acc_row.is_empty(), "some pinnable command carries an accelerator (%s)" % String(acc_row.get("title", "none")))
	if not acc_row.is_empty():
		await _set_pins([String(acc_row["pin_key"])])
		var ac := _chip(String(acc_row["pin_key"]))
		var pp: PopupMenu = acc_row["_popup"]
		var accel := OS.get_keycode_string(pp.get_item_accelerator(pp.get_item_index(int(acc_row["_id"]))))
		_ok(accel != "" and ac.tooltip_text.contains(accel) and ac.tooltip_text.begins_with(String(acc_row["title"])), "tooltip '%s' names the title and the accelerator '%s'" % [ac.tooltip_text, accel])

	print("== 6. unavailable, removed ==")
	var plain := {}
	for r in _idx().all():
		if bool(r["pinnable"]) and not bool(r["refreshes_on_open"]) and r.has("_popup") and String(r["pin_key"]) != vkey:
			plain = r
			break
	_ok(not plain.is_empty(), "a plain pinnable command exists to disable (%s)" % String(plain.get("title", "none")))
	var pkey := String(plain["pin_key"])
	await _set_pins([pkey])
	_ok(not _chip(pkey).disabled, "control: the chip is live while its menu row is enabled")
	var ppop: PopupMenu = plain["_popup"]
	var pix := ppop.get_item_index(int(plain["_id"]))
	var old_tip := ppop.get_item_tooltip(pix)
	ppop.set_item_disabled(pix, true)
	ppop.set_item_tooltip(pix, "PROBE: no world")
	_qb.refresh()
	await _settle(3)
	_ok(_chip(pkey) != null and _chip(pkey).disabled, "a pin on a disabled command is VISIBLE and DISABLED")
	_ok(_chip(pkey).tooltip_text.contains("PROBE: no world"), "its tooltip carries the menu's reason: '%s'" % _chip(pkey).tooltip_text)
	_ok(_qb.run_pin(pkey) != "", "and running it answers a reason")
	ppop.set_item_disabled(pix, false)
	ppop.set_item_tooltip(pix, old_tip)
	await _set_pins([pkey, "menu:99999"])
	var ghost := _chip("menu:99999")
	_ok(ghost != null and ghost.disabled and ghost.tooltip_text.begins_with("Not in this build"), "a removed command renders as 'Not in this build': '%s'" % (ghost.tooltip_text if ghost else "?"))
	_ok(_stored_keys() == [pkey, "menu:99999"], "and is NOT deleted from the store")
	DccSettings._loaded = false
	DccSettings._cfg = null
	_qb.refresh()
	await _settle(2)
	_ok(_stored_keys() == [pkey, "menu:99999"] and _chip("menu:99999") != null, "it survives a reload from disk")

	print("== 7. workspace changes lose no pin ==")
	var before := _stored_keys()
	for d in ["civilization", "cartography", "world"]:
		_app._select_domain(d)
		await _settle(4)
		_ok(_stored_keys() == before and _qb.pin_count() == before.size(), "after switching to %s: %d pins, same order" % [d, _qb.pin_count()])
	var order: Array = []
	for c in _qb.chips():
		order.append(String(c.get_meta("pin_key")))
	_ok(order == before, "the chips are in pin order")

	print("== 9. the chip's right-click menu ==")
	var keys3 := ["menu:%d" % DccMenus.ID_VAULT, "menu:%d" % DccMenus.ID_STORY_JOURNEY_PLANNER, pkey]
	await _set_pins(keys3)
	await _click(_chip(keys3[0]), 0, MOUSE_BUTTON_RIGHT)
	var cm: PopupMenu = _qb._chip_menu
	_ok(cm.visible and cm.item_count == 5 and cm.get_item_text(4) == "Customise…" and cm.is_item_separator(3), "a real right-click opens Remove / Move left / Move right, a rule and Customise… (%d rows)" % cm.item_count)
	_ok(cm.is_item_disabled(1) and not cm.is_item_disabled(2), "the first pin cannot move left, can move right")
	await _shot("04_chip_menu")
	cm.id_pressed.emit(2)
	await _settle(3)
	cm.hide()
	_ok(_stored_keys() == [keys3[1], keys3[0], keys3[2]], "Move right swaps it with its neighbour (literal order): %s" % str(_stored_keys()))
	DccSettings._loaded = false
	DccSettings._cfg = null
	_ok(_stored_keys() == [keys3[1], keys3[0], keys3[2]], "and the order persisted")
	await _click(_chip(keys3[2]), 0, MOUSE_BUTTON_RIGHT)
	_ok(_qb._chip_menu.is_item_disabled(2) and not _qb._chip_menu.is_item_disabled(1), "the last pin cannot move right")
	_qb._chip_menu.id_pressed.emit(0)
	await _settle(3)
	_qb._chip_menu.hide()
	_ok(_stored_keys() == [keys3[1], keys3[0]], "Remove drops it")
	for k in [keys3[1], keys3[0]]:
		DccSettings.quick_bar_remove("bar", String(k))
	_qb.refresh()
	await _settle(2)
	DccSettings._loaded = false
	DccSettings._cfg = null
	_ok(DccSettings.has_quick_bar_value("bar") and _stored_keys() == [], "clearing the last pin persists the EMPTY list")

	print("== 4/5/8b. width: identical minimum width at 0 and 24 pins, overflow ==")
	## Positive control: the measure must MOVE when something wide enters the palette row,
	## or "identical at 0 and 24 pins" proves nothing.
	var base_min := _min_width()
	_ok(base_min > 400.0, "the shell's minimum width reads a real value: %.1f" % base_min)
	var wide := Control.new()
	wide.custom_minimum_size = Vector2(3000, 10)
	_app._tool_palette_row.add_child(wide)
	await _settle(3)
	_ok(_min_width() > base_min + 2000.0, "control: a 3000 px control in the palette row raises it to %.1f" % _min_width())
	wide.queue_free()
	await _settle(3)
	_ok(is_equal_approx(_min_width(), base_min), "and it returns to %.1f when removed" % _min_width())
	var pins24 := _pinnable_keys(24)
	_ok(pins24.size() == 24, "24 distinct pinnable commands exist: %d" % pins24.size())
	var widths := [1280, 1600, 1920]
	var seeds := [1011, 2022, 3033]
	var mins_empty := {}
	var mins_full := {}
	var overflow_seen := false
	for si in seeds.size():
		_bridge.generate({"seed": seeds[si], "width_km": 900.0, "grid_w": 192, "grid_h": 128, "archetype": "", "villages": true})
		while _bridge.generating:
			await get_tree().create_timer(0.25).timeout
		await _settle(10)
		if _app.open_project_dialog:
			_app.open_project_dialog.hide()
		for dom in (["world", "civilization", "cartography"] if si == 0 else ["world"]):
			_app._select_domain(dom)
			await _settle(4)
			for w in widths:
				get_window().size = Vector2i(w, int(vp.y))
				await _settle(6)
				await _set_pins([])
				var m0 := _min_width()
				var qmin0 := _qb.get_combined_minimum_size().x
				var bar0 := (_app.tool_palette_bar as Control).get_combined_minimum_size().x
				await _set_pins(pins24)
				var m24 := _min_width()
				var qmin24 := _qb.get_combined_minimum_size().x
				var bar24 := (_app.tool_palette_bar as Control).get_combined_minimum_size().x
				var shown := _qb.shown_count()
				var hidden := _qb.overflow_titles().size()
				var key := "seed %d %s @%d" % [seeds[si], dom, w]
				_ok(is_equal_approx(m0, m24), "%s: shell min width %.1f with 0 pins == %.1f with 24" % [key, m0, m24])
				_ok(is_equal_approx(bar0, bar24), "%s: the palette bar's own minimum %.1f == %.1f" % [key, bar0, bar24])
				_ok(is_equal_approx(qmin0, qmin24) and is_equal_approx(qmin0, min_empty), "%s: the cluster's own minimum is constant (%.1f)" % [key, qmin24])
				_ok(shown + hidden == 24 and shown >= 1, "%s: shown %d + overflow %d == 24 pins" % [key, shown, hidden])
				if hidden > 0:
					overflow_seen = true
					_ok(_qb.more_chip().visible, "%s: the ⋯ chip is shown while pins overflow" % key)
				mins_empty[key] = m0
				mins_full[key] = m24
				# every shown chip is wholly inside the cluster's rect (drawn, not clipped)
				var inside := true
				var small := 0
				for c in _qb.chips():
					if c.visible:
						if not _qb.get_global_rect().grow(0.6).encloses(c.get_global_rect()):
							inside = false
						if c.size.x < floor_px - 0.01 or c.size.y < floor_px - 0.01:
							small += 1
				_ok(inside, "%s: every shown chip is inside the cluster" % key)
				_ok(small == 0, "%s: every shown chip is at least %.0f x %.0f (%d smaller)" % [key, floor_px, floor_px, small])
		if si == 0:
			await _shot("05_24pins_%d" % int(get_window().size.x))
	_ok(overflow_seen, "at least one width overflowed (the ⋯ path was exercised)")
	print("  info min width (0 pins -> 24 pins): ", mins_empty.values().min(), "..", mins_empty.values().max(), " -> ", mins_full.values().min(), "..", mins_full.values().max())

	print("== 5. the ⋯ chip ==")
	get_window().size = Vector2i(1280, int(vp.y))
	_app._select_domain("world")
	await _settle(6)
	await _set_pins(pins24)
	var hidden_titles := _qb.overflow_titles()
	_ok(hidden_titles.size() > 0 and _qb.more_chip().visible, "24 pins at 1280 px overflow (%d hidden)" % hidden_titles.size())
	await _click(_qb.more_chip(), 0)
	var mm: PopupMenu = _qb._more_menu
	_ok(mm.visible, "a real click on ⋯ opens its menu")
	var listed: Array = []
	for r in mm.item_count:
		listed.append(mm.get_item_text(r))
	_ok(listed.size() == hidden_titles.size() + 2 and listed.slice(0, hidden_titles.size()) == Array(hidden_titles) and listed[-1] == "Customise…", "the menu lists exactly the hidden pins in pin order, then a rule and Customise… (%d rows)" % listed.size())
	await _shot("06_overflow_menu")
	mm.hide()
	# pin the vault LAST so it is in the overflow, and run it from there
	var rest := pins24.duplicate()
	rest.erase(vkey)
	rest = rest.slice(0, 23)
	rest.append(vkey)
	await _set_pins(rest)
	_ok(not _chip(vkey).visible, "the vault pin (last of 24) is in the overflow")
	_hide_windows()
	await _click(_qb.more_chip(), 0)
	var vi := -1
	for r in _qb._more_menu.item_count:
		if _qb._more_menu.get_item_text(r) == "Markdown vault":
			vi = r
	_ok(vi >= 0, "the overflow menu offers Markdown vault")
	_qb._more_menu.id_pressed.emit(_qb._more_menu.get_item_id(vi))
	await _settle(6)
	_qb._more_menu.hide()
	_ok(_app.vault_window.visible, "choosing it from the overflow runs it")
	_hide_windows()
	get_window().size = Vector2i(int(vp.x), int(vp.y))
	await _settle(4)

	print("== 11. cost of a refresh (the index build) ==")
	var ms: Array = []
	for _n in 7:
		_qb.refresh()
		ms.append(_qb.last_index_ms)
	ms.sort()
	print("  info index build: median %.1f ms (min %.1f .. max %.1f), 7 runs, %d rows" % [ms[3], ms[0], ms[6], _idx().size()])

	await _set_pins([])
	_ok(DccSettings.has_quick_bar_value("bar") and _stored_keys() == [], "leaves the store empty (stored, not 'never configured')")
	_finish()


func _phone() -> void:
	print("== 10. phone: no palette bar, no cluster ==")
	_ok(_app.tool_palette_bar == null, "the phone has no tool palette bar")
	_ok(_app._quick_bar == null, "and so no quick cluster")
	var found := _app.find_children("*", "Container", true, false).filter(func(n): return n is DccQuickBar)
	_ok(found.is_empty(), "no DccQuickBar anywhere in the phone's tree")
	var vp := get_viewport().get_visible_rect().size
	var worst := _widest_visible_minimum(_app)
	_ok(float(worst[0]) > 100.0, "the walk reads real minimums (widest visible node: %s at %.1f)" % [worst[1], float(worst[0])])
	_ok(float(worst[0]) <= vp.x + 0.5, "no visible node's combined minimum width (%.1f, %s) exceeds the %.0f px viewport" % [float(worst[0]), worst[1], vp.x])
	await _shot("10_phone")
	_finish()


## The widest `get_combined_minimum_size().x` among visible Controls under `n` (internal children
## included, `get_children(true)`), with its path: the phone has no single root container to ask.
func _widest_visible_minimum(n: Node) -> Array:
	var best := [0.0, ""]
	if n is Control and (n as Control).is_visible_in_tree():
		best = [(n as Control).get_combined_minimum_size().x, str(n.get_path())]
	for c in n.get_children(true):
		var r := _widest_visible_minimum(c)
		if float(r[0]) > float(best[0]):
			best = r
	return best
