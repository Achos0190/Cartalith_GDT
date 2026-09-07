extends Node
## FIX-VIEWPORT lane, 2026-09-07. Two things `_dockfit_probe` and
## `_ldwidth_probe` between them could not hold:
##
##  A. the planner's left panel no longer nests a second `ScrollContainer`
##     inside the dock's own -- **and still scrolls**, which is the entire risk
##     of removing it, proven by driving the dock's scrollbar to its end and
##     watching the bottom-most row come into the visible rect;
##  B. the `LAPTOP` band (`--ldW:330px`, `ENV:1819`) diagnosed per node: what
##     the pixels over 330 actually ARE at `cartography/style|labels|icons`.
##
##   Godot_v4.7.1 --headless --path . _jpdock_probe.tscn -- --vp 1920x1080
##   Godot_v4.7.1 --headless --path . _jpdock_probe.tscn -- --vp 1366x768
##
## Reads `-- --vp WxH` and nothing else; an unrecognised argument is fatal.
## Headless is correct and stated rather than assumed (`MISTAKES.md`): every
## assertion is a layout integer off `get_combined_minimum_size()` / `size` /
## `ScrollBar`, none of it rasterises and none of it is timed.

## **The number this lane moved, pinned as a literal, not as a relationship.**
## `left_dock_body`'s content minimum with `civilization/planner` open. It was
## **364** before this change and is **336** after it; the 28 px is the nested
## `ScrollContainer` the planner used to add, and it is 8 + 20:
##   *  8 px -- the scroller's own vertical `ScrollBar` minimum width, folded
##      into `ScrollContainer::get_minimum_size()` because the horizontal axis
##      is DISABLED there, i.e. a second copy of the bar the dock already has;
##   * 20 px -- the default `panel` stylebox's content margins, which the
##      dock's own `_scroll()` kills with `DccTheme.empty()` and the planner's
##      never did.
## Both halves are printed below at every run, so the split above is checkable
## and not just asserted. Viewport-independent: content minimum, not a frame.
const BODY_MIN_PLANNER := 336.0

## `ENV:25` `--ldW:372px`; `ENV:1819` `--ldW:330px` at `w1366`. The dock's own
## width with the planner open, as a LITERAL at each frame -- deliberately not
## `DccTheme.W_LEFT_DOCK ± furniture`, which would assert a relationship and
## stay green at any value of the constant (`_dockfit_probe`'s own header names
## that trap). 1920: the panel now fits inside `--ldW`, so the dock is exactly
## the canvas number.
##
## **1366 is 351 and the first draft of this file said 343, which is the one
## thing worth writing down here.** Removing the nested scroller gives the dock
## back a content height it never saw, so its OWN vertical scrollbar now shows
## at this node where it did not before -- and `ScrollContainer` folds that
## bar's 8 px into its horizontal minimum. So the planner does not just lose
## 28 px, it loses 28 and pays back 8: 336 content + 7 furniture (1 px right
## border + the 6 px drag handle) + 8 dock scrollbar = **351**, down from 371.
## Every other `LAPTOP`-band node was already paying that 8 (section B prints
## the same arithmetic for each); the planner was the outlier, not the fixed
## one. Reported as the residue over 330, not waived.
const DOCK_1920 := 372.0
const DOCK_1366 := 351.0

## The party form is tall; the assertion that it is non-empty needs a number no
## plausible collapse can reach, not `> 0`. Measured ~1500 px at 1920x1080.
const PARTY_MIN_H := 400.0

var _fail := 0
var _vp: SubViewport

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## Any path that never reaches `[RESULT]` exits non-zero instead of hanging the
## harness for its full timeout (`MISTAKES.md`, "a failed script load makes
## `godot --headless` hang").
func _watchdog(seconds: float) -> void:
	await get_tree().create_timer(seconds).timeout
	print("[RESULT] WATCHDOG after %.0fs failures=%d" % [seconds, _fail + 1])
	get_tree().quit(2)

