extends Node
## Protects: FH-0, the Faction hub's tabbed shell (`FACTION_HUB_DESIGN.md`,
## `shell/faction_roster_window.gd`). Every leg below is a claim that a refactor
## of that window could silently break, read off the live window rather than a
## constant:
##
##   TITLE   the window is called "Factions"
##   LAZY    `open()` runs neither O(cells) engine pass and builds ONE pane;
##           Territory is what fetches the terrain fits and Military is what
##           fetches the military summary
##   STRIP   all seven tabs exist, in order, each reachable on screen without a
##           horizontal scroll; the phone form is a segmented grid of cells at
##           least `DccTheme.PHONE_TAP_MIN` tall, never a `TabContainer`, and
##           the phone window keeps exactly one scroller
##   BUILD   every tab builds a non-empty pane, exactly one pane is visible, and
##           each carries the section(s) it owns (Currency/Tariffs keep their
##           `Currency_<key>` / `Tariff_<id>` node names); Relations and History
##           are honest placeholders with no controls at all
##   OPEN    `open(select_faction, tab)`: a tab id is honoured, "" or an unknown
##           id means the last tab used, an unknown faction id is ignored
##   FR02    a half-typed field commits to the faction it was typed for, both
##           when the TAB changes and when the FACTION changes -- the two ways a
##           pane can be torn down under a focused LineEdit
##   STALE   an edit that moves another tab's numbers (government) drops the
##           hidden panes and marks both caches stale; a new world (RF-03)
##           rebuilds only the visible tab and resets the selection
##
## Windowed, never `--headless` (layout and focus are the subject):
##   Godot_v4.7.1-stable_win64_console.exe --path . _factionhub_probe.tscn
##   ... _factionhub_probe.tscn -- --force-touch --vp 1080x2340      (phone)
## `FACTIONHUB_SHOT_DIR`, if set, receives PNGs of the live viewport.
##
## A run prints `PROBE-FAIL` per failed check and exits 0 either way (the suite's
## convention); read the `FACTIONHUB RESULT` line. `ABORT` (exit 2) means the
## probe could not run, which must never be read as a pass.

var _app: Node
var _bridge
var _fr
var _vp: SubViewport = null
var _checks := 0
var _fails := 0
var _phone := false
var _tablet := false


func _ok(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if cond:
		print("FACTIONHUB ok    %s" % name)
	else:
		_fails += 1
		print("FACTIONHUB FAIL  %s  -- %s" % [name, detail])
		print("PROBE-FAIL %s" % name)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _shot(tag: String) -> void:
	var dir := OS.get_environment("FACTIONHUB_SHOT_DIR")
	if dir == "":
		return
	await _frames(3)
	var tex := _vp.get_texture() if _vp != null else get_viewport().get_texture()
	tex.get_image().save_png(dir.path_join("factionhub_%s.png" % tag))


func _texts(node: Node, out: PackedStringArray) -> void:
	if node is Label:
		out.append((node as Label).text)
	elif node is RichTextLabel:
		out.append((node as RichTextLabel).get_parsed_text())
	elif node is Button:
		out.append((node as Button).text)
	for c in node.get_children():
		_texts(c, out)


func _text_of(node: Node) -> String:
	var out := PackedStringArray()
	_texts(node, out)
	return "\n".join(out)


## Counts descendants matching `pred` -- used to prove the placeholders carry no
## control and that the phone window holds one scroller.
func _count(node: Node, pred: Callable) -> int:
	var n := 1 if pred.call(node) else 0
	for c in node.get_children():
		n += _count(c, pred)
	return n


func _names() -> Dictionary:
	var out := {}
	for f in _bridge.get_factions():
		out[int(f.get("id", 0))] = String(f.get("name", "?"))
	return out


func _pane(id: String) -> Control:
	return _fr._tab_panes.get(id, null)


func _visible_panes() -> Array:
	var out: Array = []
	for id in _fr._tab_panes:
		if (_fr._tab_panes[id] as Control).visible:
			out.append(id)
	return out


## Gives a text field keyboard focus the way a user can on this form. On the
## phone the shell parks every `LineEdit` under a scroller at `FOCUS_NONE` (the
## `PgField` tap-versus-swipe arbitration, `DccWidgets.touch_focus_field`) and a
## completed tap restores its `stock_focus`; so a bare `grab_focus()` is a no-op
## there by design, not a defect. This does what the tap does -- restore the
## stock mode, then focus -- and changes nothing on desktop or tablet fields.
func _focus(le: LineEdit) -> void:
	if le.focus_mode == Control.FOCUS_NONE and le.get("stock_focus") != null:
		le.focus_mode = int(le.get("stock_focus")) as Control.FocusMode
	le.grab_focus()


## Press a strip/chooser cell the way a user does, then let layout settle.
func _press(id: String) -> void:
	var b: Button = _fr._tab_buttons[id]
	b.pressed.emit()
	await _frames(4)


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 250.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("FACTIONHUB WATCHDOG"); get_tree().quit(3))
	wd.start()

	var vp_size := Vector2i.ZERO
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a: String = args[i]
		if a == "--vp" and i + 1 < args.size():
			var wh := args[i + 1].split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
			continue
		if a == "--force-touch":
			i += 1
			continue
		print("FACTIONHUB ABORT: unknown argument '%s'" % a)
		get_tree().quit(2)
		return
	if vp_size != Vector2i.ZERO:
		_vp = SubViewport.new()
		_vp.size = vp_size
		_vp.transparent_bg = false
		_vp.gui_embed_subwindows = true
		_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
		add_child(_vp)
		_app = load("res://shell/app.tscn").instantiate()
		_vp.add_child(_app)
	else:
		_app = load("res://shell/app.tscn").instantiate()
		add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	_bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)

	_phone = DccTheme.is_phone()
	_tablet = DccTheme.is_tablet()
	print("FACTIONHUB form=%s" % ("phone" if _phone else ("tablet" if _tablet else "desktop")))
	_fr = _app.faction_roster_window
	if _bridge.get_factions().size() < 2:
		print("FACTIONHUB ABORT: fewer than two factions -- nothing to switch between")
		get_tree().quit(2)
		return
	await _run()
	print("FACTIONHUB RESULT %s  checks=%d fails=%d" % [
		"PASS" if _fails == 0 else "FAIL", _checks, _fails])
	get_tree().quit(0)


