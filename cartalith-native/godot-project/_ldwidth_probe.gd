extends Node
## Diagnostic for `_ds03fit_probe`'s one red pair: `civilization/planner`
## measures the left dock at **404** against a `--ldW` of 372. Prints the
## additive chain from the widest leaf up to `left_dock`, then runs three
## in-memory experiments that separate the *cause* from the number, then sweeps
## every rail node for its headroom against the same budget.
##
##   Godot_v4.7.1 --headless --path . _ldwidth_probe.tscn -- --vp 1920x1080
##   Godot_v4.7.1 --headless --path . _ldwidth_probe.tscn -- --vp 1366x768
##
## Reads `-- --vp WxH` and nothing else; an unrecognised argument is fatal
## rather than silently defaulting, so the header cannot drift from the body.
## The window's own `--resolution` is irrelevant here: the shell is built into
## a `SubViewport` of the size given above, exactly as `_ds03fit_probe` does.

var _vp: SubViewport

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## The dock's own furniture, subtracted once: `_build_left_dock()` puts the
## body and a 6 px `_dock_drag_handle()` in one `HBoxContainer` inside a
## `PanelContainer` styled `panel(…, {"right": 1})`. Both are carved OUT of the
## dock's reserved width, so the body's budget is the dock minus the two.
const FURNITURE := 7

func _chain(n: Control, depth: int, budget: float) -> void:
	var comb := n.get_combined_minimum_size().x
	if comb <= budget:
		return
	var t: Variant = n.get("text")
	print("      %s%s  comb=%.0f own=%.0f  %s" % ["  ".repeat(depth), n.get_class(),
		comb, n.custom_minimum_size.x, (String(t).left(44) if t != null else "")])
	for c in n.get_children(true):
		if c is Control and (c as Control).is_visible_in_tree():
			_chain(c as Control, depth + 1, budget)

