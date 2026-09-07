extends Node
## **`DccShell.dock_fit()`'s guard**, and the three controls that make it one.
##
##   Godot_v4.7.1 --headless --path . _dockfit_probe.tscn -- --vp 1920x1080
##   Godot_v4.7.1 --headless --path . _dockfit_probe.tscn -- --vp 1366x768
##
## Reads `-- --vp WxH` and nothing else; an unrecognised argument is fatal.
## The window's own `--resolution` is not read -- the shell is built into a
## `SubViewport` of the size above, the way `_ds03fit_probe.gd` does it.
##
## `_ds03fit_probe.gd` already asserts the *invariant* (no dock panel forces
## its dock open) and would go red if `dock_fit()` were deleted outright. What
## it cannot see is the two ways a narrower version of the rule goes wrong,
## which is what this file is for:
##
## - dropping the `SIZE_EXPAND` guard, which would take the content minimum
##   off a dropdown that has nothing else to size from, and
## - widening the walk past the two docks, which would reach
##   `tool_options_row` -- where `cartography_workspace.gd`'s Family/Variant
##   pair MUST keep `fit_to_longest_item`, because `set_tool_options()` fits
##   that row with `wide = true` for exactly that reason.
##
## Both are asserted with an injected control rather than by finding a natural
## one, so the assertion does not depend on which panels a seed happens to
## build, and so a `NEGATIVE` control exists at all.

## **The `LAPTOP` band is over budget on four panels, and this is not that
## pass's doing.** `--ldW` is 330 at `w1366` (`ENV:1819`) and
## `_ds03fit_probe.gd` never runs there -- it boots one 2560 x 1600 viewport.
## Measured with `_ldwidth_probe.gd` at 1366 x 768, before that change, after
## it, and after FIX-VIEWPORT's planner fix (2026-09-07):
##
##   world/a               336 -> 330 -> 330   fixed by `dock_fit()`
##   civilization/planner  404 -> 371 -> 351   the nested scroller, dropped
##   cartography/style     333 -> 333 -> 333   untouched: no dropdown drives it
##   cartography/labels    332 -> 332 -> 332   untouched
##   cartography/icons     338 -> 338 -> 338   untouched
##
## The remaining four are content-side. `_jpdock_probe.gd` section B prints the
## additive chain for each and it is the same chain three times over: a
## `DccWidgets` slider row's three canvas-fixed minimums (`ROW_LABEL_W` 132 +
## `TRACK_W` 78 + `ROW_VALUE_W` 44, plus separation = 278) inside `group()`'s
## 14 + 12 body pad, plus **8 px of dock scrollbar that `--ldW:330` does not
## budget** -- `_scroll()` disables the horizontal axis, so `ScrollContainer`
## folds the visible vertical bar into its own horizontal minimum. They are
## pinned at their MEASURED values rather than waived, so a regression still
## fails and a fifth entry fails immediately.
##
## **The planner's entry was a ceiling 8 px loose and is now an equality.** It
## read 379 while the rail sweep measured 371, because the dock's own `_scroll()`
## raised a vertical scrollbar between the two measurements and folded its 8 px
## into the dock's minimum -- so the worse of two real numbers was pinned, and a
## regression of up to 8 px passed there unnoticed. Dropping the planner's
## nested `ScrollContainer` hands the dock back the content height that scroller
## was hiding, so the bar is up in BOTH states now and both sites measure the
## same **351**. The header's own advice was "do a per-site expectation when a
## third site appears, or sooner if this dock is worked on again"; the dock was
## worked on again, and one number that is exact at both sites is the tighter
## answer than two. The earlier `412 -> 379` in this header was corrected once
## already: a clean-worktree measurement gave 404 -> 371.
const KNOWN_330 := {
	## **330 since 2026-09-07, and this pin said 351 until then.** The planner
	## category now fits the cap exactly, so it is no longer an exception at
	## all -- it is listed here only because removing the key would silently
	## drop the site from the per-site equality check below.
	##
	## **The cause is NOT established, and is deliberately not guessed at.** It
	## is not the sculpt-body gate added the same day: mutating that back to
	## always-visible leaves this at 330. Something between the 351 measurement
	## and now took 21 px out of this category, and a probe that PASSES for an
	## unknown reason is worth less than one that fails for a known one --
	## so this note stands until someone bisects it.
	"civilization/planner": 330.0,
	"cartography/style": 333.0,
	"cartography/labels": 332.0,
	"cartography/icons": 338.0,
}

var _fail := 0
var _vp: SubViewport

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

