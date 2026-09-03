extends Node
## **DS-03's reflow guard.** The owner's ruling is "keep everything, reflow
## only", and the property that makes that true is narrower than it sounds:
## *no dock panel may force its dock wider than the dock is.* Both docks scroll
## through `DccShell._scroll()`, which sets
## `horizontal_scroll_mode = SCROLL_MODE_DISABLED` -- and a disabled axis folds
## the child's minimum size into the container's own on that axis, so a single
## over-wide leaf propagates all the way out and the dock *grows*, eating the
## map. There is no horizontal scrollbar to reveal it and nothing was asserting
## it; `MISTAKES.md` records three earlier instances of the same trap.
##
##   Godot_v4.7.1 --headless --path . --resolution 1600x900 _ds03fit_probe.tscn -- --force-touch
##   Godot_v4.7.1 --headless --path . --resolution 1600x900 _ds03fit_probe.tscn
##
## The second invocation is the desktop leg. `DccTheme._touch` is latched for
## the life of the process, so the two densities cannot share a run.
##
## Measured at HEAD before this pass, tablet 2560x1600, left dock nominally
## 400 px wide:
##
##   CARTO > Labels          dock grew to 1 589 px   (one un-wrapped Label)
##   CIVIL > Factions        dock grew to   555 px   (one un-wrapped Button)
##   CIVIL > Landmarks       dock grew to   417 px   (a five-chip HBoxContainer)
##
## and eight `action()` buttons carried minimum widths of 434-753 px inside
## collapsed sections, waiting for the first reader to open one.

var _fail := 0

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _le(name: String, got: float, cap: float) -> void:
	var good := got <= cap + 0.5
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=%.0f cap=%.0f" % [got, cap])

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

var _vp: SubViewport

func _boot(w: int, h: int) -> Node:
	_vp = SubViewport.new()
	_vp.size = Vector2i(w, h)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(30)
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(30)
	return app

## Visible content, per dock. Present so the invariant above cannot be met the
## one way the owner's ruling forbids -- by deleting rows until the dock fits.
func _content(root: Node) -> int:
	if root == null:
		return 0
	var n := 0
	var stack: Array = [root]
	while not stack.is_empty():
		var x: Node = stack.pop_back()
		for c in x.get_children(true):
			stack.append(c)
		if x is Control and (x as Control).is_visible_in_tree():
			if x is Label and (x as Label).text.strip_edges() != "":
				n += 1
			elif x is BaseButton:
				n += 1
	return n

## Leaves whose own minimum width exceeds the dock, **hidden ones included**.
## A collapsed `group()` body contributes nothing to its parent's minimum while
## it is closed, so a walk that skipped it would pass today and fail the first
## time a reader clicks a caret. That is the latent set, and it must be empty.
func _latent(root: Node, cap: float) -> Array:
	var out: Array = []
	if root == null:
		return out
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
		if n is Control and n.get_child_count() == 0:
			var mn := (n as Control).get_combined_minimum_size().x
			if mn > cap:
				var t: Variant = n.get("text")
				out.append("%s min_x=%.0f > %.0f  %s" % [n.get_class(), mn, cap,
					(String(t).left(60) if t != null else "")])
	return out