func _leaves(root: Node) -> Array:
	var out: Array = []
	var st: Array = [root]
	while not st.is_empty():
		var n: Node = st.pop_back()
		for c in n.get_children(true):
			st.append(c)
		if n is Control and (n as Control).is_visible_in_tree():
			out.append(n)
	return out

## Bottom-most visible control under `root`, in viewport coordinates.
func _bottom(root: Node) -> float:
	var b := -1e9
	for c in _leaves(root):
		var ctl := c as Control
		b = maxf(b, ctl.global_position.y + ctl.size.y)
	return b

## Every `ScrollContainer` strictly between `n` and `stop`.
func _scrolls_between(n: Node, stop: Node) -> int:
	var k := 0
	var p := n.get_parent()
	while p != null and p != stop:
		if p is ScrollContainer:
			k += 1
		p = p.get_parent()
	return k

## `_ldwidth_probe`'s chain, kept for section B: everything whose content
## minimum is over `budget`, from the dock down to the leaf that causes it.
func _chain(n: Control, depth: int, budget: float) -> void:
	var comb := n.get_combined_minimum_size().x
	if comb <= budget:
		return
	var t: Variant = n.get("text")
	print("      %s%s  comb=%.0f own=%.0f  %s" % ["  ".repeat(depth), n.get_class(),
		comb, n.custom_minimum_size.x, (String(t).left(46) if t != null else "")])
	var widest: Control = null
	var over := false
	for c in n.get_children(true):
		if not (c is Control) or not (c as Control).is_visible_in_tree():
			continue
		var cc := c as Control
		if widest == null or cc.get_combined_minimum_size().x > widest.get_combined_minimum_size().x:
			widest = cc
		if cc.get_combined_minimum_size().x > budget:
			over = true
			_chain(cc, depth + 1, budget)
	## **The terminal node is the interesting one and the first draft stopped
	## short of it.** Where a container is over budget but none of its children
	## are, the pixels are the container's OWN -- margins, separation, a
	## stylebox -- so the walk switches from "everything over budget" to
	## "follow the widest child to the leaf", which is what actually names the
	## control at fault. Each line prints what its own container adds, so the
	## additive chain from leaf to dock reads straight down the column.
	if not over and widest != null:
		var cw := widest.get_combined_minimum_size().x
		print("      %s`- +%-3.0f -> %s comb=%.0f  %s" % ["  ".repeat(depth),
			comb - cw, widest.get_class(), cw,
			(String(widest.get("text")).left(46) if widest.get("text") != null else "")])
		if widest.get_child_count() > 0:
			_chain(widest, depth + 1, -1.0)

