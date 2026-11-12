extends Node
## MS-9 probe (`MOBILE_SHELL_PROPOSAL.md` section 6, MS-9: "Search runs every command -- a check, radio or
## parameter result does what its row does"), inside the real shell, on one form per run.
##
##   godot --path . --resolution 1600x1000 _ms9_probe.tscn -- --vp 1600x1000 --form desktop
##   godot --path . --resolution 1600x1000 _ms9_probe.tscn -- --vp 1600x1000 --form tablet --force-touch --density 1
##   godot --path . --resolution 1600x1000 _ms9_probe.tscn -- --vp 1080x2340 --form phone --force-touch --density 2.625
##
## **Windowed, never `--headless`**, from a wiped user dir, under the window lock (`MISTAKES.md`, "Run a
## pixel probe"). Arguments read here: `--vp WxH`, `--form desktop|tablet|phone` (what the shell MUST classify
## as; a run that boots as another form aborts, so a "tablet" log can never be a desktop one), `--density D`,
## `--only param|tool|check|plain|blocked|guards` and `--stride N` (mutation runs: sweep one bucket, every Nth row), `--force-touch`, `--skip-sweep` (only the guards-free real-click and floor sections; for debugging the probe). Anything else aborts with exit 2.
##
## What it protects. It enumerates EVERY row of the app's own `CommandIndex`, opens the search surface of the
## form (the phone's overlay, the desktop dialog), narrows it to the row with a `.` command query, presses
## the row's own result button, and compares what happened with invoking the row directly:
##
##   - **check / radio**: the owning menu's item bits (and `row_state()`) after the search press, then after a
##     DIRECT `id_pressed.emit(id)` on the same popup. A check must flip on the search press and flip back on
##     the direct one (one toggle each, so the search path is exactly one invocation of the same handler); a
##     radio must end selected and a direct press must leave it so. The row must still be on screen, saying
##     the new state in its second line. A handler that declines (the 3D tiers before the tier test, say) is
##     counted as `declined`, not as a pass of anything: the state did not move under either route.
##   - **plain menu row**: a spy replaces the popup's `id_pressed` connections for the press (so no dialog,
##     export or quit is actually run), and must receive exactly `[id]`; the surface must close. A row whose
##     item is disabled, a readout, or a pointer row must receive NOTHING and say why where the form says it
##     (the dialog's count line on desktop and tablet, a toast on the phone).
##   - **tool row**: `armed_tool` equals the tool id, as a direct `arm_tool()` leaves it.
##   - **parameter row**: the world dock opens on the right category, the control is visible in the tree and
##     inside its scroll container's viewport, the dock is shown (re-expanded, or the phone's sheet), and the
##     surface closed. A key with no control says so and leaves the surface open (listed, with its reason).
##   - **stale and live-disabled guards**: a row whose item was renamed since the build, and one disabled
##     since, refuse with a reason and run nothing.
##
## Expectations come from the POPUPS (disabled bit, check shape, tooltip, item id), never from the index's own
## `available`/`pinnable` fields, so the index cannot grade itself. Real pointer events (a touch and its
## emulated mouse button on touch forms) press one row of each kind; the rest use the result button's own
## `pressed` signal, which is what a pointer ends in. 44 dp floors are measured on the drawn rows and the star.
##
## Result: `COUNT <bucket> = n` lines, `LIST` lines for every row that could not run (with the reason),
## `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument or headless.

var _vp: SubViewport
var _app: Node
var _bridge
var _form := ""
var _touch := false
var _fails := 0
var _checks := 0
var _counts := {}
var _lists: Array = []


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fails += 1
		print("  FAIL  " + what)


func _note(what: String) -> void:
	print("  info  " + what)


