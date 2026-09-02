extends Node
## Committed probe -- the cross-domain "→ Civilization ▸ Territories"
## jump buttons, which until 2026-08-25 switched the rail and stopped there.
##
## For each one: press it for real, then assert BOTH halves -- the domain is the
## one the label names, and the category the label names is the open one.
## Also re-asserts IN-01's river note and sweeps every rendered string for the
## nine category names v3 retired.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _jump_probe.tscn
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

var _app: Node
var _bridge
var _fail := 0

## Retired by the v3 pass. A rendered string naming one of these as a
## destination is a stale pointer. `Roads`/`Trade`/`Rivers`/`Politics` are
## excluded from the raw-word scan -- they are ordinary English here.
const RETIRED_PATHS := [
	"Roads ▸", "Ports ▸", "Logistics ▸", "Rivers ▸", "Population ▸",
	"Timeline ▸ ", "World ▸ Sculpt", "WORLD ▸ Finalize",
	"Generation Pipeline", "Politics ▸ Recalculate",
]


func _p(s: String) -> void:
	print("JUMP  %s" % s)


func _bad(s: String) -> void:
	_fail += 1
	print("JUMP  FAIL  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


func _find_button(root: Node, exact_text: String) -> Button:
	var all: Array = []
	_walk(root, all)
	for n in all:
		if n is Button and (n as Button).text == exact_text:
			return n
	return null


func _open_category(ws) -> String:
	for e in ws.categories:
		if (e["body"] as Control).visible:
			return String(e["title"])
	return "<none>"


func _all_text() -> Array:
	var all: Array = []
	_walk(_app, all)
	var out: Array = []
	for n in all:
		if n is Label:
			out.append((n as Label).text)
		elif n is RichTextLabel:
			out.append((n as RichTextLabel).get_parsed_text())
		elif n is Button:
			out.append((n as Button).text)
		if n is Control and (n as Control).tooltip_text != "":
			out.append((n as Control).tooltip_text)
	return out


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

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

	var panels: Dictionary = _app._workspace_panels
	var carto = panels["cartography"]
	var civ = panels["civilization"]

	var cases := [
		["cartography", carto, "Draw and edit ways → Civilization ▸ Routes & ways", "civilization", "Routes & ways"],
		["cartography", carto, "Edit territories → Civilization ▸ Territories", "civilization", "Territories"],
		["civilization", civ, "Place an icon → Cartography ▸ Assets & landmarks", "cartography", "Assets & landmarks"],
	]
	for c in cases:
		_app.select_domain(String(c[0]))
		await _frames(4)
		var b := _find_button(c[1], String(c[2]))
		if b == null:
			_bad("button not found: '%s'" % c[2])
			continue
		b.pressed.emit()
		await _frames(6)
		var dom: String = _app.active_domain()
		var cat := _open_category(panels[String(c[3])])
		var ok := dom == String(c[3]) and cat == String(c[4])
		_p("%s  '%s' -> domain=%s open=%s (want %s / %s)" % [
			"PASS " if ok else "FAIL ", c[2], dom, cat, c[3], c[4]])
		if not ok:
			_fail += 1

	## ------------------------------------------------------ the timeline strip
	## **What this stanza used to assert, and why it no longer can.** Until
	## 2026-08-31 the strip held a `TIMELINE` label, a clipped sentence and an
	## `Open Timeline` button that pressed the CIVIL Politics accordion; this
	## probe pressed it and asserted the jump. That button was deliberately
	## deleted -- `shell/app.gd`, the §10 block comment above
	## `_fill_timeline_strip()` ("What this replaces, and why the old reason is
	## gone rather than argued with"), which builds the strip
	## `01-frame-and-tokens.md` §3.7 and `05-right-dock-and-bars.md` §4.2
	## actually author: a collapsed one-liner that expands into a transport, a
	## speed pill group, a scrub track and a year footer. **The design changed;
	## the code is right.** The panel that button jumped to still exists in the
	## CIVIL dock, so nothing was lost -- which is why the assertion is replaced
	## rather than dropped, by one that guards the contract that replaced it.
	_app.select_domain("civilization")
	await _frames(4)
	await _check_timeline_strip()

	# -------------------------------------------------------------- IN-01
	_app.select_domain("world")
	await _frames(4)
	var found := false
	for t in _all_text():
		## Anchored on the note's single owner rather than a hardcoded copy.
		## The 2026-09-01 pass moved Rivers to WORLD ▸ Hydrology and reworded
		## the disclosure; two probes carrying their own copy of the text broke
		## on a wording change that was not a defect. Reading it from
		## `rivers_note()` means only its *absence* can fail this.
		if String(t).find(InfrastructureWorkspace.rivers_note().substr(0, 32)) >= 0:
			found = true
	if found:
		_p("PASS  IN-01 river note present in WORLD")
	else:
		_bad("IN-01 river note missing from WORLD")

	# ------------------------------------------------- retired-name sweep
	## Open every category in all three rails first -- a closed accordion body
	## still has its Labels in the tree, but tooltips on lazily-built rows do
	## not exist until the row does.
	for dom in ["world", "civilization", "cartography"]:
		_app.select_domain(dom)
		await _frames(2)
		var ws = panels[dom]
		for e in ws.categories:
			(e["button"] as Button).pressed.emit()
			await _frames(1)
	await _frames(4)

	var hits := {}
	for t in _all_text():
		var s := String(t)
		for r in RETIRED_PATHS:
			if s.find(r) >= 0:
				hits[r] = s.substr(maxi(0, s.find(r) - 40), 130)
	if hits.is_empty():
		_p("PASS  no retired category name is named as a destination anywhere")
	else:
		for k in hits:
			_bad("retired pointer '%s' still rendered: …%s…" % [k, hits[k]])

	_p("DONE fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)


## Text rendered inside ONE subtree. `_all_text()` is app-wide and could not
## tell a strip assertion from a dock one.
func _sub_text(root: Node) -> String:
	var all: Array = []
	_walk(root, all)
	var out: Array = []
	for n in all:
		if n is Button:
			out.append((n as Button).text)
		elif n is Label:
			out.append((n as Label).text)
	return "\n".join(out)


func _want(cond: bool, what: String) -> void:
	if cond:
		_p("PASS  %s" % what)
	else:
		_bad(what)


## The timeline strip's real contract, asserted by driving the drawn controls
## rather than by calling `DccShell`'s API behind them -- a pill that is drawn
## but never connected passes the second test and fails this one.
##
## In the order §3.7/§4.2 give them: the collapsed form, the whole-strip expand
## target, the transport, the speed ladder, the scrub track and the year footer.
## Plus the six layer toggles, which are the one place this shell draws a
## live-looking control over a missing renderer and which `BUILD_ANSWERS.md`
## §3 therefore requires to carry their own disclosure.
func _check_timeline_strip() -> void:
	var row: Control = _app.timeline_row
	if row == null or not _app.timeline_bar.visible:
		_bad("timeline region is not visible -- the strip cannot be asserted")
		return
	## Every transport control is drawn dead without a world (`TL_UNAVAILABLE`),
	## so a false here would leave the rest of this function asserting the
	## disabled form of a strip the probe meant to exercise live.
	if not _app.tl_available():
		_bad("tl_available() is false after generate() -- the strip would draw dead")
		return

	# -- 1. the collapsed form is the default (`tlOpen` false)
	_want(not bool(_app._tl_expanded), "strip opens collapsed")
	var collapsed_text := _sub_text(row)
	_want(collapsed_text.find("TIMELINE") >= 0, "collapsed strip is labelled TIMELINE")
	_want(collapsed_text.find("expand") >= 0, "collapsed strip offers expand")
	_want(collapsed_text.find(String(_app._tl_year_label())) >= 0,
		"collapsed strip prints the live year (%s)" % _app._tl_year_label())

	# -- 2. the WHOLE strip is the expand target, not a button sitting in it
	var buttons: Array = []
	var all: Array = []
	_walk(row, all)
	for n in all:
		if n is Button:
			buttons.append(n)
	_want(buttons.size() == 1,
		"collapsed strip is one whole-width hit target (found %d buttons)" % buttons.size())
	if buttons.is_empty():
		return
	(buttons[0] as Button).pressed.emit()
	await _frames(6)
	_want(bool(_app._tl_expanded), "pressing the strip expands it")

	# -- 3. transport
	var play: Button = _app._tl_play_button
	if play == null:
		_bad("expanded strip has no play button")
	else:
		_want(not play.disabled, "transport is enabled with a world loaded")
		play.pressed.emit()
		await _frames(2)
		_want(bool(_app.tl_playing), "play arms the clock")
		play.pressed.emit()
		await _frames(2)
		_want(not bool(_app.tl_playing), "and pressing it again stops it")

	# -- 4. the speed ladder binds
	var segs: Dictionary = _app._tl_speed_segments
	var keys: Array = segs.keys()
	keys.sort()
	_want(keys == DccShell.TL_SPEEDS,
		"speed pills are the %s ladder (found %s)" % [str(DccShell.TL_SPEEDS), str(keys)])
	var want_speed: int = 1 if int(_app.tl_speed) != 1 else 100
	if segs.has(want_speed):
		(segs[want_speed] as Button).pressed.emit()
		await _frames(2)
		_want(int(_app.tl_speed) == want_speed, "pressing \u00d7%d sets the speed" % want_speed)
	_app.tl_set_speed(10)
	await _frames(2)

	# -- 5. the scrub track moves the cursor, and the playhead follows it
	var track: Control = _app._tl_track
	if track == null:
		_bad("expanded strip has no scrub track")
	else:
		_want(track.size.x > 0.0, "scrub track has width (%.0f px)" % track.size.x)
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = true
		ev.position = Vector2(track.size.x * 0.5, track.size.y * 0.5)
		track.gui_input.emit(ev)
		await _frames(4)
		var mid := DccShell.TL_YEAR_MIN + int(round(
			0.5 * float(DccShell.TL_YEAR_MAX - DccShell.TL_YEAR_MIN)))
		_want(absi(int(_app.tl_year()) - mid) <= 8,
			"clicking the track's midpoint scrubs to ~%d (got %d)" % [mid, _app.tl_year()])
		var head: ColorRect = _app._tl_head
		_want(head != null and absf(head.position.x - track.size.x * 0.5) <= 4.0,
			"the playhead followed the cursor")

	# -- 6. the year footer: the fixed axis, and the live year between its ends
	var expanded_text := _sub_text(row)
	_want(expanded_text.find("YEAR %d" % DccShell.TL_YEAR_MIN) >= 0,
		"footer prints the low end of the fixed axis")
	_want(expanded_text.find("YEAR %d" % DccShell.TL_YEAR_MAX) >= 0,
		"footer prints the high end")
	_want(expanded_text.find(String(_app._tl_year_label())) >= 0,
		"footer prints the live year (%s)" % _app._tl_year_label())

	# -- 7. the six layer pills, and the disclosure they are required to carry
	var noted := 0
	for lrow in DccShell.TL_LAYERS:
		var lb := _find_button(row, String((lrow as Array)[0]))
		if lb != null and lb.tooltip_text.find(DccShell.TL_LAYER_NOTE) >= 0:
			noted += 1
	_want(noted == DccShell.TL_LAYERS.size(),
		"all %d layer pills carry the 'no layer renders yet' note (%d do)"
			% [DccShell.TL_LAYERS.size(), noted])
	_want(expanded_text.find(DccShell.TL_LAYER_NOTE) >= 0,
		"and the strip itself states it once, on the timeline")

	# -- 8. and it collapses again, restoring the default for the sweep below
	var collapse := _find_button(row, DccIcons.SYMBOLS["chevron"])
	if collapse == null:
		_bad("expanded strip has no collapse control")
		_app._tl_expanded = false
		_app._fill_timeline_strip()
	else:
		collapse.pressed.emit()
		await _frames(4)
		_want(not bool(_app._tl_expanded), "the chevron collapses it again")
	await _frames(2)