func _ready() -> void:
	_watchdog(300.0)
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
	var outer := app.get("_left_dock_scroll") as ScrollContainer
	print("[VP] %dx%d  _left_width=%.0f laptop=%s tablet=%s" % [w, h, lw,
		str(DccTheme.is_laptop()), str(DccTheme.is_tablet())])

	# ================================================== A. the planner =======
	app.call("_on_rail_node_pressed", "civilization", "planner")
	await _frames(20)
	var jp: Node = app.get("journey_planner_view")
	var party := jp.get("_left_party_body") as Control
	_ok("the planner's party body was built", party != null, true)
	if party == null:
		print("[RESULT] failures=%d" % (_fail))
		get_tree().quit(1)
		return

	print("")
	print("-- A1. the nested ScrollContainer is gone --")
	_ok("no ScrollContainer between the party form and left_dock_body",
		_scrolls_between(party, body), 0)
	_ok("left_dock_body's content minimum (was 364, the 28 px is the nesting)",
		"%.0f" % body.get_combined_minimum_size().x, "%.0f" % BODY_MIN_PLANNER)
	_ok("and the dock's own width at this frame",
		"%.0f" % ld.size.x, "%.0f" % (DOCK_1920 if w >= 1920 else DOCK_1366))

	## The 8 + 20 split of that 28, priced off a throwaway scroller built with
	## exactly the settings the planner used, so the header's arithmetic is
	## measured at every run instead of being a claim about a deleted line.
	var ghost := ScrollContainer.new()
	ghost.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	var filler := Control.new()
	filler.custom_minimum_size = Vector2(100, 4000)
	ghost.add_child(filler)
	body.add_child(ghost)
	await _frames(6)
	var bar := ghost.get_v_scroll_bar().get_combined_minimum_size().x
	var sb := ghost.get_theme_stylebox("panel").get_minimum_size().x
	print("     the 28 px, priced: v_scroll=%.0f + panel stylebox=%.0f = %.0f"
		% [bar, sb, bar + sb])
	body.remove_child(ghost)
	ghost.queue_free()
	await _frames(4)

	print("")
	print("-- A2. it STILL SCROLLS, which is the whole risk --")
	## Positive control first: if the panel did not overflow the dock there
	## would be nothing to prove and every assertion below would pass vacuously.
	var vs := outer.get_v_scroll_bar()
	print("     party.size=%s  outer page=%.0f max=%.0f" % [str(party.size),
		vs.page, vs.max_value])
	_ok("the party form has real height (a collapsed one would read 0)",
		party.size.y > PARTY_MIN_H, true)
	_ok("precondition: the dock's own scrollbar has somewhere to go",
		vs.max_value > vs.page + 1.0, true)
	var visible_bottom := outer.global_position.y + outer.size.y
	var deep_before := _bottom(party)
	_ok("precondition: the last row starts out clipped below the dock",
		deep_before > visible_bottom + 1.0, true)
	outer.scroll_vertical = int(vs.max_value)
	await _frames(8)
	print("     scroll_vertical=%d  bottom %.0f -> %.0f  (visible ends %.0f)"
		% [outer.scroll_vertical, deep_before, _bottom(party), visible_bottom])
	_ok("the dock scrolled", outer.scroll_vertical > 0, true)
	_ok("and the last row of the party form is now inside the dock",
		_bottom(party) <= visible_bottom + 1.0, true)
	## The other direction, so "it scrolled" is not one-way luck.
	outer.scroll_vertical = 0
	await _frames(8)
	_ok("and back to the top", outer.scroll_vertical, 0)
	_ok("which re-clips the last row", _bottom(party) > visible_bottom + 1.0, true)

	# ============================== B. the LAPTOP band, diagnosed ============
	## Diagnosis, not assertion: `_dockfit_probe.KNOWN_330` is where those three
	## numbers are pinned. What is printed here is WHY each is what it is, which
	## nothing in the tree said before. `FURNITURE` = 7 (1 px right border + the
	## 6 px drag handle) is `_ldwidth_probe`'s, restated.
	if w < 1920:
		print("")
		print("-- B. the LAPTOP band (--ldW:330, ENV:1819), per node --")
		for id in ["cartography/style", "cartography/labels", "cartography/icons",
				"civilization/planner"]:
			var pair: PackedStringArray = String(id).split("/")
			app.call("_on_rail_node_pressed", String(pair[0]), String(pair[1]))
			await _frames(16)
			var bmin := body.get_combined_minimum_size().x
			var dmin := ld.get_combined_minimum_size().x
			print("  %-22s body.min=%3.0f  dock.min=%3.0f  outer v_scroll shown=%s"
				% [id, bmin, dmin, str(outer.get_v_scroll_bar().visible)])
			print("      %3.0f content + 7 furniture + %2.0f dock scrollbar = %3.0f  (over 330 by %+.0f)"
				% [bmin, dmin - bmin - 7.0, dmin, dmin - 330.0])
			_chain(ld, 0, 330.0 - 7.0 - (dmin - bmin - 7.0))

	print("")
	print("[RESULT] _jpdock_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