func _count(bucket: String) -> void:
	_counts[bucket] = int(_counts.get(bucket, 0)) + 1


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _arg(name: String, dflt: String) -> String:
	var a := OS.get_cmdline_user_args()
	var i := a.find(name)
	return String(a[i + 1]) if i >= 0 and i + 1 < a.size() else dflt


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("ABORT: run WINDOWED")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--force-touch":
			_touch = true
			i += 1
		elif a == "--skip-sweep":
			i += 1
		elif a in ["--vp", "--form", "--density", "--only", "--stride"]:
			i += 2
		else:
			print("ABORT: unknown argument ", a)
			get_tree().quit(2)
			return
	_form = _arg("--form", "")
	if not _form in ["desktop", "tablet", "phone"]:
		print("ABORT: needs --form desktop|tablet|phone")
		get_tree().quit(2)
		return
	get_tree().create_timer(1500.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	var parts := _arg("--vp", "").split("x")
	if parts.size() != 2:
		print("ABORT: needs --vp WxH")
		get_tree().quit(2)
		return
	await _run(int(parts[0]), int(parts[1]))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _run(vw: int, vh: int) -> void:
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(2)
		return
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.6).timeout
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if "phone_project_picker" in _app and _app.phone_project_picker != null:
		_app.phone_project_picker.hide()
	await _frames(6)
	var got := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	_ok(got == _form, "the shell booted as %s (wanted %s)" % [got, _form])
	if got != _form:
		return
	_bridge = _app.bridge
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await _frames(2)
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _frames(10)
	await _frames(30)
	_ok(bool(_bridge.has_world), "a world was generated")
	if not "--skip-sweep" in OS.get_cmdline_user_args():
		await _enumerate_and_press()
		await _guards()
	await _real_clicks()
	await _floors()
	print("== SUMMARY (%s) ==" % _form)
	var keys := _counts.keys()
	keys.sort()
	for k in keys:
		print("COUNT %s = %d" % [k, _counts[k]])
	for l in _lists:
		print("LIST " + String(l))


# ---- the search surface ---------------------------------------------------------------------------------------

func _phone() -> bool:
	return _form == "phone"


func _open() -> void:
	_app.open_find_on_map()


func _is_open() -> bool:
	if _phone():
		return _app._phone_search_overlay != null and _app._phone_search_overlay.visible
	return _app._desktop_search_dialog != null and _app._desktop_search_dialog.visible


func _close() -> void:
	if _phone():
		_app._set_search_open(false)
	elif _app._desktop_search_dialog != null:
		_app._desktop_search_dialog.hide()


func _query(q: String) -> void:
	if _phone():
		_app._phone_search_field.text = q
		_app._refresh_phone_search()
	else:
		_app._desktop_search_field.text = q
		_app._refresh_desktop_search()
	await _frames(3)


func _results() -> Control:
	return _app._phone_search_results if _phone() else _app._desktop_search_results


func _button_for(row: Dictionary) -> Button:
	for b in _results().find_children("*", "Button", true, false):
		if (b as Button).has_meta("search_row") and is_same((b as Button).get_meta("search_row"), row):
			return b as Button
	return null


func _row_buttons() -> Array:
	var out: Array = []
	for b in _results().find_children("*", "Button", true, false):
		if (b as Button).has_meta("search_row"):
			out.append(b)
	return out


## The text the form shows when a row would not run: the dialog's count line (desktop, tablet) or the
## newest toast (phone).
func _reason_shown() -> String:
	if not _phone():
		return String(_app._desktop_search_count.text)
	var best := ""
	for c in _app._phone_root.get_children():
		if c is PanelContainer and (c as Control).z_index == 50:
			for l in (c as Node).find_children("*", "Label", true, false):
				best = (l as Label).text
	return best


func _second_line(btn: Button) -> String:
	var labels := btn.find_children("*", "Label", true, false)
	return (labels[1] as Label).text if labels.size() > 1 else ""


## A row's signature, stable across index rebuilds AND a shell remount (a class swap frees every popup, so no
## instance id may be part of it): kind, menu, item id, parameter key. Rows that share one (index popups number
## their items from 0, and a submenu repeats ids) are told apart by "#n", their order of appearance in the index
## (`_sig_of()`); the build order is deterministic while the menus are unchanged.
func _sig(r: Dictionary) -> String:
	return "%s|%s|%d|%s" % [r["kind"], r["group"], int(r.get("_id", -1)), r.get("key", "")]


## `row`'s full signature within `idx`: the base plus "#n" for the n-th row sharing it.
func _sig_of(idx: CommandIndex, row: Dictionary) -> String:
	var base := _sig(row)
	var n := 0
	for r in idx.all():
		if is_same(r, row):
			return "%s#%d" % [base, n]
		if _sig(r) == base:
			n += 1
	return "%s#?" % base


## `sig` may end in "#n": the n-th row (0-based) sharing that signature, for the few that share one (index
## popups number their items from 0, so two sibling rows can share an id).
func _find_row(idx: CommandIndex, sig: String) -> Dictionary:
	var h := sig.rfind("#")
	var want := sig.substr(0, h)
	var nth := int(sig.substr(h + 1))
	var seen := 0
	for r in idx.all():
		if _sig(r) == want:
			if seen == nth:
				return r
			seen += 1
	return {}