## Widest visible leaf under `root`, and the OptionButton among them.
func _find(root: Node, cls: String, text: String) -> Control:
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
		if n is Control and n.get_class() == cls:
			if text == "" or String((n as Control).get("text")) == text:
				return n as Control
	return null

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(1)
		return
	var w := 1920
	var h := 1080
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--vp" and i + 1 < args.size():
			var parts := String(args[i + 1]).split("x")
			w = int(parts[0])
			h = int(parts[1])
			i += 2
		else:
			print("[FATAL] unrecognised argument: ", args[i])
			get_tree().quit(2)
			return

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

	var lw := float(app.get("_left_width"))
	var budget := lw - FURNITURE
	print("[VP] ", w, "x", h, "  tablet=", DccTheme.is_tablet(),
		" laptop=", DccTheme.is_laptop(),
		"  _left_width=", lw, "  body budget=", budget)

	var ld := app.get("left_dock") as Control
	var body := app.get("left_dock_body") as Control

	print("")
	print("-- headroom, every rail node --")
	for n in DccShell.RAIL_NODES:
		if String(n.get("kind", "")) != "node":
			continue
		app.call("_on_rail_node_pressed", String(n["domain"]), String(n["mode"]))
		await _frames(12)
		print("  %-28s dock.size=%4.0f dock.min=%4.0f body.min=%4.0f  headroom=%+.0f" % [
			String(n["domain"]) + "/" + String(n["mode"]), ld.size.x,
			ld.get_combined_minimum_size().x, body.get_combined_minimum_size().x,
			budget - body.get_combined_minimum_size().x])

	print("")
	print("-- the planner chain, everything above the body budget --")
	app.call("_on_rail_node_pressed", "civilization", "planner")
	await _frames(16)
	_chain(ld, 0, budget)

	## Experiment 1: the nested `ScrollContainer`. `_build_left_panel()` puts
	## one inside `app.left_dock_body`, which is already inside the dock's own
	## `_scroll()`. Measured by asking the scroll what it costs above its child.
	print("")
	print("-- experiment 1: what the nested ScrollContainer adds --")
	var inner: ScrollContainer = null
	var stack: Array = [body]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
		## Only a scroller that actually holds content: the walk also reaches
		## the `ScrollContainer`s Godot builds inside other widgets, and one of
		## those has no children at all.
		if n is ScrollContainer and (n as ScrollContainer).get_child_count() > 0 \
				and (n as ScrollContainer).get_child(0) is Control:
			inner = n
	if inner == null:
		print("  (no nested ScrollContainer found)")
	else:
		var child := inner.get_child(0) as Control
		print("  scroll.min=%.0f  child.min=%.0f  tax=%.0f" % [
			inner.get_combined_minimum_size().x, child.get_combined_minimum_size().x,
			inner.get_combined_minimum_size().x - child.get_combined_minimum_size().x])
		print("  v_scroll.min=%.0f visible=%s   panel_sb=%s" % [
			inner.get_v_scroll_bar().get_combined_minimum_size().x,
			str(inner.get_v_scroll_bar().visible),
			str(inner.has_theme_stylebox_override("panel"))])

	## Experiment 2: the widest row. `DccWidgets.choice()` leaves
	## `OptionButton.fit_to_longest_item` at its Godot default of `true`, so a
	## dropdown's minimum is its longest ITEM, not the text it is showing.
	print("")
	print("-- experiment 2: OptionButton.fit_to_longest_item --")
	var ob := _find(body, "OptionButton", "Auto")
	if ob == null:
		print("  (no 'Auto' OptionButton found)")
	else:
		var opt := ob as OptionButton
		print("  fit_to_longest_item=", opt.fit_to_longest_item, "  items=", opt.item_count)
		for k in opt.item_count:
			print("      [%d] %s" % [k, opt.get_item_text(k)])
		var before := opt.get_combined_minimum_size().x
		var row_before := (opt.get_parent() as Control).get_combined_minimum_size().x
		var body_before := body.get_combined_minimum_size().x
		opt.fit_to_longest_item = false
		await _frames(6)
		print("  ob.min  %.0f -> %.0f" % [before, opt.get_combined_minimum_size().x])
		print("  row.min %.0f -> %.0f" % [row_before,
			(opt.get_parent() as Control).get_combined_minimum_size().x])
		print("  body.min %.0f -> %.0f   (budget %.0f)" % [body_before,
			body.get_combined_minimum_size().x, budget])
		print("  dock.size %.0f" % ld.size.x)
		opt.fit_to_longest_item = true
		await _frames(6)

	## Experiment 3: both together — every OptionButton in the panel, plus the
	## nested scroll's stylebox. States what a complete content-side fix would
	## leave the dock at, without editing anything on disk.
	print("")
	## Restricted to the ones that EXPAND, which is the guard `phone_fit()`
	## already applies to `Button` and `Label` for the same reason: a control
	## sized by its own text and not expanding has nothing else to take a width
	## from, so removing its content minimum collapses it.
	print("-- experiment 3: every EXPANDING OptionButton in the panel, fit off --")
	var obs: Array = []
	var st2: Array = [body]
	while not st2.is_empty():
		var n: Node = st2.pop_back()
		for c in n.get_children(true):
			st2.append(c)
		if n is OptionButton \
				and ((n as Control).size_flags_horizontal & Control.SIZE_EXPAND) != 0:
			obs.append(n)
	for o in obs:
		(o as OptionButton).fit_to_longest_item = false
	await _frames(10)
	print("  option buttons=%d   body.min=%.0f  dock.size=%.0f  (budget %.0f)" % [
		obs.size(), body.get_combined_minimum_size().x, ld.size.x, budget])
	_chain(ld, 0, budget)


	## Experiment 4: the fix's own risk, measured rather than argued. Clearing
	## `fit_to_longest_item` makes a dropdown's minimum its SELECTION, so the
	## question it raises is whether any dock dropdown now draws narrower than
	## the text it has to show. Reported per control: drawn width, the width
	## the longest item would have asked for, and whether the control still
	## expands (the guard `dock_width_fit()` relies on).
	print("")
	print("-- experiment 4: no dock dropdown collapsed or clipped --")
	var bad := 0
	for node in [app.get("left_dock"), app.get("right_dock")]:
		var st3: Array = [node]
		while not st3.is_empty():
			var n: Node = st3.pop_back()
			for c in n.get_children(true):
				st3.append(c)
			if not (n is OptionButton):
				continue
			var o := n as OptionButton
			if not o.is_visible_in_tree():
				continue
			var was := o.fit_to_longest_item
			o.fit_to_longest_item = true
			await _frames(2)
			var longest := o.get_combined_minimum_size().x
			o.fit_to_longest_item = was
			await _frames(2)
			var expands := (o.size_flags_horizontal & Control.SIZE_EXPAND) != 0
			var ok := o.size.x + 0.5 >= longest and o.size.x >= 50.0
			if not ok:
				bad += 1
			print("  %s fit=%s expand=%s drawn=%.0f longest_item=%.0f  %s" % [
				"ok  " if ok else "FLAG", str(o.fit_to_longest_item), str(expands),
				o.size.x, longest, o.text])
	print("  controls below their longest item or under 50 px: ", bad)

	get_tree().quit(0)
