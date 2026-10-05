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
##   ENTRY   (FH-1, `_run_entry_points`) every shipped way in lands on the
##           faction and tab `FACTION_HUB_DESIGN.md` §4 gives it: the Factions
##           category's "Open factions…" and Data ▸ Factions… (bare: last faction
##           and tab), the dock card's "Open in Factions…" (Identity, or Relations
##           with `pair_with`), every Military row, and the
##           Cartography identity-colours jump. The Relationships pair rows are asserted
##           to keep their RL-01 dock behaviour (dock card on a, b marked, hub not
##           opened) until FH-6 re-routes them. The context-card row is
##           `_ctxcard_probe.gd` / `_ctxphone_probe.gd`'s.
##   CULTURE (FH-2, `_run_culture_fold`) the retired Culture profiles window's job
##           on the Identity tab: the profile, the Name pool of real settlements
##           with reroll chips, "Also used by", the ported
##           `_cultureprofiles_probe.gd` checks (seven cultures; the real picker
##           writes the engine; the surface follows), the FR-02 guards, the dock
##           link, and the old window's absence.
##   DEFTYPE (FH-3, `_run_default_type`) the "Default settlement type" picker on
##           the Identity tab, over the real `SettlementTypeStore`: None first,
##           a pick per faction that survives switching faction and back, None
##           restoring, a deleted type (or a stale id) showing None, the FR-02
##           guard, and the Settlement types window's per-faction column being
##           gone in favour of a link that opens Identity.
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
var _entry_done := false  ## set by the last line of `_run_entry_points`
var _culture_done := false  ## set by the last line of `_run_culture_fold`
var _deftype_done := false  ## set by the last line of `_run_default_type`


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

	# -- ENTRY: every shipped way in (FH-1) -------------------------------------
	await _run_entry_points(ids)
	_ok("FH1 the entry-point leg ran to its end (a script error inside it aborts it silently)", _entry_done)

	# -- CULTURE: the Culture profiles window folded into Identity (FH-2) ---------
	await _run_culture_fold(ids)
	_ok("FH2 the culture-fold leg ran to its end (a script error inside it aborts it silently)", _culture_done)

	# -- DEFTYPE: the default settlement type moved onto Identity (FH-3) ------------
	await _run_default_type(ids)
	_ok("FH3 the default-type leg ran to its end (a script error inside it aborts it silently)", _deftype_done)

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