## Opens a fresh search surface and narrows it to `row`'s title. Returns the CURRENT row dictionary
## (the surface rebuilds its index on open) and leaves its result button findable.
func _present(sig: String) -> Dictionary:
	_close()
	await _frames(2)
	_open()
	await _frames(3)
	var idx: CommandIndex = _app._ensure_command_index()
	var row := _find_row(idx, sig)
	if row.is_empty():
		return row
	await _query(". " + String(row["title"]))
	if _button_for(row) == null:
		## A title with a character the query normalises (or text longer than a query is meant to be): the group
		## name selects the whole menu, and the row is still found by identity.
		await _query(". " + String(row["group"]))
	return row


func _bits(p: PopupMenu) -> Array:
	var out: Array = []
	for i in p.item_count:
		out.append([p.get_item_id(i), p.get_item_text(i), p.is_item_checked(i)])
	return out


func _item(p: PopupMenu, id: int, text: String) -> int:
	for i in p.item_count:
		if p.get_item_id(i) == id and p.get_item_text(i) == text:
			return i
	return -1


# ---- the sweep ------------------------------------------------------------------------------------------------

func _enumerate_and_press() -> void:
	if _arg("--only", "") == "guards":
		return
	_open()
	await _frames(3)
	await _query(". ")
	var idx: CommandIndex = _app._ensure_command_index()
	var sigs: Array = []
	for r in idx.all():
		sigs.append(_sig_of(idx, r))
	_ok(sigs.size() > 400, "the index is not silently empty (%d rows)" % sigs.size())
	var unnamed := 0
	for r in idx.all():
		if String(r["kind"]) == "menu" and String(r["group"]).strip_edges() == "":
			unnamed += 1
	_ok(unnamed == 0, "every menu row names the menu it lives in (%d rows of embedded-control popups indexed)" % unnamed)
	_note("index rows: %d" % sigs.size())
	var only := _arg("--only", "")
	var stride := maxi(1, int(_arg("--stride", "1")))
	var seen_in_bucket := {}
	for sig in sigs:
		var row := await _present(sig)
		if row.is_empty():
			_ok(false, "row vanished on re-present: " + sig)
			continue
		var kind := String(row["kind"])
		var bucket := "check" if (kind == "menu" and bool(row.get("checkable", false))) else 			("plain" if kind == "menu" else ("blocked" if kind in ["readout", "action"] else kind))
		var nb := int(seen_in_bucket.get(bucket, 0))
		seen_in_bucket[bucket] = nb + 1
		if (only != "" and bucket != only) or nb % stride != 0:
			continue
		match kind:
			"param":
				await _press_param(sig, row)
			"tool":
				await _press_tool(sig, row)
			"menu":
				if bool(row.get("checkable", false)):
					await _press_check(sig, row)
				else:
					await _press_plain(sig, row)
			"readout", "action":
				await _press_blocked(row, "readout" if kind == "readout" else "pointer")
			_:
				_ok(false, "unknown row kind %s" % kind)
	_close()


func _press(row: Dictionary) -> bool:
	var b := _button_for(row)
	if b == null:
		_ok(false, "the result row for '%s' (%s) is on the surface" % [row["title"], row["group"]])
		return false
	b.pressed.emit()
	await _frames(3)
	return true


func _press_blocked(row: Dictionary, label: String) -> void:
	var before := _is_open()
	if not await _press(row):
		return
	var why := _reason_shown()
	_ok(why != "", "%s row '%s' says why it does not run" % [label, row["title"]])
	_ok(_is_open() and before, "%s row '%s' leaves the surface open" % [label, row["title"]])
	_count("blocked_" + label)
	_lists.append("%s | %s | %s | %s" % [label, row["group"], row["title"], why.left(90)])


func _press_tool(sig: String, row: Dictionary) -> void:
	var tid := String(row["_tool"])
	_app.arm_tool("inspect" if tid != "inspect" else "measure")
	await _frames(2)
	row = await _present(sig)
	if not await _press(row):
		return
	_ok(String(_app.armed_tool) == tid, "tool row '%s' armed %s (armed: %s)" % [row["title"], tid, _app.armed_tool])
	_ok(not _is_open(), "tool row '%s' closed the surface" % row["title"])
	_count("run_tool")


