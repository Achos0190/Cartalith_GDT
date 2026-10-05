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
##           `Currency_<key>` / `Tariff_<id>` node names); History carries its Linked-
##           notes and Manual-entries sections and no input control (FH-8's own
##           leg, below, checks its content)
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
##           Cartography identity-colours jump. The Relationships pair rows (re-routed
##           by FH-6) land on the Relations tab with the other party carried and
##           marked; the dock's "Open in Factions..." carries its pair the same way.
##           The context-card row is `_ctxcard_probe.gd` / `_ctxphone_probe.gd`'s.
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
##   ECONOMY / RELATIONS (FH-6, `_run_economy_relations`) the Economy tab's
##           per-faction readout equals `civ_faction_economy()`'s row; Currency and
##           Tariffs keep their node names; "Trade flows" leaves the hub for
##           Civilization > Economy. The Relations tab lists exactly this faction's
##           pairs with the engine's own stance and score, mentions each pair's
##           tariff in BOTH directions (read-only: no edit control on the tab; the
##           direction is told apart by an asymmetric fixture), links to Economy,
##           marks the carried pair, reads the Faith group without running the
##           model, keeps one honest Diplomacy line, refreshes on a roster change,
##           and on the phone keeps >= 44 px taps and one scroller.
##   TERRITORY (FH-4, `_run_territory_tab`) the Territory tab: claim figures,
##           provinces list and influence-by-neighbour each equal the engine's own
##           answer for this world (influence only after its button, then cached);
##           the claim button arms the Territory tool with this faction picked, as
##           the context card's row does; Focus on capital keeps working; no
##           "clear" control; a faction switch refreshes; the stale chip follows
##           `stale_stages()`; phone taps >= 44 px and one scroller.
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
var _territory_done := false  ## set by the last line of `_run_territory_tab`
var _fh6_done := false  ## set by the last line of `_run_economy_relations`
var _fh7_done := false  ## set by the last line of `_run_military_tab`
const FH8_VAULT := "user://_fh8_vault"  ## throwaway vault for the Linked-notes row; removed after use
var _fh8_done := false  ## set by the last line of `_run_history_tab`


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
	## History (FH-8): a non-empty pane with its two sections, and no input control.
	await _press("history")
	var hpane := _pane("history")
	var hinputs := _count(hpane, func(n): return n is LineEdit or n is TextEdit or n is OptionButton or n is CheckBox or n is ColorPickerButton or n is SpinBox or n is Range)
	_ok("BUILD history builds its Linked notes and Manual entries sections and carries no input control",
		_text_of(hpane).to_lower().contains("linked notes") and _text_of(hpane).to_lower().contains("manual entries")
		and hpane.find_child("HistoryManualNotBuilt", true, false) != null and hinputs == 0, "inputs=%d" % hinputs)
	await _press("relations")
	_ok("BUILD relations builds a non-empty pane naming Relations, Faith and the not-built Diplomacy line",
		_text_of(_pane("relations")).to_lower().contains("faith") and _text_of(_pane("relations")).contains("Diplomacy: not built"),
		_text_of(_pane("relations")).left(200))

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

	# -- TERRITORY: the Territory tab's own content (FH-4) -----------------------------
	await _run_territory_tab(ids)
	_ok("FH4 the territory leg ran to its end (a script error inside it aborts it silently)", _territory_done)

	# -- ECONOMY + RELATIONS: the readout and the pair list (FH-6) ------------------------
	await _run_economy_relations(ids)
	_ok("FH6 the economy/relations leg ran to its end (a script error inside it aborts it silently)", _fh6_done)

	# -- MILITARY: garrisons and conflicts (FH-7) -----------------------------------------
	await _run_military_tab(ids)
	_ok("FH7 the military leg ran to its end (a script error inside it aborts it silently)", _fh7_done)

	# -- HISTORY: ownership periods, linked notes, the manual-entries line (FH-8) -------
	await _run_history_tab(ids)
	_ok("FH8 the history leg ran to its end (a script error inside it aborts it silently)", _fh8_done)

	# -- RF-03: a new world ----------------------------------------------------------
	_fr.open(fb, "military")
	await _frames(6)
	await _press("economy")
	await _press("history")  ## FH-8: fill the history cache while a timeline exists
	await _press("military")
	_ok("RF03 precondition: panes built, the selection is not the default, the history cache is warm",
		_fr._tab_panes.size() >= 3 and _fr._selected == fb and _fr._history_ready
		and not _bridge.get_civ_timeline_years().is_empty(),
		"%s sel=%d hist=%s" % [str(_fr._tab_panes.keys()), _fr._selected, str(_fr._history_ready)])
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
	_ok("RF03 ... and the history cache was dropped with the old world's timeline",
		not _fr._history_ready and _fr._history_rows.is_empty() and _bridge.get_civ_timeline_years().is_empty(),
		"ready=%s years=%s" % [str(_fr._history_ready), str(_bridge.get_civ_timeline_years())])
	await _press("history")
	var hb := _pane("history").find_child("HistoryBody", true, false)
	_ok("RF03 ... and the History tab then says there is no timeline, not the old world's spans",
		hb != null and hb.get_child_count() == 1 and _text_of(hb).begins_with("No timeline is recorded"),
		_text_of(_pane("history")).left(200))
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
			## FH-6: the pair rides along, so the Relations tab marks the same
			## row the dock card marks (and no pair means none is marked).
			_ok("%s: the hub carries the pair (-1 when none)" % tag,
				_fr._pair == (int(pair) if int(pair) > 0 and int(pair) != fa else -1), "pair=%d" % _fr._pair)
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

	# 5. Relationships > Every pair rows -> the hub's RELATIONS tab on a, with b
	# carried and marked (FH-6; RL-01 before it). Protects: a pair row opens the
	# hub on the first party's Relations tab with the SECOND party marked --
	# never one side of the pair (RL-01's measured defect: 5 of 15 rows were a
	# press with no visible effect). FH-1 first routed these to the hub, FH-1's
	# review pulled them back to the dock while the tab was a placeholder, and
	# FH-6 sent them to the hub for good. (Retargeted from the dock assertions,
	# not deleted: the dock card still marks a pair -- leg 3 and `_wiredfix`.)
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
		# Park the hub hidden on another faction and tab so "lands on a, Relations"
		# cannot pass by the window merely still sitting there.
		await _park(fb if want_a != fb else fa, park_tab)
		await _sheet("left")
		(b as Button).pressed.emit()
		await _frames(6)
		var tag5 := "FH6 relations row a=%d b=%d" % [want_a, want_b]
		_land(tag5, want_a, "relations", true)
		_ok("%s: the hub was told b as the pair (RL-01)" % tag5, _fr._pair == want_b and want_b >= 0, "pair=%d" % _fr._pair)
		var mark: Node = _fr.find_child("Relation_%d" % want_b, true, false)
		_ok("%s: b's card exists and is marked" % tag5,
			mark != null and bool(mark.get_meta("marked", false)), str(mark))
		_ok("%s: b is marked in the text ('▸ %s')" % [tag5, want_b_name],
			_text_of(_fr).to_lower().contains(("▸ " + want_b_name).to_lower()))
		_fr.hide()
		await _frames(2)
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


# -- TERRITORY (FH-4) -------------------------------------------------------------

## Names of every node under `root` whose name starts with `prefix`, in tree order.
func _named_like(root: Node, prefix: String) -> Array:
	var out: Array = []
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_front()
		if String(n.name).begins_with(prefix):
			out.append(n)
		for c in n.get_children():
			stack.append(c)
	return out


## The civilization workspace, found by capability (`claim_for_faction`), the way
## `app.claim_cells_for_faction` finds it.
func _civ_workspace() -> Node:
	for ws in _app._workspaces:
		if ws.has_method("claim_for_faction"):
			return ws
	return null


