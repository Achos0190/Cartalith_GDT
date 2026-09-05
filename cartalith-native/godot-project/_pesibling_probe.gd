extends Node
## Does the place editor and the right dock's Settlement context ever disagree
## about the same settlement?
##
## The brief for the 2026-09-05 no-design batch named this hazard by name: the
## two surfaces draw the same entity and "must not disagree". Reasoning from the
## signal graph is not enough -- `place_editor_window.gd` emits `place_changed`,
## `civilization_workspace.gd` connects it to `_on_civ_edited`, and the question
## is what is *on screen* in the dock afterwards, which only a booted app can
## answer.
##
## Both directions are exercised, because they fail differently:
##
##   A. edit in the place editor -> read the dock back
##   B. edit through the same engine call the dock's own path uses, then
##      re-select -> confirms the dock CAN show a new name, so a failure in A
##      is a refresh gap and not a dock that cannot read
##
## The name is the probe's instrument because it is the one field both surfaces
## draw verbatim: `right_dock.gd::_build_settlement` -> `_field(sec, "Name", …)`
## and `place_editor_window.gd::_build_identity` -> the `LineEdit`'s text.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _pesibling_probe.tscn
##
## No flags. `--headless` is safe here: every assertion reads `Label.text` off
## the live scene tree, never a rendered pixel (MISTAKES: `ImageTexture.update()`
## is a no-op under `--headless`, which is why no capture is taken).
##
## ## THIS PROBE IS RED ON PURPOSE, AND WAS RED WHEN IT WAS WRITTEN
##
## It exits **1** at the commit that added it. That is the finding, not a
## regression introduced with it. Measured 2026-09-05:
##
##     PES ok    A0: the dock draws the settlement's current name -- 'Sevjuniana'
##     PES ok    A1 (control): the engine took the edit -- engine='PROBE_RENAME_A'
##     PES FAIL  A2: the dock no longer shows the OLD name -- 'Sevjuniana' still drawn
##     PES FAIL  A3: the dock shows the NEW name -- 'PROBE_RENAME_A' absent
##     PES ok    B1: after a re-select the dock shows the new name
##     PES ok    C1 (control): the engine deleted it -- 40 -> 39
##     PES FAIL  C2: the dock does not still draw the deleted settlement
##     PES 3 failure(s)
##
## **What would turn it green, and it is not in the three windows.** The place
## editor emits `place_changed` / `place_deleted` correctly.
## `civilization_workspace.gd` owns both connections — its own comment explains
## why it rather than either window does — and its `_on_civ_edited()` refreshes
## the map overlay, the faction-religion column and its own dock categories,
## and never re-pushes `right_dock_ctrl.on_settlement_selected(...)`. B1 passing
## is what proves the dock can render the change once it is handed one, so the
## gap is in the refresh path and not in `right_dock.gd`'s reader.

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("PES %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _collect(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append((c as Label).text)
		_collect(c, out)

## Every `Label` the right dock is currently drawing.
func _dock_texts() -> Array:
	var out: Array = []
	_collect(app.right_dock_body, out)
	return out


func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 1800:
		await get_tree().process_frame
		waited += 1
	print("PES world generated: has_world=%s (%d frames)" % [app.bridge.has_world, waited])
	await _frames(8)
	if not app.bridge.has_world:
		print("PES  !! generate failed -- nothing here can run")
		get_tree().quit(1)
		return

	var places: Array = app.bridge.settlements()
	if places.is_empty():
		print("PES  !! no settlement -- the whole probe is about one")
		get_tree().quit(1)
		return
	var idx := 0
	var before := String((places[idx] as Dictionary).get("name", ""))
	_check("the settlement has a name to watch", before != "", "name='%s'" % before)

	## The dock, showing this settlement, exactly as a map click leaves it.
	app.right_dock_ctrl.on_settlement_selected(places[idx], idx)
	await _frames(4)
	_check("A0: the dock draws the settlement's current name",
		_dock_texts().has(before), "looking for '%s'" % before)

	# -- A. edit in the place editor, read the dock back -----------------------
	var pe = app.place_editor_window
	pe.open_for(idx)
	await _frames(4)
	var typed := "PROBE_RENAME_A"
	pe._apply({"name": typed})
	await _frames(8)

	var engine_now := String((app.bridge.settlements()[idx] as Dictionary).get("name", ""))
	_check("A1 (control): the engine took the edit", engine_now == typed,
		"engine='%s'" % engine_now)
	var texts := _dock_texts()
	_check("A2: the dock no longer shows the OLD name", not texts.has(before),
		"old='%s' still drawn" % before)
	_check("A3: the dock shows the NEW name", texts.has(typed),
		"new='%s' absent; dock still reads '%s'" % [typed, before])

	# -- B. can the dock show a new name at all? -------------------------------
	## Re-selecting pushes a fresh snapshot. If B passes while A fails, A is a
	## refresh gap in the edit path, not a dock that cannot read the field.
	app.right_dock_ctrl.on_settlement_selected(app.bridge.settlements()[idx], idx)
	await _frames(4)
	_check("B1: after a re-select the dock shows the new name",
		_dock_texts().has(typed), "dock=%s" % [_dock_texts().slice(0, 12)])

	# -- C. the same question for a delete --------------------------------------
	## `place_deleted` runs the same listener. A dock left holding a snapshot of
	## a settlement that no longer exists is the same defect with a worse ending,
	## because every action row in it is keyed to `_settlement_index`.
	var n_before: int = app.bridge.settlements().size()
	if n_before > 1:
		var doomed := String((app.bridge.settlements()[1] as Dictionary).get("name", ""))
		app.right_dock_ctrl.on_settlement_selected(app.bridge.settlements()[1], 1)
		await _frames(4)
		var drew_doomed := _dock_texts().has(doomed)
		app.bridge.civ_delete_settlement(1)
		pe.place_deleted.emit()
		await _frames(8)
		_check("C1 (control): the engine deleted it",
			app.bridge.settlements().size() == n_before - 1,
			"%d -> %d" % [n_before, app.bridge.settlements().size()])
		_check("C2: the dock does not still draw the deleted settlement",
			not (drew_doomed and _dock_texts().has(doomed)),
			"'%s' still on screen after delete" % doomed)

	print("PES %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