func _press_plain(sig: String, row: Dictionary) -> void:
	var p: PopupMenu = row["_popup"]
	var id := int(row["_id"])
	var i := _item(p, id, String(row["_text"]))
	_ok(i >= 0, "item for '%s' exists in its popup" % row["title"])
	if i < 0:
		return
	## Expectation from the POPUP, not from the index row.
	var marker = p.get_item_metadata(i)
	var is_readout: bool = typeof(marker) == TYPE_STRING and String(marker) == DccMenus.META_READOUT
	var disabled := p.is_item_disabled(i)
	var tip := p.get_item_tooltip(i)
	var saved: Array = []
	for c in p.id_pressed.get_connections():
		saved.append(c)
	for c in saved:
		p.id_pressed.disconnect(c["callable"])
	var hits: Array = []
	var spy := func(x: int) -> void: hits.append(x)
	p.id_pressed.connect(spy)
	if is_readout or disabled:
		await _press_blocked(row, "readout" if is_readout else "disabled")
		_ok(hits.is_empty(), "'%s' (%s) ran nothing (%s)" % [row["title"], row["group"], str(hits)])
		if disabled and not is_readout and tip.strip_edges() != "":
			_ok(_reason_shown() == tip, "disabled row '%s' shows the menu's own reason" % row["title"])
	else:
		var ok := await _press(row)
		if ok:
			_ok(hits == [id], "'%s' (%s) delivered id_pressed(%d) exactly once (%s)" % [row["title"], row["group"], id, str(hits)])
			_ok(not _is_open(), "'%s' closed the surface" % row["title"])
			_count("run_menu")
	p.id_pressed.disconnect(spy)
	for c in saved:
		p.id_pressed.connect(c["callable"], c["flags"])
	_app.arm_tool("inspect")
	await _frames(1)


