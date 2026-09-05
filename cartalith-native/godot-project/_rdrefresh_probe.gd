extends Node
## Right-dock refresh gaps -- the two rows in `OUTSTANDING_WORK.md` that share
## one shape: a state change no signal reaches, so the dock keeps drawing a body
## built for the old state.
##
##   Row 1  a sculpt draft created without a tool-arm never rebuilds the dock
##   Row 2  committing paint from the tool bar leaves the dock's paint context stale
##
## **Drives the real entry point and reads the dock back.** Arranging the state
## and then rebuilding by hand would prove nothing about either row: both are
## "the dock did not notice", so every assertion below is made with NO
## intervening rebuild of the probe's own.
##
## The rebuild COUNT is measured, not assumed. `RightDock._rebuild()` opens by
## removing every child of `right_dock_body`, so counting `child_exiting_tree`
## on that node and comparing it against the child count taken immediately
## before the action separates "one rebuild" from "two" without instrumenting
## the shell: one rebuild retires exactly the children that were there; a
## second also retires the ones the first just made.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _rdrefresh_probe.tscn

var app: Node
var _fail := 0
var _exits := 0
## What `sculpt_stamp_count()` answers AT THE MOMENT `sculpt_draft_changed`
## fires -- the signal's own contract, read by whoever listens on it.
var _count_at_emit := -99

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("RDR %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _collect(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append((c as Label).text)
		_collect(c, out)

func _texts() -> Array:
	var out: Array = []
	_collect(app.right_dock_body, out)
	return out

func _headers() -> Array:
	var out: Array = []
	for t in _texts():
		var s := String(t)
		if s.begins_with("§ "):
			out.append(s.substr(2))
	return out

func _hdr(prefix: String) -> int:
	var hs := _headers()
	var want := prefix.to_upper()
	for i in hs.size():
		if String(hs[i]).begins_with(want):
			return i
	return -1

## The first Button under the right dock whose text contains `needle`, or null.
func _find_btn(n: Node, needle: String) -> Button:
	for c in n.get_children():
		if c is Button and String((c as Button).text).findn(needle) >= 0:
			return c
		var deep := _find_btn(c, needle)
		if deep != null:
			return deep
	return null

func _btn(needle: String) -> Button:
	return _find_btn(app.right_dock_body, needle)

func _collect_btns(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Button:
			out.append("%s|disabled=%s" % [(c as Button).text, (c as Button).disabled])
		_collect_btns(c, out)

func _btn_states() -> Array:
	var out: Array = []
	_collect_btns(app.right_dock_body, out)
	return out

## Any label under the right dock containing `needle`.
func _says(needle: String) -> bool:
	for t in _texts():
		if String(t).findn(needle) >= 0:
			return true
	return false

func _on_exit(_n: Node) -> void:
	_exits += 1

## Zero the rebuild counter and return the number of children a single rebuild
## would retire, which is the expected count when exactly one runs.
func _arm_exit_counter() -> int:
	_exits = 0
	return app.right_dock_body.get_child_count()

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
	print("RDR world generated: has_world=%s (%d frames)" % [app.bridge.has_world, waited])
	await _frames(8)
	if not app.bridge.has_world:
		print("RDR  !! generate failed -- nothing else here can run")
		get_tree().quit(1)
		return

	var rd = app.right_dock_ctrl
	var bridge = app.bridge
	var ws = app._world_workspace()
	var gs: Vector2 = bridge.grid_size()
	app.right_dock_body.child_exiting_tree.connect(_on_exit)

	print("RDR domain=%s armed=%s stamps=%d headers=%s"
		% [app.active_domain(), app.armed_tool, bridge.sculpt_stamp_count(), _headers()])

	# == ROW 1 =================================================================
	# T1 -- a draft created with NO tool arm and NO manual rebuild.
	_check("T1 pre: Inspect is armed and the draft is empty",
		app.armed_tool == "inspect" and bridge.sculpt_stamp_count() == 0,
		"armed=%s stamps=%d" % [app.armed_tool, bridge.sculpt_stamp_count()])
	_check("T1 pre: the dock draws no Stamp stack", _hdr("STAMP STACK") < 0,
		"headers=%s" % [_headers()])
	if bridge.sculpt_get_globals().is_empty():
		print("RDR  !! no sculpt editor on this world -- Row 1 cannot be probed")
		get_tree().quit(1)
		return

	## T0 -- what the signal promises a listener. `world_workspace.gd`'s
	## `_refresh_sculpt_draft` re-reads `sculpt_stamp_count()` synchronously
	## inside this emit and re-gates its Commit/Discard on the answer, so an
	## emit that runs BEFORE the engine call hands it the pre-change count.
	bridge.sculpt_draft_changed.connect(func(): _count_at_emit = bridge.sculpt_stamp_count())

	var before := _arm_exit_counter()
	bridge.sculpt_begin_stroke()
	bridge.sculpt_add_point(gs.x * 0.40, gs.y * 0.50)
	bridge.sculpt_add_point(gs.x * 0.44, gs.y * 0.52)
	bridge.sculpt_end_stroke()
	await _frames(4)
	print("RDR [T1] stamps=%d headers=%s exits=%d (children before=%d)"
		% [bridge.sculpt_stamp_count(), _headers(), _exits, before])
	_check("T1: the stroke really landed", bridge.sculpt_stamp_count() == 1,
		"stamps=%d" % bridge.sculpt_stamp_count())
	_check("T1: the dock rebuilt and now draws the Stamp stack", _hdr("STAMP STACK") >= 0,
		"headers=%s" % [_headers()])
	## The Label the collapsed dock actually shows, not `_dock_readout_text()`
	## -- that one recomputes on call and would pass over a dock that never
	## rebuilt, which is the whole defect.
	var readout := String((app._dock_readouts["right"] as Label).text)
	_check("T1: the collapsed readout on screen agrees", readout.findn("stamp") >= 0,
		"readout=%s" % readout)
	_check("T1: exactly ONE rebuild", _exits == before,
		"exits=%d expected=%d" % [_exits, before])
	_check("T0: sculpt_draft_changed carries the POST-change count",
		_count_at_emit == 1, "count at emit=%d, live=%d" % [_count_at_emit, bridge.sculpt_stamp_count()])

	# T2 -- a path that ALREADY rebuilds must not gain a second one.
	before = _arm_exit_counter()
	rd._on_sculpt_stack_discard()
	await _frames(4)
	print("RDR [T2] stamps=%d exits=%d (children before=%d) headers=%s"
		% [bridge.sculpt_stamp_count(), _exits, before, _headers()])
	_check("T2: the discard emptied the draft", bridge.sculpt_stamp_count() == 0,
		"stamps=%d" % bridge.sculpt_stamp_count())
	_check("T2: still exactly ONE rebuild on a covered path", _exits == before,
		"exits=%d expected=%d" % [_exits, before])

	# T3 -- the covered creator: a real sculpt stroke through world_workspace.
	if ws != null and ws._sculpt_body != null:
		app.arm_tool("sculpt")
		await _frames(4)
		before = _arm_exit_counter()
		ws._sculpt_click(gs.x * 0.55, gs.y * 0.45)
		ws._sculpt_release(gs.x * 0.55, gs.y * 0.45, true)
		await _frames(4)
		print("RDR [T3] stamps=%d exits=%d (children before=%d)"
			% [bridge.sculpt_stamp_count(), _exits, before])
		_check("T3: the covered stroke landed", bridge.sculpt_stamp_count() == 1,
			"stamps=%d" % bridge.sculpt_stamp_count())
		_check("T3: still exactly ONE rebuild on the covered stroke path", _exits == before,
			"exits=%d expected=%d" % [_exits, before])
		rd._on_sculpt_stack_discard()
		await _frames(3)
		app.arm_tool("inspect")
		await _frames(3)
	else:
		print("RDR  -- T3 skipped: no _sculpt_body on the WORLD workspace")

	# == ROW 2 =================================================================
	app.arm_tool("paint")
	await _frames(6)
	print("RDR [R2] armed=%s headers=%s" % [app.armed_tool, _headers()])
	_check("R2 pre: the dock draws its paint context", _hdr("PAINT") >= 0,
		"headers=%s" % [_headers()])
	ws._paint_click(gs.x * 0.35, gs.y * 0.60)
	ws._paint_release(gs.x * 0.35, gs.y * 0.60, true)
	await _frames(4)
	var pending: int = bridge.paint_draft_count()
	print("RDR [R2] pending=%d dock says pending? %s" % [pending, _says("pending across every layer")])
	_check("R2 pre: a dab is pending", pending > 0, "paint_draft_count=%d" % pending)
	## `Discard draft`, not `Commit`: `DccWidgets.group()` builds its header as a
	## flat `Button`, and the pair sits inside a group titled `Commit` -- so a
	## text search for "Commit" finds the header first and reads its `disabled`,
	## which is never set. The first run of this probe passed both R2 button
	## checks that way, in both directions. The two actions are gated on the
	## same `pending == 0`, and only one of them has a unique label.
	print("RDR [R2] dock buttons: %s" % [_btn_states()])
	var cb := _btn("Discard draft")
	_check("R2 pre: the dock's Discard is live", cb != null and not cb.disabled,
		"btn=%s" % ("null" if cb == null else str(cb.disabled)))

	before = _arm_exit_counter()
	app.tool_bar._on_paint_commit()
	await _frames(4)
	var cb2 := _btn("Discard draft")
	print("RDR [R2] after bar commit: pending=%d nothing_pending=%s discard_disabled=%s exits=%d before=%d"
		% [bridge.paint_draft_count(), _says("Nothing pending"),
			("null" if cb2 == null else str(cb2.disabled)), _exits, before])
	_check("R2: the commit really emptied the draft", bridge.paint_draft_count() == 0,
		"paint_draft_count=%d" % bridge.paint_draft_count())
	_check("R2: the dock's paint context noticed", _says("Nothing pending"),
		"looking for 'Nothing pending' in the dock; headers=%s" % [_headers()])
	_check("R2: the dock's Discard is greyed over the emptied draft",
		cb2 != null and cb2.disabled, "btn=%s" % ("null" if cb2 == null else str(cb2.disabled)))
	_check("R2: exactly ONE dock rebuild for the bar commit", _exits == before,
		"exits=%d expected=%d" % [_exits, before])

	print("RDR done -- %d failure%s" % [_fail, "" if _fail == 1 else "s"])
	get_tree().quit(1 if _fail > 0 else 0)