# -- FH-1: entry points -------------------------------------------------------
## Protects: FH-1 (`FACTION_HUB_DESIGN.md` §4) -- every shipped way into the
## Factions hub names the faction and tab the design gives it, and none opens it
## "somewhere". Each entry is parked-from first (the hub is opened on a DIFFERENT
## faction and tab, then hidden), so "lands on X" cannot pass by the window
## merely still sitting there; every faction-id entry also reads the phone detail
## screen (`_phone_list_pane` hidden) where the form is the phone's.
## What it cannot see: the context-card row (`_ctxcard` / `_ctxphone` own it,
## they need a real right-click / long-press) and the placeholder tabs' content
## beyond "non-empty, no controls" (FH-6 / FH-8 own that).
func _run_entry_points(ids: Array) -> void:
	var fa: int = ids[0]
	var fb: int = ids[1]
	var park_tab := "economy"

	# 1. Factions category: "Open factions…", Culture/Settlement types kept.
	await _park(fb, park_tab)
	## select_domain first: on the phone the CIVIL dock is not built until the
	## domain is entered (`_civilcensus_probe` enters through MORE first too).
	_app.select_domain("civilization")
	await _frames(4)
	await _sheet("left")
	_app.select_domain_category("civilization", "Factions")
	await _frames(4)
	var cpanel: Control = _app.workspace_panel("civilization")
	var cbtns: Array = []
	_collect_buttons(cpanel, cbtns)
	_ok("FH1 cat: the CIVIL dock drew buttons at all (the control for every absence below)", cbtns.size() > 5, str(cbtns.size()))
	var open_btns := cbtns.filter(func(b): return _lc(b) == "open factions…")
	var old_btns := cbtns.filter(func(b): return _lc(b) == "faction roster…")
	var keep_culture := cbtns.filter(func(b): return _lc(b) == "culture profiles…")
	var keep_types := cbtns.filter(func(b): return _lc(b) == "settlement types…")
	_ok("FH1 cat: exactly one 'Open factions…' button", open_btns.size() == 1,
		"%d; first buttons: %s" % [open_btns.size(), _btn_texts(cbtns, 14)])
	_ok("FH1 cat: the old 'Faction roster…' label is gone", old_btns.is_empty(), str(old_btns.size()))
	_ok("FH1 cat: Culture profiles… is retired (FH-2) and Settlement types… is still there (FH-3)",
		keep_culture.is_empty() and keep_types.size() == 1, "%d/%d" % [keep_culture.size(), keep_types.size()])
	if open_btns.size() == 1:
		(open_btns[0] as Button).pressed.emit()
		await _frames(6)
		_land("FH1 cat bare open (last faction, last tab)", fb, park_tab, false)
	_fr.hide()
	await _frames(2)

	# 2. Data > Factions…
	await _park(fb, park_tab)
	var mb := _find_menu_button(_app, "Data")
	_ok("FH1 data: the Data menu exists", mb != null)
	if mb != null:
		var popup: PopupMenu = mb.get_popup()
		var idx := -1
		var n_rows := 0
		for i in popup.item_count:
			if popup.get_item_id(i) == DccMenus.ID_FACTIONS:
				idx = i
				n_rows += 1
		_ok("FH1 data: exactly one row carries ID_FACTIONS", n_rows == 1, str(n_rows))
		if idx >= 0:
			_ok("FH1 data: the row reads 'Factions…'", popup.get_item_text(idx) == "Factions…", popup.get_item_text(idx))
			popup.id_pressed.emit(DccMenus.ID_FACTIONS)
			await _frames(6)
			_land("FH1 data bare open (last faction, last tab)", fb, park_tab, false)
	_fr.hide()
	await _frames(2)

	# 3. Right dock Faction card.
	var dock = _app.right_dock_ctrl  ## a RightDock is a plain Node, not a Control
	for pair in [-1, fb]:
		var want_tab := "identity" if int(pair) < 0 else "relations"
		await _park(fb, park_tab)
		await _sheet("right")
		dock.show_faction(fa, int(pair))
		await _frames(4)
		var dbtns: Array = []
		_collect_buttons(_app.right_dock_body, dbtns)
		var dact := dbtns.filter(func(b): return _lc(b) == "open in factions…")
		var tag := "FH1 dock pair_with=%d" % int(pair)
		_ok("%s: exactly one 'Open in Factions…' action" % tag, dact.size() == 1,
			"%d; buttons: %s" % [dact.size(), _btn_texts(dbtns, 14)])
		if dact.size() == 1:
			var db: Button = dact[0]
			if _phone:
				_ok("%s: the row is a phone-safe tap target (>= %d px)" % [tag, DccTheme.PHONE_TAP_MIN],
					db.get_combined_minimum_size().y >= DccTheme.PHONE_TAP_MIN,
					str(db.get_combined_minimum_size()))
			if int(pair) < 0:
				await _shot("%s_dock_faction_card" % ("phone" if _phone else "desktop"))
			db.pressed.emit()
			await _frames(6)
			_land(tag, fa, want_tab, true)
		_fr.hide()
		await _frames(2)

	# 4. Military > Faction strength rows -> (f, military).
	await _sheet("left")
	_app.select_domain_category("civilization", "Military")
	await _frames(4)
	var mbtns: Array = []
	_collect_buttons(_app.workspace_panel("civilization"), mbtns)
	var names := _names()
	var strength_rows := mbtns.filter(func(b): return _lc(b).contains("/100 · ") and _lc(b).contains("fortified"))
	_ok("FH1 military: one row per faction", strength_rows.size() == names.size() and names.size() > 0,
		"%d rows, %d factions" % [strength_rows.size(), names.size()])
	var seen_m := {}
	for b in strength_rows:
		var fid := -1
		for k in names:
			if _lc(b).begins_with(String(names[k]).to_lower() + " -- "):
				fid = int(k)
		_ok("FH1 military: row '%s' names a known faction" % _lc(b).left(30), fid >= 0)
		if fid < 0:
			continue
		seen_m[fid] = true
		await _park(fb if fid != fb else fa, park_tab)
		(b as Button).pressed.emit()
		await _frames(6)
		_land("FH1 military row f=%d" % fid, fid, "military", true)
		_fr.hide()
		await _frames(2)
	_ok("FH1 military: the rows covered every faction", seen_m.size() == names.size(), str(seen_m.keys()))

	# 5. Relationships > Every pair rows -> the DOCK card on a with b marked
	# (RL-01), NOT the hub. FH-1 first routed these to the hub's Relations tab;
	# the coordinator's review reversed that because the tab is a placeholder
	# until FH-6 and the route lost the second faction. Protects: the pair rows
	# keep opening the faction card with the named pair marked, and do not open
	# the hub. FH-6 re-routes them and rewrites this leg.
	await _sheet("left")
	_app.select_domain_category("civilization", "Relationships")
	await _frames(4)
	var pairs: Array = _bridge.civ_faction_relations()
	var rbtns: Array = []
	_collect_buttons(_app.workspace_panel("civilization"), rbtns)
	var pair_rows := rbtns.filter(func(b): return _lc(b).contains(" ↔ "))
	_ok("FH1 relations: one row per pair", pair_rows.size() == pairs.size() and pairs.size() > 0,
		"%d rows, %d pairs" % [pair_rows.size(), pairs.size()])
	var n_rel := 0
	for b in pair_rows:
		if n_rel >= 3:
			break
		var want_a := -1
		for d in pairs:
			var pd: Dictionary = d
			if _lc(b).begins_with(("%s ↔ %s -- " % [String(pd.get("a_name", "?")), String(pd.get("b_name", "?"))]).to_lower()):
				want_a = int(pd.get("a", -1))
		_ok("FH1 relations: row '%s' matches a pair" % _lc(b).left(30), want_a >= 0)
		if want_a < 0:
			continue
		var want_b := -1
		var want_b_name := ""
		for d in pairs:
			var pd2: Dictionary = d
			if int(pd2.get("a", -1)) == want_a and _lc(b).begins_with(("%s ↔ %s -- " % [String(pd2.get("a_name", "?")), String(pd2.get("b_name", "?"))]).to_lower()):
				want_b = int(pd2.get("b", -1))
				want_b_name = String(pd2.get("b_name", "?"))
		# Park the hub hidden and the dock on another faction with no pair, so
		# both "the hub stays shut" and "the press changed the dock" are real.
		await _park(fb if want_a != fb else fa, park_tab)
		await _sheet("right")
		_app.right_dock_ctrl.show_faction(fb if want_a != fb else fa, -1)
		await _frames(3)
		await _sheet("left")
		(b as Button).pressed.emit()
		await _frames(6)
		await _sheet("right")
		await _frames(3)
		var tag5 := "FH1 relations row a=%d b=%d" % [want_a, want_b]
		var dk = _app.right_dock_ctrl
		_ok("%s: the dock shows the faction card" % tag5, dk._context == RightDock.CTX_FACTION, str(dk._context))
		_ok("%s: the card is headed by a" % tag5, dk._faction_id == want_a, "id=%d" % dk._faction_id)
		_ok("%s: the card was told b as the pair (RL-01)" % tag5, dk._faction_pair == want_b and want_b >= 0, "pair=%d" % dk._faction_pair)
		_ok("%s: b is marked among the card's relations ('▸ %s')" % [tag5, want_b_name],
			_text_of(_app.right_dock_body).to_lower().contains(("▸ " + want_b_name).to_lower()))
		_ok("%s: the hub was not opened (FH-6 owns that route)" % tag5, not _fr.visible)
		n_rel += 1
	_ok("FH1 relations: the loop pressed at least one row", n_rel > 0, str(n_rel))

	# 6. Cartography > Feature style > Territories -> (last faction, identity).
	await _park(fb, park_tab)
	_app.select_domain("cartography")
	await _frames(4)
	await _sheet("left")
	_app.select_domain_category("cartography", "Feature style")
	await _frames(4)
	var kbtns: Array = []
	_collect_buttons(_app.workspace_panel("cartography"), kbtns)
	var colour := kbtns.filter(func(b): return _lc(b).begins_with("faction identity colours"))
	_ok("FH1 carto: exactly one 'Faction identity colours' jump", colour.size() == 1, str(colour.size()))
	if colour.size() == 1:
		(colour[0] as Button).pressed.emit()
		await _frames(6)
		_land("FH1 carto jump (last faction, Identity tab)", fb, "identity", false)
	_fr.hide()
	await _frames(2)
	_entry_done = true