func _press_check(sig: String, row: Dictionary) -> void:
	var p: PopupMenu = row["_popup"]
	var id := int(row["_id"])
	var i := _item(p, id, String(row["_text"]))
	_ok(i >= 0, "item for '%s' exists in its popup" % row["title"])
	if i < 0:
		return
	var radio := p.is_item_radio_checkable(i)
	var disabled := p.is_item_disabled(i)
	if disabled:
		var saved0: Array = []
		for c in p.id_pressed.get_connections():
			saved0.append(c)
		for c in saved0:
			p.id_pressed.disconnect(c["callable"])
		var h0: Array = []
		var spy0 := func(x: int) -> void: h0.append(x)
		p.id_pressed.connect(spy0)
		await _press_blocked(row, "disabled")
		_ok(h0.is_empty(), "disabled %s '%s' ran nothing" % ["radio" if radio else "check", row["title"]])
		p.id_pressed.disconnect(spy0)
		for c in saved0:
			p.id_pressed.connect(c["callable"], c["flags"])
		return
	var idx0: CommandIndex = _app._ensure_command_index()
	idx0.refresh_states()
	var st0 := idx0.row_state(row)
	var snap0 := _bits(p)
	_ok(st0 != CommandIndex.STATE_NONE, "'%s' (%s) has a readable state before the press" % [row["title"], row["group"]])
	## The search press.
	if not await _press(row):
		return
	if not is_instance_valid(p):
		## The handler remounted the shell (a class swap frees every popup). Nothing to compare against; the
		## sweep goes on with the rebuilt menus, and the row is listed so it is not mistaken for a pass.
		_count("check_remounted_shell")
		_lists.append("check remounted the shell | %s | %s" % [row["group"], row["title"]])
		return
	## On the phone Window > Left dock and Context panel open a dock SHEET, and opening any sheet closes every other
	## overlay (`_close_all_phone_overlays`), the search among them: that is what those rows do, so it is the one
	## allowed reason for the surface to be gone. Anything else leaving it is a failure.
	var surface_closed := not _is_open()
	if surface_closed:
		_ok(_phone() and String(row["group"]) == "Window" and String(row["title"]) in ["Left dock", "Context panel"],
			"'%s' (%s) closed the surface, which only a phone dock-sheet row may do" % [row["title"], row["group"]])
		_count("check_opens_phone_sheet")
		## Read the state BEFORE reopening the search (reopening closes the sheet again, which is the same
		## handler's other half and would put the state back).
		_ok(idx0.row_state(row) != st0, "'%s' (%s) flipped its state before the surface closed (%d to %d)" % [row["title"], row["group"], st0, idx0.row_state(row)])
		_open()
		await _frames(3)
		return
	else:
		_ok(true, "'%s' (%s) left the surface open so its new state can show" % [row["title"], row["group"]])
	var idx1: CommandIndex = _app._ensure_command_index()
	idx1.refresh_states()
	var row1 := _find_row(idx1, sig)
	var stS := idx1.row_state(row1) if not row1.is_empty() else CommandIndex.STATE_NONE
	var snapS := _bits(p)
	## The row on screen says the new state.
	await _query(". " + String(row["title"]))
	var btn := _button_for(_find_row(_app._ensure_command_index(), sig))
	var word := idx1.state_word(row1) if not row1.is_empty() else ""
	if btn != null and surface_closed:
		pass
	elif btn != null:
		_ok(word != "" and _second_line(btn).begins_with(word), "'%s' shows '%s' on its row (%s)" % [row["title"], word, _second_line(btn)])
	else:
		_ok(false, "'%s' is still listed after its press" % row["title"])
	## The direct invocation, from the state the search press left.
	p.id_pressed.emit(id)
	await _frames(2)
	var idx2: CommandIndex = _app._ensure_command_index()
	idx2.refresh_states()
	var row2 := _find_row(idx2, sig)
	var stD := idx2.row_state(row2) if not row2.is_empty() else CommandIndex.STATE_NONE
	var snapD := _bits(p)
	var moved := stS != st0
	if snapS != snap0 and not moved:
		_note("'%s' (%s): other items of its menu changed though its own state did not" % [row["title"], row["group"]])
	if radio:
		if moved:
			_ok(stS == 1, "radio '%s' (%s) ends selected after the search press" % [row["title"], row["group"]])
			_count("run_radio")
		elif st0 == 1:
			_count("run_radio_already_selected")
		else:
			_count("declined_radio")
			_lists.append("declined radio | %s | %s | state unchanged by search AND by the direct press" % [row["group"], row["title"]])
		_ok(stD == stS and snapD == snapS, "radio '%s' (%s): a direct press leaves what the search press left" % [row["title"], row["group"]])
	else:
		if moved:
			_ok(stS != st0, "check '%s' (%s) flipped on the search press (%d to %d)" % [row["title"], row["group"], st0, stS])
			_ok(stD == st0, "check '%s' (%s): the direct press flipped it back (one toggle each)" % [row["title"], row["group"]])
			_count("run_check")
		else:
			_count("declined_check")
			_lists.append("declined check | %s | %s | state unchanged by search AND by the direct press" % [row["group"], row["title"]])
			_ok(stD == st0, "check '%s' (%s): both routes left it alone" % [row["title"], row["group"]])
	## Put the world back where it was where the handler allows it, so later rows meet the same preferences.
	if radio and moved and st0 == 0:
		for b in snap0:
			if bool(b[2]):
				var j := _item(p, int(b[0]), String(b[1]))
				if j >= 0 and p.is_item_radio_checkable(j):
					p.id_pressed.emit(int(b[0]))
					break
	## A check needs no restore: the direct press above already flipped it back (asserted: stD == st0). A second
	## press here once left Session mode ON, and the product (correctly) refused every WORLD parameter after it.
	await _frames(1)


func _press_param(sig: String, row: Dictionary) -> void:
	var key := String(row["key"])
	var panel: Control = _app._workspace_panels.get("world")
	var cat := String(panel.call("param_category", key)) if panel != null else ""
	## Start somewhere else, with the dock put away, so "opens" cannot be "was already open".
	_app.select_domain("civilization")
	await _frames(2)
	if not _phone() and not _app.is_dock_collapsed("left"):
		_app._toggle_dock(true)
	await _frames(2)
	row = await _present(sig)
	if row.is_empty():
		_ok(false, "param row vanished: " + key)
		return
	if cat == "":
		await _press_blocked(row, "param_no_control")
		return
	if not await _press(row):
		return
	await _frames(4)
	_ok(not _is_open(), "param '%s' closed the surface" % key)
	_ok(String(_app.active_domain()) == "world", "param '%s' selected WORLD (%s)" % [key, _app.active_domain()])
	var ctl: Control = panel._param_control(key)
	_ok(ctl != null and is_instance_valid(ctl), "param '%s' has a control" % key)
	if ctl == null or not is_instance_valid(ctl):
		return
	_ok(ctl.is_visible_in_tree(), "param '%s' control is visible in the tree (category %s)" % [key, cat])
	var sc: Node = ctl.get_parent()
	while sc != null and not (sc is ScrollContainer):
		sc = sc.get_parent()
	if sc != null:
		var vr := (sc as Control).get_global_rect()
		var cr := ctl.get_global_rect()
		_ok(cr.get_center().y >= vr.position.y - 1.0 and cr.get_center().y <= vr.end.y + 1.0,
			"param '%s' control is inside its scroll viewport (y %.0f in %.0f..%.0f)" % [key, cr.get_center().y, vr.position.y, vr.end.y])
	if _phone():
		_ok(_app._dock_in_sheet and _app._phone_dom_host.visible, "param '%s': the phone's domain sheet shows the dock" % key)
	else:
		_ok(not _app.is_dock_collapsed("left"), "param '%s': the left dock is expanded" % key)
	if ctl.focus_mode != Control.FOCUS_NONE:
		_ok(_vp.gui_get_focus_owner() == ctl, "param '%s' control has focus" % key)
		_count("param_focus_taken")
	_count("run_param")


