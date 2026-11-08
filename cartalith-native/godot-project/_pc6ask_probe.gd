extends Node
## PC-6 probe: the assistant docked as the Context panel's Ask tab (`PC_SHELL_PROPOSAL.md` §3.5,
## milestone PC-6), inside the real shell, with real mouse clicks and real key events through
## `Input.parse_input_event()`.
##
##   desktop: godot --path . --resolution 1600x1000 _pc6ask_probe.tscn -- --shots <dir>
##   tablet:  godot --path . --resolution 1600x1000 _pc6ask_probe.tscn -- --force-touch --shots <dir>
##   phone:   godot --path . --resolution 393x852   _pc6ask_probe.tscn -- --force-touch --phone --shots <dir>
##   live:    add --live [--model <id>] to the desktop line: one real request to LM Studio
##            (127.0.0.1:1234; qwen3-14b unless --model names another, as LM Studio has it loaded)
##
## **Windowed, never `--headless`**, from a project copy with its own `config/name` and a wiped
## `app_userdata/<name>` (`MISTAKES.md`). Reads `--force-touch`, `--phone`, `--live`,
## `--model <id>` and `--shots <dir>`; anything else exits 2.
##
## **No network without `--live`.** Every leg but L runs with no socket: OFF has the assistant's
## master switch off (the default of a wiped user dir); ON turns it on, pointed at LM Studio with the
## origin consented, and never presses Send; `LlmWire.connect_attempts` is asserted 0 after both. The
## request a Send WOULD make is built by `LlmStream.prepare()`, which builds bytes and opens nothing,
## and its body is read for the context. No mock server is started.
##
## What it protects (PC-6's bar: "Opening, closing or reselecting never empties the transcript. The
## context at Send includes the year. A citation selects."; the brief's: every entry point opens the
## Ask tab on desktop and tablet, the phone keeps its sheet; the conversation, typed draft, cards and
## their Apply state survive closing, reopening and switching tabs; the context names the selection
## and the year). A small world is generated (seed 4242, 256 x 192, as `_pc4context_probe.gd`). Then:
##   A. TAB: the header's tab strip holds INSPECT and ASK; Ctrl+J (a real key) shows Ask with the
##      cursor in its input, Inspect's scroll hidden, the conversation docked (the dialog never shown);
##      Ctrl+1 goes back to Inspect, Ctrl+3 and a real click on ASK return.
##   B. OFF: with the master switch off, Send reads "Turn on Assistant enabled" with its reason; no
##      socket was attempted.
##   C. PERSIST: a draft typed with real keys, a transcript message (View facts, a real click), a view
##      card applied by a real click on its Apply and a second card left waiting survive: Ctrl+1/Ctrl+3,
##      hiding the panel (Window ▸ Context panel) and reopening it with Ctrl+J, a real map click on
##      another place, and Story ▸ Ask the assistant about the selection. New (a real click) empties it.
##   D. CONTEXT: with "Send: selection and year" on, the chips name the selection and the year; a real
##      map click and a year move change them while the Ask tab stays; the request arguments carry
##      `shown_selection` and `shown_year`; the body `prepare()` builds holds the `shown` block with
##      that year; a chip click leaves its half out; the source off sends neither.
##   E. CITATION: a verified world citation in a sources card is a link; a real click selects it.
##   F. WINDOW (desktop): Open in a window (real click) moves the same conversation into the dialog and
##      the tab says so; Bring it back here (real click) returns it.
##   G. READING: on the Ask tab, a Measure reading (two real map clicks) switches the panel to Inspect.
##   H. ON: master switch on (no Send): the quick bar's Assistant chip (a real click) and the ring dock's
##      item open the Ask tab; no socket was attempted.
##   L. LIVE (`--live`): Commands, "what year is it and what is selected?" typed with real keys and Send
##      clicked, with a place selected; the reply is printed and must name the year and the place.
##   P. PHONE (`--phone`): launch and selection-sheet captures for pixel comparison with the pre-PC-6
##      tree; Ask opens the assistant's sheet (not docked, no chips) and its transcript survives a reopen.
## Exit 0 pass, 1 fail, 2 bad argument, watchdog or no world, 3 a leg could not run.

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _phone := false
var _live := false
## The LM Studio model the live leg asks (`--model`); qwen3-14b unless told otherwise. The live leg
## uses whatever LM Studio has loaded and never loads, unloads or rebinds anything itself.
var _model := "qwen3-14b"
var _app: Node
var _bridge
var _win
## Set as the last line of each leg: a leg that stopped on a script error never sets it, and the
## RESULT line then counts that as a failure instead of passing on the checks run before the error.
var _leg_done := false


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
		var path := _shots.path_join("pc6_%s_%s.png" % [_mode, tag])
		img.save_png(path)
		print("SHOT %s" % path)


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