## On the phone the two docks are sheets, shown one at a time; their bodies are
## built once but a dock that is not on screen does not fill its categories
## (`ebdf6fdb`), so the sheet is opened before a button is looked for. A no-op on
## desktop and tablet, where the docks are docked.
func _sheet(side: String) -> void:
	if _phone:
		_app._set_sheet_open(side, true)
		await _frames(4)


## A button label lower-cased: the phone shell draws button labels in capitals
## (`OPEN IN FACTIONS…`), so a label is compared case-blind everywhere here.
func _lc(b) -> String:
	return (b as Button).text.to_lower()


## The first `n` button labels, for a failure message that says what was there.
func _btn_texts(btns: Array, n: int) -> String:
	var out := PackedStringArray()
	for b in btns.slice(0, n):
		out.append(_lc(b))
	return str(out)


## Parks the hub on (`fid`, `tab`) and hides it, then asserts it is parked.
func _park(fid: int, tab: String) -> void:
	_fr.open(fid, tab)
	await _frames(4)
	_fr.hide()
	await _frames(2)
	_ok("FH1 park precondition (%d, %s)" % [fid, tab],
		_fr._selected == fid and _fr._tab == tab and not _fr.visible,
		"sel=%d tab=%s visible=%s" % [_fr._selected, _fr._tab, _fr.visible])


## Asserts the hub is showing `fid` on `tab`, with exactly that pane visible; on
## the phone, with `detail` set, that the inspector (not the master list) shows.
func _land(tag: String, fid: int, tab: String, detail: bool) -> void:
	_ok("%s: the hub is showing" % tag, _fr.visible)
	_ok("%s: landed on faction %d" % [tag, fid], _fr._selected == fid, "sel=%d" % _fr._selected)
	_ok("%s: landed on tab %s" % [tag, tab], _fr._tab == tab, "tab=%s" % _fr._tab)
	_ok("%s: exactly that pane is visible" % tag, _visible_panes() == [tab], str(_visible_panes()))
	if _phone and detail:
		_ok("%s: the phone shows the inspector, not the master list" % tag,
			not (_fr._phone_list_pane as Control).visible)


func _find_menu_button(n: Node, title: String) -> MenuButton:
	if n is MenuButton and (n as MenuButton).text == title:
		return n
	for c in n.get_children(true):
		var r := _find_menu_button(c, title)
		if r != null:
			return r
	return null


# -- FH-2: the Culture profiles window folded into Identity -------------------

## Expected card contents for faction `fid`, derived from the engine directly
## (`get_factions()` + `settlements()`), never from the card's own helpers --
## the card is the thing under test, so it cannot also be the oracle.
## `matches` are `{index, pop, name}` of settlements of factions on the same
## culture, `others` the other factions on it.
func _fold_expect(fid: int) -> Dictionary:
	var key := String(_fr._faction(fid).get("culture", ""))
	var on_culture := {}
	var others: Array = []
	for f in _bridge.get_factions():
		if String(f.get("culture", "")) == key:
			on_culture[int(f.get("id", 0))] = true
			if int(f.get("id", 0)) != fid:
				others.append(f)
	var matches: Array = []
	var all: Array = _bridge.settlements()
	for i in all.size():
		var s: Dictionary = all[i]
		if on_culture.has(int(s.get("faction", -1))):
			matches.append({"index": i, "pop": int(s.get("population", 0)), "name": String(s.get("name", ""))})
	var row := {}
	for c in _bridge.get_cultures():
		if String(c.get("key", "")) == key:
			row = c
	return {"key": key, "others": others, "matches": matches, "row": row}


