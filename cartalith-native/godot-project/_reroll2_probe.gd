extends Node
## Committed probe. Isolating why the Place editor's ⟳ (re-roll
## name) is a no-op on its FIRST press and works afterwards.
##
## Hypothesis: `open_for()` focuses the name field (§4.5.3). `⟳` calls
## `_rebuild()`, whose first act is `_clear()`, which frees the focused
## LineEdit -- firing `focus_exited`, whose handler writes the field's
## (pre-roll) text back through `civ_edit_settlement`. The roll is overwritten
## before the rebuilt form reads it.
##
## Three runs, engine-side names read on every step, so the answer is a
## measurement and not a story:
##   A. open (focused) -> press ⟳
##   B. open, drop focus explicitly -> press ⟳
##   C. call the engine directly -> read back
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _reroll2_probe.tscn
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

var _app: Node
var _bridge
var _fail := 0


func _p(s: String) -> void:
	print("RR2  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


func _roll_button(pe: Node) -> Button:
	var all: Array = []
	_walk(pe, all)
	for n in all:
		if n is Button and (n as Button).text == "⟳":
			return n
	return null


func _name_of(i: int) -> String:
	var all: Array = _bridge.settlements()
	return String(all[i].get("name", "?"))


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

	var idx := 6
	var pe = _app.place_editor_window

	# -- A: open with focus (the real desktop path), then one press ----------
	pe.open_for(idx)
	await _frames(8)
	_p("A  after open: engine='%s'  field='%s'  focusOwner=%s" % [
		_name_of(idx), pe._name_edit.text,
		str(pe._name_edit.has_focus())])
	var before_a := _name_of(idx)
	var rb := _roll_button(pe)
	rb.pressed.emit()
	await _frames(8)
	var after_a := _name_of(idx)
	_p("A  after one ⟳: engine '%s' -> '%s'  field='%s'" % [before_a, after_a, pe._name_edit.text])
	if after_a == before_a:
		_p("A  CONFIRMED: the first ⟳ press left the engine name unchanged")
		_fail += 1
	else:
		_p("A  the first ⟳ press changed the engine name")
	pe.hide()
	await _frames(4)

	# -- B: same, but with focus released first ------------------------------
	pe.open_for(idx)
	await _frames(8)
	pe._name_edit.release_focus()
	await _frames(4)
	var before_b := _name_of(idx)
	rb = _roll_button(pe)
	rb.pressed.emit()
	await _frames(8)
	var after_b := _name_of(idx)
	_p("B  focus released, one ⟳: engine '%s' -> '%s'  field='%s'" % [before_b, after_b, pe._name_edit.text])
	pe.hide()
	await _frames(4)

	# -- C: engine only ------------------------------------------------------
	var before_c := _name_of(idx)
	var ret: String = _bridge.civ_reroll_settlement_name(idx)
	_p("C  engine only: '%s' -> returned '%s', read back '%s'" % [before_c, ret, _name_of(idx)])

	## Gated. The defect this probe was written for (2026-08-25, PE-01: the
	## focused name field writing its pre-roll text back over the roll) is
	## fixed -- run A now changes the engine name on the first press -- so
	## `_fail` is a regression guard, and `quit(0)` threw it away.
	_p("DONE  A-was-a-noop=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
