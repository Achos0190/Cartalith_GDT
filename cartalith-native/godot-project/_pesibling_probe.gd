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
## Five cases, because they fail differently:
##
##   A. edit in the place editor -> read the dock back
##   B. edit through the same engine call the dock's own path uses, then
##      re-select -> confirms the dock CAN show a new name, so a failure in A
##      is a refresh gap and not a dock that cannot read
##   C. delete the settlement the dock is drawing -> the panel must not go on
##      drawing it, and must not be left pointed at a renumbered index
##   D. regenerate the world under an open panel -> the same question with the
##      identity test removed, since `tid`s are re-issued from 1 per world
##   E. close the world under an open panel -> D's clause on the other
##      world-replacement signal, `world_loaded`
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
## ## THIS PROBE WAS A DEMONSTRATION AND IS NOW A GUARD
##
## It was written **red on purpose** and exited 1 at the commit that added it:
## the failures were the finding, not a regression shipped with it. That is no
## longer its job. The defect it demonstrated was fixed on 2026-09-05 and the
## same assertions now hold, so **a non-zero exit from this file is a
## regression** -- read it that way, not as the standing state it used to be.
## Nothing was inverted to achieve that: every `_check` below is worded as the
## behaviour that should hold and always was, which is why the file needed no
## assertion rewritten when the code was fixed.
##
## Before, measured 2026-09-05 (shell build a2ba0f18b78d, 40 settlements):
##
##     PES ok    A1 (control): the engine took the edit -- engine='PROBE_RENAME_A'
##     PES FAIL  A2: the dock no longer shows the OLD name -- 'Sevjuniana' still drawn
##     PES FAIL  A3: the dock shows the NEW name -- 'PROBE_RENAME_A' absent
##     PES ok    B1: after a re-select the dock shows the new name
##     PES FAIL  C2: the dock does not still draw the deleted settlement
##     PES 3 failure(s)
##
## **The fix, and why it is where it is.** `place_editor_window.gd` emitted
## `place_changed` / `place_deleted` correctly all along and B1 passing proved
## `right_dock.gd`'s reader was never the problem -- the gap was that nothing
## re-pushed the snapshot. `civilization_workspace.gd::_on_civ_edited()` owns
## both connections and now ends with `app.right_dock_ctrl.refresh_settlement()`,
## which re-pushes the live entry when `_live_settlement()` still resolves and
## closes the panel when it does not. C2 is the second half: closing is what
## makes the delete safe, because `explain_settlement` and `urban_layouts` are
## both keyed to the `_settlement_index` a delete renumbers, and `lib.rs`'s
## `civ_delete_settlement` states that `explanations` is not re-indexed.
##
## D and E cover the path `refresh_settlement()` deliberately does NOT serve: a
## world replacement re-issues every `tid` from 1, so the identity test would
## match a stranger, and `right_dock.gd::setup()` drops the context outright on
## both `generation_finished` and `world_loaded` instead. Neither clause existed
## before this change, next to a `CTX_RIVER` reset that had been there all along
## for the same reason.
##
## **The settlement pin was not the only context those handlers left standing,
## and it is the only one this change fixes.** `CTX_ROUTE`, `CTX_FACTION`,
## `CTX_MEASURE`, `CTX_REGION`, `CTX_WILDLIFE` and `CTX_HISTORY` each still
## survive a world replacement holding an index or an entry into a world that is
## gone. Same defect, unmeasured, and deliberately out of this change's scope --
## stated here so the next reader does not take D and E passing as a claim about
## the other six.
##
## After, same day, exit **0** with all 15 checks ok (shell build
## 91bcbcdbdee3, 40 settlements) -- the three that were red are A2, A3 and C2,
## and D and E are new:
##
##     PES ok    A2: the dock no longer shows the OLD name -- old='Sevjuniana'
##     PES ok    A3: the dock shows the NEW name -- new='PROBE_RENAME_A'
##     PES ok    C2: the dock does not still draw the deleted settlement
##     PES ok    C3: the panel closed rather than re-pointing at a renumbered
##                   index -- dock=["§ SAMPLE", "Position", "—", "Cell"]
##     PES ok    D2: the dock does not still draw the previous world's town
##     PES ok    E2: the dock does not still draw a town from the closed world
##     PES 0 failure(s)
##
## **Each of the three fixed assertions was falsified against its own fix**, so
## none of them is a check that cannot fail: `refresh_settlement()` defined but
## uncalled reproduces the A2/A3/C2 block above; `if false and` on the
## `generation_finished` clause makes D2 the only failure; the same on the
## `world_loaded` clause makes E2 the only failure. All three restored and the
## file re-hashed afterwards.

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
		"old='%s'" % before)
	_check("A3: the dock shows the NEW name", texts.has(typed),
		"new='%s', old='%s'" % [typed, before])

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
			"doomed='%s'" % doomed)
		## What C2 must NOT be satisfied by: a panel re-pointed at whatever town
		## now holds the deleted one's index. `explain_settlement` and
		## `urban_layouts` are both keyed to that index and the engine does not
		## re-index `explanations` on a delete, so a redraw there would trade a
		## stale name for a neighbour's causal chain. The panel closes instead,
		## and the section header is how that reads on screen.
		_check("C3: the panel closed rather than re-pointing at a renumbered index",
			_dock_texts().has("§ SAMPLE"),
			"dock=%s" % [_dock_texts().slice(0, 4)])

	# -- D. a world replacement under an open panel -----------------------------
	## The one path `refresh_settlement()` must NOT be used on, and the reason
	## `right_dock.gd::setup()` drops the context outright there instead:
	## `compute_civilisation` starts the counter at 1 on the auto-populate path,
	## so settlement 0 of the second world carries the first world's tid and a
	## tid match would "refresh" the pin into a stranger.
	##
	## The pin is renamed to a sentinel first, through the bridge and without
	## emitting anything, so D2 cannot be passed or failed by two worlds happening
	## to generate the same name.
	var survivors: Array = app.bridge.settlements()
	if not survivors.is_empty():
		var pinned := "PROBE_PIN_D"
		app.bridge.civ_edit_settlement(0, {"name": pinned})
		app.right_dock_ctrl.on_settlement_selected(app.bridge.settlements()[0], 0)
		await _frames(4)
		var drew_pinned := _dock_texts().has(pinned)
		_check("D0 (control): the dock draws the pin before the regenerate",
			drew_pinned, "pinned='%s'" % pinned)
		app._run_pipeline()
		var w2 := 0
		while app.bridge.generating and w2 < 1800:
			await get_tree().process_frame
			w2 += 1
		await _frames(8)
		_check("D1 (control): a second world generated", app.bridge.has_world,
			"%d settlements in %d frames" % [app.bridge.settlements().size(), w2])
		_check("D2: the dock does not still draw the previous world's town",
			not (drew_pinned and _dock_texts().has(pinned)), "pinned='%s'" % pinned)

	# -- E. the same clause on the other world-replacement signal ---------------
	## `world_loaded` carries `close_world()` and a project open as well as the
	## in-place ops, and `right_dock.gd::setup()` mirrors D's clause onto it.
	## `close_world()` is the reachable half from here -- a project open needs a
	## file on disk -- and it is the harsher one: the pin's world is gone
	## entirely, so a panel still drawing it is reading a snapshot of nothing.
	var post: Array = app.bridge.settlements()
	if not post.is_empty():
		var pin_e := "PROBE_PIN_E"
		app.bridge.civ_edit_settlement(0, {"name": pin_e})
		app.right_dock_ctrl.on_settlement_selected(app.bridge.settlements()[0], 0)
		await _frames(4)
		var drew_e := _dock_texts().has(pin_e)
		_check("E0 (control): the dock draws the pin before the close", drew_e,
			"pinned='%s'" % pin_e)
		app.bridge.close_world()
		await _frames(8)
		_check("E1 (control): the world is gone", not app.bridge.has_world,
			"has_world=%s" % app.bridge.has_world)
		_check("E2: the dock does not still draw a town from the closed world",
			not (drew_e and _dock_texts().has(pin_e)), "pinned='%s'" % pin_e)

	print("PES %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