## The Identity pane's Culture `OptionButton`: the one whose items are exactly the
## capitalised engine vocabulary (the pane also holds religion/government/ag.
## tech pickers, which share the class).
func _culture_picker() -> OptionButton:
	var obs: Array = []
	_collect_of(_pane("identity"), obs, OptionButton)
	var want: Array = []
	for k in _bridge.civ_culture_vocabulary():
		want.append(String(k).capitalize())
	for ob in obs:
		var texts: Array = []
		for i in (ob as OptionButton).item_count:
			texts.append((ob as OptionButton).get_item_text(i))
		if texts == want:
			return ob
	return null


## The Name-pool chips under the card, in draw order.
func _pool_chips() -> Array:
	var host: Node = _fr._culture_card_host
	if host == null or not is_instance_valid(host):
		return []
	var out: Array = []
	for n in host.find_children("Reroll_*", "Button", true, false):
		out.append(n)
	return out


## Asserts the card on the Identity pane matches the engine for `fid`.
func _fold_check_card(tag: String, fid: int) -> void:
	var ex := _fold_expect(fid)
	var host: Control = _fr._culture_card_host
	var live := host != null and is_instance_valid(host) and host.is_inside_tree()
	_ok("%s: the Culture card exists inside the Identity pane" % tag,
		live and _pane("identity") != null and _pane("identity").is_ancestor_of(host))
	if not live:
		return
	_ok("%s: exactly one card in the window" % tag,
		_fr.find_children("CultureCard", "", true, false).size() == 1)
	var txt := _text_of(host)
	var row: Dictionary = ex.row
	_ok("%s: the profile names the faction's culture (%s)" % [tag, ex.key],
		not row.is_empty() and txt.contains("Culture profile — %s" % String(row.get("name", "?"))), txt.left(160))
	var aff := String(row.get("terrain_affinity", ""))
	_ok("%s: terrain theme read from terrain_affinity (%s)" % [tag, aff],
		txt.contains("Terrain theme: %s." % aff.capitalize()) if aff != "" else txt.contains("No terrain theme"), txt.left(260))
	var fc := int(row.get("faction_count", 0))
	_ok("%s: the reach line states faction_count (%d)" % [tag, fc], txt.contains("%d faction" % fc))
	var matches: Array = ex.matches
	var chips := _pool_chips()
	_ok("%s: the culture has real settlements (the control for the pool checks)" % tag, matches.size() > 0)
	_ok("%s: Name pool shows min(8, real settlements) chips" % tag,
		chips.size() == mini(8, matches.size()), "chips=%d real=%d" % [chips.size(), matches.size()])
	var real_names := {}
	for m in matches:
		real_names[int(m.index)] = String(m.name)
	var names_ok := true
	var sorted_ok := true
	var last_pop := 1 << 30
	var all: Array = _bridge.settlements()
	for c in chips:
		var idx := int(String((c as Button).name).trim_prefix("Reroll_"))
		if not real_names.has(idx) or (c as Button).text != String(real_names[idx]):
			names_ok = false
		var p := int((all[idx] as Dictionary).get("population", 0))
		if p > last_pop:
			sorted_ok = false
		last_pop = p
	_ok("%s: every chip is a real settlement of a faction on this culture, by its real name" % tag, names_ok)
	_ok("%s: chips are sorted by population, largest first" % tag, sorted_ok)
	var others: Array = ex.others
	_ok("%s: 'Also used by (%d)' header" % [tag, others.size()], txt.to_lower().contains("also used by (%d)" % others.size()), txt)
	var also: Node = host.find_child("AlsoUsedBy", true, false)
	_ok("%s: the Also-used-by group exists" % tag, also != null)
	if also != null:
		var banners := _count(also, func(n): return n is FactionBanner)
		_ok("%s: one banner row per other faction (%d)" % [tag, others.size()], banners == others.size(), str(banners))
		var listed := true
		for f in others:
			if not _text_of(also).contains(String(f.get("name", "?"))):
				listed = false
		_ok("%s: each other faction is named" % tag, listed)
		if others.is_empty():
			_ok("%s: no-others state says so" % tag, _text_of(also).contains("No other faction uses this culture"))
	if _phone:
		var small := 0
		var worst := ""
		var btns: Array = []
		_collect_buttons(host, btns)
		for b in btns:
			if (b as Button).visible and (b as Button).size.y < DccTheme.PHONE_TAP_MIN - 0.5:
				small += 1
				worst = "%s h=%.1f" % [(b as Button).text, (b as Button).size.y]
		_ok("%s: PHONE every tappable in the card is at least %d px tall" % [tag, DccTheme.PHONE_TAP_MIN],
			small == 0 and btns.size() > 0, "%d too small, e.g. %s (of %d)" % [small, worst, btns.size()])


var _roster_emits := 0