func _run() -> void:
	var ids: Array = []
	for f in _bridge.get_factions():
		ids.append(int(f.get("id", 0)))
	var fa: int = ids[0]
	var fb: int = ids[1]

	# -- TITLE / first open ----------------------------------------------------
	_ok("TITLE the window is called Factions", _fr.title == "Factions", "title='%s'" % _fr.title)
	_ok("OPEN the very first tab is identity", _fr._tab == "identity", "tab=%s" % _fr._tab)
	_ok("TAB_IDS is the seven agreed ids, in order",
		_fr.TAB_IDS == ["identity", "territory", "settlements", "economy", "military", "relations", "history"],
		str(_fr.TAB_IDS))
	_ok("signals unchanged: roster_changed and tariff_changed",
		_fr.has_signal("roster_changed") and _fr.has_signal("tariff_changed"))

	# -- LAZY --------------------------------------------------------------------
	_app.open_faction_roster()
	await _frames(8)
	_ok("LAZY open() fetched neither O(cells) pass",
		not _fr._fits_ready and not _fr._military_ready,
		"fits=%s military=%s" % [_fr._fits_ready, _fr._military_ready])
	_ok("LAZY ... and left the cached results empty", _fr._fits.is_empty() and _fr._military.is_empty(),
		"fits=%d military=%d" % [_fr._fits.size(), _fr._military.size()])
	_ok("LAZY only the active tab is built", _fr._tab_panes.keys() == ["identity"], str(_fr._tab_panes.keys()))
	await _shot("%s_identity" % ("phone" if _phone else "desktop"))

	# -- STRIP -------------------------------------------------------------------
	var all_there := true
	for id in _fr.TAB_IDS:
		if not _fr._tab_buttons.has(id):
			all_there = false
	_ok("STRIP all seven cells exist", all_there, str(_fr._tab_buttons.keys()))
	var strip: Control = _fr.find_child("TabStrip", true, false)
	_ok("STRIP the strip is in the window and visible", strip != null and strip.is_visible_in_tree())
	var win_rect := Rect2(Vector2.ZERO, Vector2(_fr.size))
	var on_screen := true
	var worst := ""
	for id in _fr.TAB_IDS:
		var b: Button = _fr._tab_buttons[id]
		var r := b.get_global_rect()
		if r.size.x < 8.0 or r.position.x < -0.5 or r.end.x > win_rect.end.x + 0.5:
			on_screen = false
			worst = "%s %s in window %s" % [id, r, win_rect]
	_ok("STRIP every cell is on screen horizontally with no scrolling", on_screen, worst)
	var h_scrollers := _count(_fr, func(n): return n is ScrollContainer \
		and (n as ScrollContainer).horizontal_scroll_mode != ScrollContainer.SCROLL_MODE_DISABLED)
	_ok("STRIP no horizontally scrolling container anywhere in the window", h_scrollers == 0,
		"%d scrollers allow horizontal scroll" % h_scrollers)
	if _phone:
		var tabcontainers := _count(_fr, func(n): return n is TabContainer or n is TabBar)
		_ok("PHONE the chooser is segmented, not a TabContainer/TabBar", tabcontainers == 0, str(tabcontainers))
		var grid := _count(strip, func(n): return n is GridContainer)
		_ok("PHONE the chooser is a grid of cells", grid == 1, str(grid))
		var small := ""
		for id in _fr.TAB_IDS:
			var b: Button = _fr._tab_buttons[id]
			if b.size.y + 0.5 < DccTheme.PHONE_TAP_MIN or b.custom_minimum_size.y < DccTheme.PHONE_TAP_MIN:
				small += "%s=%.0f/%.0f " % [id, b.size.y, b.custom_minimum_size.y]
		_ok("PHONE every cell is at least 44 px tall", small == "", small)
		var scrollers := _count(_fr, func(n): return n is ScrollContainer and (n as ScrollContainer).is_visible_in_tree())
		_ok("PHONE the window keeps exactly one scroller", scrollers == 1, "%d" % scrollers)
	elif _tablet:
		var short := ""
		for id in _fr.TAB_IDS:
			var b: Button = _fr._tab_buttons[id]
			if b.custom_minimum_size.y < DccTheme.role_px("btn_min_h"):
				short += "%s=%.0f " % [id, b.custom_minimum_size.y]
		_ok("TABLET every cell takes role_px(btn_min_h), not a literal", short == "", short)

	# -- BUILD, every tab --------------------------------------------------------
	for id in _fr.TAB_IDS:
		await _press(id)
		var pane := _pane(id)
		_ok("BUILD %s: built, visible, and the only visible pane" % id,
			pane != null and pane.visible and _visible_panes() == [id] and _fr._tab == id,
			"tab=%s panes=%s visible=%s" % [_fr._tab, _fr._tab_panes.keys(), _visible_panes()])
		if pane == null:
			continue
		var t := _text_of(pane)
		_ok("BUILD %s: non-empty (>= 40 chars of text)" % id, t.length() >= 40, "len=%d" % t.length())
		_ok("BUILD %s: the strip's active cell is this tab" % id,
			(_fr._tab_buttons[id] as Button).get_theme_color("font_color") == DccTheme.c("accent"))
		await _shot("%s_%s" % [("phone" if _phone else "desktop"), id])
	await _press("identity")
	var ident := _pane("identity")
	var ident_text := _text_of(ident)
	_ok("BUILD identity owns name, colour, government, culture, religion, ag. tech",
		_count(ident, func(n): return n is LineEdit) >= 1 and ident_text.contains("Colour")
		and ident_text.contains("Government") and ident_text.contains("Culture")
		and ident_text.contains("Religion") and ident_text.contains("Ag. technology"), ident_text.left(200))
	_ok("BUILD identity does NOT carry the other tabs' sections",
		not ident_text.to_lower().contains("currency") and not ident_text.to_lower().contains("tariffs")
		and not ident_text.to_lower().contains("territory fit") and not ident_text.contains("Power:"))
	await _press("territory")
	## Section titles are drawn upper-cased by `DccTheme.header()`, so compare
	## the section/group titles case-insensitively; body notes keep their case.
	var ter_text := _text_of(_pane("territory"))
	_ok("BUILD territory owns the fit and the overview", ter_text.to_lower().contains("territory fit")
		and ter_text.contains("Capital:"), ter_text.left(200))
	await _press("settlements")
	_ok("BUILD settlements owns the sublist", _text_of(_pane("settlements")).to_lower().contains("settlements ("),
		_text_of(_pane("settlements")).left(120))
	await _press("economy")
	var eco := _pane("economy")
	var eco_text := _text_of(eco).to_lower()
	_ok("BUILD economy owns Currency, Tariffs and the Not-built note",
		eco_text.contains("currency") and eco_text.contains("tariffs") and eco_text.contains("not built"), eco_text.left(200))
	_ok("BUILD economy keeps the Currency_<key> and Tariff_<id> node names",
		eco.find_child("Currency_name", true, false) != null
		and eco.find_child("Currency_symbol", true, false) != null
		and eco.find_child("Currency_rate", true, false) != null
		and eco.find_child("Tariff_%d" % fb, true, false) != null)
	await _press("military")
	_ok("BUILD military owns the military block", _text_of(_pane("military")).contains("Power:"),
		_text_of(_pane("military")).left(200))
	for id in ["relations", "history"]:
		await _press(id)
		var pane := _pane(id)
		var controls := _count(pane, func(n): return n is Button or n is LineEdit or n is OptionButton \
			or n is CheckBox or n is ColorPickerButton or n is SpinBox)
		_ok("BUILD %s is a placeholder: says so, carries no control at all" % id,
			_text_of(pane).to_lower().contains("not built") and controls == 0, "controls=%d" % controls)

	# -- LAZY, the order the fetches happen in ------------------------------------
	_app.open_faction_roster(-1, "identity")
	await _frames(6)
	_ok("LAZY reopened on identity: still neither pass", not _fr._fits_ready and not _fr._military_ready)
	await _press("territory")
	_ok("LAZY Territory fetched the fits and not the military summary",
		_fr._fits_ready and not _fr._military_ready and not _fr._fits.is_empty(),
		"fits=%s military=%s n=%d" % [_fr._fits_ready, _fr._military_ready, _fr._fits.size()])
	await _press("military")
	_ok("LAZY Military fetched the military summary", _fr._military_ready and not _fr._military.is_empty())

	# -- OPEN(select, tab) -------------------------------------------------------
	_app.open_faction_roster(-1, "economy")
	await _frames(6)
	_ok("OPEN a tab id is honoured", _fr._tab == "economy" and _pane("economy") != null, "tab=%s" % _fr._tab)
	_app.open_faction_roster(-1, "")
	await _frames(4)
	_ok("OPEN empty tab means the last tab used", _fr._tab == "economy", "tab=%s" % _fr._tab)
	_app.open_faction_roster(-1, "nonsense")
	await _frames(4)
	_ok("OPEN unknown tab means the last tab used, not a blank pane",
		_fr._tab == "economy" and _visible_panes() == ["economy"], "tab=%s %s" % [_fr._tab, _visible_panes()])
	_app.open_faction_roster(fb, "military")
	await _frames(6)
	_ok("OPEN faction and tab together", _fr._selected == fb and _fr._tab == "military",
		"sel=%d tab=%s" % [_fr._selected, _fr._tab])
	_app.open_faction_roster(99999, "territory")
	await _frames(6)
	_ok("OPEN an unknown faction id is ignored (the tab still applies)",
		_fr._selected == fb and _fr._tab == "territory", "sel=%d tab=%s" % [_fr._selected, _fr._tab])
	_fr.open(fa)
	await _frames(6)
	_ok("OPEN select alone keeps the tab", _fr._selected == fa and _fr._tab == "territory")

	# -- FR-02: a tab switch with a focused currency field ------------------------
	_fr.open(fa, "economy")
	await _frames(8)
	var cur := _fr._inspector_body.find_child("Currency_name", true, false) as LineEdit
	if cur == null:
		_ok("FR02 precondition: the Currency name field exists", false, "not found")
	else:
		_focus(cur)
		await _frames(3)
		_ok("FR02 precondition: the field really has focus", cur.has_focus(),
			"visible_in_tree=%s focus_mode=%d editable=%s owner_focus=%s" % [
				cur.is_visible_in_tree(), cur.focus_mode, cur.editable,
				str(cur.get_viewport().gui_get_focus_owner())])
		cur.text = "Zfh0a"
		await _press("military")
		await _frames(6)
		var ca: Dictionary = _bridge.civ_faction_currency(fa)
		var cb: Dictionary = _bridge.civ_faction_currency(fb)
		_ok("FR02 a tab switch commits the typed currency to the faction it was typed for",
			String(ca.get("name", "")) == "Zfh0a", "A=%s" % str(ca))
		_ok("FR02 ... and to nobody else", String(cb.get("name", "")) != "Zfh0a", "B=%s" % str(cb))
		_ok("FR02 the tab did change", _fr._tab == "military")
		_bridge.civ_set_faction_currency(fa, "name", "")

	# -- FR-02: a faction switch with a focused field on another tab ----------------
	_fr.open(fa, "economy")
	await _frames(8)
	cur = _fr._inspector_body.find_child("Currency_name", true, false) as LineEdit
	if cur != null:
		_focus(cur)
		await _frames(3)
		_ok("FR02 precondition (faction switch): the field has focus", cur.has_focus())
		cur.text = "Zfh0b"
		var row_btn: Button = null
		var rows: Array = []
		_collect_buttons(_fr._list_body, rows)
		for b in rows:
			if (b as Button).text.find("—") >= 0 and (b as Button).text.find(String(_names()[fa])) < 0:
				row_btn = b
				break
		if row_btn == null:
			_ok("FR02 precondition: a second faction row exists", false)
		else:
			row_btn.pressed.emit()
			await _frames(8)
			var ca2: Dictionary = _bridge.civ_faction_currency(fa)
			var cb2: Dictionary = _bridge.civ_faction_currency(_fr._selected)
			_ok("FR02 clicking another faction commits the field to the one it was typed for",
				String(ca2.get("name", "")) == "Zfh0b", "A=%s" % str(ca2))
			_ok("FR02 ... and the faction switched to is untouched",
				_fr._selected != fa and String(cb2.get("name", "")) != "Zfh0b", "sel=%d B=%s" % [_fr._selected, str(cb2)])
			_ok("FR02 the tab survives a faction switch", _fr._tab == "economy" and _visible_panes() == ["economy"])
			_bridge.civ_set_faction_currency(fa, "name", "")

	# -- FR-02: the name field on Identity ------------------------------------------
	_fr.open(fa, "identity")
	await _frames(8)
	var edits: Array = []
	_collect_of(_fr._inspector_body, edits, LineEdit)
	if edits.is_empty():
		_ok("FR02 precondition: the Identity name field exists", false)
	else:
		var ne := edits[0] as LineEdit
		var before: String = _names()[fa]
		_focus(ne)
		await _frames(3)
		_ok("FR02 precondition (rename): the name field has focus", ne.has_focus())
		ne.text = "Renamed fh0"
		await _press("settlements")
		_ok("FR02 a tab switch commits a half-typed rename to its own faction",
			_names()[fa] == "Renamed fh0" and _names()[fb] != "Renamed fh0",
			str(_names()))
		_bridge.civ_set_faction_field(fa, "name", before)
		_fr._rebuild()
		await _frames(4)

	# -- STALE -------------------------------------------------------------------
	_fr.open(fa, "identity")
	await _frames(6)
	await _press("military")
	await _press("territory")
	await _press("identity")
	_ok("STALE precondition: three panes built", _fr._tab_panes.size() == 3, str(_fr._tab_panes.keys()))
	var gov_key := ""
	var cur_gov: String = String(_fr._faction(fa).get("government", ""))
	for e in _bridge.civ_government_vocabulary():
		if String((e as Dictionary).get("key", "")) != cur_gov:
			gov_key = String((e as Dictionary).get("key", ""))
			break
	_fr._set_field("government", gov_key)
	await _frames(4)
	_ok("STALE a government edit drops the hidden panes and keeps the visible one",
		_fr._tab_panes.keys() == ["identity"], str(_fr._tab_panes.keys()))
	_ok("STALE ... and marks both caches stale", not _fr._fits_ready and not _fr._military_ready)
	_ok("STALE ... and the engine took the edit", String(_fr._faction(fa).get("government", "")) == gov_key)

	# -- RF-03: a new world ----------------------------------------------------------
	_fr.open(fb, "military")
	await _frames(6)
	await _press("economy")
	await _press("military")
	_ok("RF03 precondition: panes built and the selection is not the default",
		_fr._tab_panes.size() >= 2 and _fr._selected == fb, "%s sel=%d" % [str(_fr._tab_panes.keys()), _fr._selected])
	_bridge.generate({
		"seed": 71123, "width_km": 1600.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	await _frames(8)
	_ok("RF03 a new world resets the selection to 1", _fr._selected == 1, "sel=%d" % _fr._selected)
	_ok("RF03 ... rebuilds the visible tab and invalidates the others",
		_fr._tab_panes.keys() == ["military"] and _fr._tab == "military", str(_fr._tab_panes.keys()))
	_ok("RF03 ... and the rebuilt tab drew the new world's numbers",
		_text_of(_pane("military")).contains("Power:") and _fr._military_ready)
	await _shot("%s_after_generate" % ("phone" if _phone else "desktop"))
	_fr.hide()


func _collect_buttons(node: Node, out: Array) -> void:
	if node is Button:
		out.append(node)
	for c in node.get_children():
		_collect_buttons(c, out)


func _collect_of(node: Node, out: Array, cls) -> void:
	if is_instance_of(node, cls):
		out.append(node)
	for c in node.get_children():
		_collect_of(c, out, cls)