# ---- guards ---------------------------------------------------------------------------------------------------

func _guards() -> void:
	print("== guards ==")
	_open()
	await _frames(3)
	await _query(". ")
	var idx: CommandIndex = _app._ensure_command_index()
	## A plain, available, pinnable menu row to damage.
	var victim := {}
	for r in idx.all():
		if String(r["kind"]) == "menu" and not bool(r.get("checkable", false)) and bool(r["pinnable"]) and bool(r["available"]) \
				and not String(r["title"]).is_empty():
			victim = r
			break
	_ok(not victim.is_empty(), "a plain command exists to damage")
	if victim.is_empty():
		return
	var p: PopupMenu = victim["_popup"]
	var sig := _sig_of(idx, victim)
	var title := String(victim["title"])
	var id := int(victim["_id"])
	var ix := _item(p, id, String(victim["_text"]))
	var saved: Array = []
	for c in p.id_pressed.get_connections():
		saved.append(c)
	for c in saved:
		p.id_pressed.disconnect(c["callable"])
	var hits: Array = []
	var spy := func(x: int) -> void: hits.append(x)
	p.id_pressed.connect(spy)
	## 1. renamed since the build: stale. The row must be pressed on the list that was built BEFORE the rename.
	var orig_text := p.get_item_text(ix)
	var row := await _present(sig)
	p.set_item_text(ix, orig_text + " (moved)")
	var b := _button_for(row)
	_ok(b != null, "stale guard: the row is on screen")
	if b != null:
		b.pressed.emit()
		await _frames(3)
		_ok(hits.is_empty(), "stale guard: a renamed item ran nothing (%s)" % str(hits))
		_ok(_reason_shown().begins_with("That menu changed"), "stale guard says so (%s)" % _reason_shown())
	p.set_item_text(ix, orig_text)
	## 2. disabled since the build: live-disabled.
	row = await _present(sig)
	p.set_item_disabled(ix, true)
	p.set_item_tooltip(ix, "Probe reason: nothing to do here.")
	b = _button_for(row)
	if b != null:
		b.pressed.emit()
		await _frames(3)
		_ok(hits.is_empty(), "live-disabled guard: a disabled item ran nothing (%s)" % str(hits))
		_ok(_reason_shown() == "Probe reason: nothing to do here.", "live-disabled guard shows the item's tooltip (%s)" % _reason_shown())
	p.set_item_disabled(ix, false)
	p.set_item_tooltip(ix, "")
	## 3. control: undamaged, it runs.
	row = await _present(sig)
	b = _button_for(row)
	if b != null:
		b.pressed.emit()
		await _frames(3)
		_ok(hits == [id], "guard control: the undamaged row delivers id_pressed once (%s)" % str(hits))
	p.id_pressed.disconnect(spy)
	for c in saved:
		p.id_pressed.connect(c["callable"], c["flags"])
	_close()
	await _stale_bit_guard()