## Protects: FH-4, the Territory tab (`FACTION_HUB_DESIGN.md` §3.5). Every readout
## is compared with the engine's own answer for this generated world, read
## independently of the window: the claim figures (`civ_faction_territory_stats`),
## the provinces list (`bridge.provinces()` filtered by faction), the influence
## reading (`civ_territory_influence()` filtered to the faction, and ONLY after the
## button), and the stale chip (`bridge.stale_stages()["civ"]`). Also that the
## claim button runs the context card's handler (arms the Territory tool with this
## faction picked), that no control clears a faction's claims, that a selection
## change refreshes every readout, that the O(cells) influence call is never made
## unprompted, and (phone) that taps are at least 44 px and the single scroller
## holds. The absent-claims case ("—" everywhere) is `_claimsabsent_probe.gd`'s,
## which loads a save with the grid stripped.
func _run_territory_tab(ids: Array) -> void:
	var fa: int = ids[0]
	var fb: int = ids[1]
	var names := _names()

	# -- lazy: nothing O(cells) beyond the fits, and not the influence at all ------
	_fr.open(fa, "identity")
	await _frames(6)
	await _press("territory")
	await _frames(4)
	var pane: Control = _pane("territory")
	_ok("FH4 the Territory pane is built", pane != null)
	if pane == null:
		return
	_ok("FH4 LAZY building the tab never ran the influence pass", not _fr._influence_ready and _fr._influence.is_empty(),
		"ready=%s" % _fr._influence_ready)

	# -- claims, against the engine ---------------------------------------------------
	var stats: Dictionary = _bridge.civ_faction_territory_stats(fa)
	_ok("FH4 precondition: this world has a claim grid, so the stats are not empty", not stats.is_empty())
	var want_line := "Territory: %s km² over %d claimed cells (%d contested)" % [
		_fr._thousands(int(float(stats.get("area_km2", 0.0)))),
		int(stats.get("claimed_cells", 0)), int(stats.get("contested_cells", 0))]
	var claims_lbl := pane.find_child("TerritoryClaims", true, false) as Label
	_ok("FH4 the claim figures read the engine's own numbers (cells, km², contested)",
		claims_lbl != null and claims_lbl.text == want_line,
		"got '%s' want '%s'" % [claims_lbl.text if claims_lbl != null else "<none>", want_line])
	_ok("FH4 ... and they are real (a faction with settlements claims cells)",
		int(stats.get("claimed_cells", 0)) > 0 and float(stats.get("area_km2", 0.0)) > 0.0, str(stats))
	var text := _text_of(pane)
	_ok("FH4 the existing fit verdict/mix is still on the tab", text.to_lower().contains("territory fit"))
	_ok("FH4 the capital line is still on the tab", text.contains("Capital:"))

	# -- provinces, against bridge.provinces() ------------------------------------------
	var want_rows: Array = []
	for p in _bridge.provinces():
		if int((p as Dictionary).get("faction", -1)) == fa:
			want_rows.append(p)
	var got_rows := _named_like(pane, "Province_")
	_ok("FH4 precondition: faction %d owns provinces in this world" % fa, want_rows.size() > 0)
	_ok("FH4 the provinces list has one row per bridge.provinces() entry of this faction",
		got_rows.size() == want_rows.size(), "%d vs %d" % [got_rows.size(), want_rows.size()])
	var sets: Array = _bridge.settlements()
	var names_ok := true
	var bad := ""
	for p in want_rows:
		var pd: Dictionary = p
		var l := pane.find_child("Province_%d" % int(pd.get("id", 0)), true, false) as Label
		var ci := int(pd.get("capital_settlement_index", -1))
		var cname := String((sets[ci] as Dictionary).get("name", "—")) if ci >= 0 and ci < sets.size() else "—"
		var want := "%s   ·   capital %s" % [String(pd.get("name", "?")), cname]
		if l == null or l.text != want:
			names_ok = false
			bad = "'%s' vs '%s'" % [l.text if l != null else "<none>", want]
	_ok("FH4 ... each reading its own name and capital", names_ok, bad)
	_ok("FH4 ... under a heading that carries the count", text.contains("Provinces (%d)" % want_rows.size()) or text.to_lower().contains("provinces (%d)" % want_rows.size()))
	_ok("FH4 ... and says no cell counts are shown rather than inventing them", text.contains("not exposed"))

	# -- influence: behind the button ------------------------------------------------
	_ok("FH4 influence reads 'Not run yet' before the button", text.contains("Not run yet"))
	var run := pane.find_child("AnalyseInfluence", true, false) as Button
	_ok("FH4 the Analyse influence button exists", run != null)
	var inf: Dictionary = _bridge.civ_territory_influence()
	_ok("FH4 precondition: the engine can analyse influence on this world", not inf.is_empty())
	if run != null:
		if _phone:
			_ok("FH4 PHONE Analyse influence is at least 44 px tall", run.size.y + 0.5 >= DccTheme.PHONE_TAP_MIN, "%.1f" % run.size.y)
		run.pressed.emit()
		await _frames(4)
	_ok("FH4 pressing it ran the pass and cached it", _fr._influence_ready and not _fr._influence.is_empty())
	var inf_text := _text_of(pane.find_child("InfluenceBody", true, false))
	var mine: Dictionary = {}
	for r in inf.get("factions", []):
		if int((r as Dictionary).get("id", -1)) == fa:
			mine = r
	var want_reach := "Reach: %s cells, %s on a frontier; mean reach %.1f, mean contest %.3f" % [
		_fr._thousands(int(mine.get("cells", 0))), _fr._thousands(int(mine.get("frontier_cells", 0))),
		float(mine.get("mean_influence", 0.0)), float(mine.get("mean_contested", 0.0))]
	_ok("FH4 the influence reach line is this faction's engine row", not mine.is_empty() and inf_text.contains(want_reach),
		"want '%s' in '%s'" % [want_reach, inf_text.left(200)])
	var want_nb := 0
	var nb_ok := true
	for r in inf.get("borders", []):
		var b: Dictionary = r
		var a_is := int(b.get("a", -1)) == fa
		if not a_is and int(b.get("b", -1)) != fa:
			continue
		want_nb += 1
		var other := String(b.get("b_name", "?")) if a_is else String(b.get("a_name", "?"))
		var w := "%s -- %s frontier cells, mean contest %.3f" % [other, _fr._thousands(int(b.get("cells", 0))), float(b.get("mean_contested", 0.0))]
		if not inf_text.contains(w):
			nb_ok = false
	_ok("FH4 every border this faction takes part in is listed, naming the OTHER side", nb_ok)
	_ok("FH4 ... and no border it is not part of (row count matches)",
		_named_like(pane, "InfluenceNeighbour_").size() == want_nb or (want_nb == 0 and inf_text.contains("No neighbour")),
		"%d vs %d" % [_named_like(pane, "InfluenceNeighbour_").size(), want_nb])
	_ok("FH4 ... with the caption that it is not the painted claims", inf_text.contains("not from painted claims"))
	await _shot("%s_territory_influence" % ("phone" if _phone else "desktop"))

	# -- no destructive control ----------------------------------------------------------
	var all_btn: Array = []
	_collect_buttons(pane, all_btn)
	var clearish := false
	for b in all_btn:
		if (b as Button).text.to_lower().contains("clear"):
			clearish = true
	_ok("FH4 no 'clear claims' control is on the tab", not clearish)

	# -- focus on capital keeps working ---------------------------------------------------
	var focus := pane.find_child("FocusCapital", true, false) as Button
	_ok("FH4 Focus on capital is present", focus != null)
	if _phone and focus != null:
		_ok("FH4 PHONE Focus on capital is at least 44 px tall", focus.size.y + 0.5 >= DccTheme.PHONE_TAP_MIN, "%.1f" % focus.size.y)
	if focus != null:
		focus.pressed.emit()
		await _frames(3)
		_ok("FH4 ... and closes the hub to show the map", not _fr.visible)

	# -- the claim button is the context card's handler --------------------------------------
	_fr.open(fa, "territory")
	await _frames(6)
	pane = _pane("territory")
	var claim := pane.find_child("ClaimCells", true, false) as Button
	_ok("FH4 the Claim cells for this faction button exists", claim != null)
	if claim == null:
		return
	if _phone:
		_ok("FH4 PHONE Claim cells is at least 44 px tall", claim.size.y + 0.5 >= DccTheme.PHONE_TAP_MIN, "%.1f" % claim.size.y)
		_ok("FH4 PHONE the window still holds exactly one scroller",
			_count(_fr, func(n): return n is ScrollContainer and (n as ScrollContainer).is_visible_in_tree()) == 1)
	await _shot("%s_territory" % ("phone" if _phone else "desktop"))
	_app.arm_tool("inspect")
	await _frames(2)
	var ws := _civ_workspace()
	_ok("FH4 fixture: the civilization workspace is found, and no faction is picked for the tool yet", ws != null and int(ws.get("_territory_faction")) != fb)
	_ok("FH4 precondition: another tool is armed", _app.armed_tool == "inspect", _app.armed_tool)
	_fr.open(fb, "territory")
	await _frames(6)
	(_pane("territory").find_child("ClaimCells", true, false) as Button).pressed.emit()
	await _frames(4)
	_ok("FH4 pressing Claim arms the Territory tool", _app.armed_tool == "territory", _app.armed_tool)
	_ok("FH4 ... with THIS faction picked, as the context card's row does",
		ws != null and int(ws.get("_territory_faction")) == fb, str(ws.get("_territory_faction")) if ws != null else "no ws")
	_ok("FH4 ... and hides the hub so the map can be painted on", not _fr.visible)
	_app.arm_tool("inspect")

	# -- a selection change refreshes every readout ----------------------------------------------
	_fr.open(fa, "territory")
	await _frames(6)
	var stats_b: Dictionary = _bridge.civ_faction_territory_stats(fb)
	## open() marks the caches stale (RF-03), so run the analysis afresh for this
	## visit, then plant a sentinel in the cache: a re-run would replace it.
	(_pane("territory").find_child("AnalyseInfluence", true, false) as Button).pressed.emit()
	await _frames(4)
	_fr._influence["_probe_sentinel"] = 1
	var row_btn: Button = null
	var rows: Array = []
	_collect_buttons(_fr._list_body, rows)
	for b in rows:
		if (b as Button).text.find(String(names[fb])) >= 0 and (b as Button).text.find("—") >= 0:
			row_btn = b
			break
	_ok("FH4 precondition: the real list row for the second faction exists", row_btn != null)
	if row_btn == null:
		return
	row_btn.pressed.emit()
	await _frames(8)
	pane = _pane("territory")
	var lbl_b := pane.find_child("TerritoryClaims", true, false) as Label if pane != null else null
	var want_b := "Territory: %s km² over %d claimed cells (%d contested)" % [
		_fr._thousands(int(float(stats_b.get("area_km2", 0.0)))),
		int(stats_b.get("claimed_cells", 0)), int(stats_b.get("contested_cells", 0))]
	_ok("FH4 a selection change refreshes the claim figures to the new faction's",
		_fr._selected == fb and lbl_b != null and lbl_b.text == want_b,
		"sel=%d got '%s' want '%s'" % [_fr._selected, lbl_b.text if lbl_b != null else "<none>", want_b])
	var want_b_rows := 0
	for p in _bridge.provinces():
		if int((p as Dictionary).get("faction", -1)) == fb:
			want_b_rows += 1
	_ok("FH4 ... and the provinces list to the new faction's", _named_like(pane, "Province_").size() == want_b_rows,
		"%d vs %d" % [_named_like(pane, "Province_").size(), want_b_rows])
	var inf_b := _text_of(pane.find_child("InfluenceBody", true, false))
	_ok("FH4 ... and the cached influence is re-filtered, not re-run, for the new faction",
		_fr._influence_ready and _fr._influence.has("_probe_sentinel") and (inf_b.contains("Reach:") or inf_b.contains("no capital reach")),
		inf_b.left(160))

	# -- staleness: the chip follows the engine's graph ------------------------------------------------
	_ok("FH4 healthy world: no stale chip while stale_stages() has no civ entry",
		_bridge.stale_stages().has("civ") or pane.find_child("StaleClaimsChip", true, false) == null)
	var cap: Dictionary = _fr._capital_of(fa)
	var idx: int = _bridge.civ_drop_settlement(float(int(cap.get("x", 0))) + 3.0, float(int(cap.get("y", 0))) + 3.0, "hamlet", fa, "", false)
	_ok("FH4 fixture: a settlement edit was accepted", idx >= 0, "idx=%d" % idx)
	if idx >= 0:
		var civ_stale: bool = _bridge.stale_stages().has("civ")
		_ok("FH4 fixture: the engine now reports the civ stage stale", civ_stale, str(_bridge.stale_stages().keys()))
		_fr._mark_data_stale()
		_fr._rebuild_inspector()
		await _frames(6)
		var chip := _pane("territory").find_child("StaleClaimsChip", true, false) as Label
		_ok("FH4 the stale chip appears and names the engine's reason", chip != null and chip.text.contains("Claims may be stale")
			and chip.text.contains(String((_bridge.stale_stages().get("civ", {}) as Dictionary).get("reason", "?")).replace("_", " ")),
			chip.text if chip != null else "<none>")
		await _shot("%s_territory_stale" % ("phone" if _phone else "desktop"))
		_bridge.civ_delete_settlement(idx)
	_fr.hide()
	await _frames(2)
	_territory_done = true