## Protects: FH-2 (owner decision 2, `FACTION_HUB_DESIGN.md`) -- the retired
## `CultureProfilesWindow`'s whole job now lives on the Identity tab, on desktop
## and phone. Ported from the deleted `_cultureprofiles_probe.gd`: the culture
## list is real and complete (7), the REAL culture `OptionButton` (not the
## private setter) changes one faction's culture and a fresh `get_factions()`
## re-read shows the write reached the engine, and the surface reflects it
## without being reopened. New: the card's profile (terrain theme, faction
## count), its Name pool (real settlements, population order, max 8, each chip a
## reroll through `civ_reroll_settlement_name`), its "Also used by" list, the
## FR-02 guards (a half-typed field survives a reroll and is committed by a
## culture pick), the Culture dock link opening Identity, and the old window,
## opener and Factions-category button being gone. A reroll must emit
## `roster_changed` (map labels) and drop the hidden panes (the Settlements tab
## lists the names).
## What it cannot see: the Culture dock category's read-only list (unchanged,
## `_civilcensus_probe`'s) or pixels (the screenshot is the look-and-feel check).
func _run_culture_fold(ids: Array) -> void:
	var fa := -1
	for f in _bridge.get_factions():
		if int(f.get("settlement_count", 0)) > 0:
			fa = int(f.get("id", 0))
			break
	_ok("FH2 precondition: a faction with settlements exists", fa > 0)
	if fa < 0:
		return
	var fb := -1
	for id in ids:
		if int(id) != fa:
			fb = int(id)
			break
	var orig_a := String(_fr._faction(fa).get("culture", "common"))
	var orig_b := String(_fr._faction(fb).get("culture", "common"))
	var name_a: String = _names()[fa]
	_fr.roster_changed.connect(func(): _roster_emits += 1)

	# -- ported: the culture list is real and complete -------------------------
	var cultures: Array = _bridge.get_cultures()
	_ok("FH2 get_cultures() returns the seven cultures", cultures.size() == 7, str(cultures.size()))
	var any_terrain := false
	for c in cultures:
		if String(c.get("terrain_affinity", "")) != "":
			any_terrain = true
	_ok("FH2 ... and the rows carry real terrain affinities", any_terrain)

	# -- the card on the faction's current culture ---------------------------------
	_fr.open(fa, "identity")
	await _frames(8)
	var picker := _culture_picker()
	_ok("FH2 the Identity tab's Culture picker offers all seven cultures", picker != null and picker.item_count == 7)
	_fold_check_card("FH2 card (%s)" % orig_a, fa)
	await _shot("%s_culture_card" % ("phone" if _phone else "desktop"))

	# -- Also used by: share the culture, then read it ------------------------------
	_bridge.civ_set_faction_field(fb, "culture", orig_a)
	_fr._mark_data_stale()
	_fr._rebuild_inspector()
	await _frames(6)
	var ex := _fold_expect(fa)
	_ok("FH2 shared: the engine puts the other faction on this culture", (ex.others as Array).size() >= 1, str((ex.others as Array).size()))
	_fold_check_card("FH2 card shared with %s" % _names()[fb], fa)
	await _shot("%s_culture_card_shared" % ("phone" if _phone else "desktop"))
	_bridge.civ_set_faction_field(fb, "culture", orig_b)

	# -- ported: the REAL picker changes the culture and the engine re-reads it -------
	var new_key := ""
	for c in cultures:
		if int(c.get("faction_count", 0)) == 0 and String(c.get("key", "")) != orig_a:
			new_key = String(c.get("key", ""))
			break
	_ok("FH2 an unused culture exists to switch to", new_key != "")
	_fr._mark_data_stale()
	_fr._rebuild_inspector()
	await _frames(6)
	picker = _culture_picker()
	var idx := -1
	var keys: Array = _bridge.civ_culture_vocabulary()
	for i in keys.size():
		if String(keys[i]) == new_key:
			idx = i
	_ok("FH2 the picker offers '%s'" % new_key, picker != null and idx >= 0)
	if picker == null or idx < 0:
		return
	# FR-02: a half-typed rename in the same pane is committed by the culture pick,
	# to the faction it was typed for.
	var edits: Array = []
	_collect_of(_pane("identity"), edits, LineEdit)
	var ne: LineEdit = edits[0] if not edits.is_empty() else null
	if ne != null:
		_focus(ne)
		await _frames(3)
		ne.text = "Fh2 half"
	picker.select(idx)
	picker.item_selected.emit(idx)
	await _frames(6)
	var after_key := String(_fr._faction(fa).get("culture", ""))
	_ok("FH2 the culture change reached the engine (fresh get_factions() read)", after_key == new_key,
		"expected %s, engine says %s" % [new_key, after_key])
	_ok("FH2 ... and touched no other faction", String(_fr._faction(fb).get("culture", "")) == orig_b)
	if ne != null:
		_ok("FH2 FR02 a culture pick commits a half-typed rename to its own faction",
			_names()[fa] == "Fh2 half" and _names()[fb] != "Fh2 half", str(_names()))
	_ok("FH2 the card reflects the new culture without reopening the window",
		_text_of(_fr._culture_card_host).contains("Culture profile — %s" % _culture_name_of(new_key)))
	_fold_check_card("FH2 card on %s (nobody else)" % new_key, fa)
	_bridge.civ_set_faction_field(fa, "name", name_a)

	# -- reroll ---------------------------------------------------------------------------
	_fr._mark_data_stale()
	_fr._rebuild_inspector()
	await _frames(6)
	await _press("settlements")
	await _press("identity")
	_ok("FH2 reroll precondition: a hidden pane exists to go stale", _fr._tab_panes.size() >= 2, str(_fr._tab_panes.keys()))
	var chips := _pool_chips()
	_ok("FH2 reroll precondition: the pool has chips", chips.size() > 0)
	if chips.is_empty():
		return
	var pick := chips[0] as Button
	var ridx := int(pick.name.trim_prefix("Reroll_"))
	var original_name := String((_bridge.settlements()[ridx] as Dictionary).get("name", ""))
	edits.clear()
	_collect_of(_pane("identity"), edits, LineEdit)
	ne = edits[0] as LineEdit
	_focus(ne)
	await _frames(3)
	ne.text = "Fh2 typing"
	var emits0 := _roster_emits
	var changed := false
	var tries := 0
	var new_name := original_name
	while not changed and tries < 8:
		tries += 1
		var chip := _fr._culture_card_host.find_child("Reroll_%d" % ridx, true, false) as Button
		if chip == null:
			break
		chip.pressed.emit()
		await _frames(4)
		new_name = String((_bridge.settlements()[ridx] as Dictionary).get("name", ""))
		changed = new_name != original_name
	_ok("FH2 a reroll chip changes the real settlement's name in the engine (%d press%s)" % [tries, "" if tries == 1 else "es"],
		changed and new_name != "", "%s -> %s" % [original_name, new_name])
	_ok("FH2 ... and emits roster_changed after the engine call", _roster_emits > emits0, str(_roster_emits - emits0))
	var chip2 := _fr._culture_card_host.find_child("Reroll_%d" % ridx, true, false) as Button
	_ok("FH2 ... and the card's chip shows the new name", chip2 != null and chip2.text == new_name,
		"chip=%s engine=%s" % [chip2.text if chip2 != null else "<none>", new_name])
	_ok("FH2 ... and a reroll never changes the culture", String(_fr._faction(fa).get("culture", "")) == new_key)
	_ok("FH2 ... and drops the hidden panes, keeping the visible one", _fr._tab_panes.keys() == ["identity"], str(_fr._tab_panes.keys()))
	_ok("FH2 FR02 a half-typed field survives a reroll (card-only refill) and keeps focus",
		is_instance_valid(ne) and ne.text == "Fh2 typing" and ne.has_focus(),
		"text=%s focus=%s" % [ne.text if is_instance_valid(ne) else "<freed>", is_instance_valid(ne) and ne.has_focus()])
	_ok("FH2 ... and it was not committed to anyone by the reroll",
		_names()[fa] == name_a and _names()[fb] != "Fh2 typing", str(_names()))
	await _press("settlements")
	_ok("FH2 the Settlements tab lists the rerolled name (it was rebuilt, not stale)",
		_text_of(_pane("settlements")).to_lower().contains(new_name.to_lower()), new_name)
	_ok("FH2 FR02 ... and the tab switch committed the typed name to its own faction",
		_names()[fa] == "Fh2 typing", str(_names()))

	# -- restore the world for the legs after this one --------------------------------------
	_bridge.civ_set_faction_field(fa, "name", name_a)
	_bridge.civ_set_faction_field(fa, "culture", orig_a)
	_fr._mark_data_stale()
	_fr._rebuild()
	await _frames(4)
	_fr.hide()
	await _frames(2)

	# -- the Culture dock link opens the hub on Identity ---------------------------------
	_app.select_domain("civilization")
	await _frames(4)
	await _sheet("left")
	_app.select_domain_category("civilization", "Culture")
	await _frames(4)
	var cbtns: Array = []
	_collect_buttons(_app.workspace_panel("civilization"), cbtns)
	var link := cbtns.filter(func(b): return _lc(b).begins_with("which faction has which culture"))
	_ok("FH2 link: exactly one 'Which faction has which culture' button", link.size() == 1, "%d of %d buttons" % [link.size(), cbtns.size()])
	if link.size() == 1:
		await _park(fb, "territory")
		(link[0] as Button).pressed.emit()
		await _frames(6)
		_land("FH2 culture link (last faction, Identity)", fb, "identity", false)
		_ok("FH2 link: the Culture card is built and in the tree",
			_fr._culture_card_host != null and is_instance_valid(_fr._culture_card_host) and _fr._culture_card_host.is_inside_tree())
	_fr.hide()
	await _frames(2)

	# -- the old window is gone, everywhere ---------------------------------------------------
	_ok("FH2 retired: app has no culture_profiles_window field", _app.get("culture_profiles_window") == null)
	_ok("FH2 retired: app has no open_culture_profiles()", not _app.has_method("open_culture_profiles"))
	_ok("FH2 retired: the script file is gone", not FileAccess.file_exists("res://shell/culture_profiles_window.gd"))
	_culture_done = true