## An `OptionButton` whose longest item is far wider than anything a dock row
## can give it, so `fit_to_longest_item` is measurable in pixels rather than
## only readable as a flag.
func _probe_option(expand: bool) -> OptionButton:
	var ob := OptionButton.new()
	for s in ["A", "An item label long enough to set a dock's width on its own"]:
		ob.add_item(s)
	ob.selected = 0
	if expand:
		ob.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	else:
		ob.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	return ob

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
	var ld := app.get("left_dock") as Control
	var body := app.get("left_dock_body") as Control
	print("[VP] ", w, "x", h, "  _left_width=", lw, "  tablet=", DccTheme.is_tablet())

	print("")
	print("-- POSITIVE: an expanding dropdown inside the left dock is fitted --")
	var pos := _probe_option(true)
	var pos_row := HBoxContainer.new()
	pos_row.add_child(pos)
	body.add_child(pos_row)
	await _frames(6)
	_ok("expanding, in the dock: fit_to_longest_item is off",
		pos.fit_to_longest_item, false)
	## The pixels, not just the flag: with the flag on this control asks for
	## the width of its long item, and that is what forces a dock open.
	var fitted := pos.get_combined_minimum_size().x
	pos.fit_to_longest_item = true
	await _frames(4)
	_ok("and that is what lowers its minimum width",
		fitted < pos.get_combined_minimum_size().x, true)
	pos.fit_to_longest_item = false
	await _frames(4)

	print("")
	print("-- NEGATIVE 1: a NON-expanding dropdown in the dock keeps its item --")
	## `phone_fit()`'s guard, restated here: a control sized by its own text
	## and not expanding has nothing else to take a width from.
	var neg := _probe_option(false)
	var neg_row := HBoxContainer.new()
	neg_row.add_child(neg)
	neg_row.add_child(DccTheme.spacer())
	body.add_child(neg_row)
	await _frames(6)
	_ok("non-expanding, in the dock: fit_to_longest_item is untouched",
		neg.fit_to_longest_item, true)

	print("")
	print("-- NEGATIVE 2: the tool options row is NOT a dock --")
	## `set_tool_options()` fits this row `wide = true`, and
	## `cartography_workspace.gd`'s ICONS row builds Family and Variant into it
	## through the same `DccWidgets.choice()`. If this walk ever reaches the
	## row, that pair collapses onto its drop-down arrow.
	var tor := app.get("tool_options_row") as Control
	_ok("tool_options_row exists", tor != null, true)
	if tor != null:
		var outside := _probe_option(true)
		tor.add_child(outside)
		## **The pass has to be made to run while this control exists**, or the
		## assertion is vacuous. `_on_dock_node_added()` ignores a node whose
		## ancestors miss both docks, so adding to `tool_options_row` triggers
		## nothing on its own -- and a mutant that widened the walk to the whole
		## shell SURVIVED this check until the line below was added.
		var kick := Control.new()
		body.add_child(kick)
		await _frames(8)
		_ok("expanding, outside both docks: fit_to_longest_item is untouched",
			outside.fit_to_longest_item, true)
		body.remove_child(kick)
		kick.queue_free()
		tor.remove_child(outside)
		outside.queue_free()

	body.remove_child(pos_row)
	pos_row.queue_free()
	body.remove_child(neg_row)
	neg_row.queue_free()
	await _frames(6)

	print("")
	print("-- the canvas number itself, as a LITERAL --")
	## **`_ds03fit_probe` cannot pin this and neither could anything else in
	## the tree.** Its cap is `shell._left_width`, which is assigned from
	## `DccTheme.W_LEFT_DOCK`, so mutating that constant moves *both* sides of
	## its comparison and the probe stays green at any value -- the "assert a
	## constant against itself" shape. Until this line, 372 was held by
	## `ENV:25` (`--ldW:372px`) and by prose, and nothing in code would have
	## noticed it drifting.
	##
	## `ENV:1819` is the other half of the same declaration: `--ldW:330px` when
	## the frame is `w1366`, `--ldW:400px` on touch. The touch figure is
	## `_ds03fit_probe --force-touch`'s to assert, not this file's.
	var canvas_ldw := 372.0 if w >= 1920 else 330.0
	_ok("W_LEFT_DOCK is the canvas's --ldW (ENV:25)", DccTheme.W_LEFT_DOCK, 372)
	_ok("and the shipped dock is that number at this frame (ENV:1819)", lw, canvas_ldw)

	print("")
	print("-- the invariant itself, every rail node --")
	for n in DccShell.RAIL_NODES:
		if String(n.get("kind", "")) != "node":
			continue
		var id: String = String(n["domain"]) + "/" + String(n["mode"])
		app.call("_on_rail_node_pressed", String(n["domain"]), String(n["mode"]))
		await _frames(12)
		var cap := lw
		var known := is_equal_approx(lw, 330.0) and KNOWN_330.has(id)
		if known:
			cap = float(KNOWN_330[id])
		_le("%s: the left dock is not forced open%s" % [id,
			"  [KNOWN over 330, see the header]" if known else ""], ld.size.x, cap)
		## **A cap only fails upward.** Each `KNOWN_330` figure is a measurement,
		## so it is also asserted as an equality -- a panel that gets 8 px
		## NARROWER is progress that must be recorded here rather than passing
		## silently and leaving the constant a fiction. The `_le` above stays for
		## the unknown ids, where `lw` is the budget and not a measurement.
		if known:
			_ok("%s: and is exactly its measured figure" % id,
				"%.0f" % ld.size.x, "%.0f" % cap)
		## What the fix costs, counted rather than argued: a dropdown that no
		## longer reserves its longest item ellipsizes it instead when the row
		## is narrower than that item. Measured by asking each control what it
		## would need, then putting the flag back.
		var narrow := 0
		var total := 0
		var st: Array = [body]
		while not st.is_empty():
			var nd: Node = st.pop_back()
			for c in nd.get_children(true):
				st.append(c)
			if nd is OptionButton and not (nd as OptionButton).fit_to_longest_item:
				total += 1
				var o := nd as OptionButton
				o.fit_to_longest_item = true
				if o.get_combined_minimum_size().x > o.size.x + 0.5:
					narrow += 1
				o.fit_to_longest_item = false
		print("        dropdowns %d, of which %d are narrower than their longest item"
			% [total, narrow])

	print("")
	print("-- and the planner's own dropdowns, which is where this started --")
	app.call("_on_rail_node_pressed", "civilization", "planner")
	await _frames(16)
	var seen := 0
	var unfitted := 0
	var stack: Array = [body]
	while not stack.is_empty():
		var nd: Node = stack.pop_back()
		for c in nd.get_children(true):
			stack.append(c)
		if nd is OptionButton \
				and ((nd as Control).size_flags_horizontal & Control.SIZE_EXPAND) != 0:
			seen += 1
			if (nd as OptionButton).fit_to_longest_item:
				unfitted += 1
	_ok("the planner's expanding dropdowns were found", seen > 0, true)
	_ok("and none of them still reserves its longest item", unfitted, 0)

	print("")
	print("-- the transition INTO the widest state: pick the longest item --")
	## `fit_to_longest_item = false` alone makes the minimum track the
	## *selection*, so the dock is only safe until someone picks the long
	## option -- the "enumerate every transition into the state" trap. Driven
	## here rather than reasoned about: every dropdown in the dock is put on
	## its widest item at once, which is the worst case a user can reach.
	var widest: Array = []
	var st3: Array = [body]
	while not st3.is_empty():
		var nd2: Node = st3.pop_back()
		for c in nd2.get_children(true):
			st3.append(c)
		if nd2 is OptionButton \
				and ((nd2 as Control).size_flags_horizontal & Control.SIZE_EXPAND) != 0:
			widest.append(nd2)
	for o in widest:
		var ob2 := o as OptionButton
		var best := 0
		var best_w := -1.0
		for k in ob2.item_count:
			var f := ob2.get_theme_font("font")
			var fs := ob2.get_theme_font_size("font_size")
			var wpx := f.get_string_size(ob2.get_item_text(k), HORIZONTAL_ALIGNMENT_LEFT,
				-1.0, fs).x
			if wpx > best_w:
				best_w = wpx
				best = k
		ob2.selected = best
	await _frames(10)
	_ok("dropdowns switched to their longest item", widest.size() > 0, true)
	var tcap := float(KNOWN_330.get("civilization/planner", lw)) if is_equal_approx(lw, 330.0) else lw
	_le("and the dock still is not forced open", ld.size.x, tcap)
	## The second of the two sites the planner's figure covers. It used to
	## measure 8 px WIDER than the rail sweep -- see the header -- and now
	## measures the same 351, so it is asserted as an equality too rather than
	## left as the loose half of a shared ceiling.
	if is_equal_approx(lw, 330.0):
		_ok("and at exactly the same figure as the rail sweep",
			"%.0f" % ld.size.x, "%.0f" % tcap)

	print("")
	print("_dockfit_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