# -- FH-6: the Economy readout and the Relations tab --------------------------
## Protects: FH-6 (`FACTION_HUB_DESIGN.md` section 3.6) -- the Economy tab's
## per-faction readout is the engine's own `civ_faction_economy()` row, the
## Relations tab's pair list/stance/score/tariff mention are the engine's own
## `civ_faction_relations()` and `civ_trade_tariff` answers in the right
## direction, the tab carries NO tariff editor (owner decision 3: tariffs are
## edited on Economy only), the carried pair is marked, the Faith group never
## runs the model by itself, and the tab follows a roster change (RF-03) and
## keeps phone taps >= 44 px. Every expectation is derived from the bridge, not
## from the tab's own helpers, so the tab cannot be its own oracle.
func _run_economy_relations(ids: Array) -> void:
	var fa: int = ids[0]
	var fb: int = ids[1]
	var names := _names()

	# ---------------------------------------------------------------- Economy
	_fr.open(fa, "economy")
	await _frames(8)
	var eco := _pane("economy")
	_ok("FH6 the Economy pane is built", eco != null)
	if eco == null:
		return
	var erow := {}
	for r in _bridge.civ_faction_economy():
		if int((r as Dictionary).get("faction", -1)) == fa:
			erow = r
	_ok("FH6 precondition: the engine has an economy row for faction %d" % fa,
		not erow.is_empty() and String(erow.get("absent", "")) == "", str(erow.keys()))
	var fig := eco.find_child("EconomyFigures", true, false) as Label
	_ok("FH6 the economy figures read the engine's population and food capacity",
		fig != null and fig.text.contains("%s people" % _fr._thousands(int(float(erow.get("pop", 0.0)))))
		and fig.text.contains("food capacity %s" % _fr._thousands(int(float(erow.get("food_capacity", 0.0))))),
		"%s | pop=%s cap=%s" % [fig.text if fig != null else "<none>", erow.get("pop"), erow.get("food_capacity")])
	var want_sur := float(erow.get("food_surplus", 0.0))
	_ok("FH6 ... and say the surplus sign in words, not a bare minus",
		fig != null and (fig.text.contains("to spare") if want_sur >= 0.0 else fig.text.contains("short by")),
		fig.text if fig != null else "<none>")
	var res := eco.find_child("EconomyResources", true, false) as Label
	var strat: PackedStringArray = erow.get("strategic", PackedStringArray())
	var exps: PackedStringArray = erow.get("exports", PackedStringArray())
	var imps: PackedStringArray = erow.get("imports", PackedStringArray())
	var want_res := "Strategic: %s.  Exports: %s.  Imports: %s." % [
		"none" if strat.is_empty() else ", ".join(strat),
		"none" if exps.is_empty() else ", ".join(exps),
		"none" if imps.is_empty() else ", ".join(imps)]
	_ok("FH6 the strategic / exports / imports line equals the engine's lists",
		res != null and res.text == want_res, "got '%s' want '%s'" % [res.text if res != null else "<none>", want_res])
	_ok("FH6 Currency and Tariffs are kept verbatim (node names)",
		eco.find_child("Currency_name", true, false) != null and eco.find_child("Currency_rate", true, false) != null
		and eco.find_child("Tariff_%d" % fb, true, false) != null)
	var eco_text := _text_of(eco).to_lower()  ## section headers are drawn in capitals
	_ok("FH6 the readout sits above Currency (reading order)",
		eco_text.find("food capacity") >= 0 and eco_text.find("currency") >= 0
		and eco_text.find("food capacity") < eco_text.find("currency"))
	await _shot("%s_economy" % ("phone" if _phone else "desktop"))
	var flows := eco.find_child("TradeFlowsLink", true, false) as Button
	_ok("FH6 the Trade flows link exists", flows != null)
	if _phone and flows != null:
		_ok("FH6 PHONE Trade flows is at least 44 px tall", flows.size.y + 0.5 >= DccTheme.PHONE_TAP_MIN, "%.1f" % flows.size.y)
	if flows != null:
		flows.pressed.emit()
		await _frames(6)
		_ok("FH6 Trade flows closes the hub and opens Civilization > Economy",
			not _fr.visible and _text_of(_app.workspace_panel("civilization")).to_lower().contains("by faction"),
			"visible=%s" % _fr.visible)

	# ------------------------------------------------------------- Relations
	# Fixture: an ASYMMETRIC tariff pair, so a swapped direction cannot pass.
	# civ_trade_tariff(importer, exporter): fa charges 15% on fb's goods, fb
	# charges 40% on fa's.
	var old_ab: float = _bridge.civ_trade_tariff(fa, fb)
	var old_ba: float = _bridge.civ_trade_tariff(fb, fa)
	_ok("FH6 fixture: both tariffs were accepted",
		_bridge.civ_set_trade_tariff(fa, fb, 0.15) and _bridge.civ_set_trade_tariff(fb, fa, 0.40))
	_fr.open(fa, "relations")
	await _frames(8)
	var rel := _pane("relations")
	_ok("FH6 the Relations pane is built", rel != null)
	if rel == null:
		return
	var want_pairs: Array = []
	for d in _bridge.civ_faction_relations():
		var pd: Dictionary = d
		if int(pd.get("a", -1)) == fa or int(pd.get("b", -1)) == fa:
			want_pairs.append(pd)
	_ok("FH6 precondition: faction %d is a party to at least one pair" % fa, want_pairs.size() > 0)
	var cards := _named_like(rel, "Relation_")
	_ok("FH6 one card per pair involving this faction (and no others)", cards.size() == want_pairs.size(),
		"%d cards, %d pairs" % [cards.size(), want_pairs.size()])
	for pd in want_pairs:
		var other := int(pd.get("b", -1)) if int(pd.get("a", -1)) == fa else int(pd.get("a", -1))
		var oname := String(names.get(other, "?"))
		var card := rel.find_child("Relation_%d" % other, true, false)
		_ok("FH6 a card for the pair with %s" % oname, card != null)
		if card == null:
			continue
		var sc := (card.find_child("RelationScore", true, false) as Label).text
		var st := (card.find_child("RelationStance", true, false) as Label).text
		var want_score := "%+d" % int(round(100.0 * float(pd.get("value", 0.0))))
		_ok("FH6 %s: the score and stance are the engine's" % oname,
			sc == want_score and st == String(pd.get("stance", "?")).to_upper(),
			"%s/%s vs %s/%s" % [sc, st, want_score, pd.get("stance")])
		var tip := (card as Control).tooltip_text
		_ok("FH6 %s: the term tooltip carries the engine's culture and faith terms" % oname,
			tip.contains("culture %+d" % int(round(30.0 * float(pd.get("culture_term", 0.0)))))
			and tip.contains("faith %+d" % int(round(20.0 * float(pd.get("religion_term", 0.0))))), tip)
		# The tariff mention: "Charges X% on goods from B - B charges Y% on yours".
		# X is what THIS faction (the importer, fa) levies: civ_trade_tariff(fa, other).
		var x := float(_bridge.civ_trade_tariff(fa, other))
		var y := float(_bridge.civ_trade_tariff(other, fa))
		var want_t := "Charges %s%% on goods from %s · %s charges %s%% on yours" % [
			_pct_prose(x), oname, oname, _pct_prose(y)]
		var tl := card.find_child("RelationTariff", true, false) as Label
		_ok("FH6 %s: the tariff mention reads both directions from civ_trade_tariff" % oname,
			tl != null and tl.text == want_t, "got '%s' want '%s'" % [tl.text if tl != null else "<none>", want_t])
	# The asymmetric fixture, as literals (independent of any formatter): fa->fb 15, fb->fa 40.
	var card_b := rel.find_child("Relation_%d" % fb, true, false)
	var tl_b_text := "<none>"
	if card_b != null:
		tl_b_text = (card_b.find_child("RelationTariff", true, false) as Label).text
	_ok("FH6 DIRECTION: 'Charges 15%' is fa's own rate on fb's goods and 'charges 40%' is fb's on fa's",
		tl_b_text == "Charges 15%% on goods from %s · %s charges 40%% on yours" % [names.get(fb, "?"), names.get(fb, "?")], tl_b_text)
	# Read-only: no tariff (or any) editor on the Relations tab.
	var editors := _count(rel, func(n): return n is LineEdit or n is SpinBox or n is OptionButton \
		or n is CheckBox or n is ColorPickerButton or n is TextEdit)
	_ok("FH6 NO EDIT CONTROL on Relations (tariffs are edited on Economy only)", editors == 0, "editors=%d" % editors)
	_ok("FH6 ... and no node carries the Economy tab's Tariff_<id> name", _named_like(rel, "Tariff_").is_empty())
	var treat := _count(rel, func(n): return n is Button and (String((n as Button).text).to_lower().contains("treaty")
		or String((n as Button).text).to_lower().contains("declare") or String((n as Button).text).to_lower().contains("alliance")))
	_ok("FH6 no treaty / war / alliance control", treat == 0, str(treat))
	_ok("FH6 one honest Diplomacy: not built line", _text_of(rel).count("Diplomacy: not built") == 1)
	# Economy link.
	var lnk: Button = null
	if card_b != null:
		lnk = card_b.find_child("EconomyLink", true, false) as Button
	_ok("FH6 each card has an Economy link", lnk != null)
	await _shot("%s_relations" % ("phone" if _phone else "desktop"))
	if lnk != null:
		lnk.pressed.emit()
		await _frames(6)
		_ok("FH6 the Economy link switches the hub to its Economy tab",
			_fr.visible and _fr._tab == "economy" and _visible_panes() == ["economy"], "tab=%s" % _fr._tab)

	# -------------------------------------------------------------- highlight
	_fr.open(fa, "relations", fb)
	await _frames(8)
	rel = _pane("relations")
	var marked_ids: Array = []
	for c in _named_like(rel, "Relation_"):
		if bool((c as Node).get_meta("marked", false)):
			marked_ids.append(String(c.name))
	_ok("FH6 the carried pair is the ONE marked row", marked_ids == ["Relation_%d" % fb], str(marked_ids))
	_ok("FH6 ... drawn with the dock's own marker", _text_of(rel).contains("▸ %s" % names.get(fb, "?")))
	for bad_pair in [fa, 9999, -1]:
		_fr.open(fa, "relations", bad_pair)
		await _frames(6)
		var any_marked := false
		for c in _named_like(_pane("relations"), "Relation_"):
			any_marked = any_marked or bool((c as Node).get_meta("marked", false))
		_ok("FH6 a carried pair of %d (self / unknown / none) marks nothing" % bad_pair, not any_marked)

	# ------------------------------------------------------------------ Faith
	_fr.open(fa, "relations")
	await _frames(8)
	rel = _pane("relations")
	var places: Array = _bridge.settlements()
	_ok("FH6 precondition: the belief model has not been run in this world",
		places.size() > 0 and not (places[0] as Dictionary).has("religion"))
	_ok("FH6 the Faith group says so and offers the run, and opening the tab did not run it",
		rel.find_child("FaithNotRun", true, false) != null and rel.find_child("RunBelief", true, false) != null
		and not (_bridge.settlements()[0] as Dictionary).has("religion"))
	var run := rel.find_child("RunBelief", true, false) as Button
	if run != null:
		if _phone:
			_ok("FH6 PHONE Run belief model is at least 44 px tall", run.size.y + 0.5 >= DccTheme.PHONE_TAP_MIN, "%.1f" % run.size.y)
		run.pressed.emit()
		await _frames(10)
		rel = _pane("relations")
		_ok("FH6 pressing Run belief model ran the existing consumer (settlements now carry a faith)",
			(_bridge.settlements()[0] as Dictionary).has("religion"))
		var total := 0
		var counts := {}
		for s2 in _bridge.settlements():
			var sd: Dictionary = s2
			if int(sd.get("faction", 0)) != fa:
				continue
			var ad: Dictionary = sd.get("adherents", {})
			for k in ad.keys():
				counts[k] = int(counts.get(k, 0)) + int(ad[k])
				total += int(ad[k])
		var comp := rel.find_child("FaithComposition", true, false) as Label
		if total > 0:
			var lead := ""
			var lead_n := -1
			for k in counts.keys():
				if int(counts[k]) > lead_n or (int(counts[k]) == lead_n and String(k) < lead):
					lead = String(k)
					lead_n = int(counts[k])
			_ok("FH6 the Faith composition names this faction's head-count and its leading faith",
				comp != null and comp.text.begins_with("%s people" % _fr._thousands(total))
				and comp.text.to_lower().contains("no religion" if lead == "none" else lead.capitalize().to_lower()),
				"%s | total=%d lead=%s" % [comp.text if comp != null else "<none>", total, lead])
		else:
			_ok("FH6 a faction with no adherents says so with a dash", rel.find_child("FaithNone", true, false) != null)
		await _shot("%s_relations_faith" % ("phone" if _phone else "desktop"))

	# ------------------------------------------------- RF-03: roster change
	var before := _named_like(_pane("relations"), "Relation_").size()
	_fr._add_faction()
	await _frames(8)
	_fr.open(fa, "relations")
	await _frames(8)
	var after := _named_like(_pane("relations"), "Relation_").size()
	_ok("FH6 RF-03: a new faction adds one pair to this tab (the cache follows _mark_data_stale)",
		after == before + 1, "%d -> %d" % [before, after])
	_fr._ensure_relations()
	_fr._ensure_economy()
	_fr._mark_data_stale()
	_ok("FH6 RF-03: _mark_data_stale clears both caches' flags and payloads",
		not _fr._relations_ready and _fr._relations.is_empty() and not _fr._economy_ready and _fr._economy_rows.is_empty())
	_bridge.civ_remove_faction()
	_fr._mark_data_stale()
	_fr.open(fa, "relations")
	await _frames(8)
	_ok("FH6 fixture restored: the pair count is back", _named_like(_pane("relations"), "Relation_").size() == before)

	# -------------------------------------------------------------- tariff edit refreshes the mention
	_fr.open(fa, "economy")
	await _frames(6)
	await _press("relations")
	await _press("economy")
	_ok("FH6 precondition: Relations is built while Economy shows", _fr._tab_panes.has("relations"))
	var tf := _pane("economy").find_child("Tariff_%d" % fb, true, false) as LineEdit
	if tf != null:
		tf.text = "25"
		_fr._set_tariff(fb, "25", tf)
		_ok("FH6 a tariff edit on Economy drops the built Relations pane", not _fr._tab_panes.has("relations"), str(_fr._tab_panes.keys()))
		await _press("relations")
		var cb2 := _pane("relations").find_child("Relation_%d" % fb, true, false)
		var t2 := (cb2.find_child("RelationTariff", true, false) as Label).text if cb2 != null else "<none>"
		_ok("FH6 ... and the rebuilt mention shows the new rate", t2.begins_with("Charges 25% on goods from"), t2)
	else:
		_ok("FH6 the Economy tab has the Tariff_<fb> field", false)

	# -------------------------------------------------------------- phone
	if _phone:
		await _press("relations")
		var small := _count(_pane("relations"), func(n): return n is Button and (n as Button).visible and (n as Button).size.y + 0.5 < DccTheme.PHONE_TAP_MIN)
		_ok("FH6 PHONE every Relations button is at least 44 px tall", small == 0, "short=%d" % small)
		_ok("FH6 PHONE one scroller and no TabContainer",
			_count(_fr, func(n): return n is ScrollContainer) == 1 and _count(_fr, func(n): return n is TabContainer) == 0)

	# restore the world
	_bridge.civ_set_trade_tariff(fa, fb, old_ab)
	_bridge.civ_set_trade_tariff(fb, fa, old_ba)
	_fr.hide()
	await _frames(2)
	_fh6_done = true