## Types `text` into whatever has focus, one real key event per character.
func _type(text: String) -> void:
	for ch in text:
		var ev := InputEventKey.new()
		ev.unicode = ch.unicode_at(0)
		ev.keycode = KEY_NONE
		ev.pressed = true
		Input.parse_input_event(ev)
		var up := ev.duplicate()
		up.pressed = false
		Input.parse_input_event(up)
		await get_tree().process_frame
	await _settle(4)


## Clicks `c`'s centre for real, after checking it is drawn inside the window and visible.
func _press(c: Control, what: String) -> bool:
	if c == null or not is_instance_valid(c):
		_ok(false, "%s exists to be pressed" % what)
		return false
	for sc in [_app._right_dock_scroll, _dock().get("_ask_scroll"), _win.get("_scroll")]:
		if sc != null and (sc as ScrollContainer).is_ancestor_of(c) and (sc as Control).is_visible_in_tree():
			(sc as ScrollContainer).ensure_control_visible(c)
			await _settle(4)
	await _settle(2)
	for sc in [_app._right_dock_scroll, _dock().get("_ask_scroll"), _win.get("_scroll")]:
		if sc != null and (sc as ScrollContainer).is_ancestor_of(c) and (sc as Control).is_visible_in_tree():
			(sc as ScrollContainer).ensure_control_visible(c)
			await _settle(4)
	var at := c.get_global_rect().get_center()
	var vis := Rect2(Vector2.ZERO, get_viewport().get_visible_rect().size)
	var inside := vis.has_point(at) and c.is_visible_in_tree()
	_ok(inside, "%s is drawn on screen (centre %s)" % [what, str(at)])
	if not inside:
		return false
	await _click_at(at)
	await _settle(4)
	return true


## Diagnostic: the widest rows under the conversation, so a row that widens the dock can be named.
func _widths(tag: String) -> void:
	var host := _win.get("_host") as Control
	var frame := _dock().get("_ask_frame") as Control
	print("  [%s] frame %.0f, host min %.0f" % [tag, frame.size.x if frame != null else -1.0, host.get_combined_minimum_size().x])
	var n := 0
	for ch in host.find_children("*", "Container", true, false):
		var mx := (ch as Control).get_combined_minimum_size().x
		if mx > 250.0 and not (ch is ScrollContainer) and n < 12:
			var kids := PackedStringArray()
			for g in ch.get_children():
				if g is Control:
					kids.append("%s%s:%.0f" % [g.get_class(), "(" + (g as Button).text.left(24) + ")" if g is Button else ("(" + (g as Label).text.left(24) + ")" if g is Label else ""), (g as Control).get_combined_minimum_size().x])
			print("    %s min %.0f [%s]" % [ch.get_class(), mx, ", ".join(kids)])
			n += 1


func _dock():
	return _app.right_dock_ctrl


func _c(name: String) -> Control:
	return _win.controls.get(name) as Control


## The named header control in the Ask frame (or the Inspect body when `inspect`).
func _hdr(node_name: String, inspect := false) -> Control:
	var root: Node = _app.right_dock_body if inspect else _dock().get("_ask_head")
	return null if root == null else root.find_child(node_name, true, false) as Control


func _screen_of(cell: Vector2) -> Vector2:
	var ov = _app.viewport.overlay
	return (ov as Control).get_global_transform() * ov._cell_to_screen(cell, ov._displayed_rect())


## Named settlements whose marker is inside the map view, as indices.
func _visible_settlements() -> Array:
	## Inside the map's own host rect, which ends at both docks: the overlay runs beneath them, and a
	## click there lands in a panel (measured: a pick at x 350 hit the left dock).
	var r: Rect2 = (_app.viewport as Control).get_global_rect().grow(-40.0)
	var all: Array = _bridge.settlements()
	var out: Array = []
	for i in all.size():
		var s: Dictionary = all[i]
		if String(s.get("name", "")).strip_edges() == "":
			continue
		if r.has_point(_screen_of(Vector2(float(s.get("x", 0)), float(s.get("y", 0))))):
			out.append(i)
	## Largest first: a village's marker is not drawn at every zoom, and a click on an undrawn one lands
	## on whatever is under it (measured: a river, which the panel then shows).
	out.sort_custom(func(p, q): return float(all[p].get("population", 0)) > float(all[q].get("population", 0)))
	return out


func _click_settlement(i: int) -> void:
	var s: Dictionary = _bridge.settlements()[i]
	await _click_at(_screen_of(Vector2(float(s.get("x", 0)), float(s.get("y", 0)))))
	await _settle(4)