## The engine's display name for culture `key`, read from `get_cultures()`.
func _culture_name_of(key: String) -> String:
	for c in _bridge.get_cultures():
		if String(c.get("key", "")) == key:
			return String(c.get("name", key.capitalize()))
	return key.capitalize()


# -- FH-3: default settlement type ---------------------------------------------

## The Identity tab's "Default settlement type" picker (`DefaultType`), or null.
func _default_picker() -> OptionButton:
	return _fr.find_child("DefaultType", true, false) as OptionButton


## The picker's selected item text, or "<none>" when it is missing.
func _picker_text(ob: OptionButton) -> String:
	return ob.get_item_text(ob.selected) if ob != null and ob.selected >= 0 else "<none>"


## Picks entry `i` the way a user does (select + the signal the popup emits).
func _pick_default(ob: OptionButton, i: int) -> void:
	ob.select(i)
	ob.item_selected.emit(i)


## The `faction_defaults` object of the store's saved document, as a Dictionary.
func _doc_defaults() -> Dictionary:
	var parsed = JSON.parse_string(SettlementTypeStore.document())
	if parsed is Dictionary:
		return (parsed as Dictionary).get("faction_defaults", {})
	return {}


## Protects: FH-3 (`FACTION_HUB_DESIGN.md` §3.5/§7) -- the per-faction default
## settlement type is chosen on the Identity tab, backed by the REAL
## `SettlementTypeStore` (not a copy), on desktop and phone:
##   * the picker exists in the Identity pane, offers "None" first then every
##     library type, and starts on None (today's behaviour);
##   * a pick reaches the store for THAT faction only, and survives switching to
##     another faction and back (the pane is rebuilt on each switch);
##   * "None" restores the store's empty default and drops the faction from
##     `document()`, whose save/restore round trip is unchanged;
##   * a type deleted in the library leaves the faction on None (shown as None
##     after the pane is rebuilt), and so does a stored id naming no type;
##   * a pick does not rebuild the pane, so a half-typed rename keeps its text
##     and focus and is committed to its own faction by the next tab switch (FR-02);
##   * the Settlement types window has lost its per-faction column -- no picker
##     per faction, a link button instead -- and the link opens the hub on Identity;
##   * on the phone the picker and the link are at least `PHONE_TAP_MIN` tall.
## What it cannot see: that the drop tool then APPLIES the default (store-level,
## `_settlementtypes_probe.gd`'s leg 3) or pixels (the screenshot is that check).
func _run_default_type(ids: Array) -> void:
	var fa: int = ids[0]
	var fb: int = ids[1]
	var name_a: String = _names()[fa]
	SettlementTypeStore.clear()
	var t1 := SettlementTypeStore.new_type("Probe walled town")
	var t2 := SettlementTypeStore.new_type("Probe harbour")

	# -- the picker, and its starting state ---------------------------------------
	_fr.open(fa, "identity")
	await _frames(8)
	var ob := _default_picker()
	_ok("FH3 the Identity tab has a Default settlement type picker", ob != null and _pane("identity").is_ancestor_of(ob))
	if ob == null:
		return
	_ok("FH3 ... labelled in the pane", _text_of(_pane("identity")).contains("Default settlement type"))
	_ok("FH3 ... offering None then each library type, in library order",
		ob.item_count == 3 and ob.get_item_text(0) == "None" and ob.get_item_text(1) == "Probe walled town"
		and ob.get_item_text(2) == "Probe harbour",
		"%d items, first '%s'" % [ob.item_count, ob.get_item_text(0) if ob.item_count > 0 else ""])
	_ok("FH3 ... starting on None, with the store holding no default",
		ob.selected == 0 and SettlementTypeStore.faction_default(fa) == "" and SettlementTypeStore.faction_default(fb) == "",
		"selected=%d" % ob.selected)
	if _phone:
		_ok("FH3 PHONE the picker is at least 44 px tall", ob.size.y + 0.5 >= DccTheme.PHONE_TAP_MIN, "%.1f" % ob.size.y)
		_ok("FH3 PHONE the window still holds exactly one scroller",
			_count(_fr, func(n): return n is ScrollContainer and (n as ScrollContainer).is_visible_in_tree()) == 1)
	## Scroll the picker into view first, so the screenshot shows the control itself
	## rather than the top of a long Identity pane.
	var sc: Node = ob.get_parent()
	while sc != null and not (sc is ScrollContainer):
		sc = sc.get_parent()
	if sc != null:
		(sc as ScrollContainer).ensure_control_visible(ob)
		await _frames(3)
	await _shot("%s_default_type" % ("phone" if _phone else "desktop"))

	# -- a pick reaches the store, for that faction only --------------------------
	var emits := [0]
	_fr.roster_changed.connect(func(): emits[0] += 1)
	_pick_default(ob, 1)
	await _frames(3)
	_ok("FH3 a pick sets THIS faction's default in the real store", SettlementTypeStore.faction_default(fa) == t1,
		"'%s' (want '%s')" % [SettlementTypeStore.faction_default(fa), t1])
	_ok("FH3 ... and touches no other faction", SettlementTypeStore.faction_default(fb) == "")
	_ok("FH3 ... without announcing a roster change (nothing the roster shows moved)", emits[0] == 0, str(emits[0]))
	_ok("FH3 ... and the document carries it under the faction's id", _doc_defaults().get(str(fa), "") == t1,
		SettlementTypeStore.document())

	# -- it survives switching faction and back -----------------------------------
	_fr.open(fb, "identity")
	await _frames(8)
	var ob_b := _default_picker()
	_ok("FH3 the other faction's picker shows None, not the pick", ob_b != null and ob_b.selected == 0 and _picker_text(ob_b) == "None",
		_picker_text(ob_b))
	_fr.open(fa, "identity")
	await _frames(8)
	var ob_a := _default_picker()
	_ok("FH3 switching back, the pick is still shown", _picker_text(ob_a) == "Probe walled town", _picker_text(ob_a))
	## Through the real list row too, not only open(): the click path commits focus
	## and rebuilds the same pane.
	var rows: Array = []
	_collect_buttons(_fr._list_body, rows)
	var other_row: Button = null
	for b in rows:
		if (b as Button).text.find(String(_names()[fb])) >= 0:
			other_row = b
			break
	_ok("FH3 precondition: the other faction's list row exists", other_row != null)
	if other_row != null:
		other_row.pressed.emit()
		await _frames(8)
		_ok("FH3 a list-row switch lands on the other faction's own picker (None)",
			_fr._selected == fb and _picker_text(_default_picker()) == "None", "sel=%d %s" % [_fr._selected, _picker_text(_default_picker())])
		_fr.open(fa, "identity")
		await _frames(8)
		_ok("FH3 ... and back again", _picker_text(_default_picker()) == "Probe walled town")

	# -- FR-02: a pick does not tear down a half-typed field ----------------------
	var edits: Array = []
	_collect_of(_pane("identity"), edits, LineEdit)
	var ne: LineEdit = edits[0] if not edits.is_empty() else null
	_ok("FH3 FR02 precondition: the name field exists", ne != null)
	if ne != null:
		_focus(ne)
		await _frames(3)
		ne.text = "Fh3 half"
		_pick_default(_default_picker(), 2)
		await _frames(4)
		_ok("FH3 FR02 a pick keeps a half-typed name and its focus (no rebuild under it)",
			is_instance_valid(ne) and ne.text == "Fh3 half" and ne.has_focus(),
			"text=%s focus=%s" % [ne.text if is_instance_valid(ne) else "<freed>", is_instance_valid(ne) and ne.has_focus()])
		_ok("FH3 ... and the pick itself landed", SettlementTypeStore.faction_default(fa) == t2)
		await _press("settlements")
		_ok("FH3 FR02 ... and the tab switch committed the name to its own faction",
			_names()[fa] == "Fh3 half" and _names()[fb] != "Fh3 half", str(_names()))
		_bridge.civ_set_faction_field(fa, "name", name_a)
		_fr.open(fa, "identity")
		await _frames(6)

	# -- None restores ----------------------------------------------------------------
	_pick_default(_default_picker(), 0)
	await _frames(3)
	_ok("FH3 None restores the empty default", SettlementTypeStore.faction_default(fa) == "")
	_ok("FH3 ... and the document drops the faction", not _doc_defaults().has(str(fa)), SettlementTypeStore.document())
	var doc := SettlementTypeStore.document()
	SettlementTypeStore.restore_document(doc)
	_ok("FH3 the store's document round-trips unchanged", SettlementTypeStore.document() == doc)

	# -- deleting the type reverts to None --------------------------------------------
	_pick_default(_default_picker(), 2)
	SettlementTypeStore.delete_type(t2)
	_fr.open(fa, "identity")
	await _frames(8)
	ob_a = _default_picker()
	_ok("FH3 a deleted type is gone from the picker and the faction shows None",
		ob_a != null and ob_a.item_count == 2 and _picker_text(ob_a) == "None" and SettlementTypeStore.faction_default(fa) == "",
		"%d items, shows %s, store '%s'" % [ob_a.item_count if ob_a != null else -1, _picker_text(ob_a), SettlementTypeStore.faction_default(fa)])
	SettlementTypeStore.set_faction_default(fa, "t99")
	_fr.open(fa, "identity")
	await _frames(8)
	_ok("FH3 a stored id naming no type is shown as None, not as the first type", _picker_text(_default_picker()) == "None", _picker_text(_default_picker()))
	SettlementTypeStore.set_faction_default(fa, t1)
	_fr.hide()
	await _frames(2)

	# -- the Settlement types window: library only, with a link ---------------------------
	_app.open_settlement_types()
	await _frames(8)
	var stw = _app.settlement_types_window
	var pickers := _count(stw._body, func(n): return n is OptionButton)
	_ok("FH3 the window's per-faction column is gone: only the type editor's four pickers remain (not four plus one per faction)",
		pickers == 4, "%d OptionButtons for %d factions" % [pickers, ids.size()])
	var stw_text: String = _text_of(stw._body)
	var names_listed := false
	for fid in ids:
		if stw_text.contains(String(_names()[fid])):
			names_listed = true
	_ok("FH3 ... and no faction is listed in it", not names_listed, stw_text.left(160))
	_ok("FH3 ... which no longer carries the old column's note", not stw_text.contains("This column is the whole reason"))
	var link := stw._body.find_child("FactionDefaultsLink", true, false) as Button
	_ok("FH3 the window carries the Factions > Identity link",
		link != null and link.text.to_lower() == "faction defaults now live in factions ▸ identity", str(link.text if link != null else "<none>"))
	if link != null:
		if _phone:
			_ok("FH3 PHONE the link is at least 44 px tall", link.size.y + 0.5 >= DccTheme.PHONE_TAP_MIN, "%.1f" % link.size.y)
		stw.hide()
		await _park(fb, "territory")
		_app.open_settlement_types()
		await _frames(6)
		link = stw._body.find_child("FactionDefaultsLink", true, false) as Button
		link.pressed.emit()
		await _frames(8)
		_ok("FH3 the link closes the library and opens the hub on Identity", not stw.visible and _fr.visible and _fr._tab == "identity",
			"library visible=%s hub visible=%s tab=%s" % [stw.visible, _fr.visible, _fr._tab])
		_ok("FH3 ... where the picker is built", _default_picker() != null)
		if _phone:
			_ok("FH3 PHONE ... on the master list (no faction was named, so open() reopens on the list, tab kept)", _fr._phone_list_pane != null and (_fr._phone_list_pane as Control).visible)
	SettlementTypeStore.clear()
	_fr.hide()
	await _frames(2)
	_deftype_done = true