## A tariff fraction as the Relations mention prints it, derived here
## independently of the tab: whole percentages bare, otherwise one decimal.
func _pct_prose(rate: float) -> String:
	if rate <= 0.0:
		return "0"
	var pct := rate * 100.0
	return "%d" % int(round(pct)) if absf(pct - round(pct)) < 0.05 else "%.1f" % pct



## Protects: FH-7, the Military tab's two new blocks (`FACTION_HUB_DESIGN.md`
## §3.5). Read off the live window and compared with answers the probe derives
## independently of it:
##   GARRISON   one `Garrison_<index>` row per settlement this faction holds, the
##              count taken from `bridge.settlements()` filtered by faction (NOT from
##              the summary the tab itself reads), each row naming the settlement at
##              its index and carrying the engine's own garrison figure; a row opens
##              the Place editor on that very settlement and closes the hub; past the
##              cap the block says so and gives the true count; a place with no
##              garrison figure sorts last and is never drawn as 0.
##   CONFLICTS  for every faction, the `Conflict_<id>` rows equal `conflict_list()`
##              filtered to conflicts naming it in `sides` -- recomputed here from
##              rows the probe seeded through the real `conflict_add` -- including a
##              faction on no conflict (one honest line, no rows) and a conflict with
##              no sides (shown under nobody); a row opens the right dock's Conflict
##              context on that conflict id and closes the hub; a side edited in the
##              dock moves the row on the next open.
##   NO-EDIT    the Military pane has no toggle, picker, field, spinner or slider at
##              all, and the conflicts block's only buttons are its rows: side editing
##              is the dock's alone.
##   KEPT       "Full breakdown" navigation still closes the hub and opens
##              Civilization > Military; RF-03: a stale mark and rebuild leave the
##              same rows; phone: every Military button >= 44 px, one scroller.
func _run_military_tab(ids: Array) -> void:
	var fa: int = ids[0]
	var fb: int = ids[1]
	var fc: int = ids[2] if ids.size() > 2 else -1
	var names := _names()
	var year: int = _bridge.get_civ_year()

	# ----------------------------------------------------- seed the conflicts
	# C1: fa vs fb.  C2: fb alone (so fa and fb are on different sets, and the
	# last faction fc is on NONE).  C3: no sides at all (a side of nobody).
	# A battle takes one point; kinds and years are the store's own.
	var r1: Dictionary = _bridge.conflict_add({"name": "FH7 war of A and B", "kind": "battle",
		"start_year": year, "sides": PackedInt32Array([fa, fb]),
		"points": PackedVector2Array([Vector2(60.0, 60.0)])})
	var r2: Dictionary = _bridge.conflict_add({"name": "FH7 B alone", "kind": "battle",
		"start_year": year, "sides": PackedInt32Array([fb]),
		"points": PackedVector2Array([Vector2(70.0, 70.0)])})
	var r3: Dictionary = _bridge.conflict_add({"name": "", "kind": "battle",
		"start_year": year, "sides": PackedInt32Array(),
		"points": PackedVector2Array([Vector2(80.0, 80.0)])})
	_ok("FH7 fixture: the three conflicts were accepted",
		bool(r1.get("ok", false)) and bool(r2.get("ok", false)) and bool(r3.get("ok", false)),
		"%s %s %s" % [str(r1), str(r2), str(r3)])
	var c1 := int(r1.get("id", 0))
	var c2 := int(r2.get("id", 0))
	var c3 := int(r3.get("id", 0))
	var seeded: Array = [c1, c2, c3]
	_ok("FH7 precondition: a faction on no conflict exists (needs a third faction)", fc > 0,
		"factions=%d" % ids.size())

	# ----------------------------------------------------------- the garrisons
	_fr.open(fa, "military")
	await _frames(8)
	var pane := _pane("military")
	_ok("FH7 the Military pane is built", pane != null)
	if pane == null:
		return
	var roster: Array = _bridge.settlements()
	var want_idx: Array = []
	for i in roster.size():
		if int((roster[i] as Dictionary).get("faction", 0)) == fa:
			want_idx.append(i)
	_ok("FH7 precondition: faction %d holds at least two settlements" % fa, want_idx.size() >= 2,
		"holds %d" % want_idx.size())
	var want_n: int = mini(want_idx.size(), _fr.GARRISON_ROW_CAP)
	var grows := _named_like(pane, "Garrison_")
	_ok("FH7 GARRISON one row per settlement this faction holds, up to the cap (%d)" % want_n,
		grows.size() == want_n, "%d rows, %d settlements" % [grows.size(), want_idx.size()])
	_ok("FH7 GARRISON no truncation line while every place fits",
		want_idx.size() > _fr.GARRISON_ROW_CAP or pane.find_child("GarrisonTruncated", true, false) == null)
	var all_mine := true
	var names_ok := true
	for g in grows:
		var idx := int(String(g.name).trim_prefix("Garrison_"))
		if idx not in want_idx:
			all_mine = false
		elif not (g as Button).text.to_lower().begins_with(String((roster[idx] as Dictionary).get("name", "?")).to_lower() + " --"):
			names_ok = false
	_ok("FH7 GARRISON every row is one of this faction's own settlements", all_mine)
	_ok("FH7 GARRISON every row names the settlement at its index", names_ok)
	# The engine's own figure for each row, read through the real call, not the tab.
	var summary: Dictionary = _bridge.civ_military_summary()
	var by_idx := {}
	for p in (summary.get("settlements", []) as Array):
		by_idx[int((p as Dictionary).get("index", -1))] = p
	var fig_ok := true
	var fig_detail := ""
	var with_fig := 0
	for g in grows:
		var idx := int(String(g.name).trim_prefix("Garrison_"))
		var p: Dictionary = by_idx.get(idx, {})
		if p.has("garrison"):
			with_fig += 1
			if not (g as Button).text.to_lower().contains("garrison %s " % _fr._thousands(int(p["garrison"]))):
				fig_ok = false
				fig_detail = "%s vs %d" % [(g as Button).text, int(p["garrison"])]
		elif not (g as Button).text.to_lower().contains("garrison —"):
			fig_ok = false
			fig_detail = (g as Button).text
	_ok("FH7 GARRISON each row carries the engine's garrison figure (or a dash with none)", fig_ok, fig_detail)
	_ok("FH7 GARRISON precondition: at least one row has a real figure to compare", with_fig > 0, "%d" % with_fig)
	_ok("FH7 GARRISON the header says the faction's garrison total",
		(pane.find_child("GarrisonNote", true, false) as Label).text.contains(
			_fr._thousands(int(_fr._military_row(fa).get("garrison_total", -1)))))
	# Ordering is by garrison, descending.
	var order_ok := true
	var last := 1 << 40
	for g in grows:
		var idx := int(String(g.name).trim_prefix("Garrison_"))
		if (by_idx.get(idx, {}) as Dictionary).has("garrison"):
			var gv := int(by_idx[idx]["garrison"])
			if gv > last:
				order_ok = false
			last = gv
	_ok("FH7 GARRISON rows run largest garrison first", order_ok)
	# The sort never reads an absent garrison as 0, and the filter is by faction.
	var fix: Array = [
		{"index": 0, "faction": 1, "pop": 900, "garrison": 5},
		{"index": 1, "faction": 1, "pop": 5000},
		{"index": 2, "faction": 2, "pop": 100, "garrison": 99},
		{"index": 3, "faction": 1, "pop": 100, "garrison": 5},
		{"index": 4, "faction": 1, "pop": 100, "garrison": 7}]
	var ord: Array = []
	for p in _fr._garrison_places(fix, 1):
		ord.append(int((p as Dictionary)["index"]))
	_ok("FH7 GARRISON order is garrison desc, ties by population, no-figure last, other factions out",
		ord == [4, 0, 3, 1], str(ord))
	await _shot("%s_military" % ("phone" if _phone else "desktop"))

	# A row opens the Place editor on that settlement and closes the hub.
	if grows.size() > 0:
		var pick := grows[grows.size() - 1] as Button
		var pidx := int(String(pick.name).trim_prefix("Garrison_"))
		pick.pressed.emit()
		await _frames(6)
		_ok("FH7 GARRISON a row closes the hub", not _fr.visible)
		_ok("FH7 GARRISON ... and opens the Place editor on that settlement",
			_app.place_editor_window.visible and _app.place_editor_window._index == pidx,
			"visible=%s index=%s want=%d" % [_app.place_editor_window.visible, str(_app.place_editor_window._index), pidx])
		_app.place_editor_window.hide()
		await _frames(2)

	# Truncation, stated on screen with the true count.
	_fr._garrison_cap = 1
	_fr.open(fa, "military")
	await _frames(8)
	_ok("FH7 GARRISON truncated: one row drawn when the cap is 1", _named_like(_pane("military"), "Garrison_").size() == 1)
	var tl := _pane("military").find_child("GarrisonTruncated", true, false) as Label
	_ok("FH7 GARRISON ... and the block says so with the true place count",
		tl != null and tl.text.contains("1 largest of %d places" % want_idx.size()),
		tl.text if tl != null else "<none>")
	_fr._garrison_cap = _fr.GARRISON_ROW_CAP

	# ------------------------------------------------------------- the conflicts
	var all_conf: Array = _bridge.conflict_list()
	var covered_empty := false
	for f in ids:
		_fr.open(int(f), "military")
		await _frames(6)
		var cp := _pane("military")
		var want: Array = []
		for c in all_conf:
			if Array((c as Dictionary).get("sides", PackedInt32Array())).has(int(f)):
				want.append(int((c as Dictionary).get("id", 0)))
		want.sort()
		var got: Array = []
		for b in _named_like(cp, "Conflict_"):
			got.append(int(String(b.name).trim_prefix("Conflict_")))
		got.sort()
		_ok("FH7 CONFLICTS faction %s: the rows are exactly the conflicts naming it" % names.get(int(f), "?"),
			got == want, "got %s want %s" % [str(got), str(want)])
		if want.is_empty():
			var note := cp.find_child("ConflictsNote", true, false) as Label
			_ok("FH7 CONFLICTS faction %s on no conflict: one honest line, no rows" % names.get(int(f), "?"),
				note != null and note.text.contains("None of the %d drawn conflicts names this faction" % all_conf.size()),
				note.text if note != null else "<none>")
			covered_empty = true
	_ok("FH7 CONFLICTS the probe covered a faction on no conflict", covered_empty)
	# The empty-sided conflict is under nobody.
	var any3 := false
	for f in ids:
		_fr.open(int(f), "military")
		await _frames(4)
		if _pane("military").find_child("Conflict_%d" % c3, true, false) != null:
			any3 = true
	_ok("FH7 CONFLICTS a conflict with no sides appears under no faction", not any3)
	_fr.open(fa, "military")
	await _frames(6)
	var cpa := _pane("military")
	_ok("FH7 CONFLICTS fa sees C1 and not C2", cpa.find_child("Conflict_%d" % c1, true, false) != null
		and cpa.find_child("Conflict_%d" % c2, true, false) == null)
	var row1 := cpa.find_child("Conflict_%d" % c1, true, false) as Button
	_ok("FH7 CONFLICTS the row names the conflict, its kind and the OTHER side only",
		row1 != null and row1.text.to_lower().contains("fh7 war of a and b") and row1.text.to_lower().contains("battle")
		and row1.text.to_lower().contains("against " + String(names.get(fb, "?")).to_lower())
		and not row1.text.to_lower().contains("against " + String(names.get(fa, "?")).to_lower()),
		row1.text if row1 != null else "<none>")
	_ok("FH7 CONFLICTS the filter is the static `_conflicts_of` the tab uses (fixture)",
		_fr._conflicts_of([{"id": 1, "sides": PackedInt32Array([3, 4])}, {"id": 2, "sides": PackedInt32Array()},
			{"id": 3, "sides": PackedInt32Array([4])}], 4).size() == 2
		and _fr._conflicts_of([{"id": 1, "sides": PackedInt32Array([3, 4])}], 5).is_empty())

	# NO-EDIT: no editing control anywhere in the pane; the block's buttons are its rows.
	var edit_ctl := _count(cpa, func(n): return n is CheckBox or n is OptionButton or n is LineEdit \
		or n is SpinBox or n is Slider or n is TextEdit)
	_ok("FH7 NO-EDIT the Military pane has no toggle, picker, field, spinner or slider", edit_ctl == 0,
		"%d editing controls" % edit_ctl)
	var cblock := cpa.find_child("ConflictsBlock", true, false)
	var cbtn := _count(cblock, func(n): return n is Button)
	_ok("FH7 NO-EDIT the conflicts block's only buttons are its rows", cbtn == _named_like(cblock, "Conflict_").size(),
		"%d buttons, %d rows" % [cbtn, _named_like(cblock, "Conflict_").size()])

	# A side edited in the dock moves the row on the next open (no stale cache).
	_ok("FH7 fixture: the dock-side edit (C2 gains fa) was accepted",
		bool(_bridge.conflict_update(c2, {"sides": PackedInt32Array([fb, fa])}).get("ok", false)))
	_fr.open(fa, "military")
	await _frames(6)
	_ok("FH7 CONFLICTS a side added elsewhere shows on the next open",
		_pane("military").find_child("Conflict_%d" % c2, true, false) != null)
	_bridge.conflict_update(c2, {"sides": PackedInt32Array([fb])})

	# A row opens the right dock's Conflict context on that id and closes the hub.
	_fr.open(fa, "military")
	await _frames(6)
	var open_btn := _pane("military").find_child("Conflict_%d" % c1, true, false) as Button
	_ok("FH7 CONFLICTS the row to open exists", open_btn != null)
	if open_btn != null:
		open_btn.pressed.emit()
		await _frames(6)
		_ok("FH7 CONFLICTS a row closes the hub", not _fr.visible)
		_ok("FH7 CONFLICTS ... and opens the right dock's Conflict context on that conflict",
			_app.right_dock_ctrl._context == "conflict" and _app.right_dock_ctrl._conflict_id == c1,
			"ctx=%s id=%s want=%d" % [str(_app.right_dock_ctrl._context), str(_app.right_dock_ctrl._conflict_id), c1])
		await _shot("%s_military_conflict_dock" % ("phone" if _phone else "desktop"))

	# ---------------------------------------------------------------- KEPT / RF-03
	_fr.open(fa, "military")
	await _frames(6)
	var go := _full_breakdown()
	_ok("FH7 KEPT the Full breakdown navigation is still there", go != null)
	var n_before := _named_like(_pane("military"), "Garrison_").size()
	_fr._mark_data_stale()
	_ok("FH7 RF-03 _mark_data_stale marks the military cache stale", not _fr._military_ready)
	_fr.refresh_after_edit()
	await _frames(6)
	_ok("FH7 RF-03 a stale mark and rebuild leave the same garrison rows",
		_named_like(_pane("military"), "Garrison_").size() == n_before and _fr._military_ready,
		"%d vs %d" % [_named_like(_pane("military"), "Garrison_").size(), n_before])
	_app.select_domain("cartography")
	await _frames(4)
	_fr.open(fa, "military")
	await _frames(6)
	go = _full_breakdown()
	if go != null:
		go.pressed.emit()
		await _frames(6)
		await _sheet("left")
		_ok("FH7 KEPT Full breakdown closes the hub and opens Civilization > Military",
			not _fr.visible and _count(_app.workspace_panel("civilization"), func(n): return n is Button and (n as Button).is_visible_in_tree() and (n as Button).text.to_lower().contains("fortified")) > 0,
			"visible=%s" % _fr.visible)

	# ------------------------------------------------------------------ phone
	if _phone:
		_fr.open(fa, "military")
		await _frames(8)
		var small := _count(_pane("military"), func(n): return n is Button and (n as Button).visible and (n as Button).size.y + 0.5 < DccTheme.PHONE_TAP_MIN)
		_ok("FH7 PHONE every Military button is at least 44 px tall", small == 0, "short=%d" % small)
		_ok("FH7 PHONE the Military tab has Garrison and Conflict rows to measure",
			_named_like(_pane("military"), "Garrison_").size() > 0 and _named_like(_pane("military"), "Conflict_").size() > 0)
		_ok("FH7 PHONE one scroller and no TabContainer",
			_count(_fr, func(n): return n is ScrollContainer) == 1 and _count(_fr, func(n): return n is TabContainer) == 0)

	# restore the world
	for id in seeded:
		_bridge.conflict_delete(int(id))
	_fr.hide()
	await _frames(2)
	_fh7_done = true