## 4. A check whose menu item bit disagrees with the app's truth (a context module closed by its own window,
## say) must still be SHOWN as the truth: the index is built with the menu's own refreshers run first. The bit is
## set wrong on purpose on a Window > context-module check, the search is opened fresh, and the row's second line
## must lead with the state the refresher computes, not the planted one.
func _stale_bit_guard() -> void:
	_open()
	await _frames(3)
	await _query(". ")
	var idx: CommandIndex = _app._ensure_command_index()
	var target := {}
	for r in idx.all():
		if String(r["kind"]) != "menu" or not bool(r.get("checkable", false)) or bool(r.get("toggle", false)) or not bool(r["available"]):
			continue
		## A row whose popup has a NAMED refresher (`_refresh_*`, not a lambda and not one the index refuses to run):
		## only such a menu rewrites its bits from the app's truth when it opens.
		var named := false
		for c in (r["_popup"] as PopupMenu).about_to_popup.get_connections():
			var nm := String((c["callable"] as Callable).get_method())
			if nm.begins_with("_refresh_") and not CommandIndex.UNSAFE_OPEN_HANDLERS.has(nm):
				named = true
		if named:
			target = r
			break
	_ok(not target.is_empty(), "stale-bit guard: a check or radio with a named menu refresher exists")
	if target.is_empty():
		_close()
		return
	var sig := _sig_of(idx, target)
	var p: PopupMenu = target["_popup"]
	var ix := _item(p, int(target["_id"]), String(target["_text"]))
	idx.refresh_states()
	var truth := idx.row_state(target)
	_ok(truth != CommandIndex.STATE_NONE, "stale-bit guard: the module's state is readable")
	p.set_item_checked(ix, truth == 0)
	var row := await _present(sig)
	var b := _button_for(row)
	_ok(b != null, "stale-bit guard: the row is listed")
	if b != null:
		var want := "On" if truth == 1 else "Off"
		_ok(_second_line(b).begins_with(want), "stale-bit guard: a planted wrong bit is shown as the app's truth '%s' (%s)" % [want, _second_line(b)])
	_close()


# ---- real pointer events --------------------------------------------------------------------------------------

func _click(btn: Button) -> void:
	## Where the person's pointer would be: the dialog is an embedded Window, so the host viewport sees the
	## button at its SCREEN position (window origin plus its rect); the phone's overlay is a plain Control in the host.
	var vp: Viewport = _vp
	var pos: Vector2 = btn.get_screen_position() + btn.size * 0.5 if btn.get_viewport() is Window else btn.get_global_rect().get_center()
	_note("click at %s (button viewport %s, rect %s)" % [pos, btn.get_viewport().get_class(), btn.get_global_rect()])
	var motion := InputEventMouseMotion.new()
	motion.position = pos
	motion.global_position = pos
	vp.push_input(motion)
	await _frames(1)
	## A touch is added only where the host delivers it (the phone's overlay, as `_ms4_probe.gd` does); the embedded
	## dialog on the tablet gets the emulated mouse alone, since the embedder does not emulate a mouse from a touch
	## and a pair would press twice.
	var with_touch := _touch and not (btn.get_viewport() is Window)
	for down in [true, false]:
		if with_touch:
			var t := InputEventScreenTouch.new()
			t.index = 0
			t.pressed = down
			t.position = pos
			vp.push_input(t)
		var m := InputEventMouseButton.new()
		m.button_index = MOUSE_BUTTON_LEFT
		m.pressed = down
		m.position = pos
		m.global_position = pos
		m.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
		vp.push_input(m)
		await _frames(2)
	await _frames(3)
	var hov := vp.gui_get_hovered_control() if vp is Viewport else null
	_note("after the click: hovered %s, search open %s" % [hov.name if hov != null else "none", _is_open()])


func _first(pred: Callable) -> Dictionary:
	_open()
	await _frames(3)
	await _query(". ")
	for r in _app._ensure_command_index().all():
		if pred.call(r):
			return r
	return {}


## Opens the search the way the form's person does: Ctrl+F (Edit > Find on map) on desktop and tablet, a tap on
## the app bar's magnifier on the phone. Returns whether the surface opened.
func _open_by_ui() -> bool:
	if _phone():
		for b in _app._phone_root.find_children("*", "Button", true, false):
			if (b as Button).accessibility_name == "Search" and (b as Button).is_visible_in_tree():
				await _click(b as Button)
				break
	else:
		for down in [true, false]:
			var k := InputEventKey.new()
			k.keycode = KEY_F
			k.physical_keycode = KEY_F
			k.ctrl_pressed = true
			k.pressed = down
			_vp.push_input(k)
			await _frames(2)
	await _frames(4)
	return _is_open()


## Hides every window the sweep's real handlers left open (a check that opens a context window, a dialog): an
## exclusive window swallows every pointer event aimed at the main viewport, which would make the real clicks
## below fail for a reason that has nothing to do with search. Named in the log so it is not a silent reset.
func _clear_windows() -> void:
	var shut: Array = []
	for w in _vp.find_children("*", "Window", true, false):
		if (w as Window).visible:
			shut.append(String((w as Window).name))
			(w as Window).hide()
	if not shut.is_empty():
		_note("windows left open by the sweep, hidden before the real clicks: %s" % str(shut))
	await _frames(3)