func _sweep(app: Node, tag: String) -> void:
	var shell: Node = app
	var ld := shell.get("left_dock") as Control
	var rd := shell.get("right_dock") as Control
	var lw := float(shell.get("_left_width"))
	var rw := float(shell.get("_right_width"))
	## The band above the docks. This one is here because the first version of
	## this pass's own fix broke it: an unguarded `autowrap_mode` on
	## `DccWidgets.action()` collapsed the minimum width of the five buttons
	## `app.gd::_tool_options_generate()` builds straight into the bar's own
	## `HBoxContainer`, every label wrapped, and the 40 px band became 265 px.
	## A dock-only assertion would not have seen it.
	var band_h := float(DccTheme.role_px("h_tool_options"))
	for n in DccShell.RAIL_NODES:
		if String(n.get("kind", "")) != "node":
			continue
		var id: String = String(n["domain"]) + "/" + String(n["mode"])
		shell.call("_on_rail_node_pressed", String(n["domain"]), String(n["mode"]))
		await _frames(12)
		_le("%s %s: left dock is not forced open" % [tag, id], ld.size.x, lw)
		_le("%s %s: right dock is not forced open" % [tag, id], rd.size.x, rw)
		var tob := shell.get("tool_options_row") as Control
		if tob != null and tob.get_parent() != null:
			var band := tob.get_parent().get_parent() as Control
			if band != null:
				_le("%s %s: the tool-options band keeps its height" % [tag, id],
					band.size.y, band_h)
		var lat: Array = _latent(shell.get("left_dock_body"), lw) \
			+ _latent(shell.get("right_dock_body"), rw)
		_ok("%s %s: no latent over-wide leaf, collapsed sections included" % [tag, id],
			lat.size(), 0)
		for s in lat:
			print("        ", s)
		## The ruling's own half. A dock that fits because it was emptied is
		## the failure this exists to forbid, so the floor is set below the
		## measured counts rather than at them -- it catches a *deletion*, not
		## a row added or removed by an unrelated change.
		var lc := _content(shell.get("left_dock_body"))
		var good := lc >= 30
		if not good:
			_fail += 1
		print("  ", "ok  " if good else "FAIL",
			" %s %s: the panel still carries its inventory   got=%d want>=30" % [tag, id, lc])

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(1)
		return
	var forced := "--force-touch" in OS.get_cmdline_user_args()
	print("[BOOT] force-touch=", forced)

	var app := await _boot(2560, 1600)
	print("[MODE] touch=", DccTheme.is_touch(), " tablet=", DccTheme.is_tablet(),
		" phone=", DccTheme.is_phone(), " docks=",
		app.get("_left_width"), "/", app.get("_right_width"))
	_ok("classification matches the invocation", DccTheme.is_tablet(), forced)

	print("")
	print("-- the factory rule the fix rests on --")
	## Asserted directly, so the guard survives a "simplification" that drops
	## the parent test: `action()` must wrap in a column and must NOT wrap in a
	## row. Both directions, because each failure is a different bug -- no wrap
	## in a column is the over-wide dock, a wrap in a row is the 265 px band.
	var col := VBoxContainer.new()
	var row := HBoxContainer.new()
	add_child(col)
	add_child(row)
	var in_col := DccWidgets.action(col, "A sentence long enough to need wrapping", func(): pass)
	var in_row := DccWidgets.action(row, "A sentence long enough to need wrapping", func(): pass)
	_ok("action() in a column wraps", in_col.autowrap_mode, TextServer.AUTOWRAP_WORD_SMART)
	_ok("action() in a row does not", in_row.autowrap_mode, TextServer.AUTOWRAP_OFF)
	_ok("and wrapping is what lowers the minimum width",
		in_col.get_combined_minimum_size().x < in_row.get_combined_minimum_size().x, true)

	print("")
	print("-- the one row whose text has no length bound --")
	## `_settlement_row()` labels its button with a **generated** place name,
	## and it is the single call site that has to overrule `action()`'s row
	## rule (it is the row's `SIZE_EXPAND_FILL` member, so wrapping is safe
	## there and nothing else is).
	##
	## Asserted through a name this probe supplies rather than through whatever
	## the generator happened to produce, because the sweep below is **not** a
	## guard for it: with the fix removed, the world sweep passed on the seed
	## it drew and failed on the next -- a mutant that survives on one run and
	## is killed on another is not a guard at all. A fixed 60-character name
	## makes the assertion deterministic at both densities.
	var civ: Node = null
	for w in app.get("_workspaces"):
		if String(w.name) == "CivilizationWorkspace":
			civ = w
	if civ == null:
		_fail += 1
		print("  FAIL  CivilizationWorkspace not found; the row assertion could not run")
	else:
		var host := VBoxContainer.new()
		add_child(host)
		civ.call("_settlement_row", host,
			{"name": "Thorndunthornbaldstonewithanotherlongsuffixappended",
			"kind": "capital", "population": 22647}, 0)
		await _frames(4)
		var built := host.get_child(0) as Control
		_le("a generated place name cannot force the dock open",
			built.get_combined_minimum_size().x, float(app.get("_left_width")))

	print("")
	print("-- every (domain, mode), no world --")
	await _sweep(app, "boot")

	print("")
	print("-- the same sweep over a generated world --")
	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 3600:
		await get_tree().process_frame
		waited += 1
	print("  world: has_world=", app.bridge.has_world, " (", waited, " frames)")
	if not app.bridge.has_world:
		_fail += 1
		print("  FAIL  the world leg could not run")
	else:
		await _frames(20)
		await _sweep(app, "world")

	print("")
	print("_ds03fit_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