## The Military tab's "Full breakdown" navigation button, or null.
func _full_breakdown() -> Button:
	var btns: Array = []
	_collect_of(_pane("military"), btns, Button)
	for b in btns:
		if (b as Button).text.to_lower().begins_with("full breakdown"):
			return b
	return null


# -- FH-8: the History tab -----------------------------------------------------
## Protects: FH-8, the History tab (`FACTION_HUB_DESIGN.md` §3.7). Every readout is
## compared with something the tab did not compute:
##   NOTIMELINE  with no timeline recorded the tab says so in ONE line and draws no
##               table, no row and no year.
##   AGGREGATE   the "held by year span" rows equal, for three factions, the live
##               roster's own `settlement_count` read at the moment each year was
##               recorded (an anchor that never touches the periods call), and the
##               engine's `civ_settlement_ownership_periods` re-aggregated here by a
##               separate loop agrees with that anchor.
##   CHANGES     the ownership-change rows equal the edits the probe itself made
##               (three places moved A to B, one B to A, one deleted), by name and
##               direction and window; a faction with no change says so in one line.
##   STATIC      `_history_rollup` against a hand-built fixture with literal
##               expected values, covering gaps, current spans, appears/vanishes and
##               the omitted-key rule -- the faction filter is what a mutation of it breaks.
##   VAULT       the Linked-notes row equals `vault_entity_summary` for zero links and
##               for an attached one, and its action opens the Vault.
##   NO-EDIT     the pane holds no field, picker, toggle, spinner or slider, and its
##               only buttons are place rows and the vault action; manual entries
##               are one NOT BUILT line.
##   CACHE       one fetch per staleness across faction switches; stale after
##               `refresh_after_edit` and `open`; the scan cap is stated on screen.
##   PHONE       every History button >= 44 px on both axes, one scroller.
## Years 10 / 75 / 130 are used on purpose: a result that defaulted a missing year to
## 0 could not pass for a real one. Prints a TIMING line (median with min..max).
## What it cannot see: a timeline of thousands of settlements (the cap's reason is the
## measurement printed here, not a test).
func _run_history_tab(ids: Array) -> void:
	var fa: int = ids[0]
	var fb: int = ids[1]
	var fc: int = ids[2] if ids.size() > 2 else -1
	var names := _names()
	var form := "phone" if _phone else "desktop"
	var no_year_zero := RegEx.create_from_string("\\byears? 0\\b")

	# ------------------------------------------------------------ NOTIMELINE
	_ok("FH8 precondition: the world has no recorded timeline yet",
		_bridge.get_civ_timeline_years().is_empty(), str(_bridge.get_civ_timeline_years()))
	_fr.open(fa, "history")
	await _frames(8)
	var hp := _pane("history")
	_ok("FH8 the History pane is built", hp != null)
	if hp == null:
		return
	var body := hp.find_child("HistoryBody", true, false) as VBoxContainer
	_ok("FH8 NOTIMELINE the body exists", body != null)
	if body == null:
		return
	_ok("FH8 NOTIMELINE ONE line, and it says no timeline is recorded",
		body.get_child_count() == 1 and body.get_child(0) is Label
		and (body.get_child(0) as Label).text.begins_with("No timeline is recorded"),
		"%d children: %s" % [body.get_child_count(), _text_of(body).left(160)])
	_ok("FH8 NOTIMELINE no table, row or year is drawn",
		_named_like(hp, "HistoryRun_").is_empty() and _named_like(hp, "HistoryPlace_").is_empty()
		and _named_like(hp, "HistoryChange_").is_empty()
		and no_year_zero.search(_text_of(hp)) == null
		and not _text_of(body).contains("held"), _text_of(body))
	_ok("FH8 NOTIMELINE the manual-entries line and the vault row are still there",
		hp.find_child("HistoryManualNotBuilt", true, false) != null
		and hp.find_child("HistoryVaultOpen", true, false) != null)
	_ok("FH8 NO-EDIT (no timeline) no edit control on the pane", _history_violations(hp).is_empty(),
		str(_history_violations(hp)))
	await _shot("%s_history_notimeline" % form)

	# ----------------------------------------------------- the manual-entries line
	var manual := hp.find_child("HistoryManualNotBuilt", true, false) as Label
	_ok("FH8 MANUAL one honest line: not built, and why (no precedence rule)",
		manual != null and manual.text.begins_with("Not built") and manual.text.contains("which wins"),
		manual.text if manual != null else "missing")
	var msec := hp.find_child("HistoryManual", true, false)
	_ok("FH8 MANUAL its section holds nothing but that line (no control)",
		msec != null and _count(msec, func(n): return n is Button or n is LineEdit or n is TextEdit or n is Range) == 0)

	# -------------------------------------------------------------- the fixture
	var base: Array = _bridge.settlements()
	var of_a: Array = []
	var of_b: Array = []
	for i in base.size():
		var f := int((base[i] as Dictionary).get("faction", 0))
		if f == fa:
			of_a.append(i)
		elif f == fb:
			of_b.append(i)
	_ok("FH8 precondition: faction A holds >= 5 settlements and B >= 1", of_a.size() >= 5 and of_b.size() >= 1,
		"A=%d B=%d" % [of_a.size(), of_b.size()])
	if of_a.size() < 5 or of_b.size() < 1:
		return
	var moved: Array = []  # [{tid, name}]
	for k in 3:
		var s: Dictionary = base[int(of_a[k])]
		moved.append({"tid": int(s["tid"]), "name": String(s["name"])})
	var back_s: Dictionary = base[int(of_b[0])]
	var back := {"tid": int(back_s["tid"]), "name": String(back_s["name"])}
	var doom_s: Dictionary = base[int(of_a[3])]
	var doomed := {"tid": int(doom_s["tid"]), "name": String(doom_s["name"])}
	var all_tids: Array = []
	for s in base:
		all_tids.append(int((s as Dictionary)["tid"]))

	_bridge.civ_add_year(10)
	var cnt10 := _counts()
	_bridge.civ_add_year(75)
	var edits_ok := true
	for m in moved:
		edits_ok = bool(_bridge.civ_edit_settlement(_index_of(int(m["tid"])), {"faction": fb})) and edits_ok
	edits_ok = bool(_bridge.civ_edit_settlement(_index_of(int(back["tid"])), {"faction": fa})) and edits_ok
	edits_ok = bool(_bridge.civ_delete_settlement(_index_of(int(doomed["tid"])))) and edits_ok
	var cnt75 := _counts()
	_bridge.civ_add_year(130)
	_ok("FH8 fixture: every edit was accepted and the years are 10, 75, 130",
		edits_ok and _bridge.get_civ_timeline_years() == PackedInt64Array([10, 75, 130]),
		"%s %s" % [str(edits_ok), str(_bridge.get_civ_timeline_years())])
	_ok("FH8 fixture: the edits moved the live counts as intended (A -3 +1 -1, B +3 -1)",
		int(cnt75[fa]) == int(cnt10[fa]) - 3, "A %d -> %d" % [int(cnt10[fa]), int(cnt75[fa])])

	# ---------------------------------------------- AGGREGATE vs two independent reads
	var spans_of := {}
	for t in all_tids:
		spans_of[t] = _bridge.civ_settlement_ownership_periods(int(t))
	var agrees := true
	var detail := ""
	var anchors := {10: cnt10, 75: cnt75, 130: cnt75}
	for f in ids:
		for y in [10, 75, 130]:
			var held := 0
			for t in all_tids:
				for sp in (spans_of[t] as Array):
					var e := int(sp["end_year"]) if sp.has("end_year") else 130
					if int(sp["faction_id"]) == int(f) and int(sp["start_year"]) <= y and y <= e:
						held += 1
			if held != int((anchors[y] as Dictionary)[int(f)]):
				agrees = false
				detail += " f%d y%d periods=%d roster=%d" % [int(f), y, held, int((anchors[y] as Dictionary)[int(f)])]
	_ok("FH8 AGGREGATE the engine's periods, re-aggregated here, equal the roster counts at each recorded year",
		agrees, detail)

	_fr.open(fa, "history")
	await _frames(8)
	hp = _pane("history")
	body = hp.find_child("HistoryBody", true, false) as VBoxContainer
	_ok("FH8 the body filled on its first show and the cache is warm",
		_fr._history_ready and body.get_child_count() > 1, "ready=%s kids=%d" % [str(_fr._history_ready), body.get_child_count()])
	var checked: Array = [fa, fb]
	if fc > 0:
		checked.append(fc)
	for f in checked:
		_fr.open(int(f), "history")
		await _frames(6)
		hp = _pane("history")
		var want := _expected_runs(cnt10, cnt75, int(f))
		var got := _run_texts(hp)
		_ok("FH8 AGGREGATE faction %d: the year-span rows equal the roster anchors" % int(f), got == want,
			"got %s want %s" % [str(got), str(want)])
		_ok("FH8 AGGREGATE faction %d: no year 0 anywhere" % int(f), no_year_zero.search(_text_of(hp)) == null)

	# --------------------------------------------------------------- CHANGES
	var name_a: String = names[fa]
	var name_b: String = names[fb]
	var want_a: Array = []
	for m in moved:
		want_a.append([int(m["tid"]), "%s: %s → %s (years 10-75)" % [m["name"], name_a, name_b]])
	want_a.append([int(doomed["tid"]), "%s: %s → not in the records (years 10-75)" % [doomed["name"], name_a]])
	want_a.append([int(back["tid"]), "%s: %s → %s (years 10-75)" % [back["name"], name_b, name_a]])
	want_a.sort_custom(func(x, y): return int(x[0]) < int(y[0]))
	var want_a_txt: Array = []
	for w in want_a:
		want_a_txt.append(String(w[1]))
	_fr.open(fa, "history")
	await _frames(8)
	hp = _pane("history")
	_ok("FH8 CHANGES faction A: its five changes, in order, from the edits the probe made",
		_change_texts(hp) == _lcs(want_a_txt), "got %s\nwant %s" % [str(_change_texts(hp)), str(want_a_txt)])
	var live_rows := _named_like(hp, "HistoryPlace_")
	_ok("FH8 CHANGES a place that still exists is a button; the deleted one is a plain label",
		live_rows.size() == 4 and _named_like(hp, "HistoryChange_").size() == 1,
		"%d buttons, %d labels" % [live_rows.size(), _named_like(hp, "HistoryChange_").size()])
	var want_b: Array = []
	for m in moved:
		want_b.append([int(m["tid"]), "%s: %s → %s (years 10-75)" % [m["name"], name_a, name_b]])
	want_b.append([int(back["tid"]), "%s: %s → %s (years 10-75)" % [back["name"], name_b, name_a]])
	want_b.sort_custom(func(x, y): return int(x[0]) < int(y[0]))
	var want_b_txt: Array = []
	for w in want_b:
		want_b_txt.append(String(w[1]))
	_fr.open(fb, "history")
	await _frames(8)
	_ok("FH8 CHANGES faction B: three gained and one lost, the same rows seen from the other side",
		_change_texts(_pane("history")) == _lcs(want_b_txt), "got %s\nwant %s" % [str(_change_texts(_pane("history"))), str(want_b_txt)])
	if fc > 0:
		_fr.open(fc, "history")
		await _frames(8)
		var cnone := _pane("history").find_child("HistoryChangesNone", true, false) as Label
		_ok("FH8 CHANGES a faction untouched by the edits says so in one line and lists none",
			cnone != null and cnone.text == "None between the recorded years."
			and _change_texts(_pane("history")).is_empty(), cnone.text if cnone != null else "missing")

	_fr.open(fa, "history")
	await _frames(8)
	await _shot("%s_history_fa" % form)  ## taken while the hub still shows (the place row below hides it)
	# A place row opens that place's editor and closes the hub.
	var prow := _named_like(_pane("history"), "HistoryPlace_")
	if not prow.is_empty():
		(prow[0] as Button).pressed.emit()
		await _frames(6)
		_ok("FH8 CHANGES a place row closes the hub and opens that place's editor",
			not _fr.visible and _app.place_editor_window.visible, "hub=%s editor=%s" % [str(_fr.visible), str(_app.place_editor_window.visible)])
		_app.place_editor_window.hide()

	# ----------------------------------------------------------------- STATIC
	_check_rollup_fixture()

	# ------------------------------------------------------------------ VAULT
	_fr.open(fa, "history")
	await _frames(8)
	hp = _pane("history")
	var sum0: Dictionary = _bridge.vault_entity_summary("faction", fa)
	var vline := hp.find_child("HistoryVaultSummary", true, false) as Label
	var vbtn := hp.find_child("HistoryVaultOpen", true, false) as Button
	_ok("FH8 VAULT with no link: the row reads No Markdown notes linked and offers Attach",
		int(sum0.get("link_count", 0)) == 0 and vline != null and vline.text == "No Markdown notes linked."
		and vbtn != null and vbtn.text.to_lower() == "attach a markdown note…", "%s | %s" % [str(sum0), vline.text if vline != null else "-"])
	## A real note in a throwaway vault under user:// (the way _pechronos_probe does it):
	## attaching needs a connected vault. Removed again below; the settings file is untouched.
	DirAccess.make_dir_recursive_absolute(FH8_VAULT + "/Factions")
	var nf := FileAccess.open(FH8_VAULT + "/Factions/FH8.md", FileAccess.WRITE)
	nf.store_string("# FH8

A faction note.
")
	nf.close()
	var vinfo: Dictionary = _bridge.vault_connect(ProjectSettings.globalize_path(FH8_VAULT), "FH8 probe vault")
	_ok("FH8 fixture: the throwaway vault connected", bool(vinfo.get("ok", false)), str(vinfo))
	var att: Dictionary = _bridge.vault_attach("faction", fa, name_a, "Factions/FH8.md", "")
	_ok("FH8 fixture: a vault link on faction A was accepted", bool(att.get("ok", false)), str(att))
	_fr.open(fa, "history")
	await _frames(8)
	hp = _pane("history")
	var sum1: Dictionary = _bridge.vault_entity_summary("faction", fa)
	vline = hp.find_child("HistoryVaultSummary", true, false) as Label
	vbtn = hp.find_child("HistoryVaultOpen", true, false) as Button
	var want_line := "%d linked note%s — %s" % [int(sum1.get("link_count", 0)),
		"" if int(sum1.get("link_count", 0)) == 1 else "s", String({"connected": "✓ Connected", "stale": "● Source changed", "local_changes": "● Local changes", "cached": "● Cached — source unavailable", "missing": "✕ Source missing", "unbound": "● Vault not connected on this device"}.get(String(sum1.get("status", "")), ""))]
	_ok("FH8 VAULT with a link: the row equals vault_entity_summary and offers Linked notes",
		int(sum1.get("link_count", 0)) == 1 and vline != null and vline.text == want_line
		and vbtn != null and vbtn.text.to_lower() == "linked notes…", "%s | %s | want %s" % [str(sum1), vline.text if vline != null else "-", want_line])
	if vbtn != null:
		vbtn.pressed.emit()
		await _frames(6)
		_ok("FH8 VAULT the action closes the hub and opens the Vault", not _fr.visible and _app.vault_window.visible,
			"hub=%s vault=%s" % [str(_fr.visible), str(_app.vault_window.visible)])
		_app.vault_window.hide()
	var links: Array = _bridge.vault_links_for("faction", fa)
	for l in links:
		_bridge.vault_detach(String((l as Dictionary).get("link_id", "")))
	_ok("FH8 fixture: the vault link was detached again", _bridge.vault_links_for("faction", fa).is_empty())
	_bridge.vault_disconnect()
	DirAccess.remove_absolute(FH8_VAULT + "/Factions/FH8.md")
	DirAccess.remove_absolute(FH8_VAULT + "/Factions")
	DirAccess.remove_absolute(FH8_VAULT)

	# --------------------------------------------------------------- NO-EDIT
	_fr.open(fa, "history")
	await _frames(8)
	hp = _pane("history")
	_ok("FH8 NO-EDIT the pane holds no field, picker, toggle, spinner or slider; its only buttons are place rows and the vault action",
		_history_violations(hp).is_empty(), str(_history_violations(hp)))
	_ok("FH8 NO-EDIT there is something to be a violation of (place rows and the vault action exist)",
		_named_like(hp, "HistoryPlace_").size() == 4 and hp.find_child("HistoryVaultOpen", true, false) != null)

	# ----------------------------------------------------------------- CACHE
	_fr._history_usec = -1
	_fr.open(fb, "history")  # open() marks stale and refetches
	await _frames(6)
	_ok("FH8 CACHE open() refetches (stale mark honoured)", int(_fr._history_usec) >= 0, "usec=%d" % int(_fr._history_usec))
	_fr._history_usec = -1
	_fr._selected = fa
	_fr._drop_panes()
	_fr._show_active_tab()
	await _frames(6)
	_ok("FH8 CACHE a faction switch reuses the cached rows (no second fetch)",
		int(_fr._history_usec) == -1 and _fr._history_ready, "usec=%d" % int(_fr._history_usec))
	_fr.refresh_after_edit()
	await _frames(2)
	_ok("FH8 RF-03 refresh_after_edit marks the history cache stale and refetches on the visible tab",
		_fr._history_ready and int(_fr._history_usec) >= 0, "ready=%s usec=%d" % [str(_fr._history_ready), int(_fr._history_usec)])
	_fr._mark_data_stale()
	_ok("FH8 RF-03 _mark_data_stale drops flag and payload together",
		not _fr._history_ready and _fr._history_rows.is_empty() and _fr._history_years.is_empty())
	# The scan cap is stated on screen and never silent.
	var total_tids := int((_bridge.civ_year_diff(10) as Dictionary)["present"].size())
	_fr._history_tid_cap = 3
	_fr.open(fa, "history")
	await _frames(8)
	var capped := _pane("history").find_child("HistoryScanCapped", true, false) as Label
	_ok("FH8 CACHE a lowered scan cap says how many were read and how many left out",
		capped != null and capped.text.begins_with("Read 3 of the %d settlements" % total_tids)
		and capped.text.contains("the other %d are left out" % (total_tids - 3)), capped.text if capped != null else "missing")
	_fr._history_tid_cap = _fr.HISTORY_TID_CAP
	_fr.open(fa, "history")
	await _frames(8)
	_ok("FH8 CACHE ... and at the real cap nothing is left out, so no such line",
		_pane("history").find_child("HistoryScanCapped", true, false) == null and _fr._history_scanned == _fr._history_total
		and _fr._history_total == total_tids, "scanned %d of %d, independent total %d" % [_fr._history_scanned, _fr._history_total, total_tids])

	# ------------------------------------------------------------------ PHONE
	if _phone:
		_fr.open(fa, "history")
		await _frames(8)
		var hpp := _pane("history")
		var short := _count(hpp, func(n): return n is Button and (n as Button).is_visible_in_tree() and ((n as Button).size.y + 0.5 < DccTheme.PHONE_TAP_MIN or (n as Button).size.x + 0.5 < DccTheme.PHONE_TAP_MIN))
		_ok("FH8 PHONE every History button is at least 44 px on both axes", short == 0, "short=%d" % short)
		_ok("FH8 PHONE there are place rows and the vault action to measure",
			_named_like(hpp, "HistoryPlace_").size() > 0 and hpp.find_child("HistoryVaultOpen", true, false) != null)
		_ok("FH8 PHONE one scroller and no TabContainer",
			_count(_fr, func(n): return n is ScrollContainer) == 1 and _count(_fr, func(n): return n is TabContainer) == 0)
		await _shot("phone_history_fa_phonecheck")

	# ----------------------------------------------------------------- TIMING
	var t3 := await _time_history(7)
	print("FACTIONHUB TIMING history (%s) years=3: median %.1f ms, min %.1f, max %.1f, %d tids scanned" % [
		form, t3[0], t3[1], t3[2], int(_fr._history_scanned)])
	for y in range(131, 178):
		_bridge.civ_add_year(y)
	var t50 := await _time_history(7)
	print("FACTIONHUB TIMING history (%s) years=%d: median %.1f ms, min %.1f, max %.1f, %d tids scanned" % [
		form, _bridge.get_civ_timeline_years().size(), t50[0], t50[1], t50[2], int(_fr._history_scanned)])
	## The per-faction rollup alone (pure, over the cached rows), on the biggest faction.
	var big := fa
	for f in ids:
		if int(_counts().get(int(f), 0)) > int(_counts().get(big, 0)):
			big = int(f)
	var ru: Array = []
	for k in 7:
		var t0 := Time.get_ticks_usec()
		_fr._history_rollup(_fr._history_rows, _fr._history_years, big)
		ru.append(float(Time.get_ticks_usec() - t0) / 1000.0)
	ru.sort()
	print("FACTIONHUB TIMING history rollup (%s) years=%d rows=%d faction=%d: median %.1f ms, min %.1f, max %.1f" % [
		form, _fr._history_years.size(), _fr._history_rows.size(), big, ru[3], ru[0], ru[6]])
	_fr.open(fa, "history")
	await _frames(8)
	var runs_after := _run_texts(_pane("history"))
	_ok("FH8 a 50-year timeline still draws a bounded, merged run list",
		runs_after.size() >= 1 and runs_after.size() <= _fr.HISTORY_ROW_CAP, str(runs_after.size()))
	_fr.hide()
	await _frames(2)
	_fh8_done = true


## Median, min and max, in milliseconds, of `n` cold `_ensure_history()` calls (each
## after `_mark_data_stale()`), read from the window's own `Time.get_ticks_usec()` figure.
func _time_history(n: int) -> Array:
	var us: Array = []
	for k in n:
		_fr._mark_data_stale()
		_fr._ensure_history()
		us.append(float(_fr._history_usec))
	us.sort()
	return [us[n / 2] / 1000.0, us[0] / 1000.0, us[n - 1] / 1000.0]


## fid -> settlement_count from the live roster: the anchor the periods are checked against.
func _counts() -> Dictionary:
	var out := {}
	for f in _bridge.get_factions():
		out[int((f as Dictionary).get("id", 0))] = int((f as Dictionary).get("settlement_count", 0))
	return out


## Index in `settlements()` of the settlement with stable id `tid`, or -1.
func _index_of(tid: int) -> int:
	var roster: Array = _bridge.settlements()
	for i in roster.size():
		if int((roster[i] as Dictionary).get("tid", 0)) == tid:
			return i
	return -1


## "settlements held" wording, written out here rather than shared with the shell.
func _held_text(n: int) -> String:
	if n == 0:
		return "none held"
	return "%d settlement%s held" % [n, "" if n == 1 else "s"]


## The year-span rows the tab must show for faction `f`, from the roster anchors:
## years 10, 75, 130 hold `c10`, `c75`, `c75` settlements, merged where equal.
func _expected_runs(c10: Dictionary, c75: Dictionary, f: int) -> Array:
	var a := int(c10[f])
	var b := int(c75[f])
	if a == b:
		return ["years 10-130: %s" % _held_text(a)]
	return ["year 10: %s" % _held_text(a), "years 75-130: %s" % _held_text(b)]


## The year-span rows on screen, in order.
func _run_texts(pane: Node) -> Array:
	var rows := _named_like(pane, "HistoryRun_")
	rows.sort_custom(func(x, y): return int(String(x.name).trim_prefix("HistoryRun_")) < int(String(y.name).trim_prefix("HistoryRun_")))
	var out: Array = []
	for r in rows:
		if (r as Label).name.begins_with("HistoryRunsHead") or (r as Label).name.begins_with("HistoryRunsTrunc"):
			continue
		out.append((r as Label).text)
	return out


## Lower-cased copies of `texts`: the phone draws button captions in capitals
## (`DccWidgets.action`), so row text is compared case-insensitively.
func _lcs(texts: Array) -> Array:
	var out: Array = []
	for s in texts:
		out.append(String(s).to_lower())
	return out


## The change rows on screen (buttons and labels alike), in tree order, lower-cased
## (see `_lcs`).
func _change_texts(pane: Node) -> Array:
	var rows: Array = []
	rows.append_array(_named_like(pane, "HistoryPlace_"))
	rows.append_array(_named_like(pane, "HistoryChange_"))
	rows.sort_custom(func(x, y): return int(String(x.name).get_slice("_", 1)) < int(String(y.name).get_slice("_", 1)))
	var out: Array = []
	for r in rows:
		out.append(((r as Button).text if r is Button else (r as Label).text).to_lower())
	return out


## Everything on `pane` that must not be there: any text field, range or tree, and
## any button that is not a place row (`HistoryPlace_*`) or the vault action
## (`HistoryVaultOpen`) or is a Button subclass (picker, toggle, colour button, menu).
## Empty means the tab is read-only plus navigation.
func _history_violations(pane: Node) -> Array:
	var bad: Array = []
	var stack: Array = [pane]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is LineEdit or n is TextEdit or n is Range or n is ItemList or n is Tree:
			bad.append("%s (%s)" % [n.name, n.get_class()])
		elif n is Button:
			var ok_name := String(n.name).begins_with("HistoryPlace_") or n.name == &"HistoryVaultOpen"
			if not ok_name or n.get_class() != "Button":
				bad.append("%s (%s '%s')" % [n.name, n.get_class(), (n as Button).text])
		for c in n.get_children():
			stack.append(c)
	return bad


## `_history_rollup` against a hand-built fixture with literal answers. Years 1-4.
## tid 5: faction 7 in year 1, absent in 2 (a gap), faction 8 from 3 (current).
## tid 6: 7 in 2, 8 in 3 (an immediate transfer), absent in 4.
## tid 9: 7 from 2, current (first appears after the first recorded year).
func _check_rollup_fixture() -> void:
	var years := [1, 2, 3, 4]
	var rows := [
		{"tid": 5, "spans": [
			{"start_year": 1, "faction_id": 7, "current": false, "end_year": 1},
			{"start_year": 3, "faction_id": 8, "current": true}]},
		{"tid": 6, "spans": [
			{"start_year": 2, "faction_id": 7, "current": false, "end_year": 2},
			{"start_year": 3, "faction_id": 8, "current": false, "end_year": 3}]},
		{"tid": 9, "spans": [{"start_year": 2, "faction_id": 7, "current": true}]},
	]
	var r7: Dictionary = _fr._history_rollup(rows, years, 7)
	_ok("FH8 STATIC faction 7 runs: 1, 2, then 1 for years 3-4",
		r7["runs"] == [{"y0": 1, "y1": 1, "held": 1}, {"y0": 2, "y1": 2, "held": 2}, {"y0": 3, "y1": 4, "held": 1}], str(r7["runs"]))
	var c7: Array = r7["changes"]
	_ok("FH8 STATIC faction 7 changes: tid 5 vanishes 1-2, tids 6 and 9 appear at 2 (absent in year 1), tid 6 lost to 8 over 2-3, nothing else",
		c7.size() == 4
		and c7[0]["tid"] == 5 and c7[0]["kind"] == "vanishes" and c7[0]["y0"] == 1 and c7[0]["y1"] == 2
		and c7[1]["tid"] == 6 and c7[1]["kind"] == "appears" and c7[1]["y0"] == 2 and c7[1]["y1"] == 2
		and c7[2]["tid"] == 9 and c7[2]["kind"] == "appears" and c7[2]["y0"] == 2 and c7[2]["y1"] == 2
		and c7[3]["tid"] == 6 and c7[3]["kind"] == "lost" and c7[3]["to"] == 8 and c7[3]["y0"] == 2 and c7[3]["y1"] == 3, str(c7))
	_ok("FH8 STATIC an absent side is an omitted key, never a sentinel faction id",
		not (c7[0] as Dictionary).has("to") and not (c7[1] as Dictionary).has("from"), str(c7))
	var r8: Dictionary = _fr._history_rollup(rows, years, 8)
	_ok("FH8 STATIC faction 8 runs: none for years 1-2, 2 in year 3, 1 in year 4",
		r8["runs"] == [{"y0": 1, "y1": 2, "held": 0}, {"y0": 3, "y1": 3, "held": 2}, {"y0": 4, "y1": 4, "held": 1}], str(r8["runs"]))
	var c8: Array = r8["changes"]
	_ok("FH8 STATIC faction 8 changes: tid 6 gained from 7 over 2-3, tid 5 appears at 3, tid 6 vanishes 3-4",
		c8.size() == 3
		and c8[0]["tid"] == 6 and c8[0]["kind"] == "gained" and c8[0]["from"] == 7 and c8[0]["y0"] == 2 and c8[0]["y1"] == 3
		and c8[1]["tid"] == 5 and c8[1]["kind"] == "appears" and c8[1]["y0"] == 3 and c8[1]["y1"] == 3
		and c8[2]["tid"] == 6 and c8[2]["kind"] == "vanishes" and c8[2]["y0"] == 3 and c8[2]["y1"] == 4, str(c8))
	var r9: Dictionary = _fr._history_rollup(rows, years, 99)
	_ok("FH8 STATIC a faction no span names has zero held in every year and no change",
		r9["runs"] == [{"y0": 1, "y1": 4, "held": 0}] and (r9["changes"] as Array).is_empty(), str(r9))
	var none: Dictionary = _fr._history_rollup([], [], 7)
	_ok("FH8 STATIC no years gives no runs and no changes (no invented year)",
		(none["runs"] as Array).is_empty() and (none["changes"] as Array).is_empty())