func _real_clicks() -> void:
	print("== real pointer events (%s) ==" % ("touch" if _touch else "mouse"))
	_close()
	await _frames(3)
	await _clear_windows()
	_ok(await _open_by_ui(), "the search opened from the form's own control (Ctrl+F, or the phone's magnifier)")
	_close()
	## A check with a state source and one without; a radio; a parameter; a plain command; a disabled one.
	var chk := await _first(func(r): return String(r["kind"]) == "menu" and bool(r.get("checkable", false)) and not bool(r.get("radio", false)) \
		and not bool(r.get("toggle", false)) and bool(r["available"]) and String(r["group"]) == "Preferences" \
		and String(r["title"]) in ["Grid", "Colors", "Labels", "Show tile borders on the map"])
	_ok(not chk.is_empty(), "real click: a plain Preferences check exists to press")
	if not chk.is_empty():
		var sig := _sig_of(_app._ensure_command_index(), chk)
		var p: PopupMenu = chk["_popup"]
		var i := _item(p, int(chk["_id"]), String(chk["_text"]))
		var before := p.is_item_checked(i)
		var row := await _present(sig)
		var b := _button_for(row)
		_ok(b != null, "real click: the %s check row is on screen" % chk["title"])
		if b != null:
			await _click(b)
			_ok(p.is_item_checked(i) != before, "real click toggled the %s check (%s to %s)" % [chk["title"], before, p.is_item_checked(i)])
			_ok(_is_open(), "real click on a check kept the surface open")
			var again := _button_for(_find_row(_app._ensure_command_index(), sig))
			if again != null:
				await _click(again)
				_ok(p.is_item_checked(i) == before, "a second real click toggled it back")
	var key := "geo.age"
	var prm := await _first(func(r): return String(r["kind"]) == "param" and String(r["key"]) == key)
	if not prm.is_empty():
		_app.select_domain("civilization")
		await _frames(3)
		var row2 := await _present(_sig_of(_app._ensure_command_index(), prm))
		var b2 := _button_for(row2)
		if b2 != null:
			await _click(b2)
			await _frames(4)
			var panel: Control = _app._workspace_panels.get("world")
			var ctl: Control = panel._param_control(key)
			_ok(String(_app.active_domain()) == "world" and ctl != null and ctl.is_visible_in_tree() and not _is_open(),
				"real click on a parameter row opened WORLD with the control visible and closed the surface (domain %s, surface open %s, shown: %s)" % [_app.active_domain(), _is_open(), _reason_shown()])
		else:
			_ok(false, "real click: the parameter row is on screen")
	var dis := await _first(func(r): return String(r["kind"]) == "menu" and not bool(r["available"]) and String(r["why"]) != "")
	if not dis.is_empty():
		var row3 := await _present(_sig_of(_app._ensure_command_index(), dis))
		var b3 := _button_for(row3)
		if b3 != null:
			await _click(b3)
			_ok(_is_open() and _reason_shown() == String(dis["why"]), "real click on a disabled row shows its reason and stays open (%s)" % _reason_shown())
	_close()


# ---- 44 dp floors ---------------------------------------------------------------------------------------------

func _floors() -> void:
	print("== touch floors ==")
	_open()
	await _frames(3)
	await _query(". ")
	var rows := _row_buttons()
	var tallest_short := 1.0e9
	for b in rows:
		tallest_short = minf(tallest_short, (b as Button).size.y)
	_note("%s: %d drawn command rows, shortest %.1f layout px" % [_form, rows.size(), tallest_short])
	_ok(rows.size() > 10, "rows were drawn to measure")
	if _form != "desktop":
		_ok(tallest_short >= 44.0, "every command row is at least 44 tall (shortest %.1f)" % tallest_short)
		var stars := _results().find_children("*PinStar*", "Button", true, false)
		var s_short := 1.0e9
		for s in stars:
			s_short = minf(s_short, (s as Button).size.x)
			s_short = minf(s_short, (s as Button).size.y)
		_note("%d pin stars, smallest side %.1f" % [stars.size(), s_short])
		_ok(stars.size() == 0 or s_short >= 44.0, "every pin star is at least 44 square (%.1f)" % s_short)
	_close()