func _name_of(i: int) -> String:
	return String((_bridge.settlements()[i] as Dictionary).get("name", ""))


## The transcript's entries (message boxes and cards), as nodes.
func _transcript() -> Array:
	var msgs: Node = _win.get("_msgs")
	return [] if msgs == null else msgs.get_children()


## The popup holding `title`'s rows: the bar's menu of that name, else ☰ ▸ `title` (the narrow touch
## tablet's bar holds four menus and puts the rest under ☰, `_pc4context_probe.gd::_window_popup`).
func _menu(title: String) -> PopupMenu:
	var burger: PopupMenu = null
	for ch in _app.menu_bar_row.get_children():
		if ch is MenuButton and (ch as MenuButton).text == title:
			return (ch as MenuButton).get_popup()
		if ch is MenuButton and (ch as MenuButton).text == "☰":
			burger = (ch as MenuButton).get_popup()
	if burger != null:
		for i in burger.item_count:
			if not burger.is_item_separator(i) and burger.get_item_text(i) == title and burger.get_item_submenu(i) != "":
				return burger.get_node_or_null(NodePath(burger.get_item_submenu(i))) as PopupMenu
	return null


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var k := 0
	while k < args.size():
		var a := args[k]
		if a == "--force-touch":
			pass
		elif a == "--phone":
			_phone = true
		elif a == "--live":
			_live = true
		elif a == "--model" and k + 1 < args.size():
			k += 1
			_model = args[k]
		elif a == "--shots" and k + 1 < args.size():
			k += 1
			_shots = args[k]
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
		k += 1
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(2)
		return
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_mode = "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("== mode %s ==" % _mode)
	if _phone and _mode != "phone":
		print("[FATAL] --phone asked for and the shell is not the phone (%s)" % _mode)
		get_tree().quit(3)
		return
	if _phone:
		await _shot("launch")
	_bridge = _app.bridge
	_win = _app.assistant_window
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await _settle(2)
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _settle(10)
	await _settle(30)
	if not bool(_bridge.has_world) or _bridge.settlements().size() < 3:
		print("[FATAL] no world with three settlements")
		get_tree().quit(2)
		return
	if _phone:
		await _leg_phone()
	else:
		await _leg_desktop()
	_ok(_leg_done, "the leg ran to its end")
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _leg_phone() -> void:
	print("== P. phone ==")
	_app.select("settlement", 0)
	await _settle(10)
	await _shot("sheet")
	_ok(_app.context_tabs().has("ask"), "the phone lists Ask (its route is the assistant's sheet)")
	_ok(_app.open_context_tab("ask"), "open_context_tab(\"ask\") answers true on the phone")
	await _settle(10)
	_ok(_win.visible and not _win.is_docked(), "on the phone Ask opens the assistant's sheet, not a docked panel")
	_ok(not _win.controls.has("chips_row"), "the phone's sheet draws no chip row (its rows were measured at the sheet's edge)")
	## A real tap cannot reach an embedded dialog at the phone's content scale (`MISTAKES.md`), so the
	## transcript entry is made by the View facts handler itself, labelled as such.
	_win._on_view_facts()
	await _settle(4)
	var n := _transcript().size()
	_ok(n >= 2, "the sheet's transcript holds the facts message (handler call; %d entries)" % n)
	_win.hide()
	await _settle(6)
	_app.open_context_tab("ask")
	await _settle(10)
	_ok(_win.visible and _transcript().size() == n, "a reopened sheet keeps the transcript (%d of %d entries)" % [_transcript().size(), n])
	_win.hide()
	await _settle(4)
	_leg_done = true


