extends Node
## TEMPORARY, untracked probe for the 2026-08-25 conformance sweep.
##
##   1. WW-13 -- paint Commit / Discard gate on the pending draft, not on the
##      committed-plus-pending composite. Driven with real dabs and a real
##      commit, both docks asserted, both directions.
##   2. IN-01 -- the river-entity disclosure really is drawn under WORLD ▸
##      Hydrology now (it was written for that and never wired up).
##   3. Every `A ▸ B` pointer rendered anywhere in the app, dumped for
##      cross-checking against the real v3 structure.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _conform_probe.tscn

var _app: Node
var _bridge
var _fail := 0


func _p(s: String) -> void:
	print("CONFORM  %s" % s)


func _bad(s: String) -> void:
	_fail += 1
	print("CONFORM  FAIL  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


## Every button whose text starts with `needle`, anywhere under `root`.
func _buttons(root: Node, needle: String) -> Array:
	var all: Array = []
	_walk(root, all)
	var hits: Array = []
	for n in all:
		if n is Button and (n as Button).text.strip_edges().ends_with(needle):
			hits.append(n)
	return hits


func _texts(root: Node) -> Array:
	var all: Array = []
	_walk(root, all)
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
	wd.wait_time = 240.0
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

	# ---------------------------------------------------------------- 1. WW-13
	_p("=== WW-13 : paint Commit / Discard ===")
	if not _bridge._has("paint_draft_count"):
		_bad("paint_draft_count is not in the loaded extension -- stale .dll?")
	_app.select_domain("world")
	await _frames(4)
	_app.arm_tool("paint")
	await _frames(6)

	var ws = _app._world_workspace()

	## Find the dock pair and the tool-bar chip by their real text.
	var dock_commit := _buttons(ws, "Commit")
	var dock_discard := _buttons(ws._paint_body, "Discard draft")
	var bar_commit := _buttons(_app.tool_options_row, "Commit")
	_p("found: dock Commit x%d, dock Discard x%d, tool-bar Commit x%d" % [
		dock_commit.size(), dock_discard.size(), bar_commit.size()])
	for b in dock_commit:
		_p("   dockCommit candidate text=%s disabled=%s" % [b.text, str(b.disabled)])
	for b in dock_discard:
		_p("   dockDiscard candidate text=%s disabled=%s" % [b.text, str(b.disabled)])
	for b in bar_commit:
		_p("   barCommit candidate text=%s disabled=%s" % [b.text, str(b.disabled)])

	func_state("before any dab", dock_commit, dock_discard, bar_commit)
	if dock_commit.size() > 0 and not dock_commit[0].disabled:
		_bad("dock Commit is live with an empty draft")

	## Real dabs, through the same handler the map click uses.
	ws._paint_apply_dab(120.0, 90.0)
	ws._paint_apply_dab(122.0, 92.0)
	ws._paint_release(122.0, 92.0, true)
	await _frames(6)
	var pending_after_dabs: int = _bridge.paint_draft_count()
	var total_after_dabs: int = int(_bridge.paint_painted_counts().get("total", 0))
	_p("after 2 dabs: pending=%d  composite total=%d" % [pending_after_dabs, total_after_dabs])
	if pending_after_dabs <= 0:
		_bad("two dabs left nothing pending")

	dock_commit = _buttons(ws, "Commit")
	dock_discard = _buttons(ws._paint_body, "Discard draft")
	bar_commit = _buttons(_app.tool_options_row, "Commit")
	func_state("after dabs", dock_commit, dock_discard, bar_commit)
	if dock_commit.size() > 0 and dock_commit[0].disabled:
		_bad("dock Commit is dead with a real pending draft")

	## Commit for real, through the button.
	if dock_commit.size() > 0:
		dock_commit[0].pressed.emit()
	await _frames(8)
	var pending_after_commit: int = _bridge.paint_draft_count()
	var total_after_commit: int = int(_bridge.paint_painted_counts().get("total", 0))
	_p("after commit: pending=%d  composite total=%d" % [pending_after_commit, total_after_commit])
	if pending_after_commit != 0:
		_bad("commit left %d stamps pending" % pending_after_commit)
	if total_after_commit != total_after_dabs:
		_bad("the composite total changed across a commit (%d -> %d) -- the premise is wrong" % [
			total_after_dabs, total_after_commit])

	dock_commit = _buttons(ws, "Commit")
	dock_discard = _buttons(ws._paint_body, "Discard draft")
	bar_commit = _buttons(_app.tool_options_row, "Commit")
	func_state("after commit", dock_commit, dock_discard, bar_commit)
	if dock_commit.size() > 0 and not dock_commit[0].disabled:
		_bad("dock Commit still live after a commit -- WW-13 not fixed")
	if dock_discard.size() > 0 and not dock_discard[0].disabled:
		_bad("dock 'Discard draft' still live after a commit -- WW-13 not fixed")
	if bar_commit.size() > 0 and not bar_commit[0].disabled:
		_bad("tool-bar Commit still live after a dock commit -- cross-refresh missing")

	## And the reverse direction: dab again, commit from the TOOL BAR, assert
	## the dock's pair went dead too.
	ws._paint_apply_dab(150.0, 100.0)
	ws._paint_release(150.0, 100.0, true)
	await _frames(6)
	bar_commit = _buttons(_app.tool_options_row, "Commit")
	if bar_commit.size() > 0:
		bar_commit[0].pressed.emit()
	await _frames(8)
	dock_commit = _buttons(ws, "Commit")
	dock_discard = _buttons(ws._paint_body, "Discard draft")
	func_state("after TOOL-BAR commit", dock_commit, dock_discard, _buttons(_app.tool_options_row, "Commit"))
	if _bridge.paint_draft_count() != 0:
		_bad("tool-bar commit left a draft pending")
	if dock_commit.size() > 0 and not dock_commit[0].disabled:
		_bad("dock Commit still live after a TOOL-BAR commit -- cross-refresh missing")

	_app.arm_tool("inspect")
	await _frames(4)

	# ---------------------------------------------------------------- 2. IN-01
	_p("=== IN-01 : the river-entity disclosure under WORLD ▸ Hydrology ===")
	var world_texts := _texts(ws)
	var found := false
	for t in world_texts:
		if String(t).find("No hydrological river entity is exposed to Godot") >= 0:
			found = true
	if found:
		_p("PASS  the IN-01 note is drawn in the WORLD dock")
	else:
		_bad("the IN-01 river note is nowhere in the WORLD dock")

	# ------------------------------------------------------- 3. pointer dump
	_p("=== every rendered A ▸ B pointer ===")
	var seen := {}
	var re := RegEx.new()
	re.compile("[A-Za-z][A-Za-z&/ ']{0,26} ▸ [A-Za-z][A-Za-z&/ '\\u2026]{0,26}( ▸ [A-Za-z][A-Za-z&/ '\\u2026]{0,26})?")
	var everything: Array = _texts(_app)
	for t in everything:
		for m in re.search_all(String(t)):
			var s: String = m.get_string().strip_edges()
			seen[s] = int(seen.get(s, 0)) + 1
	var keys := seen.keys()
	keys.sort()
	for k in keys:
		_p("   PTR  %s" % k)

	_p("DONE fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)


func func_state(when: String, dc: Array, dd: Array, bc: Array) -> void:
	_p("  %-24s dockCommit=%s dockDiscard=%s barCommit=%s" % [
		when,
		"disabled" if (dc.size() > 0 and dc[0].disabled) else ("enabled" if dc.size() > 0 else "-"),
		"disabled" if (dd.size() > 0 and dd[0].disabled) else ("enabled" if dd.size() > 0 else "-"),
		"disabled" if (bc.size() > 0 and bc[0].disabled) else ("enabled" if bc.size() > 0 else "-")])