func _leg_desktop() -> void:
	_app.select_domain("civilization")
	_app.arm_tool("inspect")
	await _settle(8)
	var vis := _visible_settlements()
	if vis.size() < 3:
		print("[FATAL] fewer than three settlements in view (%d)" % vis.size())
		_ok(false, "three settlements in view")
		return
	var a: int = vis[0]
	var b: int = vis[1]
	var c: int = vis[2]
	var wire0 := int(LlmWire.connect_attempts)

	print("== A. tab ==")
	await _click_settlement(a)
	_ok(_app.selection == {"kind": "settlement", "id": a}, "a real map click selects %s" % _name_of(a))
	_ok(_hdr("CtxTabInspect", true) != null and _hdr("CtxTabAsk", true) != null, "Inspect's header draws INSPECT and ASK")
	_ok(_dock().current_tab() == "inspect" and not _win.is_open(), "the panel starts on Inspect, the assistant closed")
	await _key(KEY_J, true)
	await _settle(6)
	var frame: Control = _dock().get("_ask_frame")
	_ok(_dock().current_tab() == "ask" and frame != null and frame.is_visible_in_tree() and not _app._right_dock_scroll.visible,
		"Ctrl+J shows the Ask tab, Inspect's scroll hidden")
	_ok(_win.is_docked() and not _win.visible and _win.is_open(), "the conversation is docked in the tab; the dialog is not shown")
	_ok(_c("input") != null and (_c("input") as Control).has_focus(), "Ctrl+J puts the cursor in the input")
	_ok(_hdr("CtxName") != null and (_hdr("CtxName") as Label).text == _name_of(a), "the Ask tab's header names the selection (%s)" % (_hdr("CtxName") as Label).text if _hdr("CtxName") != null else "no header")
	var lit := func(b: Control) -> bool: return b != null and (b as Button).get_theme_color("font_color") == DccTheme.c("accent")
	_ok(lit.call(_hdr("CtxTabAsk")) and not lit.call(_hdr("CtxTabInspect")), "ASK, not INSPECT, is the lit tab in the Ask header")
	await _shot("ask_off")
	await _key(KEY_1, true)
	_ok(_dock().current_tab() == "inspect" and _app._right_dock_scroll.visible and not frame.visible, "Ctrl+1 goes back to Inspect")
	await _key(KEY_3, true)
	_ok(_dock().current_tab() == "ask" and frame.visible, "Ctrl+3 shows Ask")
	await _press(_hdr("CtxTabInspect"), "the Ask header's INSPECT tab")
	_ok(_dock().current_tab() == "inspect", "a real click on INSPECT shows Inspect")
	await _press(_hdr("CtxTabAsk", true), "Inspect's ASK tab")
	_ok(_dock().current_tab() == "ask", "a real click on ASK shows Ask")
	print("  widths: frame %.0f (min %.0f), head min %.0f, host min %.0f, body min %.0f, dock col %.0f" % [frame.size.x, frame.get_combined_minimum_size().x,
		(_dock().get("_ask_head") as Control).get_combined_minimum_size().x, (_win.get("_host") as Control).get_combined_minimum_size().x,
		(_win.get("_body") as Control).get_combined_minimum_size().x, (frame.get_parent() as Control).size.x])
	for ch in (_win.get("_body") as Node).get_children():
		if (ch as Control).get_combined_minimum_size().x > 200.0:
			var kids := PackedStringArray()
			for g in ch.get_children():
				kids.append("%s:%.0f" % [(g as Control).get_class() + ("(" + (g as Button).text + ")" if g is Button else ("(" + (g as Label).text.left(20) + ")" if g is Label else "")), (g as Control).get_combined_minimum_size().x])
			print("  wide row: %s %s min %.0f [%s]" % [ch.get_class(), ch.name, (ch as Control).get_combined_minimum_size().x, ", ".join(kids)])
	var docked_w := frame.get_global_rect().size.x
	_ok(docked_w <= float(DccTheme.role_px("w_right_dock")) + 0.5 if DccTheme.is_tablet() else docked_w <= float(DccTheme.W_RIGHT_DOCK) + 0.5,
		"the Ask tab does not widen the dock (frame %.0f px)" % docked_w)
	var min_w := (_win.get("_host") as Control).get_combined_minimum_size().x
	_ok(min_w <= docked_w + 0.5, "the docked conversation fits the dock's width (minimum %.0f, frame %.0f)" % [min_w, docked_w])

	## The map is narrower with the dock open; pick the places again from what is in view now.
	vis = _visible_settlements()
	if vis.size() >= 3:
		var rest := vis.filter(func(i): return i != a)
		b = rest[0]
		c = rest[1]

	print("== B. master switch off ==")
	_ok(not bool(DccSettings.llm_value("enabled")), "control: the wiped user dir has the assistant off")
	_ok((_c("send") as Button).text == "Turn on Assistant enabled" and (_c("reason") as Label).visible,
		"Send reads Turn on Assistant enabled, with its reason ('%s')" % (_c("reason") as Label).text.left(60))

	print("== C. persistence ==")
	(_c("input") as Control).grab_focus()
	await _settle(2)
	await _type("draft one")
	_ok((_c("input") as TextEdit).text == "draft one", "a draft typed with real keys is in the input ('%s')" % (_c("input") as TextEdit).text)
	var t_before := _transcript().size()
	await _press(_c("facts_view"), "View facts")
	await _settle(4)
	_ok(_transcript().size() == t_before + 1, "View facts adds one transcript entry (%d -> %d)" % [t_before, _transcript().size()])
	## Two view cards (the handler a model reply reaches), built from the live view facts: one applied by
	## a real click, one left waiting.
	var lay := ""
	var lay_label := ""
	for l in (_win._VIEW.facts(_app, _bridge) as Dictionary).get("layers", []):
		if bool((l as Dictionary).get("on", false)):
			lay = String(l["id"])
			lay_label = String(l["label"])
			break
	_ok(lay != "", "control: a shown layer exists for the card (%s)" % lay)
	var card_v := {"summary": "probe card", "valid": [{"action": "set_layer_visible", "label": lay_label, "from": "shown", "to": "hidden",
		"args": {"layer": lay, "visible": false}, "undoable": false}], "rejected": [], "noop": PackedStringArray(), "undo": PackedStringArray()}
	_win._on_view_proposal({"view": card_v})
	_win._on_view_proposal({"view": card_v.duplicate(true)})
	await _settle(6)
	var cards: Array = _win.cards
	_ok(cards.size() == 2, "two cards are drawn (%d)" % cards.size())
	var c1: Dictionary = cards[0] if cards.size() > 0 else {}
	var c2: Dictionary = cards[1] if cards.size() > 1 else {}
	if not c1.is_empty():
		await _press(c1["apply"], "the first card's Apply")
	_ok(not c1.is_empty() and c1["state"] == "applied" and not bool(_app.viewport.layer_visible(lay)), "a real click on Apply applies the first card (layer %s hidden)" % lay)
	_ok(not c2.is_empty() and c2["state"] == "new" and not (c2["apply"] as Button).disabled, "the second card waits with Apply enabled")
	var snap := func() -> Dictionary:
		return {"n": _transcript().size(), "draft": (_c("input") as TextEdit).text if _c("input") != null else "",
			"c1": String(c1.get("state", "")), "c2": String(c2.get("state", "")),
			"c2_apply": not c2.is_empty() and is_instance_valid(c2["apply"]) and not (c2["apply"] as Button).disabled,
			"in_tree": not c1.is_empty() and is_instance_valid(c1["box"]) and (c1["box"] as Node).is_inside_tree() and is_instance_valid(c2["box"]) and (c2["box"] as Node).is_inside_tree(),
			"cards": (_win.cards as Array).size()}
	_widths("after cards")
	var s0: Dictionary = snap.call()
	_ok(s0["n"] >= t_before + 3 and s0["in_tree"], "the transcript holds the facts and both cards (%s)" % str(s0))
	await _key(KEY_1, true)
	await _key(KEY_3, true)
	_ok(snap.call() == s0, "Ctrl+1 then Ctrl+3: transcript, draft, cards and Apply state unchanged (%s)" % str(snap.call()))
	var wp := _menu("Window")
	_ok(wp != null, "Window's menu is found (bar or ☰)")
	if wp == null:
		return
	var rnode: CanvasItem = _app._region_nodes[DccMenus.ID_WIN_RIGHT]
	wp.id_pressed.emit(DccMenus.ID_WIN_RIGHT)
	await _settle(6)
	_ok(not rnode.visible and not _win.is_open(), "Window ▸ Context panel hides the panel (and the conversation with it)")
	await _key(KEY_J, true)
	await _settle(6)
	_ok(rnode.visible and _dock().current_tab() == "ask" and _win.is_open(), "Ctrl+J shows the hidden panel on Ask")
	_ok(snap.call() == s0, "closing and reopening the panel keeps everything (%s)" % str(snap.call()))
	await _click_settlement(b)
	_ok(_app.selection == {"kind": "settlement", "id": b} and _dock().current_tab() == "ask", "a real map click on %s selects it and the Ask tab stays (selection %s, tab %s, at %s)" % [_name_of(b), str(_app.selection), _dock().current_tab(), str(_screen_of(Vector2(float(_bridge.settlements()[b].get("x", 0)), float(_bridge.settlements()[b].get("y", 0)))))])
	_ok(snap.call() == s0, "reselecting keeps everything (%s)" % str(snap.call()))
	var story := _menu("Story")
	var si := story.get_item_index(DccMenus.ID_STORY_ASSISTANT) if story != null else -1
	_ok(si >= 0 and story.get_item_text(si) == "Ask the assistant about the selection" and story.get_item_accelerator(si) == (KEY_MASK_CTRL | KEY_J),
		"Story's row reads Ask the assistant about the selection, on Ctrl+J")
	await _key(KEY_1, true)
	if si >= 0:
		story.id_pressed.emit(DccMenus.ID_STORY_ASSISTANT)
		await _settle(6)
	_ok(_dock().current_tab() == "ask" and snap.call() == s0, "Story's row opens the same conversation in the Ask tab")
	await _shot("ask_persist")

	print("== D. context ==")
	_ok(not _win._shown_args().has("shown_selection") and not _win._shown_args().has("shown_year"),
		"control: with Send: selection and year off (the default), neither is in the request")
	_ok((_c("chip_selection") as Button).disabled and (_c("chip_year") as Button).disabled, "...and both chips are disabled with a reason ('%s')" % (_c("chip_year") as Button).tooltip_text.left(50))
	DccSettings.set_llm_value("ctx_selection", true)
	_win._refresh()
	await _settle(4)
	var y0 := int(_app.tl_year())
	var sa: Dictionary = _win._shown_args()
	_ok(String(sa.get("shown_selection", "")) == "settlement " + _name_of(b) and sa.has("shown_year") and int(sa["shown_year"]) == y0,
		"with the source on, the request carries the selection and the year (%s; year %d)" % [str(sa), y0])
	_ok((_c("chip_selection") as Button).text == "Selection: settlement " + _name_of(b) and (_c("chip_year") as Button).text == "Year: %d" % y0,
		"the chips name them ('%s', '%s')" % [(_c("chip_selection") as Button).text, (_c("chip_year") as Button).text])
	_widths("context on")
	var y1 := y0 + 37
	_app.tl_set_year(y1, true)
	await _settle(6)
	_ok(int(_app.tl_year()) == y1 and (_c("chip_year") as Button).text == "Year: %d" % y1 and int(_win._shown_args().get("shown_year", -99999)) == y1,
		"moving the year moves the chip and the request (%s)" % (_c("chip_year") as Button).text)
	await _click_settlement(c)
	_ok((_c("chip_selection") as Button).text == "Selection: settlement " + _name_of(c) and _dock().current_tab() == "ask",
		"a real map click on %s moves the selection chip; the tab stays Ask" % _name_of(c))
	## The request a Send would make, built and not sent (no socket: `prepare` builds bytes only).
	DccSettings.set_llm_value("enabled", true)
	DccSettings.set_llm_value("prose", true)
	DccSettings.set_llm_value("local_allowed", true)
	DccSettings.set_llm_value("preset_id", "lm_studio")
	DccSettings.set_llm_value("base_url", "http://127.0.0.1:1234/v1")
	DccSettings.set_llm_value("model", "qwen3-14b")
	var ep: Dictionary = _bridge.llm_endpoint_info("lm_studio", "http://127.0.0.1:1234/v1")
	DccSettings.llm_add_consent(String(ep.get("origin", "")))
	_win._refresh()
	await _settle(4)
	_ok(String((_c("sent") as Label).text).contains("the year the timeline shows (%d)" % y1) and String((_c("sent") as Label).text).contains(_name_of(c)),
		"the this-will-send line names the year and the place ('%s')" % String((_c("sent") as Label).text).right(160))
	var req: Dictionary = _win.run.stream.prepare(DccSettings.llm_gate_dict(), "prose", DccSettings.llm_consented_origins(), _win._args("What year is it?"))
	var body := (req.get("body", PackedByteArray()) as PackedByteArray).get_string_from_utf8()
	if not bool(req.get("ok", false)):
		print("  prepare refused: %s %s" % [String(req.get("code", "")), String(req.get("reason", ""))])
	_ok(bool(req.get("ok", false)) and body.contains("year shown on the timeline: %d" % y1) and body.contains("selected: settlement " + _name_of(c).replace("\"", "")),
		"the request Send would make carries the shown block with year %d and %s (ok %s, %d bytes)" % [y1, _name_of(c), str(req.get("ok", false)), body.length()])
	_ok(bool((req.get("sent", {}) as Dictionary).get("shown", false)), "prepare's summary counts the shown block")
	await _press(_c("chip_year"), "the year chip")
	_ok(not _win._shown_args().has("shown_year") and _win._shown_args().has("shown_selection") and not (_c("chip_year") as Button).button_pressed,
		"a real click on the year chip leaves the year out (%s)" % str(_win._shown_args()))
	var req2: Dictionary = _win.run.stream.prepare(DccSettings.llm_gate_dict(), "prose", DccSettings.llm_consented_origins(), _win._args("What year is it?"))
	var body2 := (req2.get("body", PackedByteArray()) as PackedByteArray).get_string_from_utf8()
	_ok(bool(req2.get("ok", false)) and not body2.contains("year shown on the timeline") and body2.contains("selected: settlement"),
		"...and the built request then has no year line")
	await _press(_c("chip_year"), "the year chip again")
	_ok(_win._shown_args().has("shown_year"), "a second click puts the year back")
	DccSettings.set_llm_value("ctx_selection", false)
	_win._refresh()
	var req3: Dictionary = _win.run.stream.prepare(DccSettings.llm_gate_dict(), "prose", DccSettings.llm_consented_origins(), _win._args("What year is it?"))
	var body3 := (req3.get("body", PackedByteArray()) as PackedByteArray).get_string_from_utf8()
	_ok(bool(req3.get("ok", false)) and not body3.contains("kind=\\\"shown\\\"") and not body3.contains("year shown on the timeline"),
		"with the source off the built request has no shown block")
	## Rust re-checks the source: keys forced into the args with the source off still send nothing.
	var forced: Dictionary = _win._args("What year is it?")
	forced["shown_selection"] = "settlement Forced"
	forced["shown_year"] = 1
	var req4: Dictionary = _win.run.stream.prepare(DccSettings.llm_gate_dict(), "prose", DccSettings.llm_consented_origins(), forced)
	_ok(not (req4.get("body", PackedByteArray()) as PackedByteArray).get_string_from_utf8().contains("Forced"), "Rust drops shown keys when the source is off")
	DccSettings.set_llm_value("ctx_selection", true)
	DccSettings.set_llm_value("enabled", false)
	_win._refresh()
	await _settle(4)

	print("== E. citation ==")
	var tgt: Dictionary = _bridge.settlements()[a]
	await _shot("before_citation")
	_win._on_sources({"sources": [{"label": "WORLD DATA", "source": "world", "id": "W:settlement:%d" % int(tgt.get("tid", -1)), "title": _name_of(a), "locator": "settlement"}],
		"citations": [{"verified": true, "id": "W:settlement:%d" % int(tgt.get("tid", -1)), "label": "1", "title": _name_of(a)},
			{"verified": false, "id": "W:settlement:999999"}]})
	await _settle(6)
	var link: Button = null
	var box: Node = (_win.source_cards as Array).back()["box"] if not (_win.source_cards as Array).is_empty() else null
	if box != null:
		for bt in box.find_children("*", "Button", true, false):
			if (bt as Node).has_meta("cite_target"):
				link = bt
	_ok(link != null and (link.get_meta("cite_target") as Dictionary) == {"kind": "settlement", "id": a}, "the verified citation is a link to %s" % _name_of(a))
	var nlinks := 0
	if box != null:
		for bt in box.find_children("*", "Button", true, false):
			if (bt as Node).has_meta("cite_target"):
				nlinks += 1
	_ok(nlinks == 1, "the unverified citation is not a link (%d links)" % nlinks)
	if link != null:
		var msc: ScrollContainer = _win.get("_scroll")
		print("  citation: tab %s, scroll visible %s, scroll v %d of %.0f, link y %.0f" % [_dock().current_tab(), str(msc.is_visible_in_tree()), msc.scroll_vertical, msc.get_v_scroll_bar().max_value, link.get_global_rect().position.y])
		await _press(link, "the citation link")
	_ok(_app.selection == {"kind": "settlement", "id": a} and _dock().current_tab() == "ask", "a real click on the citation selects %s; Ask stays" % _name_of(a))

	if _mode == "desktop":
		print("== F. window ==")
		var n_before := _transcript().size()
		await _press(_c("open_window"), "Open in a window")
		await _settle(10)
		_ok(_win.visible and not _win.is_docked() and _transcript().size() == n_before, "Open in a window shows the same conversation in the dialog (%d entries)" % _transcript().size())
		var note: Control = (_dock().get("_ask_body") as Node).find_child("AskInWindowNote", true, false) as Control
		_ok(note != null and note.is_visible_in_tree(), "the Ask tab says the conversation is in its own window")
		await _shot("window")
		## The window's own Close button, through its `pressed` (a pixel push into an embedded window does
		## not reach it, `_ac3_probe.gd`'s finding).
		_win.get_ok_button().pressed.emit()
		await _settle(10)
		_ok(not _win.visible and _win.is_docked() and _win.is_open() and _transcript().size() == n_before, "closing the window returns the conversation to the tab (%d entries)" % _transcript().size())
		_ok((_dock().get("_ask_body") as Node).find_child("AskInWindowNote", true, false) == null, "...and the notice is gone")
	else:
		_ok(_c("open_window") == null, "the tablet has no Open in a window (its windows stay sheets, §3.11)")

	print("== G. reading ==")
	_app.arm_tool("measure")
	await _settle(6)
	await _click_settlement(a)
	await _click_settlement(b)
	await _settle(6)
	_ok(_dock().current_tab() == "inspect" and _app._right_dock_scroll.visible, "a Measure reading on the Ask tab switches the panel to Inspect")
	_app.arm_tool("inspect")
	await _settle(6)

	print("== H. master switch on, no Send ==")
	DccSettings.set_llm_value("enabled", true)
	_ok(bool(DccSettings.quick_bar_value("assistant_button")), "control: the quick bar's Assistant switch is on (its default)")
	_app._quick_bar.refresh()
	await _settle(8)
	var chip: Button = _app._quick_bar.assistant_chip()
	_ok(chip != null and chip.visible and not chip.disabled, "the quick bar's Assistant chip is shown and live (%s)" % ("visible" if chip != null and chip.visible else "hidden"))
	await _key(KEY_1, true)
	if chip != null and chip.visible:
		await _press(chip, "the quick bar's Assistant chip")
		await _settle(6)
	_ok(_dock().current_tab() == "ask" and _win.is_docked() and not _win.visible, "the chip opens the Ask tab")
	await _key(KEY_1, true)
	## The ring dock's row is `QuickBarRing.assistant_item()`; its callable is what a press on it runs
	## (`radial_ring.gd`), called here (`_quickbar_ring_probe` drives the ring's own gestures).
	var it: Dictionary = QuickBarRing.assistant_item(_app)
	_ok(it.has("callable") and bool(it.get("enabled", false)), "the ring dock's Assistant row is live")
	if it.has("callable"):
		(it["callable"] as Callable).call()
	await _settle(6)
	_ok(_dock().current_tab() == "ask" and _win.is_docked(), "the ring dock's Assistant row opens the Ask tab")
	_ok((_c("new") as Button) != null and not (_c("new") as Button).disabled, "New is live while idle")
	var w_now := int(LlmWire.connect_attempts)
	_ok(w_now == wire0, "no socket was attempted by any of this (%d -> %d)" % [wire0, w_now])

	if _live:
		await _leg_live()

	print("== C2. New ==")
	await _press(_c("new"), "New")
	await _settle(6)
	_ok(_transcript().size() <= 2 and (_win.cards as Array).is_empty() and (_c("input") as TextEdit).text == "",
		"New empties the transcript, the cards and the draft (%d entries, %d cards)" % [_transcript().size(), (_win.cards as Array).size()])
	_ok(_win.is_docked() and _dock().current_tab() == "ask", "...and the new conversation is still in the Ask tab")
	DccSettings.set_llm_value("enabled", false)
	_leg_done = true


func _leg_live() -> void:
	print("== L. live: LM Studio %s ==" % _model)
	DccSettings.set_llm_value("commands", true)
	DccSettings.set_llm_value("ctx_params", true)
	DccSettings.set_llm_value("ctx_selection", true)
	DccSettings.set_llm_value("thinking", "none")
	DccSettings.set_llm_value("model", _model)
	_win._refresh()
	await _key(KEY_J, true)
	var a: int = _visible_settlements()[0]
	await _click_settlement(a)
	await _press(_c("task_commands"), "the Commands chip")
	(_c("input") as TextEdit).text = ""
	(_c("input") as Control).grab_focus()
	await _type("what year is it and what is selected?")
	_win._refresh()
	await _settle(4)
	var yr := int(_app.tl_year())
	var nm := _name_of(a)
	print("LIVE asking with %s selected, year %d" % [nm, yr])
	var done := [false]
	var fail := [""]
	_win.run.finished.connect(func(_i): done[0] = true, CONNECT_ONE_SHOT)
	_win.run.failed.connect(func(m): fail[0] = m; done[0] = true, CONNECT_ONE_SHOT)
	var t0 := Time.get_ticks_msec()
	await _press(_c("send"), "Send")
	while not done[0] and Time.get_ticks_msec() - t0 < 300000:
		await get_tree().create_timer(0.25).timeout
	while _win.run.busy() and Time.get_ticks_msec() - t0 < 320000:
		await get_tree().create_timer(0.25).timeout
	var reply := String(_win.get("_reply"))
	var all_text := ""
	for m in _transcript():
		for l in (m as Node).find_children("*", "Label", true, false):
			all_text += (l as Label).text + "\n"
	print("LIVE ms=%d failed='%s'" % [Time.get_ticks_msec() - t0, fail[0]])
	print("LIVE reply: %s" % reply.replace("\n", " ").left(600))
	_ok(fail[0] == "" and reply.strip_edges() != "", "the live request finished with a reply")
	_ok(reply.contains(str(yr)), "the reply states the year shown (%d)" % yr)
	_ok(reply.contains(nm), "the reply names the selected place (%s)" % nm)
	_ok(_c("last_sent_with") != null and (_c("last_sent_with") as Label).text.contains("year %d" % yr), "the sent message records what went with it ('%s')" % ((_c("last_sent_with") as Label).text if _c("last_sent_with") != null else ""))
	await _shot("live")
